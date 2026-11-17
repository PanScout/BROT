#[cfg(test)]

    use arbitrary_int::{i40, traits::Integer, u5, u40};
    use isa::{
        Imm, ImmTypeLabel, InstructionLabel, TypeLabel::{self, *}, Word,
    };
    use test_case::test_case;

    use crate::{BvmState, alu::shl, emulate::{alu::*, process_r_type}};

    const MASK: u64 = (1 << 40) - 1;
    const I40_MAX: i64 = (1 << 39) - 1;
    const I40_MIN: i64 = -(1 << 39);
    use super::*;

    fn w(tag: TypeLabel, raw: u64) -> Word
    {
        Word { tag, data: u40::masked_new(raw) }
    }

    fn i(tag: ImmTypeLabel, raw: u64) -> Imm
    {
        Imm{ datatype: tag, imm_val: isa::ImmVal::Full(u40::masked_new(raw))}
    }

    fn fp(x: f64, frac: i32) -> i64
    {
        let ret = u40::new((f64::abs(x) * 2.0_f64.powi(frac)).round() as u64);
        if x < 0.0
        {
            return ((!ret.value() + 1)).try_into().unwrap();
        }
        ret.value() as i64
    }

    // ---------- raw bit-pattern results (unsigned + packed) ----------

    #[test_case(UINT, 2, 3 => 5 ; "uint basic")]
    #[test_case(UINT, MASK, 1 => MASK ; "uint saturates")]
    #[test_case(UMOD, MASK, 1 => 0 ; "umod wraps to zero")]
    #[test_case(UMOD, MASK, 2 => 1 ; "umod wraps to one")]
    #[test_case(UPACKED, 0x01_02_03_04_05, 0x01_01_01_01_01 => 0x02_03_04_05_06 ; "upacked basic lanes")]
    #[test_case(UPACKED, 0xFF_10_FF_00_80, 0x01_10_01_00_80 => 0xFF_20_FF_00_FF ; "upacked saturates per lane")]
    #[test_case(UPACKED, 0xFF, 0x01 => 0xFF ; "upacked no carry between lanes")]
    #[test_case(SPACKED, 0x05, 0xFD => 0x02 ; "spacked mixed signs")]
    #[test_case(SPACKED, 0x80_7F, 0xFF_01 => 0x80_7F ; "spacked saturates both directions")]
    #[test_case(SPACKED, 0xFF, 0x00 => 0xFF ; "spacked negative lane does not smear")]
    fn add_raw(tag: TypeLabel, a: u64, b: u64) -> u64
    {
        add(&w(tag, a), &w(tag, b)).unwrap().data.value()
    }

    // ---------- signed results (read back via get_i40) ----------

    #[test_case(SINT, 5, -3 => 2 ; "sint positive result")]
    #[test_case(SINT, -5, 3 => -2 ; "sint negative result")]
    #[test_case(SINT, I40_MAX, 1 => I40_MAX ; "sint saturates high")]
    #[test_case(SINT, I40_MIN, -1 => I40_MIN ; "sint saturates low")]
    #[test_case(Q31_8, -7, 2 => -5 ; "q31_8 negative")]
    #[test_case(Q15_24, -7, 2 => -5 ; "q15_24 negative")]
    #[test_case(Q19_20, -7, 2 => -5 ; "q19_20 negative")]
    #[test_case(Q23_16, -7, 2 => -5 ; "q23_16 negative")]
    #[test_case(Q7_32, I40_MAX, 1 => I40_MAX ; "q7_32 saturates")]
    #[test_case(SMOD, I40_MAX, 1 => I40_MIN ; "smod wraps high to low")]
    #[test_case(SMOD, -4, -6 => -10 ; "smod negative inputs")]
    #[test_case(SMOD, -4, 6 => 2 ; "smod mixed inputs")]
    #[test_case(SMOD, 4, -6 => -2 ; "smod mixed inputs")]
    fn add_signed(tag: TypeLabel, a: i64, b: i64) -> i64
    {
        add(&w(tag, a as u64), &w(tag, b as u64)).unwrap().get_i40().value()
    }
    #[test_case(UMOD, 5, 4 => 1;"basic sub test")]
    #[test_case(UMOD, 0, 1 => MASK;"mod wrapping")]
    #[test_case(UINT, 0, 1 => 0;"uint saturating")]
    #[test_case(UINT, 5, 1 => 4;"uint basic")]
    #[test_case(SPACKED, 0x80_80_80_80_80, 0x01_01_01_01_01 => 0x80_80_80_80_80 ; "signed sub saturates at min")]
    #[test_case(SPACKED, 0x7F_7F_7F_7F_7F, 0xFF_FF_FF_FF_FF => 0x7F_7F_7F_7F_7F; "signed sub saturates at max")]
    fn sub_unsigned(tag: TypeLabel, a: u64, b: u64) -> u64
    {
        let w1 = w(tag, a);
        let w2 = w(tag, b);
        sub(&w1, &w2).unwrap().get_data().as_u64()
    }

    #[test_case(SMOD, -10, -1 => -9;"smod basic")]
    #[test_case(SMOD, -1, 1 => -2;"smod mixed sign")]
    #[test_case(SMOD, (i40::MIN).value(), 1 => i40::MAX.value();"smod wraps bottom")]
    #[test_case(SINT, (i40::MIN).value(), 1 => i40::MIN.value();"sint satures bottom")]
    #[test_case(SINT, -24, 4 => -28;"sint basic")]

    fn sub_signed(tag: TypeLabel, a: i64, b: i64) -> i64
    {
        let w1 = w(tag, a as u64);
        let w2 = w(tag, b as u64);
        sub(&w1, &w2).unwrap().get_i40().as_i64()
    }

    #[test_case( TypeLabel::BOOL, 1 => 0; "bool case 1")]
    #[test_case( TypeLabel::BOOL, 0 => 1; "bool case 0")]
    fn not_test(tag: TypeLabel, a: u64) -> u64
    {
        let w1 = w(tag, a);
        not(&w1).unwrap().data.value()
    }


    // ---------- error paths ----------
    #[test_case(UMOD ; "umod")]
    #[test_case(SPACKED ; "spacked")]
    fn add_keeps_tag(tag: TypeLabel)
    {
        assert_eq!(add(&w(tag, 1), &w(tag, 1)).unwrap().tag, tag);
    }

    #[test_case(UINT, 0b1, 4 => 0b10000 ;"shl test basic")]
    #[test_case(UINT, 0b100, 4 => 0b01000000 ;"shl test basic 2")]
    #[test_case(UMOD, 0b100, 4 => 0b01000000 ;"shl test basic 2")]
    #[test_case(SINT, 0b100, 4 => 0b01000000 ;"shl test signed basic 1")]
    #[test_case(SINT, 0b11111100, 4 => 0b111111000000 ;"shl test signed basic 1")]
    fn test_shl(l: TypeLabel, a: u64, b: u64) -> u64
    {
        
       u40::masked_new(shl(&Word { tag: l, data: u40::masked_new(a) }, &Word { tag: l, data: u40::masked_new(b) } ).unwrap().data).value()
    }
