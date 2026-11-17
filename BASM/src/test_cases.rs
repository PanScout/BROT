use std::ops::RangeInclusive;
use test_case::test_case;

use crate::string_handling::*;
use arbitrary_int::u5;
use arbitrary_int::{i18, traits::Integer, u18, u23, u40};
use isa::utils::insert_bits;

use crate::*;
use isa::ImmTypeLabel::{self, *};
use isa::ImmVal::{self, *};
use isa::{Imm, InstructionGroup, InstructionLabel, Operands, TokenStruct, TypeLabel, Word, constants::*};
use isa::utils::*;

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
#[test_case("262143", UINT => Short(u18::new(UNSIGNED_SHORT_RANGE.max().unwrap() as u32)) ; "uint short max")]
#[test_case("262144", UINT => Long(u23::new(262144)) ; "uint promotes to long")]
// assumes LI zero-extends for unsigned types
#[test_case("8388607", UINT => Long(u23::new(8_388_607)) ; "uint long max")]
#[test_case("8388608", UINT => Full(u40::new(8_388_608)) ; "uint promotes to full")]
#[test_case("1099511627775", UINT => Full(u40::new((1 << 40) - 1)) ; "uint full max")]
#[test_case("40", UMOD => Short(u18::new(40)) ; "umod 40")]
// ---- signed integers ----
#[test_case("-40", SINT => Short(neg_to_u18(-40)) ; "sint neg 40")]
#[test_case("-131072", SINT => Short(neg_to_u18(SIGNED_SHORT_RANGE.min().unwrap() as i32)) ; "sint short min")]
#[test_case("131072", SINT => Long(neg_to_u23(131072)) ; "sint promotes to long")]
#[test_case("-40", SMOD => Short(neg_to_u18(-40)) ; "smod neg 40")]
#[test_case("131071", SMOD => Short(neg_to_u18(SIGNED_SHORT_RANGE.max().unwrap() as i32)) ; "smod short max")]
#[test_case("131072", SMOD => Long(neg_to_u23(131072)) ; "smod promotes to long")]
#[test_case("-131073", SMOD => Long(neg_to_u23(-131073)) ; "smod neg promotes to long")]
#[test_case("4194303", SMOD => Long(neg_to_u23(SIGNED_LONG_RANGE.max().unwrap() as i32)) ; "smod long max")]
#[test_case("-4194304", SMOD => Long(neg_to_u23(SIGNED_LONG_RANGE.min().unwrap() as i32)) ; "smod long min")]
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
#[test_case("[5 4 3 2 1]", UPACKED => Full(u40::new(0x05_04_03_02_01)) ; "upacked")]
#[test_case("[-5 -4 -3 -2 -1]", SPACKED => Full(u40::new(0xFB_FC_FD_FE_FF)) ; "spacked")]
#[test_case("TRUE", BOOL => Short(u18::new(1)) ; "bool true")]
#[test_case("FALSE", BOOL => Short(u18::new(0)) ; "bool false")]
fn process_imm_accepts(input: &str, label: ImmTypeLabel) -> ImmVal
{
    let imm = process_imm(&format!("{input}:{}", <&str>::from(label))).unwrap();
    assert_eq!(imm.datatype, label);
    imm.imm_val
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
    let r = process_imm(&format!("{input}:{}", <&str>::from(label)));
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

fn r_helper(lab: InstructionLabel, rd: u8, rs1: u8, rs2: u8) -> TokenStruct
{
    TokenStruct {
        label: lab,
        group: InstructionGroup::RTYPE,
        operands: Operands::RType3 { rd: u5::new(rd), rs1: u5::new(rs1), rs2: u5::new(rs2) },
    }
}

fn r_two_helper(lab: InstructionLabel, rd: u8, rs1: u8) -> TokenStruct
{
    TokenStruct {
        label: lab,
        group: InstructionGroup::RTYPE,
        operands: Operands::RType2 { rd: u5::new(rd), rs1: u5::new(rs1) },
    }
}

/// Builds an expected instruction word from its 40-bit data field
fn instr_word(data: u64) -> Word
{
    Word::from_bin((u64::from(TypeLabel::INSTRUCTION) << 40) | data).unwrap()
}

#[test_case(&r_helper(InstructionLabel::ADD, 3, 4, 5) => instr_word(0b00000000_00011_00100_00101_00000000000000000) ; "basic ADD test")]
#[test_case(&r_helper(InstructionLabel::XOR, 3, 4, 5) => instr_word(0b00000010_00011_00100_00101_00000000000000000) ; "basic XOR test")]
#[test_case(&r_helper(InstructionLabel::EXTB, 3, 4, 5) => instr_word(0b00001111_00011_00100_00101_00000000000000000) ; "basic EXTB test")]
#[test_case(&r_two_helper(InstructionLabel::NOT, 3, 4) => instr_word(0b00000101_00011_00100_00000_00000000000000000) ; "basic NOT test")]
#[test_case(&r_two_helper(InstructionLabel::MV, 3, 4) => instr_word(0b00001110_00011_00100_00000_00000000000000000) ; "basic MV test")]
fn test_r_type(instr: &TokenStruct) -> Word
{
    match assemble_r_type(instr).unwrap()
    {
        VarArgRet::One(word) => word,
        _ => panic!("R-type should assemble to a single word"),
    }
}

// ============================================================
// Encoding helpers for expected values
// ============================================================

/// opcode | rd | rs1 | rs2 in their fixed positions
fn fields(op: InstructionLabel, rd: u64, rs1: u64, rs2: u64) -> u64
{
    (u64::from(op) << 32) | (rd << 27) | (rs1 << 22) | (rs2 << 17)
}

/// Short immediate field: value in bits 0–17, type tag in 18–21
fn short_imm(val: u64, tag: ImmTypeLabel) -> u64
{
    val | (u64::from(tag) << 18)
}

/// Long immediate field: value in bits 0–22, type tag in 23–26
fn long_imm(val: u64, tag: ImmTypeLabel) -> u64
{
    val | (u64::from(tag) << 23)
}

fn i3_helper(lab: InstructionLabel, rd: u8, rs1: u8, imm_val: ImmVal, datatype: ImmTypeLabel) -> TokenStruct
{
    TokenStruct {
        label: lab,
        group: InstructionGroup::ITYPE,
        operands: Operands::IType3 { rd: u5::new(rd), rs1: u5::new(rs1), imm: Imm { imm_val, datatype } },
    }
}

fn i2_helper(lab: InstructionLabel, rd: u8, imm_val: ImmVal, datatype: ImmTypeLabel) -> TokenStruct
{
    TokenStruct {
        label: lab,
        group: InstructionGroup::ITYPE,
        operands: Operands::IType2 { rd: u5::new(rd), imm: Imm { imm_val, datatype } },
    }
}

fn words(ret: VarArgRet) -> Vec<Word>
{
    match ret
    {
        VarArgRet::One(a) => vec![a],
        VarArgRet::Two((a, b)) => vec![a, b],
        VarArgRet::Three((a, b, c)) => vec![a, b, c],
    }
}

// Full immediate whose halves are easy to check: upper 17 bits = 5, lower 23 bits = 7
const FULL_TEST_VAL: u64 = (5 << 23) | 7;

// ============================================================
// I-type assembler
// ============================================================

// ---- single word ----
#[test_case(&i3_helper(InstructionLabel::ADDI, 3, 4, Short(u18::new(40)), UINT)
    => vec![instr_word(0b00010001_00011_00100_0000_000000000000101000)] ; "addi short literal encoding")]
#[test_case(&i3_helper(InstructionLabel::ADDI, 3, 4, Short(neg_to_u18(-40)), SINT)
    => vec![instr_word(fields(InstructionLabel::ADDI, 3, 4, 0) | short_imm(neg_to_u18(-40).value() as u64, SINT))]
    ; "addi short negative is not sign extended")]
#[test_case(&i2_helper(InstructionLabel::LI, 3, Short(u18::new(40)), UINT)
    => vec![instr_word(fields(InstructionLabel::LI, 3, 0, 0) | long_imm(40, UINT))] ; "li short unsigned uses long field")]
#[test_case(&i2_helper(InstructionLabel::LI, 3, Short(neg_to_u18(-40)), SINT)
    => vec![instr_word(fields(InstructionLabel::LI, 3, 0, 0) | long_imm((1 << 23) - 40, SINT))] ; "li short signed is sign extended")]
#[test_case(&i2_helper(InstructionLabel::LUI, 3, Short(neg_to_u18(-40)), SINT)
    => vec![instr_word(fields(InstructionLabel::LUI, 3, 0, 0) | long_imm((1 << 23) - 40, SINT))] ; "lui short signed is sign extended")]
#[test_case(&i2_helper(InstructionLabel::LI, 3, Long(u23::new(300_000)), UINT)
    => vec![instr_word(fields(InstructionLabel::LI, 3, 0, 0) | long_imm(300_000, UINT))] ; "li long")]
// ---- multi word ----
#[test_case(&i2_helper(InstructionLabel::LI, 3, Full(u40::new(FULL_TEST_VAL)), UINT)
    => vec![
        instr_word(fields(InstructionLabel::LI, 3, 0, 0) | long_imm(7, UINT)),
        instr_word(fields(InstructionLabel::LUI, 3, 0, 0) | long_imm(5, UINT)),
    ] ; "li full splits into li lui")]
#[test_case(&i3_helper(InstructionLabel::ADDI, 3, 4, Long(u23::new(300_000)), UINT)
    => vec![
        instr_word(fields(InstructionLabel::LI, 3, 0, 0) | long_imm(300_000, UINT)),
        instr_word(fields(InstructionLabel::ADD, 3, 4, 3)),
    ] ; "addi long becomes li plus add")]
#[test_case(&i3_helper(InstructionLabel::ADDI, 3, 4, Full(u40::new(FULL_TEST_VAL)), UINT)
    => vec![
        instr_word(fields(InstructionLabel::LI, 3, 0, 0) | long_imm(7, UINT)),
        instr_word(fields(InstructionLabel::LUI, 3, 0, 0) | long_imm(5, UINT)),
        instr_word(fields(InstructionLabel::ADD, 3, 4, 3)),
    ] ; "addi full becomes li lui add")]
fn assemble_i_type_accepts(instr: &TokenStruct) -> Vec<Word>
{
    words(assemble_i_type(instr).unwrap())
}

#[test_case(&i2_helper(InstructionLabel::LUI, 3, Full(u40::new(FULL_TEST_VAL)), UINT) ; "lui full rejected")]
#[test_case(&i3_helper(InstructionLabel::ADDI, 3, 3, Long(u23::new(300_000)), UINT) ; "long with rd equal rs1")]
#[test_case(&i3_helper(InstructionLabel::ADDI, 3, 3, Full(u40::new(FULL_TEST_VAL)), UINT) ; "full with rd equal rs1")]
// documents current behavior: SUBI has no R-type equivalent mapped yet
#[test_case(&i3_helper(InstructionLabel::SUBI, 3, 4, Long(u23::new(300_000)), UINT) ; "subi long has no r equivalent")]
fn assemble_i_type_rejects(instr: &TokenStruct)
{
    let r = assemble_i_type(instr);
    assert!(r.is_err(), "expected Err, got Ok");
}

// ============================================================
// Tokenizer: more inputs and output structure
// ============================================================

#[test_case("NOT R1, R2" ; "two operand r-type")]
#[test_case("LI R1, 40:UINT" ; "li")]
#[test_case("LUI R1, 5:UINT" ; "lui")]
#[test_case("ADD R0, R31, R15" ; "register bounds r-type")]
fn tokenize_instruction_accepts_more(instruction: &str)
{
    tokenize_instruction(instruction).unwrap();
}

#[test_case("ADD R32, R1, R2" ; "register just past max")]
#[test_case("ADD 1, R2, R3" ; "register missing prefix")]
#[test_case("ADD R1 R2 R3" ; "missing commas")]
#[test_case("ADD" ; "mnemonic only")]
#[test_case("FOO R1, R2, R3" ; "unknown mnemonic")]
#[test_case("LI R1, R2, 5:UINT" ; "li with 3 operands")]
#[test_case("ADD R1, R2, R3, R4" ; "too many operands")]
fn tokenize_instruction_fails_more(instruction: &str)
{
    let r = tokenize_instruction(instruction);
    assert!(r.is_err(), "expected Err, got {r:?}");
}

#[test_case("ADD R1, R2, R3" => r_helper(InstructionLabel::ADD, 1, 2, 3) ; "add structure")]
#[test_case("NOT R4, R5" => r_two_helper(InstructionLabel::NOT, 4, 5) ; "not structure")]
#[test_case("ADDI R1, R2, 40:UINT" => i3_helper(InstructionLabel::ADDI, 1, 2, Short(u18::new(40)), UINT) ; "addi structure")]
#[test_case("LI R5, 40:UINT" => i2_helper(InstructionLabel::LI, 5, Short(u18::new(40)), UINT) ; "li structure")]
fn tokenize_instruction_structure(instruction: &str) -> TokenStruct
{
    tokenize_instruction(instruction).unwrap()
}

fn size_class(n: i128, label: ImmTypeLabel) -> &'static str
{
    match process_imm(&format!("{n}:{}", <&str>::from(label))).unwrap().imm_val
    {
        Short(_) => "Short",
        Long(_) => "Long",
        Full(_) => "Full",
    }
}

#[test_case(*UNSIGNED_SHORT_RANGE.end() as i128, UINT => "Short" ; "unsigned short max")]
#[test_case(*UNSIGNED_SHORT_RANGE.end() as i128 + 1, UINT => "Long" ; "unsigned short max plus one")]
#[test_case(*UNSIGNED_LONG_RANGE.end() as i128, UINT => "Long" ; "unsigned long max")]
#[test_case(*UNSIGNED_LONG_RANGE.end() as i128 + 1, UINT => "Full" ; "unsigned long max plus one")]
#[test_case(*UNSIGNED_FULL_RANGE.end() as i128, UINT => "Full" ; "unsigned full max")]
#[test_case(*SIGNED_SHORT_RANGE.start() as i128, SINT => "Short" ; "signed short min")]
#[test_case(*SIGNED_SHORT_RANGE.end() as i128, SINT => "Short" ; "signed short max")]
#[test_case(*SIGNED_SHORT_RANGE.start() as i128 - 1, SINT => "Long" ; "signed short min minus one")]
#[test_case(*SIGNED_SHORT_RANGE.end() as i128 + 1, SINT => "Long" ; "signed short max plus one")]
#[test_case(*SIGNED_LONG_RANGE.start() as i128 - 1, SINT => "Full" ; "signed long min minus one")]
#[test_case(*SIGNED_LONG_RANGE.end() as i128 + 1, SINT => "Full" ; "signed long max plus one")]
#[test_case(*SIGNED_FULL_RANGE.start() as i128, SINT => "Full" ; "signed full min")]
#[test_case(*SIGNED_FULL_RANGE.end() as i128, SINT => "Full" ; "signed full max")]
fn imm_size_boundaries(n: i128, label: ImmTypeLabel) -> &'static str
{
    size_class(n, label)
}
