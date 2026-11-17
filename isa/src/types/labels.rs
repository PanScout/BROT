use num_enum::{IntoPrimitive, TryFromPrimitive};
use strum::{EnumCount, EnumIter, EnumString, IntoStaticStr};

#[repr(u64)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive, EnumString, IntoStaticStr, EnumCount)]
pub enum InstructionGroup {
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

pub enum TypeLabel {
    #[default]
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
    INERT,
}

#[repr(u64)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, IntoPrimitive, TryFromPrimitive, EnumString, IntoStaticStr, EnumIter)]
pub enum ImmTypeLabel {
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

impl ImmTypeLabel {
    pub fn type_label_to_imm_label(label: TypeLabel) -> Result<ImmTypeLabel, String> {
        match label {
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
            TypeLabel::FREE | TypeLabel::INSTRUCTION | TypeLabel::PTRB | TypeLabel::PTRF | TypeLabel::INERT => {
                Err(format!("{label:?} is not a valid immediate type"))
            }
        }
    }

    pub fn to_type_label(self) -> Result<TypeLabel, String> {
        match self {
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
            ImmTypeLabel::Q19_20 => Ok(TypeLabel::Q19_20),
        }
    }
}

#[repr(u64)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Copy, TryFromPrimitive, IntoPrimitive, EnumString, IntoStaticStr)]
pub enum InstructionLabel {
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
    SEQ,
    NEG,
    MUL,
    MULH,
    MAC,
    MV,
    EXTB,
    INSRTB,

    // I-type
    ADDI,
    SUBI,
    XORI,
    SEQI,
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

impl InstructionLabel {
    pub fn group(&self) -> InstructionGroup {
        use InstructionLabel::*;
        match self {
            ADD | SUB | XOR | OR | AND | NOT | SHL | SHR | SLT | SEQ | NEG | MUL | MULH | MAC | MV | EXTB | INSRTB => {
                InstructionGroup::RTYPE
            }

            ADDI | SUBI | XORI | SEQI | ORI | ANDI | SHRI | SHLI | SLTI | LI | LUI | INSRTBI => InstructionGroup::ITYPE,

            CONVERT => InstructionGroup::CTYPE,
        }
    }

    pub fn arg_count(&self) -> usize {
        use InstructionLabel::*;
        match self {
            NOT | NEG | MV | LI | LUI => 2,
            CONVERT => todo!(),
            _ => 3,
        }
    }

    pub fn itype_to_rtype_equivalent(&self) -> Result<InstructionLabel, String> {
        match &self {
            InstructionLabel::ADDI => Ok(InstructionLabel::ADD),
            InstructionLabel::XORI => Ok(InstructionLabel::XOR),
            InstructionLabel::ORI => Ok(InstructionLabel::OR),
            InstructionLabel::ANDI => Ok(InstructionLabel::AND),
            InstructionLabel::SHRI => Ok(InstructionLabel::SHR),
            InstructionLabel::SHLI => Ok(InstructionLabel::SHL),
            InstructionLabel::SLTI => Ok(InstructionLabel::SLT),
            _others => {
                Err("No R-type Equivalent to break into or its LUI/LI. Value must be too large to fit.".to_string())
            }
        }
    }
}
