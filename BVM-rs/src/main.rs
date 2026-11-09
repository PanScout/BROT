use arbitrary_int::{i40, u40, u46};
use clap::Parser;
use std::path::PathBuf;

mod alu;
mod bvm;

use isa as brot_isa;

use crate::bvm::BvmState;
use brot_isa::*;

#[derive(Parser, Debug)]
struct Args
{
    #[arg(required = true, short)]
    input: PathBuf,
    //#[arg(short)]
    //output: PathBuf,
}

use std::io::Write;

fn print_status(vm: &BvmState, logfile: &mut impl Write) -> Result<(), String>
{
    writeln!(logfile, "==========================================================").map_err(|e| format!("{e}"))?;
    writeln!(logfile, "‖ {:1} {:>20} ‖ {:15} {:08b} ‖", "PC: ", vm.pc, "Status Register: ", vm.get_status())
        .map_err(|e| format!("{e}"))?;
    writeln!(logfile, "=====================================================================================")
        .map_err(|e| format!("{e}"))?;
    for i in 0..=MAX_GPR_VAL
    {
        let tag = vm.read_gpr(i)?.tag;
        let val = vm.read_gpr(i)?.data;
        writeln!(
            logfile,
            "‖ {:1} {:2}: {:>8} {:046b} | {:>15} ‖",
            "GPR",
            i,
            <&str>::from(tag),
            vm.read_gpr(i)?.data,
            val
        )
        .map_err(|e| format!("{e}"))?;
    }
    writeln!(logfile, "=====================================================================================\n")
        .map_err(|e| format!("{e}"))?;
    Ok(())
}

//fn group_from_label(label: types::InstructionLabel) -> types::InstructionGroup
//{
//   use types::InstructionGroup::{CType, IType, RType};
//    use types::InstructionLabel::{
//        Add, Addi, And, Andi, Convert, Mac, Mul, Mulh, Or, Ori, Shr, Shri, Sll, Slli, Sub, Xor, Xori,
//    };
//
//   match label
//   {
//      Add | Sub | Xor | Or | And | Sll | Shr | Mul | Mulh | Mac => RType,
//       Addi | Xori | Ori | Andi | Slli | Shri => IType,
//       Convert => CType,
//  }
//}

fn perform_arimethic(label: InstructionLabel, a: &Word, b: &Word) -> Result<Word, String>
{
    use brot_isa::InstructionLabel::*;

    let result: Word = match label
    {
        ADD => alu::add(a, b)?,
        SUB => alu::sub(a, b)?,
        XOR => alu::xor(a, b)?,
        OR => alu::or(a, b)?,
        AND => alu::and(a, b)?,
        SHL => todo!(),
        SHR => todo!(),
        MUL => todo!(),
        MULH => todo!(),
        MAC => todo!(),
        ADDI => todo!(),
        XORI => todo!(),
        ORI => todo!(),
        ANDI => todo!(),
        SHLI => todo!(),
        SHRI => todo!(),
        CONVERT => todo!(),
        SLT => todo!(),
        SLTI => todo!(),
        LI => todo!(),
        LUI => todo!(),
    };
    Ok(result)
}

fn process_r_type(vm: &mut BvmState, curr_instruction: Word, label: InstructionLabel) -> Result<(), String>
{
    let rd = extract_bits(curr_instruction.data.value(), RD_END, RD_START) as usize;
    let rs1 = extract_bits(curr_instruction.data.value(), RS1_END, RS1_START) as usize;
    let rs2 = extract_bits(curr_instruction.data.value(), RS2_END, RS1_START) as usize;

    let (rs1_word, rs2_word) = (vm.read_gpr(rs1)?, vm.read_gpr(rs2)?);

    if rs1_word.tag != rs2_word.tag
    {
        return Err(format!(
            "Tag of rs1: {} and tag of rs2: {} don't match",
            <&str>::from(rs1_word.tag),
            <&str>::from(rs2_word.tag)
        ));
    }
    vm.write_gpr(perform_arimethic(label, &rs1_word, &rs2_word)?, rd)?;
    Ok(())
}

fn process_i_type(vm: &mut BvmState, curr_instruction: Word, label: InstructionLabel) -> Result<(), String>
{
    let rd = extract_bits(curr_instruction.data.value(), RD_END, RD_START) as usize;
    let rs1 = extract_bits(curr_instruction.data.value(), RS1_END, RS1_START) as usize;

    let mut imm_val = u40::new(extract_bits(curr_instruction.data.value(), IMM_VAL_END, IMM_START));
    let imm_type = ImmTypeLabel::try_from(extract_bits(curr_instruction.data.value(), IMM_END, IMM_TAG_START))
        .map_err(|e| format!("{e}"))?
        .to_type_label()?;

    if SIGNED_ARIMETHIC_TYPES.contains(&imm_type)
    {
        imm_val = u40::new(((i40::new(imm_val.value() as i64) << 22) >> 22).value() as u64)
    }

    let imm_word = Word { tag: imm_type, data: imm_val };
    let rs1_word = vm.read_gpr(rs1)?;

    if rs1_word.tag != imm_word.tag
    {
        return Err(format!(
            "Tag of rs1: {} and tag of imm: {} don't match",
            <&str>::from(rs1_word.tag),
            <&str>::from(imm_word.tag)
        ));
    }

    vm.write_gpr(perform_arimethic(label, &rs1_word, &imm_word)?, rd)?;
    Ok(())
}

fn execute_code(vm: &mut BvmState, input_instructions: &[String], print_flag: bool) -> Result<(), String>
{
    let mut curr_instruction: Word;

    while vm.pc < input_instructions.len()
    {
        if print_flag
        {
            print_status(vm, &mut std::io::stdout());
        }

        curr_instruction = hex_to_word(&input_instructions[vm.pc]).map_err(|e| format!("{e}"))?;

        if curr_instruction.tag != TypeLabel::INSTRUCTION
        {
            return Err(format!(
                "Instruction on line {} is not an instruction type, is {}",
                vm.pc,
                <&str>::from(curr_instruction.tag)
            ));
        }

        let info_struct = InstructionInfo::label_to_info(
            extract_bits(curr_instruction.data.value(), END_OPCODE_FIELD, START_OPCODE_FIELD)
                .try_into()
                .map_err(|e| format!("{e}"))?,
            vm.pc,
        )?;

        match info_struct.group
        {
            InstructionGroup::RTYPE => process_r_type(vm, curr_instruction, info_struct.label)?,
            InstructionGroup::STYPE => todo!(),
            InstructionGroup::BTYPE => todo!(),
            InstructionGroup::ITYPE => process_i_type(vm, curr_instruction, info_struct.label)?,
            InstructionGroup::UTYPE => todo!(),
            InstructionGroup::JTYPE => todo!(),
            InstructionGroup::CTYPE => todo!(),
            InstructionGroup::LTYPE => todo!(),
            InstructionGroup::PTYPE => todo!(),
            InstructionGroup::QTYPE => todo!(),
        };
        vm.pc += 1;
    }
    if print_flag
    {
        print_status(vm, &mut std::io::stdout());
    }
    Ok(())
}

fn path_into_vector(input: &String) -> Result<Vec<String>, String>
{
    const HEX_LENGTH: usize = 12;
    let mut ret: Vec<String> = Vec::new();
    let string_bin =
        std::fs::read_to_string(input).unwrap_or_else(|input| panic!("Could not open input file: {input}!"));
    for line_string in string_bin.lines()
    {
        assert!(
            string_bin[0..=1].to_string() != r"\\",
            "Comment comes before instructions. Did you edit the binary file?"
        );

        if string_bin.starts_with(r"\\")
        {
            return Err("Comment comes before instructions. Did you edit the binary file?".to_string());
        }

        if string_bin.starts_with(r"//")
        {
            return Err("No foward Slashes!".to_string());
        }

        let trun_string =
            line_string.strip_suffix(r"\\").unwrap_or(line_string).trim().to_ascii_uppercase().to_string();
        assert!(trun_string.len() == HEX_LENGTH, "Final hex string is of improper length");
        ret.push(trun_string);
    }
    Ok(ret)
}

fn main()
{
    let x = arbitrary_int::i4::new(0b1101); 
    let args = Args::parse();

    let Ok(string_vec) = path_into_vector(&args.input.to_str().expect("No file provided!").to_string())
    else
    {
        panic!("Could not file into vector!");
    };

    let mut vm: BvmState = BvmState::new();
    match execute_code(&mut vm, &string_vec, true)
    {
        Ok(_) => println!("PROGRAM FINISHED!"),
        Err(e) => print!("Could not finish program due to {e}"),
    }
}
