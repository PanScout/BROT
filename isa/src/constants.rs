use std::ops::RangeInclusive;

use crate::types::{ImmTypeLabel, InstructionLabel, TypeLabel};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BitRange {
    hi: u8,
    low: u8,
}

impl BitRange 
{
    pub const fn new(hi: u8, low: u8) -> BitRange {
        assert!(hi > low, "Creating a backwards bit range!");
        assert!(hi < 64, "High range too high!");
        BitRange { hi, low }
    }

    pub fn low(&self) -> u8 {
        self.low
    }

    pub fn hi(&self) -> u8 {
        self.hi
    }
}

pub const MAX_GPR_VAL: usize = 31;

pub const TAG_FIELD: BitRange = BitRange::new(45, 40);
pub const OPCODE_FIELD: BitRange = BitRange::new(39, 32);
pub const RD_FIELD: BitRange = BitRange::new(31, 27);
pub const RS1_FIELD: BitRange = BitRange::new(26, 22);
pub const RS2_FIELD: BitRange = BitRange::new(21, 17);

pub const SHORT_IMM_VAL_FIELD: BitRange = BitRange::new(17, 0);
pub const SHORT_IMM_TAG_FIELD: BitRange = BitRange::new(21, 18);
pub const SHORT_IMM_FIELD: BitRange = BitRange::new(21, 0);

pub const LONG_IMM_VAL_FIELD: BitRange = BitRange::new(22, 0);
pub const LONG_IMM_TAG_FIELD: BitRange = BitRange::new(26, 23);
pub const LONG_IMM_FIELD: BitRange = BitRange::new(26, 0);

pub const DATA_FIELD: BitRange = BitRange::new(39, 0);

pub const MASK_46: u64 = (1u64 << 46) - 1;
pub const MASK_40: u64 = (1 << 40) - 1;
pub const LONG_IMM_MASK23: u64 = (1u64 << 23) - 1;

pub const UNSIGNED_SHORT_RANGE: RangeInclusive<u64> = 0..=((1u64 << 18) - 1);
pub const UNSIGNED_LONG_RANGE: RangeInclusive<u64> = 0..=((1u64 << 23) - 1);
pub const UNSIGNED_FULL_RANGE: RangeInclusive<u64> = 0..=(1u64 << 40) - 1;

pub const SIGNED_SHORT_RANGE: RangeInclusive<i64> = -(1i64 << 17)..=((1i64 << 17) - 1);
pub const SIGNED_LONG_RANGE: RangeInclusive<i64> = -(1i64 << 22)..=((1i64 << 22) - 1);
pub const SIGNED_FULL_RANGE: RangeInclusive<i64> = -(1i64 << 39)..=((1i64 << 39) - 1);

pub const ARIMETHIC_TYPES_IMM: [ImmTypeLabel; 11] = {
    use ImmTypeLabel::*;
    [UINT, SINT, UMOD, SMOD, UPACKED, SPACKED, Q31_8, Q23_16, Q19_20, Q15_24, Q7_32]
};

pub const ARIMETHIC_TYPES: [TypeLabel; 11] = {
    use TypeLabel::*;
    [UINT, SINT, UMOD, SMOD, UPACKED, SPACKED, Q31_8, Q23_16, Q19_20, Q15_24, Q7_32]
};

pub const SIGNED_ARIMETHIC_TYPES_IMM: [ImmTypeLabel; 8] = {
    use ImmTypeLabel::*;
    [SINT, SMOD, SPACKED, Q31_8, Q23_16, Q19_20, Q15_24, Q7_32]
};

pub const SIGNED_ARIMETHIC_TYPES: [TypeLabel; 8] = {
    use TypeLabel::*;
    [SINT, SMOD, SPACKED, Q31_8, Q23_16, Q15_24, Q19_20, Q7_32]
};

pub const UNSIGNED_ARIMETHIC_TYPES_IMM: [ImmTypeLabel; 3] = {
    use ImmTypeLabel::*;
    [UINT, UMOD, UPACKED]
};

pub const UNSIGNED_ARIMETHIC_TYPES: [TypeLabel; 3] = {
    use TypeLabel::*;
    [UINT, UMOD, UPACKED]
};

pub const MEM_OPS: [InstructionLabel; 3] = {
    use InstructionLabel::*;
    [MV, LI, LUI]
};
