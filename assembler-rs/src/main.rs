use std::{fmt::format, ops::RangeInclusive, path::PathBuf};

use clap::Parser;

use brot_isa::*;
use isa::{
    self as brot_isa,
    ImmTypeLabel::{SPACKED, UPACKED},
    ImmVal::{self, Full, Long, Short},
    InstructionGroup::ITYPE,
    InstructionLabel::{LI, LUI},
};

enum VarArgRet
{
    One(Word),
    Two((Word, Word)),
    Three((Word, Word, Word)),
}

use arbitrary_int::{traits::Integer, u18, u23, u40};

use crate::VarArgRet::One;

//Take a path buffer and return a list of instructions paired with its original line numbers.
//TODO Implement includes and recursively resolve them to create the one big vector
fn clean_input(input: &PathBuf) -> Result<Vec<(String, usize)>, String>
{
    let mut ret: Vec<(String, usize)> = Vec::new();

    let file_as_vec: Vec<(String, usize)> = std::fs::read_to_string(input)
        .map_err(|e| format!("Could not read input \"{}\" into string: {e}", input.display()))?
        .lines()
        .map(|s| s.to_ascii_uppercase().trim().to_string())
        .enumerate()
        .map(|(b, a)| (a, b))
        .filter(|(s, l)| !s.is_empty() || !(s.chars().all(char::is_whitespace)))
        .filter(|(s, l)| !(s.contains(r"\\") || s.contains(r"//")))
        .map(|(s, l)| (s, l + 1))
        .collect();

    if file_as_vec.is_empty()
    {
        return Err("File is empty or just comments!".to_string());
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
fn tokenize_instruction(instruction: &str, curr_line_num: usize) -> Result<InstructionInfo, String>
{
    const VALID_IMM_ARG_COUNTS: [usize; 2] = [2, 6];

    let Some(white_space_index) = instruction.find(char::is_whitespace)
    else
    {
        return Err("\nCould not find whitespace. Garbled Instruction".to_string());
    };

    let instruction_label: String = instruction[0..white_space_index].to_string().to_ascii_uppercase();

    let instruction_label: InstructionLabel = instruction_label
        .parse()
        .map_err(|e| format!("\nCould not map instruction to instruction label due to {e}"))?;

    let mut info_struct = InstructionInfo::label_to_info(instruction_label)
        .map_err(|e| format!("\nCould not map label to instruction struct because of {e}"))?;

    let operands: Vec<String> =
        instruction[white_space_index + 1..].split(',').map(|s| s.trim().to_ascii_uppercase().to_string()).collect();

    if !info_struct.valid_operand_counts.contains(&operands.len())
    {
        return Err(format!(
            "{:?} takes {:?} operands, found {}. Did you leave extra commas or use commas instead of spaces for a packed type?",
            info_struct.label,
            info_struct.valid_operand_counts,
            operands.len()
        ));
    }

    for (op_index, op) in operands.iter().enumerate()
    {
        let op = op.trim().to_string().to_ascii_uppercase();

        if op.starts_with('R')
        {
            let op_num = op
                .strip_prefix('R')
                .ok_or("Register not prefixed with 'R'")?
                .parse::<u8>()
                .map_err(|e| format!("\nCould not parse register into u8. Out of bounds?\nError{e}"))?;

            info_struct.register_args.push(RegisterIndexArg { index: op_num, kind: RegisterType::GPR });
        }
        else if op.starts_with('P')
        {
            let op_num = op
                .strip_prefix('P')
                .ok_or("Register not prefixed with 'P'")?
                .parse::<u8>()
                .map_err(|e| format!("\nCould not parse register into u8. Out of bounds?\nError{e}"))?;

            info_struct.register_args.push(RegisterIndexArg { index: op_num, kind: RegisterType::Pointer });
        }
        else if op.contains(':')
        {
            if op_index != operands.len() - 1
            {
                return Err("Only the last instruction should have \":\" ".to_string());
            }

            let imm_spilt: Vec<String> = op.split(':').map(|s| s.to_string()).collect();

            if !VALID_IMM_ARG_COUNTS.contains(&imm_spilt.len())
            {
                return Err(format!(
                    "\nParsed immediate argument and found invalid argument count: {}",
                    imm_spilt.len()
                ));
            }

            let mut imm_val_str: Vec<String> = imm_spilt[0..imm_spilt.len() - 1]
                .to_vec()
                .iter()
                .map(|s| s.to_ascii_uppercase().trim().to_string())
                .collect();

            let imm_type: ImmTypeLabel =
                ImmTypeLabel::try_from(imm_spilt[imm_spilt.len() - 1].as_str()).map_err(|e| e.to_string())?;

            if imm_type == SPACKED || imm_type == UPACKED
            {
                imm_val_str = imm_val_str
                    .iter()
                    .map(|s| s.strip_prefix("[").unwrap_or(s).strip_suffix("]").unwrap_or(s).trim().to_string())
                    .collect();
            }

            if info_struct.register_args.get(1).unwrap().index == 1
                && !matches!(info_struct.register_args.get(1).unwrap().kind, RegisterType::Pointer)
                && imm_type != ImmTypeLabel::UINT
            {
                return Err("\nYou are trying to use an immediate instruction with GPR0 that is not UINT".to_string());
            }

            let imm_bin: ImmVal = match process_imm(&imm_val_str[0], imm_type)
            {
                Ok(r) => r,
                Err(e) =>
                {
                    return Err(format!("\nUnable to process immediate due to:{}", e));
                }
            };

            info_struct.imm_arg.datatype = imm_type;
            info_struct.imm_arg.imm_val = imm_bin;
            info_struct.imm_arg.valid = true;
        }
    }

    if info_struct.register_args.clone().iter().any(|r| r.index > MAX_GPR_VAL as u8)
    {
        return Err("\nParsed a register value beyond the max range.".to_string());
    }

    if !info_struct.valid_operand_counts.contains(&(info_struct.get_arg_count() as usize))
    {
        return Err("\nNot a vaild number of operands".to_string());
    }

    if info_struct.group == ITYPE && !info_struct.imm_arg.valid
    {
        return Err("\nWas an I-type instruction but didn't find any immediate arguments".to_string());
    }

    Ok(info_struct)
}

//Takes an imm_str and its type and generates its embedded form in a u18.
fn process_imm(imm_str: &str, imm_type: ImmTypeLabel) -> Result<ImmVal, String>
{
    use brot_isa::*;

    let imm_str = &imm_str.to_ascii_uppercase();

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
        let packed_vals: Vec<String> = imm_str.split(' ').map(|s| s.trim().to_string()).collect();

        if packed_vals.len() != 5
        {
            return Err(format!("Packed value contains too few lanes {} instead of 5", packed_vals.len()));
        }

        for (i, v) in packed_vals.iter().enumerate()
        {
            let v_i8: i8 = v
                .parse::<i8>()
                .map_err(|e| format!("\nCannot process signed packed value {v}, does not fit into i8"))?;
            ret |= (v_i8 as u8 as u64) << (32 - (8 * i));
        }

        Ok(ret)
    };

    let parse_upacked = || -> Result<u64, String> {
        // 5,4,3,2,1
        let mut ret = 0u64;
        let packed_vals: Vec<String> = imm_str.split(' ').map(|s| s.trim().to_string()).collect();

        if packed_vals.len() != 5
        {
            return Err(format!("Packed value contains too few lanes {} instead of 5", packed_vals.len()));
        }

        for (i, v) in packed_vals.iter().enumerate()
        {
            let v_u8: u8 = v
                .parse::<u8>()
                .map_err(|e| format!("Cannot process unsigned packed value {v}, does not fit into u8"))?;
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

    if SIGNED_ARIMETHIC_TYPES_IMM.contains(&imm_type) && !(imm_type == UPACKED || imm_type == SPACKED)
    {
        let val = val as i64;

        if SIGNED_SHORT_RANGE.contains(&val)
        {
            Ok(ImmVal::Short(u18::masked_new(val as i32)))
        }
        else if SIGNED_LONG_RANGE.contains(&val)
        {
            Ok(ImmVal::Long(u23::masked_new(val as i32)))
        }
        else if SIGNED_FULL_RANGE.contains(&val)
        {
            Ok(ImmVal::Full(u40::masked_new(val)))
        }
        else
        {
            Err(format!(
                "Error signed immediate {}  of type {} does not fit into long immediate. Min is {} and Max is {}",
                val as i64,
                <&str>::from(imm_type),
                SIGNED_FULL_RANGE.min().unwrap(),
                SIGNED_FULL_RANGE.max().unwrap()
            ))
        }
    }
    else
    {
        if UNSIGNED_SHORT_RANGE.contains(&val)
        {
            Ok(ImmVal::Short(u18::new(val as u32)))
        }
        else if UNSIGNED_LONG_RANGE.contains(&val)
        {
            Ok(ImmVal::Long(u23::new(val as u32)))
        }
        else if UNSIGNED_FULL_RANGE.contains(&val)
        {
            Ok(ImmVal::Full(u40::new(val)))
        }
        else
        {
            Err(format!("Unsigned Immediate {val} is too large to fit!"))
        }
    }
}
fn embed_tag_opcode_from_struct(info_struct: &InstructionInfo) -> Result<Word, String>
{
    let mut ret: Word = Word { tag: TypeLabel::INSTRUCTION, data: u40::new(0) };
    ret.data |= u40::new(info_struct.label.into()) << START_OPCODE_FIELD;
    Ok(ret)
}

fn embed_tag_opcode_from_label(label: InstructionLabel) -> Result<Word, String>
{
    let mut ret: Word = Word { tag: TypeLabel::INSTRUCTION, data: u40::new(0) };
    ret.data |= u40::new(label.into()) << START_OPCODE_FIELD;
    Ok(ret)
}

fn assemble_r_type(info_struct: &InstructionInfo) -> Result<VarArgRet, String>
{
    let mut data_word_encoding: Word = embed_tag_opcode_from_struct(info_struct)?;

    if info_struct.get_arg_count() != 3
    {
        return Err(format!(
            "{} did not tokenize properly, incorrect number of args. Supposed to be 3 got {} ",
            <&str>::from(info_struct.label),
            info_struct.get_arg_count() != 3
        ));
    }

    data_word_encoding.data = u40::new(insert_bits(
        data_word_encoding.data.value(),
        info_struct.register_args[0].index as u64,
        RD_END,
        RD_START,
    ));

    data_word_encoding.data = u40::new(insert_bits(
        data_word_encoding.data.value(),
        info_struct.register_args[1].index as u64,
        RS1_END,
        RS1_START,
    ));

    data_word_encoding.data = u40::new(insert_bits(
        data_word_encoding.data.value(),
        info_struct.register_args[2].index as u64,
        RS2_END,
        RS2_START,
    ));

    Ok(One(data_word_encoding))
}

fn assemble_i_type(info_struct: &InstructionInfo) -> Result<VarArgRet, String>
{
    let mut data_word_encoding: Word = embed_tag_opcode_from_struct(info_struct)?;

    data_word_encoding.data = u40::new(insert_bits(
        data_word_encoding.data.value(),
        info_struct.register_args[0].index as u64,
        RD_END,
        RD_START,
    ));
    data_word_encoding.data = u40::new(insert_bits(
        data_word_encoding.data.value(),
        info_struct.register_args[1].index as u64,
        RS1_END,
        RS1_START,
    ));

    if info_struct.label == LI
    {
        match info_struct.imm_arg.imm_val
        {
            Short(uint) =>
            {
                let mut imm_bin: u64 = uint.value() as u64;
                //Sign extend
                if SIGNED_ARIMETHIC_TYPES_IMM.contains(&info_struct.imm_arg.datatype)
                {
                    imm_bin = (((imm_bin << 5u64) as i64) >> 5i64) as u64;
                }
                imm_bin |= <u64>::from(info_struct.imm_arg.datatype) << LONG_IMM_TAG_START;

                data_word_encoding.data =
                    u40::new(insert_bits(data_word_encoding.data.value(), imm_bin, LONG_IMM_END, IMM_START));
                Ok(One(data_word_encoding))
            }
            Long(uint) =>
            {
                let imm_bin: u64 =
                    (uint.value() as u64) | <u64>::from(info_struct.imm_arg.datatype) << LONG_IMM_TAG_START;
                data_word_encoding.data =
                    u40::new(insert_bits(data_word_encoding.data.value(), imm_bin, LONG_IMM_END, IMM_START));
                Ok(One(data_word_encoding))
            }
            Full(uint) =>
            {
                let imm_bin_lower =
                    extract_bits(uint.value(), 22, 0) | <u64>::from(info_struct.imm_arg.datatype) << LONG_IMM_TAG_START;
                let imm_bin_upper = extract_bits(uint.value(), 39, 23)
                    | <u64>::from(info_struct.imm_arg.datatype) << LONG_IMM_TAG_START;

                let mut li: Word = embed_tag_opcode_from_label(LI)?;
                let mut lui: Word = embed_tag_opcode_from_label(LUI)?;

                li.data =
                    u40::new(insert_bits(li.data.value(), info_struct.register_args[0].index as u64, RD_END, RD_START));

                lui.data = u40::new(insert_bits(
                    lui.data.value(),
                    info_struct.register_args[0].index as u64,
                    RD_END,
                    RD_START,
                ));

                li.data = u40::new(insert_bits(li.data.value(), imm_bin_lower, LONG_IMM_END, IMM_START));

                lui.data = u40::new(insert_bits(lui.data.value(), imm_bin_upper, LONG_IMM_END, LONG_IMM_START));

                Ok(VarArgRet::Two((li, lui)))
            }
        }
    }
    else if info_struct.label == LUI
    {
        match info_struct.imm_arg.imm_val
        {
            Short(uint) =>
            {
                let mut imm_bin: u64 = uint.value() as u64;
                //Sign extend
                if SIGNED_ARIMETHIC_TYPES_IMM.contains(&info_struct.imm_arg.datatype)
                {
                    imm_bin = (((imm_bin << 5u64) as i64) >> 5i64) as u64;
                }
                imm_bin |= <u64>::from(info_struct.imm_arg.datatype) << LONG_IMM_TAG_START;

                data_word_encoding.data =
                    u40::new(insert_bits(data_word_encoding.data.value(), imm_bin, LONG_IMM_END, IMM_START));
                Ok(One(data_word_encoding))
            }
            Long(uint) =>
            {
                let imm_bin: u64 =
                    (uint.value() as u64) | (<u64>::from(info_struct.imm_arg.datatype) << LONG_IMM_TAG_START);
                data_word_encoding.data =
                    u40::new(insert_bits(data_word_encoding.data.value(), imm_bin, LONG_IMM_END, IMM_START));
                Ok(One(data_word_encoding))
            }
            Full(uint) => Err("Can't expand LUI value into two instructions. Please use LI instead.".to_string()),
        }
    }
    else
    {
        match info_struct.imm_arg.imm_val
        {
            Short(uint) =>
            {
                let imm_bin: u64 = (uint.value() as u64) | (<u64>::from(info_struct.imm_arg.datatype) << IMM_TAG_START);

                data_word_encoding.data = u40::new(insert_bits(
                    data_word_encoding.data.value(),
                    info_struct.register_args[1].index as u64,
                    RS1_END,
                    RS1_START,
                ));

                data_word_encoding.data =
                    u40::new(insert_bits(data_word_encoding.data.value(), imm_bin, IMM_END, IMM_START));

                Ok(One(data_word_encoding))
            }
            Long(uint) =>
            {
                let imm_bin: u64 =
                    (uint.value() as u64) | (<u64>::from(info_struct.imm_arg.datatype) << LONG_IMM_TAG_START);

                let mut li: Word = embed_tag_opcode_from_label(LI)?;

                li.data =
                    u40::new(insert_bits(li.data.value(), info_struct.register_args[0].index as u64, RD_END, RD_START));

                li.data = u40::new(insert_bits(li.data.value(), imm_bin, LONG_IMM_END, IMM_START));

                let mut r_word: Word = embed_tag_opcode_from_label(itype_to_rtype_equivalent(info_struct.label)?)?;

                r_word.data = u40::new(insert_bits(
                    r_word.data.value(),
                    info_struct.register_args[0].index as u64,
                    RD_END,
                    RD_START,
                ));

                r_word.data = u40::new(insert_bits(
                    r_word.data.value(),
                    info_struct.register_args[1].index as u64,
                    RS1_END,
                    RS1_START,
                ));

                r_word.data = u40::new(insert_bits(
                    r_word.data.value(),
                    info_struct.register_args[0].index as u64,
                    RS2_END,
                    RS2_START,
                ));

                Ok(VarArgRet::Two((li, r_word)))
            }
            Full(uint) =>
            {
                let imm_bin_lower =
                    extract_bits(uint.value(), 22, 0) | <u64>::from(info_struct.imm_arg.datatype) << LONG_IMM_TAG_START;
                let imm_bin_upper = extract_bits(uint.value(), 39, 23)
                    | <u64>::from(info_struct.imm_arg.datatype) << LONG_IMM_TAG_START;

                let mut li: Word = embed_tag_opcode_from_label(LI)?;
                let mut lui: Word = embed_tag_opcode_from_label(LUI)?;

                li.data =
                    u40::new(insert_bits(li.data.value(), info_struct.register_args[0].index as u64, RD_END, RD_START));

                lui.data = u40::new(insert_bits(
                    lui.data.value(),
                    info_struct.register_args[0].index as u64,
                    RD_END,
                    RD_START,
                ));

                li.data = u40::new(insert_bits(li.data.value(), imm_bin_lower, LONG_IMM_END, IMM_START));

                lui.data = u40::new(insert_bits(lui.data.value(), imm_bin_upper, LONG_IMM_END, LONG_IMM_START));

                let mut r_word: Word = embed_tag_opcode_from_label(itype_to_rtype_equivalent(info_struct.label)?)?;

                r_word.data = u40::new(insert_bits(
                    r_word.data.value(),
                    info_struct.register_args[0].index as u64,
                    RD_END,
                    RD_START,
                ));

                r_word.data = u40::new(insert_bits(
                    r_word.data.value(),
                    info_struct.register_args[1].index as u64,
                    RS1_END,
                    RS1_START,
                ));

                r_word.data = u40::new(insert_bits(
                    r_word.data.value(),
                    info_struct.register_args[0].index as u64,
                    RS2_END,
                    RS2_START,
                ));

                Ok(VarArgRet::Three((li, lui, r_word)))
            }
        }
    }
}

fn assemble_c_type(_instruction_info_struct: &InstructionInfo) -> Result<VarArgRet, String>
{
    Ok(One(Word { tag: TypeLabel::FREE, data: u40::new(0) }))
}

fn assemble_s_type(_instruction_info_struct: &InstructionInfo) -> Result<VarArgRet, String>
{
    Ok(One(Word { tag: TypeLabel::FREE, data: u40::new(0) }))
}
fn assemble_b_type(_instruction_info_struct: &InstructionInfo) -> Result<VarArgRet, String>
{
    Ok(One(Word { tag: TypeLabel::FREE, data: u40::new(0) }))
}
fn assemble_u_type(_instruction_info_struct: &InstructionInfo) -> Result<VarArgRet, String>
{
    Ok(One(Word { tag: TypeLabel::FREE, data: u40::new(0) }))
}
fn assemble_j_type(_instruction_info_struct: &InstructionInfo) -> Result<VarArgRet, String>
{
    Ok(One(Word { tag: TypeLabel::FREE, data: u40::new(0) }))
}
fn assemble_l_type(_instruction_info_struct: &InstructionInfo) -> Result<VarArgRet, String>
{
    Ok(One(Word { tag: TypeLabel::FREE, data: u40::new(0) }))
}
fn assemble_p_type(_instruction_info_struct: &InstructionInfo) -> Result<VarArgRet, String>
{
    Ok(One(Word { tag: TypeLabel::FREE, data: u40::new(0) }))
}
fn assemble_q_type(_instruction_info_struct: &InstructionInfo) -> Result<VarArgRet, String>
{
    Ok(One(Word { tag: TypeLabel::FREE, data: u40::new(0) }))
}

fn assemble(instruction_vector: &Vec<(String, usize)>) -> Result<Vec<String>, Vec<(String, usize)>>
{
    let mut output_buffer: Vec<String> = Vec::with_capacity(instruction_vector.len());
    let mut error_vector: Vec<(String, usize)> = Vec::new();

    use brot_isa::{InstructionGroup::*, InstructionInfo};

    for (instruction_string, curr_line_num) in instruction_vector
    {
        let instruction_string_comment = format!("{}{}", r"\\", instruction_string);

        let instruction_info_struct: InstructionInfo = match tokenize_instruction(instruction_string, *curr_line_num)
        {
            Ok(r) => r,
            Err(e) =>
            {
                error_vector.push((e, *curr_line_num));
                continue;
            }
        };

        let instruction_bin = match instruction_info_struct.group
        {
            RTYPE => assemble_r_type(&instruction_info_struct),
            CTYPE => assemble_c_type(&instruction_info_struct),
            ITYPE => assemble_i_type(&instruction_info_struct),
            STYPE => assemble_s_type(&instruction_info_struct),
            BTYPE => assemble_b_type(&instruction_info_struct),
            UTYPE => assemble_u_type(&instruction_info_struct),
            JTYPE => assemble_j_type(&instruction_info_struct),
            LTYPE => assemble_l_type(&instruction_info_struct),
            PTYPE => assemble_p_type(&instruction_info_struct),
            QTYPE => assemble_q_type(&instruction_info_struct),
        };

        match instruction_bin
        {
            Ok(r) => match r
            {
                One(word) => output_buffer.push(format!("{:012X} {instruction_string_comment:12}", word.to_u64())),
                VarArgRet::Two((word1, word2)) =>
                {
                    output_buffer.push(format!("{:012X} {instruction_string_comment:12} LI Expanded", word1.to_u64()));
                    output_buffer
                        .push(format!("{:012X} {instruction_string_comment:12} R-Type Equivalent", word2.to_u64()));
                }
                VarArgRet::Three((word1, word2, word3)) =>
                {
                    output_buffer.push(format!(
                        "{:012X} {instruction_string_comment:12} <- Original LI Expanded term",
                        word1.to_u64()
                    ));
                    output_buffer.push(format!(
                        "{:012X} {instruction_string_comment:12} <- Original LUI Expanded",
                        word2.to_u64()
                    ));
                    output_buffer
                        .push(format!("{:012X} {instruction_string_comment:12} <- R-Type Equivalent", word2.to_u64()));
                }
            },
            Err(e) =>
            {
                error_vector.push((e, *curr_line_num));
            }
        }
    }

    if !error_vector.is_empty()
    {
        return Err(error_vector);
    }
    Ok(output_buffer)
}

fn write_binary(instructions: &[String], output_file: &PathBuf) -> Result<(), String>
{
    std::fs::write(output_file, instructions.join("\n")).map_err(|e| format!("{e}"))
}

#[derive(Parser, Debug)]
struct Args
{
    #[arg(required = true)]
    input: PathBuf,
    #[arg(short)]
    output: PathBuf,
}
fn main()
{
    //Gonaherra argument parsing
    let args = Args::parse();
    //Clean Input file and store inot a vector of lines and original line numbers

    let instruction_vector = match clean_input(&args.input)
    {
        Ok(r) => r,
        Err(e) =>
        {
            println!("{e}");
            std::process::exit(-1);
        }
    };
    //Assemble into a vector of the binary encodings
    let ret_vector: Vec<String> = match assemble(&instruction_vector)
    {
        Ok(r) => r,
        Err(e) =>
        {
            for (s, line) in e
            {
                println!("ERROR: {s} on line {line}",);
            }
            std::process::exit(-1)
        }
    };

    //Encode them
    match write_binary(&ret_vector, &args.output)
    {
        Ok(_) =>
        {
            println!("Success! File Written.");
            std::process::exit(1)
        }
        Err(e) =>
        {
            println!("Could not write output file due to: {e} ");
            std::process::exit(-1);
        }
    }
}

#[cfg(test)]
mod tests
{
    use ImmTypeLabel::*;
    use arbitrary_int::{i18, traits::Integer, u18};
    use clap::builder::Str;
    use isa::ImmVal::Short;
    use test_case::test_case;

    const UNSIGNED_SHORT_RANGE: RangeInclusive<u64> = 0..=((1u64 << 18) - 1);
    const UNSIGNED_LONG_RANGE: RangeInclusive<u64> = 0..=((1u64 << 23) - 1);
    const UNSIGNED_FULL_RANGE: RangeInclusive<u64> = 0..=(1u64 << 40) - 1;

    const SIGNED_SHORT_RANGE: RangeInclusive<i64> = -(1i64 << 17)..=((1i64 << 17) - 1);
    const SIGNED_LONG_RANGE: RangeInclusive<i64> = -(1i64 << 22)..=((1i64 << 22) - 1);
    const SIGNED_FULL_RANGE: RangeInclusive<i64> = -(1i64 << 39)..=((1i64 << 39) - 1);

    use super::*;

    fn neg_to_u18(x: i32) -> u18
    {
        let tmp = i18::new(x);
        tmp.to_unsigned()
    }
    fn neg_to_u23(x: i32) -> u23
    {
        let tmp = arbitrary_int::i23::new(x);
        tmp.to_unsigned()
    }

    fn neg_to_u40(x: i64) -> u40
    {
        let tmp = arbitrary_int::i40::new(x);
        tmp.to_unsigned()
    }

    fn float_to_uint_form(x: f64, frac: i32) -> u40
    {
        let ret = u40::new((f64::abs(x) * 2.0_f64.powi(frac)).round() as u64);
        if x < 0.0
        {
            return u40::masked_new(!ret.value() + 1);
        }
        ret
    }

    fn upacked_to_int(x: String) -> u40
    {
        let y: Vec<u8> = x.split(" ").map(|s| s.parse::<u8>().unwrap()).collect();
        let mut or_sum: u64 = 0;
        for i in 0..=4
        {
            or_sum |= (y[i] as u64) << (32u64 - 8u64 * (i as u64));
        }
        u40::new(or_sum)
    }

    fn spacked_to_int(x: String) -> u40
    {
        let y: Vec<i8> = x.split(" ").map(|s| s.parse::<i8>().unwrap()).collect();
        let mut or_sum: u64 = 0;
        for i in 0..=4
        {
            or_sum |= (y[i] as u8 as u64) << (32u8 - 8u8 * (i as u8)) as u64;
        }
        u40::masked_new(or_sum)
    }

    // ---- unsigned integers ----
    #[test_case("0", UINT => Short(u18::new(0)) ; "uint zero")]
    #[test_case("40", UINT => Short(u18::new(40)) ; "uint 40")]
    #[test_case("262143", UINT => Short(u18::new(UIMM_MAX as u32)) ; "uint short max")]
    #[test_case("262144", UINT => Long(u23::new(262144)) ; "uint promotes to long")]
    // assumes LI zero-extends for unsigned types
    #[test_case("8388607", UINT => Long(u23::new(8388607)) ; "uint long max")]
    #[test_case("8388608", UINT => Full(u40::new(8388608)) ; "uint promotes to full")]
    #[test_case("1099511627775", UINT => Full(u40::new((1 << 40) - 1)) ; "uint full max")]
    #[test_case("40", UMOD => Short(u18::new(40)) ; "umod 40")]
    // ---- signed integers ----
    #[test_case("-40", SINT => Short(neg_to_u18(-40)) ; "sint neg 40")]
    #[test_case("-131072", SINT => Short(neg_to_u18(SIMM_MIN as i32)) ; "sint short min")]
    #[test_case("131072", SINT => Long(neg_to_u23(131072)) ; "sint promotes to long")]
    #[test_case("-40", SMOD => Short(neg_to_u18(-40)) ; "smod neg 40")]
    #[test_case("131071", SMOD => Short(neg_to_u18(SIMM_MAX as i32)) ; "smod short max")]
    #[test_case("131072", SMOD => Long(neg_to_u23(131072)) ; "smod promotes to long")]
    #[test_case("-131073", SMOD => Long(neg_to_u23(-131073)) ; "smod neg promotes to long")]
    #[test_case("4194303", SMOD => Long(neg_to_u23(LONG_SIMM_MAX as i32)) ; "smod long max")]
    #[test_case("-4194304", SMOD => Long(neg_to_u23(LONG_SIMM_MIN as i32)) ; "smod long min")]
    #[test_case("4194304", SMOD => Full(u40::new(4194304)) ; "smod promotes to full")]
    #[test_case("-4194305", SMOD => Full(neg_to_u40(-4194305)) ; "smod neg promotes to full")]
    #[test_case("1073741824", SMOD => Full(u40::new(1073741824)) ; "smod full")]
    #[test_case("549755813887", SMOD => Full(u40::new((1 << 39) - 1)) ; "smod full max")]
    #[test_case("-549755813888", SMOD => Full(neg_to_u40(-(1 << 39))) ; "smod full min")]
    // ---- fixed point (assumes Q formats are signed) ----
    #[test_case("54.632", Q15_24 => Full(u40::new(916572865)) ; "q15_24 pos")]
    #[test_case("-54.632", Q15_24 => Full(u40::new((1 << 40) - 916572865)) ; "q15_24 neg")]
    #[test_case("54.6326231", Q7_32 => Full(u40::new(234645329509)) ; "q7_32 pos")]
    #[test_case("-54.632", Q7_32 => Full(u40::new((1 << 40) - 234642653315)) ; "q7_32 neg")]
    #[test_case("54.6326231", Q23_16 => Long(u23::new(3580404)) ; "q23_16 fits long")]
    #[test_case("-54.6326231", Q23_16 => Long(neg_to_u23(-3580404)) ; "q23_16 neg fits long")]
    #[test_case("54.6326231", Q31_8 => Short(u18::new(13986)) ; "q31_8 fits short")]
    #[test_case("-54.6326231", Q31_8 => Short(neg_to_u18(-13986)) ; "q31_8 neg fits short")]
    #[test_case("600.0", Q31_8 => Long(u23::new(153600)) ; "q31_8 past signed short")]
    // ---- packed / bool ----
    #[test_case("5 4 3 2 1", UPACKED => Full(u40::new(0x05_04_03_02_01)) ; "upacked")]
    #[test_case("-5 -4 -3 -2 -1", SPACKED => Full(u40::new(0xFB_FC_FD_FE_FF)) ; "spacked")]
    #[test_case("TRUE", BOOL => Short(u18::new(1)) ; "bool true")]
    #[test_case("FALSE", BOOL => Short(u18::new(0)) ; "bool false")]
    fn process_imm_accepts(input: &str, label: ImmTypeLabel) -> ImmVal
    {
        process_imm(input, label).unwrap()
    }

    // ---- must be rejected ----
    #[test_case("", UINT ; "empty")]
    #[test_case("abc", UINT ; "not a number")]
    #[test_case("54.6.3", Q15_24 ; "malformed float")]
    #[test_case("MAYBE", BOOL ; "bad bool")]
    #[test_case("1099511627776", UINT ; "uint exceeds 40 bits")]
    // ---- must be rejected: assumptions, verify ----
    #[test_case("-40", UINT ; "uint rejects negative")]
    #[test_case("549755813888", SMOD ; "smod exceeds 40 bit signed")]
    #[test_case("200.0", Q7_32 ; "q7_32 out of range")]
    #[test_case("5 4 3 2 256", UPACKED ; "upacked lane overflow")]
    #[test_case("5 4 3 2", UPACKED ; "upacked too few lanes")]
    fn process_imm_rejects(input: &str, label: ImmTypeLabel)
    {
        let r = process_imm(input, label);
        assert!(r.is_err(), "expected Err, got {r:?}");
    }

    #[test]
    fn test_assemlber()
    {
        let tests_that_should_work =
            [(("ADD", "R3", "R2", "R1"), 0), (("SUB", "R0", "R0", "R0"), 0), (("XOR", "R31", "R18", "R15"), 0)];
    }
}
