//Take a path buffer and return a list of instructions paired with its original line numbers.
//TODO Implement includes and recursively resolve them to create the one big vector

use std::ops::RangeInclusive;
use std::str::FromStr;

use arbitrary_int::traits::Integer;
use arbitrary_int::{u5, u18, u23, u40};
use isa::ImmTypeLabel::{SINT, UINT};
use isa::InstructionGroup::RTYPE;
use isa::{Imm, Operands, TokenStruct};
use isa::{
    ImmTypeLabel::{self},
    ImmVal,
    InstructionGroup::ITYPE,
    InstructionLabel::{self},
};


pub fn clean_input(input: &str) -> Result<Vec<(String, usize)>, String>
{
    let mut ret: Vec<(String, usize)> = Vec::new();

    let file_as_vec: Vec<(String, usize)> = input
        .lines()
        .map(|s| s.to_ascii_uppercase().trim().to_string())
        .enumerate()
        .map(|(b, a)| (a, b + 1))
        .map(|(mut s, l)| {
            s.truncate(s.find(r"//").unwrap_or(s.len()));
            (s.trim().to_string(), l)
        })
        .filter(|(s, _l)| !s.is_empty() || !(s.chars().all(char::is_whitespace)))
        .filter(|(s, _l)| s != ";")
        .collect();

    if file_as_vec.is_empty()
    {
        return Err("File is empty or just comments!".to_string());
    }

    if let Some(idx) = file_as_vec.iter().find(|(s, _l)| !s.is_ascii())
    {
        return Err(format!("File contains a non-ascii char at {}", idx.1));
    }

    for (line_string, i) in file_as_vec
    {
        if line_string.starts_with('#')
        {
            return Err(format!("Line {i}: includes not implemented yet"));
        }

        let line: String = match line_string.strip_suffix(";")
        {
            Some(s) => s.trim().to_string(),
            None => return Err(format!("Line {i} is missing semicolon!")),
        };

        assert!(!line.is_empty());
        ret.push((line, i));
    }

    Ok(ret)
}

//Break the string form of an instruction into a struct of enums and ints.
pub fn tokenize_instruction(instruction: &str) -> Result<TokenStruct, String>
{
    const VALID_OP_COUNTS: [usize; 2] = [2, 3];

    fn parse_register_r(s: &String) -> Result<u5, String>
    {
        let reg: u5 = u5::try_new(
            s.strip_prefix('R')
                .ok_or_else(|| format!("rd '{}' missing 'R' prefix", s))?
                .parse()
                .map_err(|e| format!("Couldn't map rd to number due to {e}"))?,
        )
        .map_err(|e| format!("Couldn't map rd index to u5 due to {e}"))?;
        Ok(reg)
    }

    let (mnemonic, rest) =
        instruction.split_once(char::is_whitespace).ok_or_else(|| format!("No whitespace found in '{instruction}'"))?;

    let instruction_label = InstructionLabel::from_str(mnemonic)
        .map_err(|e| format!("Couldn't convert label to instruction due to {e}"))?;
    let operands: Vec<String> = rest.split([',']).map(|s| s.trim().to_ascii_uppercase()).collect();

    let op_count = operands.len();

    if !VALID_OP_COUNTS.contains(&op_count)
    {
        return Err(format!(
            "Parsed an invalid number of operands. Got {} expected one of {:?}",
            operands.len(),
            VALID_OP_COUNTS
        ));
    }

    use InstructionLabel::*;

    let rd: u5 = parse_register_r(&operands[0])?;

    let tokens: TokenStruct = match instruction_label
    {
        // R-type 3
        ADD | SUB | XOR | OR | AND | SHL | SLT | SHR | MUL | SEQ | MULH | MAC | EXTB | INSRTB =>
        {
            if op_count != 3
            {
                return Err(format!("Parsed an invalid number of operands. Got {} expected 3", op_count,));
            }
            let rs1: u5 = parse_register_r(&operands[1])?;
            let rs2: u5 = parse_register_r(&operands[2])?;
            TokenStruct { label: instruction_label, group: RTYPE, operands: Operands::RType3 { rd, rs1, rs2 } }
        }
        //R-type 2
        NOT | NEG | MV =>
        {
            if op_count != 2
            {
                return Err(format!("Parsed an invalid number of operands. Got {} expected 2", op_count,));
            }
            let rs1: u5 = parse_register_r(&operands[1])?;
            TokenStruct { label: instruction_label, group: RTYPE, operands: Operands::RType2 { rd, rs1 } }
        }
        // I-type
        ADDI | SUBI | XORI | ORI | SHRI | SEQI | SLTI | ANDI | SHLI | INSRTBI =>
        {
            if op_count != 3
            {
                return Err(format!("Parsed an invalid number of operands. Got {} expected 3", op_count,));
            }
            let rs1: u5 = parse_register_r(&operands[1])?;
            let imm: Imm = process_imm(operands[2].as_str())?;
            TokenStruct { label: instruction_label, group: ITYPE, operands: Operands::IType3 { rd, rs1, imm } }
        }
        LUI | LI =>
        {
            if op_count != 2
            {
                return Err(format!("Parsed an invalid number of operands. Got {} expected 2", op_count,));
            }
            let imm: Imm = process_imm(operands[1].as_str())?;
            TokenStruct { label: instruction_label, group: ITYPE, operands: Operands::IType2 { rd, imm } }
        }
        // C-type
        CONVERT =>
        {
            todo!();
        }
    };

    check_tokens(&tokens)?;

    Ok(tokens)
}

fn check_tokens(token: &TokenStruct) -> Result<(), String>
{
    match token.operands
    {
        Operands::RType3 { rd, rs1, rs2 } =>
        {}
        Operands::RType2 { rd, rs1 } =>
        {}
        Operands::IType3 { rd, rs1, imm } =>
        {
            if token.label == InstructionLabel::INSRTBI
            {
                if imm.datatype == UINT || imm.datatype == SINT
                {
                    match imm.imm_val
                    {
                        ImmVal::Short(uint) =>
                        {
                            if uint > u8::MAX.into()
                            {
                                return Err(format!("Cannot fit immediate {uint} into unsigned packed."));
                            }
                        }
                        ImmVal::Long(uint) =>
                        {
                            if uint > u8::MAX.into()
                            {
                                return Err(format!("Cannot fit immediate {uint} into unsigned packed."));
                            }
                        }
                        ImmVal::Full(uint) =>
                        {
                            if uint > u8::MAX.into()
                            {
                                return Err(format!("Cannot fit immediate {uint} into unsigned packed."));
                            }
                        }
                    }
                }
                else
                {
                    return Err(format!("Cannot use immediate type {} with packed values", <&str>::from(imm.datatype)));
                }
            }
        }
        Operands::IType2 { rd, imm } =>
        {}
    }

    Ok(())
}

pub fn process_imm(imm_str: &str) -> Result<Imm, String>
{
    let imm_spilt_str: Vec<&str> = imm_str.split(":").collect();

    if imm_spilt_str.len() == 1
    {
        return Err(format!("Immediate did not spilt, does not contain type annotation or does not contain \":\""));
    }

    let imm_type: ImmTypeLabel =
        ImmTypeLabel::from_str(imm_spilt_str[1]).map_err(|e| format!("Couldn't convert immediate type due to {e}"))?;
    let imm_str = imm_spilt_str[0];

    let (digits, base): (&str, u32) = if imm_str.starts_with("0X")
    {
        (imm_str.strip_prefix("0X").unwrap(), 16)
    }
    else if imm_str.to_ascii_uppercase().starts_with("0B")
    {
        (imm_str.strip_prefix("0B").unwrap(), 16)
    }
    else
    {
        (imm_str, 10)
    };

    let parsed_fixed = |frac_bits: u64| -> Result<u64, String> {
        if base != 10
        {
            return Err("Fixed-point immediates must be base 10".to_string());
        }
        let float_repr: f64 =
            digits.parse().map_err(|e| format!("\nCouldn't parse fixed-point immediate '{digits}'\nError: {e}"))?;
        let scaled = (float_repr * (1u64 << frac_bits) as f64).round();
        Ok(scaled as i64 as u64)
    };

    let parse_ipacked = || -> Result<u64, String> {
        // 5,4,3,2,1
        let mut ret = 0u64;
        let mut packed_vals: Vec<String> = imm_str.split(' ').map(|s| s.trim().to_string()).collect();
        packed_vals[0] = packed_vals[0].strip_prefix("[").ok_or("No opening brace for packed immediate.")?.to_string();
        packed_vals[4] = packed_vals[4].strip_suffix("]").ok_or("No closing brace for packed immediate.")?.to_string();

        if packed_vals.len() != 5
        {
            return Err(format!("Packed value contains too few lanes {} instead of 5", packed_vals.len()));
        }

        for (i, v) in packed_vals.iter().enumerate()
        {
            let v_i8: i8 = v
                .parse::<i8>()
                .map_err(|e| format!("\nCannot process signed packed value {v}, does not fit into i8: {e}"))?;
            ret |= (v_i8 as u8 as u64) << (32 - (8 * i));
        }

        Ok(ret)
    };

    let parse_upacked = || -> Result<u64, String> {
        // 5,4,3,2,1
        let mut ret = 0u64;
        let mut packed_vals: Vec<String> = imm_str.split(' ').map(|s| s.trim().to_string()).collect();
        packed_vals[0] = packed_vals[0].strip_prefix("[").ok_or("No opening brace for packed immediate.")?.to_string();
        packed_vals[4] = packed_vals[4].strip_suffix("]").ok_or("No closing brace for packed immediate.")?.to_string();

        if packed_vals.len() != 5
        {
            return Err(format!("Packed value contains too few lanes {} instead of 5", packed_vals.len()));
        }

        for (i, v) in packed_vals.iter().enumerate()
        {
            let v_u8: u8 = v
                .parse::<u8>()
                .map_err(|e| format!("Cannot process unsigned packed value {v}, does not fit into u8: {e}"))?;
            ret |= (v_u8 as u64) << (32 - (8 * i));
        }

        Ok(ret)
    };

    let val = match imm_type
    {
        ImmTypeLabel::BOOL => match digits
        {
            "TRUE" => 1,
            "FALSE" => 0,
            _others =>
            {
                return Err(format!("\nInvalid bool, only TRUE and FALSE are valid. Got {_others}"));
            }
        },
        ImmTypeLabel::UINT | ImmTypeLabel::UMOD => match u64::from_str_radix(digits, base)
        {
            Ok(r) => r,
            Err(e) =>
            {
                return Err(format!(
                    "\nCould not convert unsigned immediate to number\nNumber: {}\nType: {}\nError: {e}",
                    digits,
                    <&str>::from(imm_type)
                ));
            }
        },
        ImmTypeLabel::SINT | ImmTypeLabel::SMOD =>
        {
            let tmp = match i64::from_str_radix(digits, base)
            {
                Ok(r) => r,
                Err(e) =>
                {
                    return Err(format!(
                        "\nCould not convert unsigned immediate to number:\nNumber: {}\nType: {}\nError: {e}",
                        digits,
                        <&str>::from(imm_type)
                    ));
                }
            };
            tmp as u64
        }
        ImmTypeLabel::Q31_8 => parsed_fixed(8)?,
        ImmTypeLabel::Q23_16 => parsed_fixed(16)?,
        ImmTypeLabel::Q19_20 => parsed_fixed(20)?,
        ImmTypeLabel::Q15_24 => parsed_fixed(24)?,
        ImmTypeLabel::Q7_32 => parsed_fixed(32)?,
        ImmTypeLabel::SPACKED => parse_ipacked()?,
        ImmTypeLabel::UPACKED => parse_upacked()?,
    };

    const UNSIGNED_SHORT_RANGE: RangeInclusive<u64> = 0..=((1u64 << 18) - 1);
    const UNSIGNED_LONG_RANGE: RangeInclusive<u64> = 0..=((1u64 << 23) - 1);
    const UNSIGNED_FULL_RANGE: RangeInclusive<u64> = 0..=(1u64 << 40) - 1;

    const SIGNED_SHORT_RANGE: RangeInclusive<i64> = -(1i64 << 17)..=((1i64 << 17) - 1);
    const SIGNED_LONG_RANGE: RangeInclusive<i64> = -(1i64 << 22)..=((1i64 << 22) - 1);
    const SIGNED_FULL_RANGE: RangeInclusive<i64> = -(1i64 << 39)..=((1i64 << 39) - 1);

    let imm_val: ImmVal;

    if isa::constants::SIGNED_ARIMETHIC_TYPES_IMM.contains(&imm_type)
        && !(imm_type == ImmTypeLabel::UPACKED || imm_type == ImmTypeLabel::SPACKED)
    {
        let val = val as i64;

        if SIGNED_SHORT_RANGE.contains(&val)
        {
            imm_val = ImmVal::Short(u18::masked_new(val as i32));
        }
        else if SIGNED_LONG_RANGE.contains(&val)
        {
            imm_val = ImmVal::Long(u23::masked_new(val as i32));
        }
        else if SIGNED_FULL_RANGE.contains(&val)
        {
            imm_val = ImmVal::Full(u40::masked_new(val));
        }
        else
        {
            return Err(format!(
                "Error signed immediate {}  of type {} does not fit into long immediate. Min is {} and Max is {}",
                val as i64,
                <&str>::from(imm_type),
                SIGNED_FULL_RANGE.min().unwrap(),
                SIGNED_FULL_RANGE.max().unwrap()
            ));
        }
    }
    else
    {
        if UNSIGNED_SHORT_RANGE.contains(&val)
        {
            imm_val = ImmVal::Short(u18::new(val as u32));
        }
        else if UNSIGNED_LONG_RANGE.contains(&val)
        {
            imm_val = ImmVal::Long(u23::new(val as u32));
        }
        else if UNSIGNED_FULL_RANGE.contains(&val)
        {
            imm_val = ImmVal::Full(u40::new(val));
        }
        else
        {
            return Err(format!("Unsigned Immediate {val} is too large to fit!"));
        }
    }

    Ok(Imm { imm_val, datatype: imm_type })
}
