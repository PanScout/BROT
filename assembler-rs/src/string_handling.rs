//Take a path buffer and return a list of instructions paired with its original line numbers.
//TODO Implement includes and recursively resolve them to create the one big vector

use isa::{
    ImmTypeLabel::{self, UINT}, ImmVal, InstructionGroup::ITYPE, InstructionInfo, InstructionLabel::{self, NOT}, MAX_GPR_VAL, RegisterIndexArg, RegisterType, sign_extend_short,
};

use crate::assemble::process_imm;

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
        .filter(|(s, l)| !s.is_empty() || !(s.chars().all(char::is_whitespace)))
        .filter(|(s, l)| s != ";")
        .collect();

    if file_as_vec.is_empty()
    {
        return Err("File is empty or just comments!".to_string());
    }

    if let Some(idx) = file_as_vec.iter().find(|(s, l)| !s.is_ascii())
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
pub fn tokenize_instruction(instruction: &str) -> Result<InstructionInfo, String>
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

            if imm_type == ImmTypeLabel::SPACKED || imm_type == ImmTypeLabel::UPACKED
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

    check_info_struct(&info_struct)?;

    Ok(info_struct)
}

fn check_info_struct(info: &InstructionInfo) -> Result<(), String>
{
    use isa::InstructionLabel::*;

    if info.register_args.clone().iter().any(|r| r.index > MAX_GPR_VAL as u8)
    {
        return Err("\nParsed a register value beyond the max range.".to_string());
    }

    if !info.valid_operand_counts.contains(&(info.get_arg_count()))
    {
        return Err("\nNot a vaild number of operands".to_string());
    }

    if info.group == ITYPE && !info.imm_arg.valid
    {
        return Err("\nWas an I-type instruction but didn't find any immediate arguments".to_string());
    }

    if (info.label == SHR || info.label == SHL) && info.imm_arg.datatype != ImmTypeLabel::UINT
    {
        return Err("Using an a non UINt value with a shift".to_string());
    }

    if info.label == INSRTBI
    {
        if !(info.imm_arg.datatype == ImmTypeLabel::UINT || info.imm_arg.datatype == ImmTypeLabel::SINT)
        {
            return Err("Using a non".to_string());
        }

        if info.imm_arg.datatype == ImmTypeLabel::UINT
        {
    
            match info.imm_arg.imm_val
            {
                ImmVal::Short(uint) =>  
                {
                    if (uint.value() >> 8) > 0
                    {
                        return Err("Trying to fit a UINT into a packed that cannot fit".to_string());
                    }
                }
                ImmVal::Long(uint) =>
                {
                    if (uint.value() >> 8) > 0
                    {
                        return Err("Trying to fit a UINT into a packed that cannot fit".to_string());
                    }
                }
                ImmVal::Full(uint) =>
                {
                    if (uint.value() >> 8) > 0
                    {
                        return Err("Trying to fit a UINT into a packed that cannot fit".to_string());
                    }
                }
            }
        }
        else if info.imm_arg.datatype == ImmTypeLabel::SINT
        {
            match info.imm_arg.imm_val
            {
                ImmVal::Short(uint) =>
                {
                    let sext = isa::sign_extend_short(uint.value() as u64);
                    if !(-128i32..=127).contains(&(sext as i32))
                    {
                        return Err("Trying to fit a UINT into a packed that cannot fit".to_string());
                    }
                }
                ImmVal::Long(uint) =>
                {
                    let sext = isa::sign_extend_long(uint.value() as u64);
                    if !(-128i32..=127).contains(&(sext as i32))
                    {
                        return Err("Trying to fit a UINT into a packed that cannot fit".to_string());
                    }
                }
                ImmVal::Full(uint) =>
                {
                    let sext = isa::sign_extend_40(uint.value());
                    if !(-128i32..=127).contains(&(sext as i32))
                    {
                        return Err("Trying to fit a UINT into a packed that cannot fit".to_string());
                    }
                }
            }
        }
    }

    Ok(())
}
