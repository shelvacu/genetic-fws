use std::fmt;
use rand::Rng;
// The Shelvacu Fast Word Square VM: SFWSVM -> SVM
// registers (32-bit each)
// 00 output
// 01..16 input
// 16..64 work registers, initialized to 0

const SVM_NUM_REGISTERS:usize = 32;
type SvmRegister = u32;
type SvmMemory = Box<[SvmRegister; SVM_NUM_REGISTERS]>;

#[derive(Debug,PartialEq,Eq,Clone,Copy,PartialOrd,Ord)]
pub enum SvmInstructionTy {
    Xor,
    Add,
    Sub,
    And,
    Oor,
    Mov,
    Shl,
    Shr,
    Seb, //set bit
    Clb, //clear bit
    Jis, //jump forward one instruction if bit is set
    Jns, //jump if not set
}

impl SvmInstructionTy {
    fn random<R: Rng + ?Sized>(rng: &mut R) -> Self {
        use SvmInstructionTy::*;
        match rng.gen_range(0..12) {
            0 => Xor,
            1 => Add,
            2 => Sub,
            3 => And,
            4 => Oor,
            5 => Mov,
            6 => Shl,
            7 => Shr,
            8 => Seb,
            9 => Clb,
            10=> Jis,
            11=> Jns,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug,PartialEq,Eq,Clone,Copy,PartialOrd,Ord)]
pub struct SvmInstruction {
    pub ty: SvmInstructionTy,
    pub dest: u8,
    pub src: u8,
}

impl SvmInstruction {
    pub fn random<R: Rng + ?Sized>(rng: &mut R) -> Self {
        SvmInstruction {
            ty: SvmInstructionTy::random(rng),
            dest: rng.gen_range(0..32),
            src: rng.gen_range(0..32),
        }
    }
}

impl fmt::Display for SvmInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?} {} {})",self.ty,self.dest,self.src)
    }
}

#[derive(Debug)]
pub struct SvmState<I: Iterator<Item=SvmInstruction> + fmt::Debug> {
    instructions: I,
    memory: SvmMemory,
}

#[must_use]
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum StepResult {
    Continue,
    Finish,
}

impl<I: Iterator<Item=SvmInstruction> + fmt::Debug> SvmState<I> {
    pub fn new(instructions: I) -> Self {
        let memory = Box::new([0u32; SVM_NUM_REGISTERS]);
        Self {
            instructions,
            memory,
        }
    }

    pub fn memory(&self) -> &[SvmRegister] {
        self.memory.as_slice()
    }

    pub fn memory_mut(&mut self) -> &mut [SvmRegister] {
        self.memory.as_mut_slice()
    }

    pub fn step(&mut self) -> StepResult {
        if let Some(ins) = self.instructions.next() {
            use SvmInstructionTy::*;
            match ins.ty {
                Xor => self.oper2(ins.dest, ins.src, |a,b| a^b),
                Add => self.oper2(ins.dest, ins.src, u32::wrapping_add),
                Sub => self.oper2(ins.dest, ins.src, u32::wrapping_sub),
                And => self.oper2(ins.dest, ins.src, |a,b| a&b),
                Oor => self.oper2(ins.dest, ins.src, |a,b| a|b),
                Mov => self.oper2(ins.dest, ins.src, |_,b| b),
                Shl => self.oper1(ins.dest, |a| a << ins.src),
                Shr => self.oper1(ins.dest, |a| a >> ins.src),
                Seb => self.oper1(ins.dest, |a| a |  (1 << ins.src)), //set bit
                Clb => self.oper1(ins.dest, |a| a & !(1 << ins.src)), //clear bit
                Jis => if self.get(ins.dest) & (1 << ins.src) != 0 {let _ = self.instructions.next();}, //jump forward one instruction if bit is set
                Jns => if self.get(ins.dest) & (1 << ins.src) == 0 {let _ = self.instructions.next();}, //jump if not set
            }
            return StepResult::Continue;
        } else {
            return StepResult::Finish;
        }
    }

    fn oper2(&mut self, dest:u8, src:u8, f: impl FnOnce(u32, u32) -> u32) {
        self.memory[dest as usize] = f(self.memory[dest as usize], self.memory[src as usize]);
    }

    fn oper1(&mut self, dest:u8, f: impl FnOnce(u32) -> u32) {
        self.memory[dest as usize] = f(self.memory[dest as usize]);
    }

    fn get(&self, i:u8) -> u32 {
        self.memory[i as usize]
    }
}