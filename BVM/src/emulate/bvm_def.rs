use std::ffi::os_str::Display;

use arbitrary_int::u5;
use isa::Word;
use isa::constants::*;

#[derive(Debug, Clone)]
pub struct BvmState
{
    pub pc: usize,
    status: [bool; 16],
    gpr: [Word; MAX_GPR_VAL + 1],
    pr: [(Word, Word); 13],
    mem: Vec<Word>,
}

impl BvmState
{
    pub fn write_gpr(&mut self, val: Word, idx: u5) -> Result<(), String>
    {
        if idx == u5::new(0)
        {
            return Err(format!("\nIndex {idx} is writing to immutable 0 register!"));
        }

        self.gpr[idx.value() as usize] = val;
        Ok(())
    }

    pub fn read_gpr(&self, idx: u5) -> Result<Word, String>
    {
        Ok(self.gpr[idx.value() as usize])
    }

    pub fn get_status_index(&self, index: usize) -> bool
    {
        if index > 16
        {
            panic!("Status register out of bounds");
        }
        self.status[index]
    }
    pub fn get_status(&self) -> u64
    {
        let mut ret: u64 = 0;
        for (i, val) in self.status.into_iter().enumerate()
        {
            ret |= (val as u64) << i;
        }
        ret
    }
}

impl Default for BvmState
{
    fn default() -> Self
    {
        let mem : Vec<Word> = Vec::with_capacity(65_536);
        Self {
            pc: Default::default(),
            status: Default::default(),
            gpr: Default::default(),
            pr: Default::default(),
            mem
        }
    }
}
