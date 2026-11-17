
use crate::types::{TypeLabel, Word};

use super::*;
use test_case::test_case;
// ============================================================
// Bit helpers and Word
// ============================================================

#[test_case(0b1011_0000, 7, 4 => 0b1011 ; "extract middle nibble")]
#[test_case(1 << 39, 39, 39 => 1 ; "extract single top bit")]
fn extract_bits_cases(bin: u64, hi: u8, low: u8) -> u64
{
    extract_bits( u40::new(bin), BitRange::new(hi,low) ).value()
}

#[test]
fn insert_bits_places_field()
{
    
    assert_eq!(insert_bits(u40::new(0), 0b101, BitRange::new(6,4)), u40::new(0b101 << 4));
}

#[test]
fn insert_bits_preserves_other_bits()
{
    let base = u40::new(0b1111_1111);
    assert_eq!(insert_bits(base, 0, BitRange::new(5, 2)), u40::new(0b1100_0011));
}

#[test]
#[should_panic]
fn insert_bits_rejects_oversized_field()
{
    
    let _ =   utils::insert_bits(u40::new(0), 0b1000, BitRange::new(2, 0));
}

#[test_case(0x3FFFF => -1 ; "short all ones is minus one")]
#[test_case(0x1FFFF => 131071 ; "short max positive")]
#[test_case(0x20000 => -131072 ; "short min negative")]
fn sign_extend_short_cases(raw: u64) -> i64
{
    utils::sign_extend_short(raw)
}

#[test_case(0x7FFFFF => -1 ; "long all ones is minus one")]
#[test_case(0x3FFFFF => 4194303 ; "long max positive")]
fn sign_extend_long_cases(raw: u64) -> i64
{
      utils::sign_extend_long(raw)
}

#[test]
fn word_round_trips_through_u64()
{
    let bin = (u64::from(TypeLabel::INSTRUCTION) << 40) | 0x12_3456_789A;
    assert_eq!(Word::from_bin(bin).unwrap().to_u64(), bin);
}

#[test]
fn word_from_bin_rejects_oversized()
{
    assert!(Word::from_bin(1 << 46).is_err());
}
