pub enum VarArgRet
{
    One(Word),
    Two((Word, Word)),
    Three((Word, Word, Word)),
}

use std::ops::RangeInclusive;

use arbitrary_int::{traits::Integer, u18, u23, u40};
use brot_isa::*;
use isa::{self as brot_isa};
//Takes an imm_str and its type and generates its embedded form in a u18.
pub fn process_imm(imm_str: &str, imm_type: ImmTypeLabel) -> Result<ImmVal, String>
{
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

    if SIGNED_ARIMETHIC_TYPES_IMM.contains(&imm_type)
        && !(imm_type == ImmTypeLabel::UPACKED || imm_type == ImmTypeLabel::SPACKED)
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
pub fn embed_tag_opcode_from_struct(info_struct: &InstructionInfo) -> Result<Word, String>
{
    let mut ret: Word = Word { tag: TypeLabel::INSTRUCTION, data: u40::new(0) };
    ret.data |= u40::new(info_struct.label.into()) << START_OPCODE_FIELD;
    Ok(ret)
}

pub fn embed_tag_opcode_from_label(label: InstructionLabel) -> Result<Word, String>
{
    let mut ret: Word = Word { tag: TypeLabel::INSTRUCTION, data: u40::new(0) };
    ret.data |= u40::new(label.into()) << START_OPCODE_FIELD;
    Ok(ret)
}

pub fn assemble_r_type(info_struct: &InstructionInfo) -> Result<VarArgRet, String>
{
    let mut data_word_encoding: Word = embed_tag_opcode_from_struct(info_struct)?;

    if !info_struct.valid_operand_counts.contains(&(info_struct.get_arg_count()))
    {
        return Err(format!(
            "{} did not tokenize properly, incorrect number of args. Supposed to be 3 got {} ",
            <&str>::from(info_struct.label),
            info_struct.get_arg_count() != 3
        ));
    }

    data_word_encoding.data =
        u40::new(insert_bits(data_word_encoding.data.value(), info_struct.get_rd_index()?, RD_END, RD_START));

    data_word_encoding.data =
        u40::new(insert_bits(data_word_encoding.data.value(), info_struct.get_rs1_index()?, RS1_END, RS1_START));

    if info_struct.get_arg_count() == 3
    {
        data_word_encoding.data =
            u40::new(insert_bits(data_word_encoding.data.value(), info_struct.get_rs2_index()?, RS2_END, RS2_START));
    }

    Ok(VarArgRet::One(data_word_encoding))
}

pub fn assemble_i_type(info_struct: &InstructionInfo) -> Result<VarArgRet, String>
{
    let mut data_word_encoding: Word = embed_tag_opcode_from_struct(info_struct)?;

    data_word_encoding.data =
        u40::new(insert_bits(data_word_encoding.data.value(), info_struct.get_rd_index()?, RD_END, RD_START));
    data_word_encoding.data =
        u40::new(insert_bits(data_word_encoding.data.value(), info_struct.get_rs1_index()?, RS1_END, RS1_START));

    use brot_isa::ImmVal::*;
    use brot_isa::InstructionLabel::*;

    if info_struct.label == LI
    {
        match info_struct.imm_arg.imm_val
        {
            Short(uint) =>
            {
                const LONG_SHORT_SIZE_DIFF: u64 = 5;

                let mut imm_bin: u64 = uint.value() as u64;
                //Sign extend
                if SIGNED_ARIMETHIC_TYPES_IMM.contains(&info_struct.imm_arg.datatype)
                {
                    imm_bin = (((imm_bin << LONG_SHORT_SIZE_DIFF) as i64) >> LONG_SHORT_SIZE_DIFF) as u64;
                }
                imm_bin |= <u64>::from(info_struct.imm_arg.datatype) << LONG_IMM_TAG_START;

                data_word_encoding.data =
                    u40::new(insert_bits(data_word_encoding.data.value(), imm_bin, LONG_IMM_END, IMM_START));
                Ok(VarArgRet::One(data_word_encoding))
            }
            Long(uint) =>
            {
                let imm_bin: u64 =
                    (uint.value() as u64) | <u64>::from(info_struct.imm_arg.datatype) << LONG_IMM_TAG_START;
                data_word_encoding.data =
                    u40::new(insert_bits(data_word_encoding.data.value(), imm_bin, LONG_IMM_END, IMM_START));
                Ok(VarArgRet::One(data_word_encoding))
            }
            Full(uint) =>
            {
                const UPPER_HALF_END: u64 = 39;
                const UPPER_HALF_START: u64 = 23;

                let imm_bin_lower = extract_bits(uint.value(), LONG_IMM_VAL_END, IMM_START)
                    | <u64>::from(info_struct.imm_arg.datatype) << LONG_IMM_TAG_START;
                let imm_bin_upper = extract_bits(uint.value(), UPPER_HALF_END, UPPER_HALF_START)
                    | <u64>::from(info_struct.imm_arg.datatype) << LONG_IMM_TAG_START;

                let mut li: Word = embed_tag_opcode_from_label(LI)?;
                let mut lui: Word = embed_tag_opcode_from_label(LUI)?;

                li.data = u40::new(insert_bits(li.data.value(), info_struct.get_rd_index()?, RD_END, RD_START));

                lui.data = u40::new(insert_bits(lui.data.value(), info_struct.get_rd_index()?, RD_END, RD_START));

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
                const LONG_SHORT_SIZE_DIFF: u64 = 5;
                //Sign extend
                if SIGNED_ARIMETHIC_TYPES_IMM.contains(&info_struct.imm_arg.datatype)
                {
                    imm_bin = (((imm_bin << LONG_SHORT_SIZE_DIFF) as i64) >> LONG_SHORT_SIZE_DIFF) as u64;
                }
                imm_bin |= <u64>::from(info_struct.imm_arg.datatype) << LONG_IMM_TAG_START;

                data_word_encoding.data =
                    u40::new(insert_bits(data_word_encoding.data.value(), imm_bin, LONG_IMM_END, IMM_START));
                Ok(VarArgRet::One(data_word_encoding))
            }
            Long(uint) =>
            {
                let imm_bin: u64 =
                    (uint.value() as u64) | (<u64>::from(info_struct.imm_arg.datatype) << LONG_IMM_TAG_START);
                data_word_encoding.data =
                    u40::new(insert_bits(data_word_encoding.data.value(), imm_bin, LONG_IMM_END, IMM_START));
                Ok(VarArgRet::One(data_word_encoding))
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
                    info_struct.get_rs1_index()?,
                    RS1_END,
                    RS1_START,
                ));

                data_word_encoding.data =
                    u40::new(insert_bits(data_word_encoding.data.value(), imm_bin, IMM_END, IMM_START));

                Ok(VarArgRet::One(data_word_encoding))
            }
            Long(uint) =>
            {
                if info_struct.get_rd_index()? == info_struct.get_rs1_index()?
                {
                    return Err(format!(
                        "When destination and source registers are equal (R{} = R{} ) it cannot be broken into a LI/LUI pair. Please load into a register before hand and use an R-type instruction.",
                        info_struct.get_rd_index()?,
                        info_struct.get_rs1_index()?
                    ));
                }

                let imm_bin: u64 =
                    (uint.value() as u64) | (<u64>::from(info_struct.imm_arg.datatype) << LONG_IMM_TAG_START);

                let mut li: Word = embed_tag_opcode_from_label(LI)?;

                li.data = u40::new(insert_bits(li.data.value(), info_struct.get_rd_index()?, RD_END, RD_START));

                li.data = u40::new(insert_bits(li.data.value(), imm_bin, LONG_IMM_END, IMM_START));

                let mut r_word: Word = embed_tag_opcode_from_label(itype_to_rtype_equivalent(info_struct.label)?)?;

                r_word.data = u40::new(insert_bits(r_word.data.value(), info_struct.get_rd_index()?, RD_END, RD_START));

                r_word.data =
                    u40::new(insert_bits(r_word.data.value(), info_struct.get_rs1_index()?, RS1_END, RS1_START));

                r_word.data =
                    u40::new(insert_bits(r_word.data.value(), info_struct.get_rs2_index()?, RS2_END, RS2_START));

                Ok(VarArgRet::Two((li, r_word)))
            }
            Full(uint) =>
            {
                const UPPER_HALF_END: u64 = 39;
                const UPPER_HALF_START: u64 = 23;

                if info_struct.get_rs1_index()? == info_struct.get_rs2_index()?
                {
                    return Err(format!(
                        "When destination and source registers are equal (R{} = R{} ) it cannot be broken into a LI/LUI pair. Please load into a register before hand and use an R-type instruction.",
                        info_struct.get_rd_index()?,
                        info_struct.get_rs1_index()?
                    ));
                }

                let imm_bin_lower = extract_bits(uint.value(), LONG_IMM_VAL_END, IMM_START)
                    | <u64>::from(info_struct.imm_arg.datatype) << LONG_IMM_TAG_START;

                let imm_bin_upper = extract_bits(uint.value(), UPPER_HALF_END, UPPER_HALF_START)
                    | <u64>::from(info_struct.imm_arg.datatype) << LONG_IMM_TAG_START;

                let mut li: Word = embed_tag_opcode_from_label(LI)?;
                let mut lui: Word = embed_tag_opcode_from_label(LUI)?;

                li.data = u40::new(insert_bits(li.data.value(), info_struct.get_rd_index()?, RD_END, RD_START));

                lui.data = u40::new(insert_bits(lui.data.value(), info_struct.get_rd_index()?, RD_END, RD_START));

                li.data = u40::new(insert_bits(li.data.value(), imm_bin_lower, LONG_IMM_END, IMM_START));

                lui.data = u40::new(insert_bits(lui.data.value(), imm_bin_upper, LONG_IMM_END, LONG_IMM_START));

                let mut r_word: Word = embed_tag_opcode_from_label(itype_to_rtype_equivalent(info_struct.label)?)?;

                r_word.data = u40::new(insert_bits(r_word.data.value(), info_struct.get_rd_index()?, RD_END, RD_START));

                r_word.data =
                    u40::new(insert_bits(r_word.data.value(), info_struct.get_rs1_index()?, RS1_END, RS1_START));

                r_word.data =
                    u40::new(insert_bits(r_word.data.value(), info_struct.get_rs2_index()?, RS2_END, RS2_START));

                Ok(VarArgRet::Three((li, lui, r_word)))
            }
        }
    }
}

pub fn assemble_c_type(_instruction_info_struct: &InstructionInfo) -> Result<VarArgRet, String>
{
    Ok(VarArgRet::One(Word { tag: TypeLabel::FREE, data: u40::new(0) }))
}

pub fn assemble_s_type(_instruction_info_struct: &InstructionInfo) -> Result<VarArgRet, String>
{
    Ok(VarArgRet::One(Word { tag: TypeLabel::FREE, data: u40::new(0) }))
}
pub fn assemble_b_type(_instruction_info_struct: &InstructionInfo) -> Result<VarArgRet, String>
{
    Ok(VarArgRet::One(Word { tag: TypeLabel::FREE, data: u40::new(0) }))
}
pub fn assemble_u_type(_instruction_info_struct: &InstructionInfo) -> Result<VarArgRet, String>
{
    Ok(VarArgRet::One(Word { tag: TypeLabel::FREE, data: u40::new(0) }))
}
pub fn assemble_j_type(_instruction_info_struct: &InstructionInfo) -> Result<VarArgRet, String>
{
    Ok(VarArgRet::One(Word { tag: TypeLabel::FREE, data: u40::new(0) }))
}
pub fn assemble_l_type(_instruction_info_struct: &InstructionInfo) -> Result<VarArgRet, String>
{
    Ok(VarArgRet::One(Word { tag: TypeLabel::FREE, data: u40::new(0) }))
}
pub fn assemble_p_type(_instruction_info_struct: &InstructionInfo) -> Result<VarArgRet, String>
{
    Ok(VarArgRet::One(Word { tag: TypeLabel::FREE, data: u40::new(0) }))
}
pub fn assemble_q_type(_instruction_info_struct: &InstructionInfo) -> Result<VarArgRet, String>
{
    Ok(VarArgRet::One(Word { tag: TypeLabel::FREE, data: u40::new(0) }))
}

pub fn assemble(instruction: &str) -> Result<VarArgRet, String>
{
    use isa::InstructionGroup::*;
    let instruction_info_struct: InstructionInfo = super::tokenize_instruction(instruction)?;

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

    let r = instruction_bin?;
    Ok(r)
}

pub fn assemble_all(instruction_vector: &Vec<(String, usize)>) -> Result<Vec<String>, Vec<(String, usize)>>
{
    let mut output_buffer: Vec<String> = Vec::with_capacity(instruction_vector.len());
    let mut error_vector: Vec<(String, usize)> = Vec::new();

    for (instruction_string, curr_line_num) in instruction_vector
    {
        let instruction_string_comment = format!("{}{}", r"\\", instruction_string);

        // {instruction_string_comment:12} <
        let instruction_bin = match assemble(instruction_string)
        {
            Ok(r) => r,
            Err(e) =>
            {
                error_vector.push((e, *curr_line_num));
                continue;
            }
        };

        match instruction_bin
        {
            VarArgRet::One(word) => output_buffer.push(format!("{:012X}", word.to_u64())),
            VarArgRet::Two((word1, word2)) =>
            {
                output_buffer.push(format!("{:012X} LI Expanded", word1.to_u64()));
                output_buffer.push(format!("{:012X} R-Type Equivalent", word2.to_u64()));
            }
            VarArgRet::Three((word1, word2, word3)) =>
            {
                output_buffer.push(format!("{:012X} Original LI Expanded term", word1.to_u64()));
                output_buffer.push(format!("{:012X} <- Original LUI Expanded", word2.to_u64()));
                output_buffer.push(format!("{:012X} <- R-Type Equivalent", word2.to_u64()));
            }
        }
    }

    if !error_vector.is_empty()
    {
        return Err(error_vector);
    }
    Ok(output_buffer)
}
