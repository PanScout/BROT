use arbitrary_int::u40;
use isa as brot_isa;

use brot_isa::{*};


pub struct BvmState {
    pub pc: usize,
    status: u64,
    gpr: [Word; MAX_GPR_VAL + 1],
    pr: [u128; 13],
    mem: Vec<u64>,
}

impl BvmState {
    pub fn new() -> BvmState
    {
        let mut ret = BvmState{ pc: 0, status: 0, gpr: [Word{tag: TypeLabel::FREE, data: u40::new(0) }; 32], pr: [0 ; 13], mem: Vec::new() };
        ret.gpr[0].data = u40::new(0);
        ret.gpr[0].tag = TypeLabel::UINT;
        ret

    }

    pub fn write_gpr(&mut self, val: Word, idx: usize) -> Result<(), String> {


        if idx > MAX_GPR_VAL
        {
            return Err(format!("\nIndex {idx} is out of bounds for register write!"));
        }

        if idx == 0
        {
            return Err(format!("\nIndex {idx} is writing to immutable 0 register!"));
        }


        self.gpr[idx] = val;
        Ok(()) 
    }

    pub fn read_gpr(&self, idx: usize) -> Result<Word, String>{
        
        if idx >  MAX_GPR_VAL
        {
            return Err(format!("\nIndex {idx} is out of bounds for register write!"));
        }

        Ok(self.gpr[idx])
    }

    pub fn get_status(&self) -> u64 {
        self.status
    }
}
