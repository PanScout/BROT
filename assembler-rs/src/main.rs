use std::path::PathBuf;

use clap::Parser;

use brot_isa::*;
use isa::{
    self as brot_isa,
    ImmTypeLabel::{SPACKED, UPACKED},
    ImmVal,
    ImmVal::Long,
    InstructionGroup::ITYPE,
    InstructionLabel::LI,
};

use arbitrary_int::{traits::Integer, u23, u40};

//Take a path buffer and return a list of instructions paired with its original line numbers.
//TODO Implement includes and recursively resolve them to create the one big vector
fn clean_input(input: &PathBuf) -> Result<Vec<(String, usize)>, String>
{
    let mut ret: Vec<(String, usize)> = Vec::new();

    let file_as_string: String = std::fs::read_to_string(input)
        .map_err(|e| format!("Could not read input \"{}\" into string: {e}", input.display()))?;

    //Grab each line, get rid of stuff after semicolon and remove white space
    for (i, line_string) in file_as_string.lines().enumerate()
    {
        let line_string = line_string.trim();
        let line_number = i + 1;
        if line_string.is_empty() || line_string.chars().all(char::is_whitespace)
        {
            continue;
        }

        if !line_string.is_ascii()
        {
            return Err(format!("Non ASCII at line {line_number}"));
        }

        //Comments
        if line_string.starts_with(r"\\") || line_string.starts_with("//")
        {
            continue;
        }

        //Include
        if line_string.starts_with('#')
        {
            return Err(format!("Line {line_number}: includes not implemented yet"));
        }

        let code = if let Some(idx) = line_string.find(';')
        {
            &line_string[..idx].trim()
        }
        else
        {
            if !line_string.contains("//")
            {
                return Err(format!(
                    "Skipping line {line_number}; might be an instruction are you sure?\n \"{line_string}\"\nMake sure to end instruction lines with a \";\" otherwise"
                ));
            }
            continue;
        };

        if !code.is_empty()
        {
            ret.push((code.to_string(), line_number));
        }
    }

    if ret.is_empty()
    {
        return Err("File contains no instructions.".to_string());
    }

    Ok(ret)
}

//Break the string form of an instruction into a struct of enums and ints.
fn tokenize_instruction(instruction: &str, curr_line_num: usize) -> Result<InstructionInfo, String>
{
    const VALID_IMM_ARG_COUNTS: [usize; 2] = [2, 6];

    let white_space_index: usize = match instruction.find(char::is_whitespace)
    {
        Some(r) => r,
        None => return Err("\nCould not find whitespace. Garbled Instruction".to_string()),
    };

    let instruction_label: String = instruction[0..white_space_index].to_string().to_ascii_uppercase();

    let instruction_label: InstructionLabel = instruction_label
        .parse()
        .map_err(|e| format!("\nCould not map instruction to instruction label due to {e}"))?;

    let mut info_struct = InstructionInfo::label_to_info(instruction_label, curr_line_num)
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
        let op_str = op.trim().to_string().to_ascii_uppercase();

        if op_str.starts_with('R')
        {
            let op_num = op_str
                .strip_prefix('R')
                .ok_or("Register not prefixed with 'r'")?
                .parse::<u8>()
                .map_err(|e| format!("\nCould not parse register into u8. Out of bounds?\nError{e}"))?;
            info_struct.register_args[op_index] =
                RegisterIndexArg { index: op_num, is_pointer_reg: false, is_valid: true };
        }
        else if op_str.starts_with('P')
        {
            let op_num = op_str
                .strip_prefix('P')
                .ok_or("Register not prefixed with 'p'")?
                .parse::<u8>()
                .map_err(|e| format!("\nCould not parse register into u8. Out of bounds?\nError{e}"))?;
            info_struct.register_args[op_index] =
                RegisterIndexArg { index: op_num, is_pointer_reg: true, is_valid: true };
        }
        else if op_str.contains(':')
        {
            if op_index != operands.len() - 1
            {
                return Err("Only the last instruction should have \":\" ".to_string());
            }

            let imm_spilt: Vec<String> = op_str.split(':').map(|s| s.to_string()).collect();

            if !VALID_IMM_ARG_COUNTS.contains(&imm_spilt.len())
            {
                return Err(format!(
                    "\nParsed immediate argument and found invalid argument count: {}",
                    imm_spilt.len()
                ));
            }

            let imm_val_str: Vec<String> = imm_spilt[0..imm_spilt.len() - 1]
                .to_vec()
                .iter()
                .map(|s| s.to_ascii_uppercase().trim().to_string())
                .collect();

            let imm_type: ImmTypeLabel =
                ImmTypeLabel::try_from(imm_spilt[imm_spilt.len() - 1].as_str()).map_err(|e| e.to_string())?;

            if imm_type == UPACKED || imm_type == SPACKED
            {
                return Err("Packed immediates are not allowed!".to_string());
            }
            else
            {
                if (info_struct.register_args[1].index == 1 && !info_struct.register_args[1].is_pointer_reg)
                    && imm_type != ImmTypeLabel::UINT
                {
                    return Err(
                        "\nYou are trying to use an immediate instruction with GPR0 that is not UINT".to_string()
                    );
                }

                let imm_bin: ImmVal = match process_imm(&imm_val_str[0], imm_type, instruction_label == LI)
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
    }

    if info_struct.register_args[0].index > MAX_GPR_VAL as u8
        || info_struct.register_args[1].index > MAX_GPR_VAL as u8
        || info_struct.register_args[1].index > MAX_GPR_VAL as u8
    {
        return Err("\nParsed a register value beyond the max range.".to_string());
    }
    if info_struct.group != ITYPE
    {
        let number_of_valid_args = info_struct.register_args.clone().iter().filter(|x| x.is_valid).count();
        if !info_struct.valid_operand_counts.contains(&number_of_valid_args)
        {
            return Err("\nNot a vaild number of operands".to_string());
        }
    }
    else
    {
        let mut number_of_valid_args = info_struct.register_args.clone().iter().filter(|x| x.is_valid).count();
        if info_struct.imm_arg.valid
        {
            number_of_valid_args += 1;
        }

        if !info_struct.valid_operand_counts.contains(&number_of_valid_args)
        {
            return Err("\nNot a vaild number of operands".to_string());
        }
    }

    if info_struct.group == ITYPE && !info_struct.imm_arg.valid
    {
        return Err("\nWas an I-type instruction but didn't find any immediate arguments".to_string());
    }

    info_struct.line_num = curr_line_num;
    Ok(info_struct)
}

//Takes an imm_str and its type and generates its embedded form in a u18.
fn process_imm(imm_str: &str, imm_type: ImmTypeLabel, is_long: bool) -> Result<ImmVal, String>
{
    use brot_isa::*;

    let imm_str = &imm_str.to_ascii_uppercase();

    if !ARIMETHIC_TYPES_IMM.contains(&imm_type)
    {
        return Err(format!("\nCannot process immediate type {} is not an aritmethic type", <&str>::from(imm_type)));
    }

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
        ImmTypeLabel::SPACKED => return Err("Signed Packed not supported for immediates".to_owned()),
        ImmTypeLabel::UPACKED => return Err("Signed Packed not supported for immediates".to_owned()),
    };

    if is_long
    {
        if SIGNED_ARIMETHIC_TYPES_IMM.contains(&imm_type)
        {
            if !((LONG_SIMM_MIN..=LONG_SIMM_MAX).contains(&(val as i64)))
            {
                return Err(format!(
                    "Error immediate {} does not fit into long immediate. Max is {}",
                    val as i64,
                    (LONG_SIMM_MIN..=LONG_SIMM_MAX).max().unwrap()
                ));
            }

            Ok(Long(u23::masked_new(val)))
        }
        else
        {
            if val > LONG_UIMM_MAX
            {
                return Err(format!(
                    "Error immediate {} does not fit into long immediate. Max is {}",
                    val, LONG_UIMM_MAX
                ));
            }
            Ok(Long(u23::masked_new(val)))
        }
    }
    else
    {
        if SIGNED_ARIMETHIC_TYPES_IMM.contains(&imm_type)
        {
            if !(SIMM_MIN..=SIMM_MAX).contains(&(val as i64))
            {
                return Err(format!("Error immediate {val} does not fit into short immediate"));
            }
            Ok(Long(u23::masked_new(val)))
        }
        else
        {
            if UIMM_MAX < val
            {
                return Err(format!("Error immediate {val} does not fit into short immediate"));
            }
            Ok(Long(u23::masked_new(val)))
        }
    }
}
fn embed_tag_opcode(info_struct: &InstructionInfo) -> Result<Word, String>
{
    let mut ret: Word = Word { tag: TypeLabel::INSTRUCTION, data: u40::new(0) };
    ret.data |= u40::new(info_struct.label.into()) << START_OPCODE_FIELD;
    Ok(ret)
}

fn assemble_r_type(info_struct: &InstructionInfo) -> Result<Word, String>
{
    let mut data_word_encoding: Word = embed_tag_opcode(info_struct)?;

    if !info_struct.register_args[0].is_valid
    {
        return Err("RD register argument is not valid".to_string());
    }

    data_word_encoding.data = u40::new(insert_bits(
        data_word_encoding.data.value(),
        info_struct.register_args[0].index as u64,
        RD_END,
        RD_START,
    ));

    if !info_struct.register_args[1].is_valid
    {
        return Err("RS1 register argument is not valid".to_string());
    }

    data_word_encoding.data = u40::new(insert_bits(
        data_word_encoding.data.value(),
        info_struct.register_args[1].index as u64,
        RS1_END,
        RS1_START,
    ));

    if !info_struct.register_args[2].is_valid
    {
        return Err("RS2 register argument is not valid".to_string());
    }

    data_word_encoding.data = u40::new(insert_bits(
        data_word_encoding.data.value(),
        info_struct.register_args[2].index as u64,
        RS2_END,
        RS2_START,
    ));

    Ok(data_word_encoding)
}

fn assemble_i_type(info_struct: &InstructionInfo) -> Result<Word, String>
{
    let mut data_word_encoding: Word = embed_tag_opcode(info_struct)?;

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
        info_struct.imm_arg.to_u23().value() as u64,
        IMM_END,
        IMM_START,
    ));

    Ok(data_word_encoding)
}

fn assemble_c_type(_instruction_info_struct: &InstructionInfo) -> Result<Word, String>
{
    Ok(Word { tag: TypeLabel::FREE, data: u40::new(0) })
}

fn assemble_s_type(_instruction_info_struct: &InstructionInfo) -> Result<Word, String>
{
    Ok(Word { tag: TypeLabel::FREE, data: u40::new(0) })
}
fn assemble_b_type(_instruction_info_struct: &InstructionInfo) -> Result<Word, String>
{
    Ok(Word { tag: TypeLabel::FREE, data: u40::new(0) })
}
fn assemble_u_type(_instruction_info_struct: &InstructionInfo) -> Result<Word, String>
{
    Ok(Word { tag: TypeLabel::FREE, data: u40::new(0) })
}
fn assemble_j_type(_instruction_info_struct: &InstructionInfo) -> Result<Word, String>
{
    Ok(Word { tag: TypeLabel::FREE, data: u40::new(0) })
}
fn assemble_l_type(_instruction_info_struct: &InstructionInfo) -> Result<Word, String>
{
    Ok(Word { tag: TypeLabel::FREE, data: u40::new(0) })
}
fn assemble_p_type(_instruction_info_struct: &InstructionInfo) -> Result<Word, String>
{
    Ok(Word { tag: TypeLabel::FREE, data: u40::new(0) })
}
fn assemble_q_type(_instruction_info_struct: &InstructionInfo) -> Result<Word, String>
{
    Ok(Word { tag: TypeLabel::FREE, data: u40::new(0) })
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
            Ok(r) =>
            {
                output_buffer.push(format!("{:012X} {instruction_string_comment:12}", r.to_u64()));
            }
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
    use arbitrary_int::{i18, traits::Integer, u18};
    use isa::ImmVal::Short;

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

    #[test]
    fn test_process_imm()
    {
        let cases = [
            ("40", ImmTypeLabel::UINT, Short(u18::new(40))),
            ("40", ImmTypeLabel::UMOD, Short(u18::new(40))),
            ("-40", ImmTypeLabel::SINT, Short(neg_to_u18(-40))),
            ("-40", ImmTypeLabel::SMOD, Short(neg_to_u18(-40))),
            ("131071", ImmTypeLabel::SMOD, Short(neg_to_u18(SIMM_MAX as i32))),
            ("-131072", ImmTypeLabel::SINT, Short(neg_to_u18(SIMM_MIN as i32))),
            ("262143", ImmTypeLabel::UINT, Short(u18::new(UIMM_MAX as u32))),
            ("40", ImmTypeLabel::UINT, Long(u23::new(40))),
            ("40", ImmTypeLabel::UMOD, Long(u23::new(40))),
            ("-40", ImmTypeLabel::SINT, Long(neg_to_u23(-40))),
            ("-40", ImmTypeLabel::SMOD, Long(neg_to_u23(-40))),
            ("4194303", ImmTypeLabel::SMOD, Long(neg_to_u23(LONG_SIMM_MAX as i32))),
            ("-4194304", ImmTypeLabel::SMOD, Long(neg_to_u23(LONG_SIMM_MIN as i32))),
        ];

        let mut failure_count = 0;
        let total_tests = cases.len();

        for (input_str, label, expected) in cases
        {
            if matches!(expected, Long(x))
            {
                if process_imm(input_str, label, true).unwrap_or_else(|e| panic!("input {input_str:?} failed: {e}"))
                    != expected
                {
                    failure_count += 1;
                }
            }
            else
            {
                if process_imm(input_str, label, false).unwrap_or_else(|e| panic!("input {input_str:?} failed: {e}"))
                    != expected
                {
                    failure_count += 1;
                }
            }
        }
    }
    #[test]
    fn test_assemlber()
    {
        let tests_that_should_work =
            [(("ADD", "R3", "R2", "R1"), 0), (("SUB", "R0", "R0", "R0"), 0), (("XOR", "R31", "R18", "R15"), 0)];
    }
}
