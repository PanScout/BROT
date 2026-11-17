pub enum VarArgRet
{
    One(Word),
    Two((Word, Word)),
    Three((Word, Word, Word)),
}

use arbitrary_int::{u5, u23, u40};
use brot_isa::*;
use isa::{self as brot_isa};
use isa::constants::*;
use isa::utils::*;

pub fn assemble_r_type(tokens: &TokenStruct) -> Result<VarArgRet, String>
{
    let mut data_word_encoding = Word::default();

    match tokens.operands
    {
        Operands::RType3 { rd, rs1, rs2 } => data_word_encoding.construct_rtype_3(tokens.label, rd, rs1, rs2),
        Operands::RType2 { rd, rs1 } => data_word_encoding.construct_rtype_2(tokens.label, rd, rs1),
        _ => panic!("Non R-type entered into R type field"),
    }

    Ok(VarArgRet::One(data_word_encoding))
}

pub fn assemble_i_type(tokens: &TokenStruct) -> Result<VarArgRet, String>
{
    // Split a 40-bit immediate into LI (bits 0–22) and LUI (bits 23–39) words
    let split_full = |rd: u5, full: u40, datatype: ImmTypeLabel| -> (Word, Word) {
        let lower = Imm {
            imm_val: ImmVal::Long(u23::new(
                extract_bits(full, LONG_IMM_VAL_FIELD).try_into().unwrap(),
            )),
            datatype,
        };
        let upper_range = BitRange::new(39, 23);
        let upper =
            Imm { imm_val: ImmVal::Long(u23::new(extract_bits(full, upper_range).try_into().unwrap())), datatype };
        let mut li = Word::default();
        let mut lui = Word::default();
        li.construct_itype_2(InstructionLabel::LI, rd, lower);
        lui.construct_itype_2(InstructionLabel::LUI, rd, upper);
        (li, lui)
    };

    match tokens.operands
    {
        // LI / LUI
        Operands::IType2 { rd, imm } => match imm.imm_val
        {
            ImmVal::Full(full) =>
            {
                if tokens.label == InstructionLabel::LUI
                {
                    return Err("Can't expand LUI value into two instructions. Please use LI instead.".to_string());
                }
                let (li, lui) = split_full(rd, full, imm.datatype);
                Ok(VarArgRet::Two((li, lui)))
            }
            _others =>
            {
                let mut word = Word::default();
                word.construct_itype_2(tokens.label, rd, imm);
                Ok(VarArgRet::One(word))
            }
        },

        // ADDI, XORI, etc.
        Operands::IType3 { rd, rs1, imm } =>
        {
            if let ImmVal::Short(_) = imm.imm_val
            {
                let mut word = Word::default();
                word.construct_itype_3(tokens.label, rd, rs1, imm);
                return Ok(VarArgRet::One(word));
            }

            // Immediate is loaded into rd first, so rd == rs1 would clobber the source
            if rd == rs1
            {
                return Err(format!(
                    "When destination and source registers are equal (R{rd} = R{rs1}) it cannot be broken into a LI/LUI pair. Please load into a register before hand and use an R-type instruction."
                ));
            }

            // rd = rs1 op rd, where rd now holds the immediate
            let mut r_word = Word::default();
            r_word.construct_rtype_3(tokens.label.itype_to_rtype_equivalent()?, rd, rs1, rd);

            if let ImmVal::Full(full) = imm.imm_val
            {
                let (li, lui) = split_full(rd, full, imm.datatype);
                Ok(VarArgRet::Three((li, lui, r_word)))
            }
            else
            {
                let mut li = Word::default();
                li.construct_itype_2(InstructionLabel::LI, rd, imm);
                Ok(VarArgRet::Two((li, r_word)))
            }
        }

        _others => Err("Non I-type entered into I-type assembler".to_string()),
    }
}

pub fn assemble_c_type(_instruction_tokens: &TokenStruct) -> Result<VarArgRet, String>
{
    let mut word = Word::default();
    word.tag = TypeLabel::INSTRUCTION;
    Ok(VarArgRet::One(word))
}

pub fn assemble_s_type(_instruction_tokens: &TokenStruct) -> Result<VarArgRet, String>
{
    let mut word = Word::default();
    word.tag = TypeLabel::INSTRUCTION;
    Ok(VarArgRet::One(word))
}

pub fn assemble_b_type(_instruction_tokens: &TokenStruct) -> Result<VarArgRet, String>
{
    let mut word = Word::default();
    word.tag = TypeLabel::INSTRUCTION;
    Ok(VarArgRet::One(word))
}

pub fn assemble_u_type(_instruction_tokens: &TokenStruct) -> Result<VarArgRet, String>
{
    let mut word = Word::default();
    word.tag = TypeLabel::INSTRUCTION;
    Ok(VarArgRet::One(word))
}

pub fn assemble_j_type(_instruction_tokens: &TokenStruct) -> Result<VarArgRet, String>
{
    let mut word = Word::default();
    word.tag = TypeLabel::INSTRUCTION;
    Ok(VarArgRet::One(word))
}

pub fn assemble_l_type(_instruction_tokens: &TokenStruct) -> Result<VarArgRet, String>
{
    let mut word = Word::default();
    word.tag = TypeLabel::INSTRUCTION;
    Ok(VarArgRet::One(word))
}

pub fn assemble_p_type(_instruction_tokens: &TokenStruct) -> Result<VarArgRet, String>
{
    let mut word = Word::default();
    word.tag = TypeLabel::INSTRUCTION;
    Ok(VarArgRet::One(word))
}

pub fn assemble_q_type(_instruction_tokens: &TokenStruct) -> Result<VarArgRet, String>
{
    let mut word = Word::default();
    word.tag = TypeLabel::INSTRUCTION;
    Ok(VarArgRet::One(word))
}

pub fn assemble(instruction: &str) -> Result<VarArgRet, String>
{
    use isa::InstructionGroup::*;
    let tokens: TokenStruct = super::tokenize_instruction(instruction)?;

    let instruction_bin = match tokens.group
    {
        RTYPE => assemble_r_type(&tokens),
        CTYPE => assemble_c_type(&tokens),
        ITYPE => assemble_i_type(&tokens),
        STYPE => assemble_s_type(&tokens),
        BTYPE => assemble_b_type(&tokens),
        UTYPE => assemble_u_type(&tokens),
        JTYPE => assemble_j_type(&tokens),
        LTYPE => assemble_l_type(&tokens),
        PTYPE => assemble_p_type(&tokens),
        QTYPE => assemble_q_type(&tokens),
    };

    let r = instruction_bin?;
    Ok(r)
}

//todo need to do a two pass. first pass assembles all instructions with immediates including branches,
//todo 2nd pass needs to resolve branch labels.
pub fn assemble_all(instruction_vector: &Vec<(String, usize)>) -> Result<Vec<String>, Vec<(String, usize)>>
{
    let mut output_buffer: Vec<String> = Vec::with_capacity(instruction_vector.len());
    let mut error_vector: Vec<(String, usize)> = vec![];

    for (instruction_string, curr_line_num) in instruction_vector
    {
        let instruction_string_comment = format!("{}{}", r"\\", instruction_string);

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
            VarArgRet::One(word) =>
            {
                output_buffer.push(format!("{:012X} \\\\{instruction_string_comment}", word.to_u64()))
            }
            VarArgRet::Two((word1, word2)) =>
            {
                output_buffer.push(format!("{:012X} LI Expanded", word1.to_u64()));
                output_buffer
                    .push(format!("{:012X} R-Type Equivalent of {instruction_string_comment}", word2.to_u64()));
            }
            VarArgRet::Three((word1, word2, word3)) =>
            {
                output_buffer.push(format!("{:012X} Original LI Expanded term", word1.to_u64()));
                output_buffer.push(format!("{:012X} <- Original LUI Expanded", word2.to_u64()));
                output_buffer
                    .push(format!("{:012X} <- R-Type Equivalent of {instruction_string_comment}", word3.to_u64()));
            }
        }
    }

    if !error_vector.is_empty()
    {
        return Err(error_vector);
    }
    Ok(output_buffer)
}
