use arbitrary_int::u40;
use num_enum::TryFromPrimitive;


use super::*;
use crate::types::labels::{*};

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

        Ok(Word {
            tag: TypeLabel::try_from_primitive(extract_bits_u64(bin, TAG_FIELD)).map_err(|e| e.to_string())?,
            data: u40::new(extract_bits_u64(bin, DATA_FIELD)),
        })
    }

    pub fn to_u64(self) -> u64
    {
        ((self.tag as u64) << 40) | (self.data.value())
    }

    pub fn get_data(self) -> u40
    {
        self.data
    }

    fn set_rd(&mut self, index: u5)
    {
        assert_eq!(extract_bits(self.data, RD_FIELD), u40::new(0));
        self.data = insert_bits(self.data, index.value() as u64, RD_FIELD);
    }

    fn set_rs1(&mut self, index: u5)
    {
        assert_eq!(extract_bits(self.data, RS1_FIELD), u40::new(0));
        self.data = insert_bits(self.data, index.value() as u64, RS1_FIELD);
    }

    fn set_rs2(&mut self, index: u5)
    {
        assert_eq!(extract_bits(self.data, RS2_FIELD), u40::new(0));
        self.data = insert_bits(self.data, index.value() as u64, RS2_FIELD);
    }

    fn get_rd(self) -> u5
    {
        u5::new(extract_bits(self.data, RD_FIELD).value() as u8)
    }

    fn get_rs1(self) -> u5
    {
        u5::new(extract_bits(self.data, RS1_FIELD).value() as u8)
    }

    fn get_rs2(self) -> u5
    {
        u5::new(extract_bits(self.data, RS2_FIELD).value() as u8)
    }

    fn get_imm(self) -> Result<Imm, String>
    {
        assert_eq!(self.get_label()?.group(), InstructionGroup::ITYPE);
        let arg_count = self.get_label()?.arg_count();
        let imm: Imm = match arg_count
        {
            2 =>
            {
                let bin = self.data;
                let imm_type = <ImmTypeLabel>::try_from(extract_bits(bin, LONG_IMM_TAG_FIELD).value())
                    .map_err(|e| format!("Could not map immediate to type: {e}"))?;
                let imm_val = u23::new(extract_bits(bin, LONG_IMM_VAL_FIELD).value() as u32);
                Imm { imm_val: ImmVal::Long(imm_val), datatype: imm_type }
            }
            3 =>
            {
                let bin = self.data;
                let imm_type = <ImmTypeLabel>::try_from(extract_bits(bin, SHORT_IMM_TAG_FIELD).value())
                    .map_err(|e| format!("Could not map immediate to type: {e}"))?;
                let imm_val = u18::new(extract_bits(bin, SHORT_IMM_VAL_FIELD).value() as u32);
                Imm { imm_val: ImmVal::Short(imm_val), datatype: imm_type }
            }
            _others => panic!("How'd you get here?"),
        };
        Ok(imm)
    }

    fn get_label(self) -> Result<InstructionLabel, String>
    {
        let bin = self.data;
        let label = InstructionLabel::try_from_primitive(extract_bits(bin, OPCODE_FIELD).value())
            .map_err(|e| format!("Couldn't map opcode to instruction!: {e}"))?;
        Ok(label)
    }

    pub fn disassemble(self) -> Result<TokenStruct, String>
    {
        let bits: u40 = self.data;
        let label = InstructionLabel::try_from_primitive(extract_bits(bits, OPCODE_FIELD).value())
            .map_err(|e| format!("Couldn't map opcode to instruction!: {e}"))?;
        let group: InstructionGroup = label.group();

        let arg_count = label.arg_count();
        let rd = self.get_rd();

        match (group, arg_count)
        {
            (types::labels::InstructionGroup::RTYPE, 3) =>
            {
                let rs1 = self.get_rs1();
                let rs2 = self.get_rs2();
                Ok(TokenStruct { label, group, operands: Operands::RType3 { rd, rs1, rs2 } })
            }
            (types::labels::InstructionGroup::RTYPE, 2) =>
            {
                let rs1 = self.get_rs1();
                Ok(TokenStruct { label, group, operands: Operands::RType2 { rd, rs1 } })
            }
            (types::labels::InstructionGroup::ITYPE, 3) =>
            {
                let rs1 = self.get_rs1();
                let imm = self.get_imm()?;
                Ok(TokenStruct { label, group, operands: Operands::IType3 { rd, rs1, imm } })
            }
            (types::labels::InstructionGroup::ITYPE, 2) =>
            {
                let imm = self.get_imm()?;
                Ok(TokenStruct { label, group, operands: Operands::IType2 { rd, imm } })
            }
            _others => todo!("not done yet"),
        }
    }

    pub fn set_imm(&mut self, imm: Imm)
    {
        fn sign_extend(v: u64) -> u64
        {
            ((((v << 46) as i64) >> 46) as u64) & LONG_IMM_MASK23
        }

        let op_bits = extract_bits(self.data, OPCODE_FIELD).value();
        let is_load = op_bits == u64::from(InstructionLabel::LI) || op_bits == u64::from(InstructionLabel::LUI);

        let (imm_val, imm_field, tag_start) = match imm.imm_val
        {
            ImmVal::Short(uint) =>
            {
                let v = uint.value() as u64;
                if is_load
                {
                    // LI/LUI: short value goes in the long field
                    let v = if SIGNED_ARIMETHIC_TYPES_IMM.contains(&imm.datatype) { sign_extend(v) } else { v };
                    (v, LONG_IMM_FIELD, LONG_IMM_TAG_FIELD.low())
                }
                else
                {
                    (v, SHORT_IMM_FIELD, SHORT_IMM_TAG_FIELD.low())
                }
            }
            ImmVal::Long(uint) => (uint.value() as u64, LONG_IMM_FIELD, LONG_IMM_TAG_FIELD.low()),
            ImmVal::Full(_) => panic!("Why are you trying to insert a long into a single word?"),
        };

        let imm_bin = imm_val | (<u64>::from(imm.datatype) << tag_start);

        assert_eq!(extract_bits(self.data, imm_field), u40::new(0));
        self.data = insert_bits(self.data, imm_bin, imm_field)
    }

    fn set_opcode(&mut self, op: InstructionLabel)
    {
        assert_eq!(extract_bits(self.data, OPCODE_FIELD), u40::new(0));
        let op_bin = u64::from(op);
        self.data = insert_bits(self.data, op_bin, OPCODE_FIELD);
    }

    pub fn construct_rtype_3(&mut self, op: InstructionLabel, rd: u5, rs1: u5, rs2: u5)
    {
        self.tag = TypeLabel::INSTRUCTION;
        self.set_opcode(op);
        self.set_rd(rd);
        self.set_rs1(rs1);
        self.set_rs2(rs2);
    }

    pub fn construct_rtype_2(&mut self, op: InstructionLabel, rd: u5, rs1: u5)
    {
        self.tag = TypeLabel::INSTRUCTION;
        self.set_opcode(op);
        self.set_rd(rd);
        self.set_rs1(rs1);
    }

    pub fn construct_itype_3(&mut self, op: InstructionLabel, rd: u5, rs1: u5, imm: Imm)
    {
        self.tag = TypeLabel::INSTRUCTION;
        self.set_opcode(op);
        self.set_rd(rd);
        self.set_rs1(rs1);
        self.set_imm(imm);
    }

    pub fn construct_itype_2(&mut self, op: InstructionLabel, rd: u5, imm: Imm)
    {
        self.tag = TypeLabel::INSTRUCTION;
        self.set_opcode(op);
        self.set_rd(rd);
        self.set_imm(imm);
    }

    pub fn get_i40(&self) -> i40
    {
        i40::from_bits(self.data.value())
    }

    pub fn get_u40(&self) -> u40
    {
        self.data
    }
}
