use std::ops::RangeInclusive;

use arbitrary_int::{i18, traits::Integer, u18, u23, u40};
use isa::RegisterType::GPR;
use test_case::test_case;
use crate::string_handling::*;


use isa::ImmTypeLabel::*;
use isa::ImmVal::*;
use isa::*;
use crate::*;

const UNSIGNED_SHORT_RANGE: RangeInclusive<u64> = 0..=((1u64 << 18) - 1);
const UNSIGNED_LONG_RANGE: RangeInclusive<u64> = 0..=((1u64 << 23) - 1);
const UNSIGNED_FULL_RANGE: RangeInclusive<u64> = 0..=(1u64 << 40) - 1;

const SIGNED_SHORT_RANGE: RangeInclusive<i64> = -(1i64 << 17)..=((1i64 << 17) - 1);
const SIGNED_LONG_RANGE: RangeInclusive<i64> = -(1i64 << 22)..=((1i64 << 22) - 1);
const SIGNED_FULL_RANGE: RangeInclusive<i64> = -(1i64 << 39)..=((1i64 << 39) - 1);

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
#[test_case("54.632", Q19_20 => Full(u40::new(57285804)) ; "q19_20 pos")]
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

#[test_case("ADD R1, R2, R3"; "accepts R-type")]
#[test_case("ADDI R1, R2, 45:UINT"; "accepts I-type")]
#[test_case("ADDI R31, R0, TRUE:BOOL"; "reg bounds")]
fn tokenize_instruction_accepts(instruction: &str)
{
    assert!(tokenize_instruction(instruction).is_ok());
}

#[test_case("ADD R40, R2, R3"; "Reg out of range")]
#[test_case("ADD R-40, R2, R3"; "Reg negative")]
#[test_case("ADDI R4, R2, R3"; "Immedaite with R")]
#[test_case("R4, R2, R3"; "No instruction")]
#[test_case("ADD R4, R2"; "Missing register")]
#[test_case("ADDI R4, R2"; "Missing immediate")]
#[test_case("ADDI R4, R2, -52:UINT"; "Wrong immediate")]
#[test_case("ADDI R4, R2, 11111111111111111111111111111111111111111111111111111:UINT"; "too big immediate")]
#[test_case("ADDI R4, R2, 0:BOOL"; "bool")]
#[test_case("INSRTBI R4, R2, 300:UINT"; "insert packed out of range unsigned")]
#[test_case("INSRTBI R4, R2, -300:SINT"; "insert packed out of range signed")]
#[test_case("NOT R4, R2, R3"; "not with 3 args")]
fn tokenize_instruction_fails(instruction: &str)
{
    assert!(tokenize_instruction(instruction).is_err());
}

fn r_helper(lab: InstructionLabel, rd: u8, rs1: u8, rs2: u8) -> InstructionInfo
{
    let mut ret = InstructionInfo::label_to_info(lab).unwrap();

    ret.register_args.push(RegisterIndexArg { index: rd, kind: GPR });
    ret.register_args.push(RegisterIndexArg { index: rs1, kind: GPR });
    ret.register_args.push(RegisterIndexArg { index: rs2, kind: GPR });

    ret
}

fn r_two_helper(lab: InstructionLabel, rd: u8, rs1: u8) -> InstructionInfo
{
    let mut ret = InstructionInfo::label_to_info(lab).unwrap();

    ret.register_args.push(RegisterIndexArg { index: rd, kind: GPR });
    ret.register_args.push(RegisterIndexArg { index: rs1, kind: GPR });

    ret
}
#[test_case(&r_helper(InstructionLabel::ADD, 3,4,5) => Word{ tag: TypeLabel::INSTRUCTION, data: u40::new(0b00000000_00011_00100_00101_00000000000000000) }; "basic ADD test")]
#[test_case(&r_helper(InstructionLabel::XOR, 3,4,5) => Word{ tag: TypeLabel::INSTRUCTION, data: u40::new(0b00000010_00011_00100_00101_00000000000000000) }; "basic XOR test")]
#[test_case(&r_helper(InstructionLabel::EXTB, 3,4,5) => Word{ tag: TypeLabel::INSTRUCTION, data: u40::new(0b00001101_00011_00100_00101_00000000000000000) }; "basic EXTB test")]
#[test_case(&r_two_helper(InstructionLabel::NOT, 3, 4) => Word{ tag: TypeLabel::INSTRUCTION, data: u40::new(0b00000101_00011_00100_00000_00000000000000000) }; "basic NOT test")]
#[test_case(&r_two_helper(InstructionLabel::MV, 3, 4) => Word{ tag: TypeLabel::INSTRUCTION, data: u40::new(0b00001100_00011_00100_00000_00000000000000000) }; "basic NOT test")]
fn test_r_type(instr: &InstructionInfo) -> Word
{
    match assemble_r_type(instr).unwrap()
    {
        VarArgRet::One(word) => word,
        _others => panic!("what?")
    }

}   
