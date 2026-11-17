use super::*;


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ImmVal
{
    Short(u18),
    Long(u23),
    Full(u40),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Imm
{
    pub imm_val: ImmVal,
    pub datatype: ImmTypeLabel,
}
impl Imm
{
    fn sext_to_i40(&self) -> i40
    {
        assert!(SIGNED_ARIMETHIC_TYPES_IMM.contains(&self.datatype));
        match self.imm_val
        {
            ImmVal::Short(uint) => i40::masked_new(((uint.value() as i64) << 46) >> 46),
            ImmVal::Long(uint) => i40::masked_new(((uint.value() as i64) << 41) >> 41),
            ImmVal::Full(uint) => i40::from_bits(uint.value()),
        }
    }

    pub fn to_u40(&self) -> u40
    {
        if SIGNED_ARIMETHIC_TYPES_IMM.contains(&self.datatype)
        {
            u40::masked_new(self.sext_to_i40())
        }
        else
        {
            match self.imm_val
            {
                ImmVal::Short(uint) => u40::new(uint.value() as u64),
                ImmVal::Long(uint) => u40::new(uint.value() as u64),
                ImmVal::Full(uint) => u40::new(uint.value() as u64),
            }
        }
    }

    pub fn to_word(&self) -> Word
    {
        Word { tag: self.datatype.to_type_label().unwrap(), data: self.to_u40() }
    }
}


#[derive(Clone, PartialEq, Eq, Debug, Copy)]
pub enum Operands
{
    RType3
    {
        rd: u5, rs1: u5, rs2: u5
    },
    RType2
    {
        rd: u5, rs1: u5
    },
    IType3
    {
        rd: u5, rs1: u5, imm: Imm
    },
    IType2
    {
        rd: u5, imm: Imm
    },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TokenStruct
{
    pub label: InstructionLabel,
    pub group: InstructionGroup,
    pub operands: Operands,
}
