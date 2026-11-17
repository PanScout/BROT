use arbitrary_int::{u5, u40};
use clap::builder::Str;
use isa::{self as brot_isa, InstructionLabel::MV, utils::{extract_bits, hex_to_instr_word}};

use brot_isa::*;
use isa::constants::*;


pub mod alu;
pub use alu::*;
pub mod mem;
pub use mem::*;
pub mod bvm_def;
pub use bvm_def::*;
#[cfg(test)]
mod emulator_tests;

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
        let i = u5::new(i as u8);
        let tag = vm.read_gpr(i)?.tag;
        let val = vm.read_gpr(i)?.get_data();
        writeln!(
            logfile,
            "‖ {:1} {:2}: {:>8} {:046b} | {:>15} ‖",
            "GPR",
            i,
            <&str>::from(tag),
            vm.read_gpr(i)?.get_data(),
            val
        )
        .map_err(|e| format!("{e}"))?;
    }
    writeln!(logfile, "=====================================================================================\n")
        .map_err(|e| format!("{e}"))?;
    Ok(())
}

fn perform_binary_arimethic(label: InstructionLabel, a: &Word, b: &Word) -> Result<Word, String>
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
        SUBI => todo!(),
        XORI => todo!(),
        ORI => todo!(),
        ANDI => todo!(),
        SHLI => todo!(),
        SHRI => todo!(),
        CONVERT => todo!(),
        SLT => todo!(),
        SLTI => todo!(),
        SEQ => todo!(),
        SEQI => todo!(),
        EXTB => todo!(),
        INSRTB => todo!(),
        INSRTBI => todo!(),
        other => return Err(format!("{:?} is not a binary arithmetic instruction", other)),
    };
    Ok(result)
}

fn perform_unary_operation(label: InstructionLabel, a: &Word) -> Result<Word, String>
{
    use brot_isa::InstructionLabel::*;

    let result: Word = match label
    {
        NOT => alu::not(a)?,
        NEG => alu::neg(a)?,
        other => return Err(format!("{:?} is not a unary arithmetic instruction", other)),
    };
    Ok(result)
}

fn perform_mem_op_imm(label: InstructionLabel, vm: &mut BvmState, index: u5, imm: &Imm) -> Result<(), String>
{
    use brot_isa::InstructionLabel::*;

    match label
    {
        LI =>
        {
            vm.write_gpr(imm.to_word(), index)?;
        }
        LUI =>
        {
            let rd_type = vm.read_gpr(index)?.tag;
            let imm_type = imm.datatype.to_type_label()?;

            if rd_type != imm.datatype.to_type_label()?
            {
                return Err(format!(
                    "Cannot load upper immediate into a different destination type.\nDest:{}\nSource:{} ",
                    <&str>::from(imm_type),
                    <&str>::from(rd_type)
                ));
            }
            let br_lower = BitRange::new(16,0);
            let rd_lower = extract_bits(vm.read_gpr(index)?.get_u40(), br_lower);
            let imm_upper = imm.to_u40() << 17;
            let insert: u40 = imm_upper | rd_lower;
            let w = Word { tag: rd_type, data: insert };
            vm.write_gpr(w, index)?;
        }
        other => return Err(format!("{:?} is not a register immediate arithmetic instruction", other)),
    };
    Ok(())
}

fn process_r_type(vm: &mut BvmState, token: &TokenStruct) -> Result<(), String>
{
    let (rd, rs1, rs2) = match token.operands
    {
        Operands::RType3 { rd, rs1, rs2 } => (rd, rs1, Some(rs2)),
        Operands::RType2 { rd, rs1 } => (rd, rs1, None),
        _others => panic!("Non R-Type on R-type branch"),
    };

    let rs1_word: Word = vm.read_gpr(rs1)?;

    if token.label == InstructionLabel::MV
    {
         vm.write_gpr(rs1_word, rd)?;
         return Ok(());
    }

    let result: Word = if rs2.is_some()
    {
        let rs2_word = vm.read_gpr(rs2.unwrap())?;
        if rs1_word.tag != rs2_word.tag
        {
            return Err(format!(
                "Tag of rs1: {} and tag of rs2: {} don't match",
                <&str>::from(rs1_word.tag),
                <&str>::from(rs2_word.tag)
            ));
        }
        perform_binary_arimethic(token.label, &rs1_word, &rs2_word)?
    }
    else
    {
        perform_unary_operation(token.label, &rs1_word)?
    };
    vm.write_gpr(result, rd)?;
    Ok(())
}

fn process_i_type(vm: &mut BvmState, token: &TokenStruct) -> Result<(), String>
{
    let (rd, rs1, imm) = match token.operands
    {
        Operands::IType3 { rd, rs1, imm } => (rd, Some(rs1), imm),
        Operands::IType2 { rd, imm } => (rd, None, imm),
        _others => panic!("Non I-type in I-type path"),
    };

    let imm_word = imm.to_word();
    let result: Word = if rs1.is_some()
    {
        let rs1_word = vm.read_gpr(rs1.unwrap())?;
        if rs1_word.tag != imm_word.tag
        {
            return Err(format!(
                "Tag of rs1: {} and tag of imm: {} don't match",
                <&str>::from(rs1_word.tag),
                <&str>::from(imm_word.tag)
            ));
        }
        perform_binary_arimethic(token.label, &rs1_word, &imm_word)?
    }
    else
    {
        return perform_mem_op_imm(token.label, vm, rd, &imm);
    };
    vm.write_gpr(result, rd)?;
    Ok(())
}

fn step(vm: &mut BvmState, token: &TokenStruct) -> Result<(), String>
{
    match token.group
    {
        InstructionGroup::RTYPE => process_r_type(vm, token)?,
        InstructionGroup::STYPE => todo!(),
        InstructionGroup::BTYPE => todo!(),
        InstructionGroup::ITYPE => process_i_type(vm, token)?,
        InstructionGroup::UTYPE => todo!(),
        InstructionGroup::JTYPE => todo!(),
        InstructionGroup::CTYPE => todo!(),
        InstructionGroup::LTYPE => todo!(),
        InstructionGroup::PTYPE => todo!(),
        InstructionGroup::QTYPE => todo!(),
    };
    vm.pc += 1;
    Ok(())
}

pub fn execute_code(vm: &mut BvmState, input_instructions: &[String], print_flag: bool) -> Result<(), String>
{
    let mut curr_instruction: Word;

    while vm.pc < input_instructions.len()
    {
        if print_flag
        {
            print_status(vm, &mut std::io::stdout())?;
        }

        curr_instruction = hex_to_instr_word(&input_instructions[vm.pc]).map_err(|e| e.to_string())?;
        let tokens = curr_instruction.disassemble()?;
        step(vm, &tokens)?;
    }

    if print_flag
    {
        print_status(vm, &mut std::io::stdout())?;
    }

    Ok(())
}
