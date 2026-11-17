use arbitrary_int::u40;
use arbitrary_int::prelude::*;

use crate::types::TypeLabel;
use crate::types::Word;

use super::*;


#[must_use]
pub fn extract_bits(bin: u40, r: BitRange) -> u40
{
    assert!(r.hi() >= r.low() && r.hi() < 40, "Error: Invalid bit range {} downto {}", r.hi(), r.low());
    let width = r.hi() - r.low() + 1;
    let mask = u64::MAX >> (64 - width);
    u40::masked_new( (bin.value() >> r.low()) & mask)
}

#[must_use]
pub fn extract_bits_u64(bin: u64, r: BitRange) -> u64
{
    assert!(r.hi() >= r.low() && r.hi() < 64, "Error: Invalid bit range {} downto {}", r.hi(), r.low());
    let width = r.hi() - r.low() + 1;
    let mask = u64::MAX >> (64 - width);
     (bin >> r.low()) & mask
}

#[must_use]
pub fn insert_bits(bin: u40, field: u64, r: BitRange) -> u40
{
    assert!(r.hi() >= r.low() && r.hi() < 40, "Error: Invalid bit range {} downto {}", r.hi(), r.low());
    let width = r.hi() - r.low() + 1;
    assert!(width == 64 || field >> width == 0, "Error: Field {field} does not fit in {width} bits");
    let mask = u40::new((u64::MAX >> (64 - width)) << r.low());
    (bin & !mask) | u40::new(field <<r.low())
}

pub fn sign_extend_40(raw: u64) -> i64
{
    (raw << 24).cast_signed() >> 24
}

pub fn sign_extend_short(raw: u64) -> i64
{
    ((raw << 46) as i64) >> 46
}

pub fn sign_extend_long(raw: u64) -> i64
{
    ((raw << 41) as i64) >> 41
}

pub fn to_i40(x: u40) -> i40
{
    i40::masked_new(x.value() as i64)
}


pub fn hex_to_instr_word(hex: &str) -> Result<Word, String>
{
    const RADIX: u32 = 16;
    let bin: u46 = u46::try_new(u64::from_str_radix(hex, RADIX).map_err(|e| format!("Could not parse hex: {e}"))?)
        .map_err(|e| format!("{e}"))?;
    let ret = Word {
        tag: TypeLabel::try_from_primitive(extract_bits_u64(bin.value(), TAG_FIELD).value())
            .map_err(|e| format!("{e}"))?,
        data: u40::masked_new(bin),
    };
    if ret.tag != TypeLabel::INSTRUCTION
    {
        return Err(format!("Instruction is not an instruction type, is {}", <&str>::from(ret.tag)));
    }
    Ok(ret)
}
