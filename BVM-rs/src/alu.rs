use std::ops::Shr;
use std::{fs::TryLockError::Error, ops::Shl};

use arbitrary_int::prelude::*;
use arbitrary_int::{i40, u40};

use isa::*;

pub fn add(a: &Word, b: &Word) -> Result<Word, String>
{
    use TypeLabel::{Q8_32, Q24_16, Q32_8, SINT, SMOD, SPACKED, UINT, UMOD, UPACKED};

    if a.tag != b.tag
    {
        return Err(format!("Tags of a and b don't match in ALU: {} and {}", <&str>::from(a.tag), <&str>::from(b.tag)));
    }

    match a.tag
    {
        UINT => Ok(Word { data: a.data.saturating_add(b.data), tag: a.tag }),
        UMOD => Ok(Word { data: a.data.wrapping_add(b.data), tag: a.tag }),
        UPACKED =>
        {
            let mut y = Word { tag: a.tag, data: u40::new(0) };

            for i in 0..=4
            {
                //Extract each of the 5 bytes
                let a = extract_bits(a.data.value(), 8 * i + 7, 8 * i) as u8;
                let b = extract_bits(b.data.value(), 8 * i + 7, 8 * i) as u8;

                let res = u64::from(a.saturating_add(b));

                y.data |= u40::new(res << (8 * i));
            }
            Ok(y)
        }
        SINT | Q32_8 | Q24_16 | Q8_32 =>
        {
            let a_val: i40 = i40::new(a.data.value() as i64);
            let b_val: i40 = i40::new(b.data.value() as i64);

            Ok(Word { tag: a.tag, data: u40::new(a_val.saturating_add(b_val).value() as u64) })
        }
        SMOD =>
        {
            let a_val: i40 = i40::new(a.data.value() as i64);
            let b_val: i40 = i40::new(b.data.value() as i64);

            Ok(Word { tag: a.tag, data: u40::new(a_val.wrapping_add(b_val).value() as u64) })
        }
        SPACKED =>
        {
            let mut y: Word = Word { tag: a.tag, data: u40::new(0) };

            for i in 0..=4
            {
                //Extract each of the 5 bytes
                let a = extract_bits(a.data.value(), 8 * i + 7, 8 * i) as i8;
                let b = extract_bits(b.data.value(), 8 * i + 7, 8 * i) as i8;

                let res = i40::from(a.saturating_add(b));

                y.data |= u40::new((res << (8 * i)).value() as u64);
            }
            Ok(y)
        }
        other => Err(format!("{} is not valid for arithmetic", <&str>::from(other))),
    }
}

pub fn sub(a: &Word, b: &Word) -> Result<Word, String>
{
    use TypeLabel::{*};

    if a.tag != b.tag
    {
        return Err(format!("Tags of a and b don't match in ALU: {} and {}", <&str>::from(a.tag), <&str>::from(b.tag)));
    }

    match a.tag
    {
        UINT => Ok(Word { data: a.data.saturating_sub(b.data), tag: a.tag }),
        UMOD => Ok(Word { data: a.data.wrapping_sub(b.data), tag: a.tag }),
        UPACKED =>
        {
            let mut y: Word = Word { tag: a.tag, data: u40::new(0) };

            for i in 0..=4
            {
                //Extract each of the 5 bytes
                let a = extract_bits(a.data.value(), 8 * i + 7, 8 * i) as u8;
                let b = extract_bits(b.data.value(), 8 * i + 7, 8 * i) as u8;

                let res = u64::from(a.saturating_sub(b));

                y.data |= u40::new(res << (8 * i));
            }
            Ok(y)
        }
        SINT | Q32_8 | Q16_24 | Q24_16 | Q8_32 =>
        {
            let a_val: i40 = i40::from_bits(a.data.value());
            let b_val: i40 = i40::from_bits(b.data.value());

            Ok(Word { tag: a.tag, data: u40::new(a_val.saturating_sub(b_val).value() as u64) })
        }
        SMOD =>
        {
            let a_val: i40 = i40::from_bits(a.data.value());
            let b_val: i40 = i40::from_bits(b.data.value());

            Ok(Word { tag: a.tag, data: u40::new(a_val.wrapping_sub(b_val).value() as u64) })
        }
        SPACKED =>
        {
            let mut y: Word = Word { tag: a.tag, data: u40::new(0) };

            for i in 0..=4
            {
                //Extract each of the 5 bytes
                let a = extract_bits(a.data.value(), 8 * i + 7, 8 * i) as i8;
                let b = extract_bits(b.data.value(), 8 * i + 7, 8 * i) as i8;

                let res = u40::from(a.saturating_sub(b) as u8);

                y.data |= u40::new((res << (8 * i)).value() as u64);
            }
            Ok(y)
        }
        other => Err(format!("{} is not valid for arithmetic", <&str>::from(other))),
    }
}

pub fn xor(a: &Word, b: &Word) -> Result<Word, String>
{
    if a.tag != b.tag
    {
        return Err(format!("Tags of a and b don't match in ALU: {} and {}", <&str>::from(a.tag), <&str>::from(b.tag)));
    }

    if ARIMETHIC_TYPES.contains(&a.tag)
    {
        Ok(Word { tag: a.tag, data: a.data ^ b.data })
    }
    else
    {
        Err(format!("{} is not an arimethic type. Can't perform XOR", <&str>::from(a.tag)))
    }
}

pub fn or(a: &Word, b: &Word) -> Result<Word, String>
{
    if a.tag != b.tag
    {
        return Err(format!("Tags of a and b don't match in ALU: {} and {}", <&str>::from(a.tag), <&str>::from(b.tag)));
    }

    if ARIMETHIC_TYPES.contains(&a.tag)
    {
        Ok(Word { tag: a.tag, data: a.data | b.data })
    }
    else
    {
        Err(format!("{} is not an arimethic type. Can't perform XOR", <&str>::from(a.tag)))
    }
}

pub fn and(a: &Word, b: &Word) -> Result<Word, String>
{
    if a.tag != b.tag
    {
        return Err(format!("Tags of a and b don't match in ALU: {} and {}", <&str>::from(a.tag), <&str>::from(b.tag)));
    }

    if ARIMETHIC_TYPES.contains(&a.tag)
    {
        Ok(Word { tag: a.tag, data: a.data & b.data })
    }
    else
    {
        Err(format!("{} is not an arimethic type. Can't perform XOR", <&str>::from(a.tag)))
    }
}
pub fn shl(a: &Word, b: &Word) -> Result<Word, String>
{
    if a.tag != b.tag
    {
        return Err(format!("Tags of a and b don't match in ALU: {} and {}", <&str>::from(a.tag), <&str>::from(b.tag)));
    }
    if ARIMETHIC_TYPES.contains(&a.tag)
    {
        let shift_amount = b.data.value().clamp(0, 40) as u32;
        Ok(Word { tag: a.tag, data: a.data.shl(shift_amount) })
    }
    else
    {
        Err(format!("{} is not an arimethic type. Can't perform XOR", <&str>::from(a.tag)))
    }
}

pub fn shr(a: &Word, b: &Word) -> Result<Word, String>
{
    if a.tag != b.tag
    {
        return Err(format!("Tags of a and b don't match in ALU: {} and {}", <&str>::from(a.tag), <&str>::from(b.tag)));
    }

    let shift_amount = b.data.value().clamp(0, 40) as u32;
    if UNSIGNED_ARIMETHIC_TYPES.contains(&a.tag)
    {
        Ok(Word { tag: a.tag, data: a.data.shr(shift_amount) })
    }
    else if SIGNED_ARIMETHIC_TYPES.contains(&a.tag)
    {
        let a_data_signed = i40::new((a.data.value() as i64));
        Ok(Word { tag: a.tag, data: u40::new(a_data_signed.shr(shift_amount).value() as u64) })
    }
    else
    {
        Err(format!("{} is not an arimethic type. Can't perform XOR", <&str>::from(a.tag)))
    }
}

pub fn mul(a: &Word, b: &Word) -> Result<Word, String>
{
    if a.tag != b.tag
    {
        return Err(format!("Tags of a and b don't match in ALU: {} and {}", <&str>::from(a.tag), <&str>::from(b.tag)));
    }
    match a.tag
    {
        TypeLabel::FREE => todo!(),
        TypeLabel::UINT => todo!(),
        TypeLabel::UMOD => todo!(),
        TypeLabel::UPACKED => todo!(),
        TypeLabel::BOOL => todo!(),
        TypeLabel::SINT => todo!(),
        TypeLabel::SMOD => todo!(),
        TypeLabel::Q32_8 => todo!(),
        TypeLabel::Q24_16 => todo!(),
        TypeLabel::Q16_24 => todo!(),
        TypeLabel::Q8_32 => todo!(),
        TypeLabel::SPACKED => todo!(),
        TypeLabel::INSTRUCTION => todo!(),
        TypeLabel::PTRF => todo!(),
        TypeLabel::PTRB => todo!(),
        TypeLabel::INERT => todo!(),
    }
}

pub fn mulh(a: &Word, b: &Word) -> Result<Word, String>
{
    if a.tag != b.tag
    {
        return Err(format!("Tags of a and b don't match in ALU: {} and {}", <&str>::from(a.tag), <&str>::from(b.tag)));
    }
    match a.tag
    {
        TypeLabel::FREE => todo!(),
        TypeLabel::UINT => todo!(),
        TypeLabel::UMOD => todo!(),
        TypeLabel::UPACKED => todo!(),
        TypeLabel::BOOL => todo!(),
        TypeLabel::SINT => todo!(),
        TypeLabel::SMOD => todo!(),
        TypeLabel::Q32_8 => todo!(),
        TypeLabel::Q24_16 => todo!(),
        TypeLabel::Q16_24 => todo!(),
        TypeLabel::Q8_32 => todo!(),
        TypeLabel::SPACKED => todo!(),
        TypeLabel::INSTRUCTION => todo!(),
        TypeLabel::PTRF => todo!(),
        TypeLabel::PTRB => todo!(),
        TypeLabel::INERT => todo!(),
    }
}

pub fn mac(a: &Word, b: &Word) -> Result<Word, String>
{
    if a.tag != b.tag
    {
        return Err(format!("Tags of a and b don't match in ALU: {} and {}", <&str>::from(a.tag), <&str>::from(b.tag)));
    }
    match a.tag
    {
        TypeLabel::FREE => todo!(),
        TypeLabel::UINT => todo!(),
        TypeLabel::UMOD => todo!(),
        TypeLabel::UPACKED => todo!(),
        TypeLabel::BOOL => todo!(),
        TypeLabel::SINT => todo!(),
        TypeLabel::SMOD => todo!(),
        TypeLabel::Q32_8 => todo!(),
        TypeLabel::Q24_16 => todo!(),
        TypeLabel::Q16_24 => todo!(),
        TypeLabel::Q8_32 => todo!(),
        TypeLabel::SPACKED => todo!(),
        TypeLabel::INSTRUCTION => todo!(),
        TypeLabel::PTRF => todo!(),
        TypeLabel::PTRB => todo!(),
        TypeLabel::INERT => todo!(),
    }
}
