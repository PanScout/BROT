use num_enum::{IntoPrimitive, TryFromPrimitive};
use strum::{EnumCount, EnumIter, EnumString, IntoStaticStr};

use arbitrary_int::{i40, u18, u23, u40, u46};

pub use strum::IntoEnumIterator;

pub const MAX_GPR_VAL: usize = 31;

pub const END_TAG_FIELD: u64 = 45;
pub const START_TAG_FIELD: u64 = 40;

pub const START_OPCODE_FIELD: u64 = 32;
pub const END_OPCODE_FIELD: u64 = 39;

pub const RD_START: u64 = 27;
pub const RD_END: u64 = 31;
pub const RS1_START: u64 = 22;
pub const RS1_END: u64 = 26;
pub const RS2_START: u64 = 17;
pub const RS2_END: u64 = 21;

pub const C_TYPE_START_REGISTER_FIELD: u64 = 22;
pub const C_TYPE_END_REGISTER_FIELD: u64 = 31;
pub const C_TYPE_REGISTER_FIELD_WIDTH: u64 = 5;
pub const C_TYPE_START_TARGET_TYPE_FIELD: u64 = 14;

pub const IMM_START: u64 = 0;
pub const IMM_VAL_END: u64 = 17;
pub const IMM_TAG_START: u64 = 18;
pub const IMM_END: u64 = 21;

pub const LONG_IMM_START: u64 = 0;
pub const LONG_IMM_VAL_END: u64 = 22;
pub const LONG_IMM_TAG_START: u64 = 23;
pub const LONG_IMM_END: u64 = 26;
pub const LONG_IMM_MASK23: u64 = (1u64 << 23) - 1;
pub const LONG_UIMM_MAX: u64 = (1u64 << 23) - 1;
pub const LONG_SIMM_MAX: i64 = (1i64 << 22) - 1;
pub const LONG_SIMM_MIN: i64 = -(1i64 << 22);

pub const UIMM_MAX: u64 = (1u64 << 18) - 1;
pub const SIMM_MAX: i64 = (1i64 << 17) - 1;
pub const SIMM_MIN: i64 = -(1i64 << 17);
//pub const MASK_IMM22: u64 = (1u64 << 22) - 1;
pub const MASK_IMM18: u64 = (1u64 << 18) - 1;
pub const MASK_IMM22: u64 = (1u64 << 22) - 1;
pub const MASK_46: u64 = (1u64 << 46) - 1;
pub const MASK_40: u64 = (1 << 40) - 1;
pub const MASK_39: i64 = (1 << 39) - 1;
pub const MASK_39N: i64 = -(1 << 39);

pub const END_DATA_FIELD: u64 = 39;
pub const START_DATA_FIELD: u64 = 0;

pub const ARIMETHIC_TYPES_IMM: [ImmTypeLabel; 10] = {
    use ImmTypeLabel::*;
    [UINT, SINT, UMOD, SMOD, UPACKED, SPACKED, Q31_8, Q23_16, Q15_24, Q7_32]
};

pub const ARIMETHIC_TYPES: [TypeLabel; 10] = {
    use TypeLabel::*;
    [UINT, SINT, UMOD, SMOD, UPACKED, SPACKED, Q31_8, Q23_16, Q15_24, Q7_32]
};

pub const SIGNED_ARIMETHIC_TYPES_IMM: [ImmTypeLabel; 7] = {
    use ImmTypeLabel::*;
    [SINT, SMOD, SPACKED, Q31_8, Q23_16, Q15_24, Q7_32]
};

pub const SIGNED_ARIMETHIC_TYPES: [TypeLabel; 7] = {
    use TypeLabel::*;
    [SINT, SMOD, SPACKED, Q31_8, Q23_16, Q15_24, Q7_32]
};

pub const UNSIGNED_ARIMETHIC_TYPES_IMM: [ImmTypeLabel; 3] = {
    use ImmTypeLabel::*;
    [UINT, UMOD, UPACKED]
};

pub const UNSIGNED_ARIMETHIC_TYPES: [TypeLabel; 3] = {
    use TypeLabel::*;
    [UINT, UMOD, UPACKED]
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RegisterType
{
    Pointer,
    GPR,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct RegisterIndexArg
{
    pub index: u8,
    pub kind: RegisterType,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]

pub enum ImmVal
{
    Short(u18),
    Long(u23),
    Full(u40),
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ImmArg
{
    pub imm_val: ImmVal,
    pub datatype: ImmTypeLabel,
    pub valid: bool,
}

#[repr(u64)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive, EnumString, IntoStaticStr, EnumCount)]
pub enum InstructionGroup
{
    RTYPE = 0,
    STYPE,
    BTYPE,
    ITYPE,
    UTYPE,
    JTYPE,
    CTYPE,
    LTYPE,
    PTYPE,
    QTYPE,
}

impl InstructionGroup {}

#[repr(u64)]
#[derive(
    Debug,
    PartialEq,
    PartialOrd,
    Eq,
    Clone,
    Copy,
    IntoPrimitive,
    TryFromPrimitive,
    EnumString,
    IntoStaticStr,
    EnumCount,
    Default,
)]

pub enum TypeLabel
{
    FREE = 0,
    UINT,
    UMOD,
    UPACKED,
    BOOL,
    SINT,
    SMOD,
    Q31_8,
    Q23_16,
    Q19_20,
    Q15_24,
    Q7_32,
    SPACKED,
    INSTRUCTION,
    PTRF,
    PTRB,
    #[default]
    INERT,
}

#[repr(u64)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, IntoPrimitive, TryFromPrimitive, EnumString, IntoStaticStr, EnumIter)]
pub enum ImmTypeLabel
{
    UINT = 0,
    UMOD = 1,
    UPACKED = 2,
    BOOL = 3,
    SINT = 4,
    SMOD = 5,
    Q31_8 = 6,
    Q23_16 = 7,
    Q19_20 = 8,
    Q15_24 = 9,
    Q7_32 = 10,
    SPACKED = 11,
}

impl ImmTypeLabel
{
    pub fn type_label_to_imm_label(label: TypeLabel) -> Result<ImmTypeLabel, String>
    {
        match label
        {
            TypeLabel::UINT => Ok(ImmTypeLabel::UINT),
            TypeLabel::UMOD => Ok(ImmTypeLabel::UMOD),
            TypeLabel::UPACKED => Ok(ImmTypeLabel::UPACKED),
            TypeLabel::BOOL => Ok(ImmTypeLabel::BOOL),
            TypeLabel::SINT => Ok(ImmTypeLabel::SINT),
            TypeLabel::SMOD => Ok(ImmTypeLabel::SMOD),
            TypeLabel::Q31_8 => Ok(ImmTypeLabel::Q31_8),
            TypeLabel::Q23_16 => Ok(ImmTypeLabel::Q23_16),
            TypeLabel::Q15_24 => Ok(ImmTypeLabel::Q15_24),
            TypeLabel::Q7_32 => Ok(ImmTypeLabel::Q7_32),
            TypeLabel::SPACKED => Ok(ImmTypeLabel::SPACKED),
            TypeLabel::Q19_20 => Ok(ImmTypeLabel::Q19_20),
            TypeLabel::FREE | TypeLabel::INSTRUCTION | TypeLabel::PTRB | TypeLabel::PTRF | TypeLabel::INERT =>
            {
                Err(format!("{label:?} is not a valid immediate type"))
            }
        }
    }

    pub fn to_type_label(self) -> Result<TypeLabel, String>
    {
        match self
        {
            ImmTypeLabel::UINT => Ok(TypeLabel::UINT),
            ImmTypeLabel::UMOD => Ok(TypeLabel::UMOD),
            ImmTypeLabel::UPACKED => Ok(TypeLabel::UPACKED),
            ImmTypeLabel::BOOL => Ok(TypeLabel::BOOL),
            ImmTypeLabel::SINT => Ok(TypeLabel::SINT),
            ImmTypeLabel::SMOD => Ok(TypeLabel::SMOD),
            ImmTypeLabel::Q31_8 => Ok(TypeLabel::Q31_8),
            ImmTypeLabel::Q23_16 => Ok(TypeLabel::Q23_16),
            ImmTypeLabel::Q15_24 => Ok(TypeLabel::Q15_24),
            ImmTypeLabel::Q7_32 => Ok(TypeLabel::Q7_32),
            ImmTypeLabel::SPACKED => Ok(TypeLabel::SPACKED),
            ImmTypeLabel::Q19_20 =>  Ok(TypeLabel::Q19_20)
        }
    }
}

#[repr(u64)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Copy, TryFromPrimitive, IntoPrimitive, EnumString, IntoStaticStr)]
pub enum InstructionLabel
{
    // R-type
    ADD = 0,
    SUB,
    XOR,
    OR,
    AND,
    NOT,
    SHL,
    SHR,
    SLT,
    MUL,
    MULH,
    MAC,
    MV,
    EXTB,
    INSRTB,

    // I-type
    ADDI,
    XORI,
    ORI,
    ANDI,
    SHRI,
    SHLI,
    SLTI,
    LI,
    LUI,
    INSRTBI,

    // C-type
    CONVERT = 29,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct InstructionInfo
{
    pub label: InstructionLabel,
    pub group: InstructionGroup,
    pub register_args: Vec<RegisterIndexArg>,
    pub imm_arg: ImmArg,
    pub valid_operand_counts: &'static [usize],
}

impl InstructionInfo
{
    pub fn label_to_info(label: InstructionLabel) -> Result<InstructionInfo, String>
    {
        use InstructionGroup::{CTYPE, ITYPE, RTYPE};
        use InstructionLabel::*;

        let (group, valid_operand_counts): (InstructionGroup, &'static [usize]) = match label
        {
            // R-type
            ADD => (RTYPE, &[3]),
            SUB => (RTYPE, &[3]),
            XOR => (RTYPE, &[3]),
            OR => (RTYPE, &[3]),
            AND => (RTYPE, &[3]),
            SHL => (RTYPE, &[3]),
            SLT => (RTYPE, &[3]),
            NOT => (RTYPE, &[2]),
            SHR => (RTYPE, &[3]),
            MUL => (RTYPE, &[3]),
            MULH => (RTYPE, &[3]),
            MAC => (RTYPE, &[3]),
            MV => (RTYPE, &[2]),
            EXTB => (RTYPE, &[3]),
            INSRTB => (RTYPE, &[3]),

            // I-type
            ADDI => (ITYPE, &[3]),
            XORI => (ITYPE, &[3]),
            ORI => (ITYPE, &[3]),
            SHRI => (ITYPE, &[3]),
            SLTI => (ITYPE, &[3]), 
            ANDI => (ITYPE, &[3]),
            SHLI => (ITYPE, &[3]),
            LI => (ITYPE, &[2]),
            LUI => (ITYPE, &[2]),
            INSRTBI => (ITYPE, (&[3])),

            // C-type
            CONVERT => (CTYPE, &[2]),
        };

        let default_imm: ImmArg =
            ImmArg { imm_val: ImmVal::Short(u18::new(0)), datatype: ImmTypeLabel::BOOL, valid: false };

        Ok(InstructionInfo {
            label,
            group,
            valid_operand_counts,
            register_args: Vec::with_capacity(0),
            imm_arg: default_imm,
        })
    }

    pub fn get_arg_count(&self) -> usize
    {
        self.register_args.len() + ({ if self.imm_arg.valid { 1u64 } else { 0u64 } }) as usize
    }

    pub fn get_rd(&self) -> Result<RegisterIndexArg, String>
    {
        let ret = self
            .register_args
            .get(0)
            .ok_or("Could not access RD")
            .map_err(|_e| format!("Could not access RD!"))?
            .clone();
        Ok(ret)
    }

    pub fn get_rd_index(&self) -> Result<u64, String>
    {
        let ret = self
            .register_args
            .get(0)
            .ok_or("Could not access RD")
            .map_err(|_e| format!("Could not access RD!"))?
            .clone();
        Ok(ret.index as u64)
    }

    pub fn get_rs1(&self) -> Result<RegisterIndexArg, String>
    {
        let ret = self
            .register_args
            .get(1)
            .ok_or("Could not access RS1")
            .map_err(|_e| format!("Could not access RS1!"))?
            .clone();
        Ok(ret)
    }

    pub fn get_rs1_index(&self) -> Result<u64, String>
    {
        let ret = self
            .register_args
            .get(1)
            .ok_or("Could not access RS1")
            .map_err(|_e| format!("Could not access RS1!"))?
            .clone();
        Ok(ret.index as u64)
    }

    pub fn get_rs2(&self) -> Result<RegisterIndexArg, String>
    {
        let ret = self
            .register_args
            .get(2)
            .ok_or("Could not access RS2")
            .map_err(|_e| format!("Could not access RS2!"))?
            .clone();
        Ok(ret)
    }

    pub fn get_rs2_index(&self) -> Result<u64, String>
    {
        let ret = self
            .register_args
            .get(2)
            .ok_or("Could not access RS2")
            .map_err(|_e| format!("Could not access RS2!"))?
            .clone();
        Ok(ret.index as u64)
    }
    
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Default)]
pub struct Word
{
    pub tag: TypeLabel,
    pub data: u40,
}

impl Word
{
    pub fn from_bin(bin: u64) -> Result<Word, String>
    {
        if (bin >> 46) != 0
        {
            return Err(format!("Binary value of {bin} is too large to be a valid word").to_string());
        }

        Ok(Word { tag: extract_tag(bin)?, data: u40::new(extract_value(bin)) })
    }

    pub fn new(tag: TypeLabel, data: u64) -> Result<Word, String>
    {
        if data >> 40 != 0
        {
            return Err(format!("Data value {data:#x} doesn't fit in 40 bits"));
        }
        Ok(Word { tag, data: u40::new(data) })
    }

    pub fn to_u64(self) -> u64
    {
        ((self.tag as u64) << 40) | (self.data.value())
    }
}

pub fn extract_bits(bin: u64, hi: u64, low: u64) -> u64
{
    assert!(hi >= low && hi < 64, "Error: Invalid bit range {hi} downto {low}");
    let width = hi - low + 1;
    let mask = u64::MAX >> (64 - width);
    (bin >> low) & mask
}

pub fn insert_bits(bin: u64, field: u64, hi: u64, low: u64) -> u64
{
    assert!(hi >= low && hi < 64, "Error: Invalid bit range {hi} downto {low}");
    let width = hi - low + 1;
    assert!(width == 64 || field >> width == 0, "Error: Field {field} does not fit in {width} bits");
    let mask = (u64::MAX >> (64 - width)) << low;
    (bin & !mask) | (field << low)
}

pub fn extract_tag(bin: u64) -> Result<TypeLabel, String>
{
    let tag_bin = extract_bits(bin, 45, 40);
    let tag_label: TypeLabel =
        TypeLabel::try_from_primitive(tag_bin).map_err(|e| format!("Could not convert to type label: {e}"))?;
    Ok(tag_label)
}

pub fn insert_taglabel(bin: &mut u64, field: TypeLabel)
{
    let hi = END_TAG_FIELD;
    let low = START_TAG_FIELD;
    let field: u64 = field.into();
    assert!(field < 2u64.pow(6) - 1);
    assert!(hi > low && hi <= 64, "Error: Invalid bit range {low}..{hi}");
    let width = hi - low + 1;
    assert!(width == 64 || field >> width == 0, "Error: Field {field} does not fit in {width} bits");
    let mask = (u64::MAX >> (64 - width)) << low;
    *bin = (*bin & !mask) | (field << low);
}

pub fn insert_tag(bin: &mut u64, tag: TypeLabel)
{
    let hi = END_TAG_FIELD;
    let low = START_TAG_FIELD;
    let field: u64 = tag.into();
    assert!(field < 2u64.pow(6) - 1);
    assert!(hi > low && hi <= 64, "Error: Invalid bit range {low}..{hi}");
    let width = hi - low + 1;
    assert!(width == 64 || field >> width == 0, "Error: Field {field} does not fit in {width} bits");
    let mask = (u64::MAX >> (64 - width)) << low;
    *bin = (*bin & !mask) | (field << low);
}

pub fn extract_opcode(bin: u64) -> u64
{
    let hi = END_OPCODE_FIELD;
    let low = START_OPCODE_FIELD;
    assert!(hi > low && hi <= 64, "Error: Invalid bit range {low}..{hi}");
    let width = hi - low + 1;
    let mask = u64::MAX >> (64 - width);
    (bin >> low) & mask
}

pub fn extract_value(bin: u64) -> u64
{
    let hi = END_DATA_FIELD;
    let low = START_DATA_FIELD;
    assert!(hi > low && hi <= 64, "Error: Invalid bit range {low}..{hi}");
    let width = hi - low + 1;
    let mask = u64::MAX >> (64 - width);
    (bin >> low) & mask
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



pub fn hex_to_word(hex: &str) -> Result<Word, String>
{
    const RADIX: u32 = 16;
    let bin: u46 = u46::try_new(u64::from_str_radix(hex, RADIX).map_err(|e| format!("Could not parse hex: {e}"))?)
        .map_err(|e| format!("{e}"))?;
    let ret = Word {
        tag: TypeLabel::try_from_primitive(extract_bits(bin.value(), END_TAG_FIELD, START_TAG_FIELD))
            .map_err(|e| format!("{e}"))?,
        data: u40::try_new(extract_bits(bin.value(), END_DATA_FIELD, START_DATA_FIELD)).map_err(|e| format!("{e}"))?,
    };
    Ok(ret)
}

pub fn to_i40(x: u40) -> i40
{
    i40::new(x.value() as i64)
}

pub fn itype_to_rtype_equivalent(label: InstructionLabel) -> Result<InstructionLabel, String>
{
    match label
    {
        InstructionLabel::ADDI => Ok(InstructionLabel::ADD),
        InstructionLabel::XORI => Ok(InstructionLabel::XOR),
        InstructionLabel::ORI => Ok(InstructionLabel::OR),
        InstructionLabel::ANDI => Ok(InstructionLabel::AND),
        InstructionLabel::SHRI => Ok(InstructionLabel::SHR),
        InstructionLabel::SHLI => Ok(InstructionLabel::SHL),
        InstructionLabel::SLTI => Ok(InstructionLabel::SLT),
        _others =>
        {
            return Err("No R-type Equivalent to break into or its LUI/LI. Value must be too large to fit.".to_string());
        }
    }
}
