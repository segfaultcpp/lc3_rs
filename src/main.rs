use std::{*, env, io::{*}};
use libc;

#[link(name = "cpp/terminal_setup", kind = "static")]
extern {
    fn check_key() -> u16;
    fn setup_term() -> core::ffi::c_void;
    fn shutdown_term() -> core::ffi::c_void;
}

struct Terminal;

impl Terminal {
    fn setup() {
        unsafe { setup_term(); }
    }

    fn shutdown() {
        unsafe { shutdown_term(); }
    }

    fn check_key() -> u16 {
        unsafe { check_key() }
    }
}

struct Registers {
    regs: [u16; 8],
    pub pc: u16,
    pub cond: u16,
}

pub const PC_START: u16 = 0x3000;

const FL_POS: u16 = 1;
const FL_ZRO: u16 = 2;
const FL_NEG: u16 = 4;

impl Registers {
    pub fn new() -> Registers {
        Registers { regs: [0; 8], pc: PC_START, cond: FL_ZRO }
    }

    pub fn get(&self, idx: usize) -> &u16 {
        &self.regs[idx]
    }

    pub fn get_mut(&mut self, idx: usize) -> &mut u16 {
        &mut self.regs[idx]
    }

    pub fn update_cond_reg(&mut self, idx: usize) {
        if *self.get(idx) == 0 {
            self.cond = FL_ZRO;
        }
        else if (*self.get(idx) >> 15) != 0 {
            self.cond = FL_NEG;
        }
        else {
            self.cond = FL_POS;
        }
    }
}

impl std::ops::Index<usize> for Registers {
    type Output = u16;

    fn index(&self, idx: usize) -> &Self::Output {
        self.get(idx)
    }
}

impl std::ops::IndexMut<usize> for Registers {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        self.get_mut(idx)
    }
}

const OP_BR: u16 = 0;      // branch 
const OP_ADD: u16 = 1;     // add  
const OP_LD: u16 = 2;      // load 
const OP_ST: u16 = 3;      // store 
const OP_JSR: u16 = 4;     // jump register 
const OP_AND: u16 = 5;     // bitwise and 
const OP_LDR: u16 = 6;     // load register 
const OP_STR: u16 = 7;     // store register 
const OP_RTI: u16 = 8;     // unused 
const OP_NOT: u16 = 9;     // bitwise not 
const OP_LDI: u16 = 10;    // load indirect 
const OP_STI: u16 = 11;    // store indirect 
const OP_JMP: u16 = 12;    // jump 
const OP_RES: u16 = 13;    // reserved (unused) 
const OP_LEA: u16 = 14;    // load effective address 
const OP_TRAP: u16 = 15;   // execute trap 

const TRAP_GETC: u16 = 0x20;  // get character from keyboard, not echoed onto the terminal 
const TRAP_OUT: u16 = 0x21;   // output a character 
const TRAP_PUTS: u16 = 0x22;  // output a word string 
const TRAP_IN: u16 = 0x23;    // get character from keyboard, echoed onto the terminal 
const TRAP_PUTSP: u16 = 0x24; // output a byte string 
const TRAP_HALT: u16 = 0x25;  // halt the program 

const MR_KBSR: u16 = 0xFE00; // keyboard status
const MR_KBDR: u16 = 0xFE02; // keyboard data

const MEM_BUF_SIZE: usize = 1 << 16;

#[inline]
fn bit(shift: u16) -> u16 {
    1 << shift
}

#[inline]
fn test_bit(value: u16, bit: u16) -> bool {
    (value & bit) > 0
}

fn sign_extend(mut x: u16, bit_count: usize) -> u16 {
    if ((x >> (bit_count as u16 - 1)) & 1) > 0 {
        x |= 0xFFFF << bit_count;
    }
    x
}

struct VM {
    memory: [u16; MEM_BUF_SIZE],
    regs: Registers,
}

impl VM {
    fn new() -> VM {
        VM {
            memory: [0; MEM_BUF_SIZE],
            regs: Registers::new(),
        }
    }

    fn with_program(mut self, program: Vec<u16>) -> VM {
        self.load_program(program);
        self
    }

    fn load_program(&mut self, program: Vec<u16>) {
        let base_addr = program.first().unwrap();
        let begin = *base_addr as usize;
        let end = begin + program.len() - 1;
        let mem = self.memory[begin..end].iter_mut();

        for (slot, code) in mem.zip(program.iter().skip(1)) {
            *slot = *code;
        }
        self.regs.pc = *base_addr;
    }

    fn run(&mut self) {
        loop {
            let instr = self.read_from_mem(self.regs.pc);
            self.inc_pc();
            let op = instr >> 12;
            match op {
                OP_BR => {
                    let cond_flags = (instr >> 9) & 7;
                    let offset = sign_extend(instr & 0x1FF, 9);

                    if (cond_flags & self.regs.cond) > 0 {
                        let val = self.regs.pc as u32 + offset as u32;
                        self.regs.pc = val as u16;
                    }
                },
                OP_ADD => {
                    let dr = (instr >> 9) & 7;
                    let sr1 = (instr >> 6) & 7;
                    let imm_flag = test_bit(instr, bit(5));

                    if imm_flag {
                        let imm5 = sign_extend(instr & 0x1F, 5);
                        let val = self.regs[sr1.into()] as u32 + imm5 as u32;
                        self.regs[dr.into()] = val as u16;
                    }
                    else {
                        let sr2 = instr & 7;
                        let val = self.regs[sr1.into()] as u32 + self.regs[sr2.into()] as u32;
                        self.regs[dr.into()] = val as u16;
                    }
                    self.regs.update_cond_reg(dr.into());
                },
                OP_LD => {
                    let offset = sign_extend(instr & 0x1FF, 9);
                    let dr = (instr >> 9) & 7;
                    let addr = self.regs.pc as u32 + offset as u32;
                    self.regs[dr.into()] = self.read_from_mem(addr as u16);
                    self.regs.update_cond_reg(dr.into());
                },
                OP_ST => {
                    let offset = sign_extend(instr & 0x1FF, 9);
                    let sr = (instr >> 9) & 7;
                    let addr = self.regs.pc as u32 + offset as u32;
                    self.write_to_mem(addr as u16, self.regs[sr.into()]);
                },
                OP_JSR => {
                    self.regs[7] = self.regs.pc;

                    if !test_bit(instr, bit(11)) {
                        let base_reg = (instr >> 6) & 7;
                        self.regs.pc = self.regs[base_reg.into()];
                    }
                    else {
                        let offset = sign_extend(instr & 0x7FF, 11); 
                        let val = self.regs.pc as u32 + offset as u32;
                        self.regs.pc = val as u16;
                    }
                },
                OP_AND => {
                    let dr = (instr >> 9) & 7;
                    let sr1 = (instr >> 6) & 7;
                    let imm_flag = test_bit(instr, bit(5));

                    if imm_flag {
                        let imm5 = sign_extend(instr & 0x1F, 5);
                        self.regs[dr.into()] = self.regs[sr1.into()] & imm5;
                    }
                    else {
                        let sr2 = instr & 7;
                        self.regs[dr.into()] = self.regs[sr1.into()] & self.regs[sr2.into()];
                    }
                    self.regs.update_cond_reg(dr.into());
                },
                OP_LDR => {
                    let offset = sign_extend(instr & 0x3F, 6);
                    let base_reg = (instr >> 6) & 7;
                    let dr = (instr >> 9) & 7;

                    let addr = self.regs[base_reg.into()] as u32 + offset as u32;
                    self.regs[dr.into()] = self.read_from_mem(addr as u16);
                    self.regs.update_cond_reg(dr.into());
                },
                OP_STR => {
                    let offset = sign_extend(instr & 0x3F, 6);
                    let base_reg = (instr >> 6) & 7;
                    let sr = (instr >> 9) & 7;
                    let addr = self.regs[base_reg.into()] as u32 + offset as u32;
                    self.write_to_mem(addr as u16, self.regs[sr.into()]);
                },
                OP_NOT => {
                    let dr = (instr >> 9) & 7;
                    let sr = (instr >> 6) & 7;

                    self.regs[dr.into()] = !self.regs[sr.into()];
                    self.regs.update_cond_reg(dr.into());
                },
                OP_LDI => {
                    let dr = (instr >> 9) & 7;
                    let offset = sign_extend(instr & 0x1FF, 9);
                    let addr = self.regs.pc as u32 + offset as u32;
                    let addr = self.read_from_mem(addr as u16);
                    self.regs[dr.into()] = self.read_from_mem(addr);
                    self.regs.update_cond_reg(dr.into());
                },
                OP_STI => {
                    let offset = sign_extend(instr & 0x1FF, 9);
                    let sr = (instr >> 9) & 7;
                    let addr = self.regs.pc as u32 + offset as u32;
                    let addr = self.read_from_mem(addr as u16);
                    self.write_to_mem(addr, self.regs[sr.into()]);
                },
                OP_JMP => {
                    let base_reg = (instr >> 6) & 7;
                    self.regs.pc = self.regs[base_reg.into()];
                },
                OP_LEA => {
                    let offset = sign_extend(instr & 0x1FF, 9);
                    let dr = (instr >> 9) & 7;

                    let val = self.regs.pc as u32 + offset as u32;
                    self.regs[dr.into()] = val as u16;
                    self.regs.update_cond_reg(dr.into());
                },
                OP_TRAP => {
                    self.regs[7] = self.regs.pc;
                    let trap_type = instr & 0xFF;

                    match trap_type {
                        TRAP_GETC => {
                            let mut buffer = [0; 1];
                            std::io::stdin().read_exact(&mut buffer).unwrap();
                            self.regs[0] = buffer[0] as u16;
                            self.regs.update_cond_reg(0);
                        },
                        TRAP_OUT => {
                            let c = self.regs[0] as u8 as char;
                            print!("{c}");
                            io::stdout().flush().expect("Failed to flush");
                        },
                        TRAP_PUTS => {
                            let start_pos = self.regs[0] as usize;
                            let string = self.memory[start_pos..].iter()
                                .map(|c| *c as u8)
                                .take_while(|c| *c != 0);
                            
                            for c in string {
                                print!("{}", c as char);
                            }
                            io::stdout().flush().expect("Failed to flush");
                        },
                        TRAP_IN => {
                            print!("Enter a character: ");
                            let char = std::io::stdin()
                                .bytes()
                                .next()
                                .and_then(|result| result.ok())
                                .map(|byte| byte as u16)
                                .unwrap();
                            self.regs[0] = char;
                            self.regs.update_cond_reg(0);
                        },
                        TRAP_PUTSP => {
                            let begin = self.regs[0] as usize;
                            let string = self.memory[begin..].iter()
                                .take_while(|c| **c != 0)
                                .map(|c| {
                                    let c1 = (*c & 0xFF) as u8;
                                    let c2 = (*c >> 8) as u8;
                                    (c1, c2)
                                });
                            
                            for (c1, c2) in string {
                                print!("{}{}", c1 as char, c2 as char);
                            }
                            io::stdout().flush().expect("Failed to flush");
                        },
                        TRAP_HALT => {
                            println!("\nHALT");
                            io::stdout().flush().expect("Failed to flush");
                            break;
                        },
                        _ => unreachable!(),
                    }
                },

                OP_RTI => panic!(),
                OP_RES => panic!(),
                _ => unreachable!(),
            }
        }
    }

    fn inc_pc(&mut self) {
        self.regs.pc += 1;
    }

    fn read_from_mem(&mut self, offset: u16) -> u16 {
        if offset == MR_KBSR {
            if Terminal::check_key() != 0 {
                let mut buffer = [0; 1];
                std::io::stdin().read_exact(&mut buffer).unwrap();
                
                self.memory[MR_KBSR as usize] = 1 << 15;
                self.memory[MR_KBDR as usize] = buffer[0] as u16;
            } 
            else {
                self.memory[MR_KBSR as usize] = 0;
            }
        }

        self.memory[offset as usize]
    }

    fn write_to_mem(&mut self, offset: u16, value: u16) {
        self.memory[offset as usize] = value;
    }
}

fn program_form_u8(bin: Vec<u8>) -> Vec<u16> {
    bin
        .chunks_exact(2)
        .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
        .collect::<Vec<_>>()
}

fn load_program_from_file(path: &str) -> Vec<u16> {
    let bytes = std::fs::read(path).expect("Couldn't open file!");
    program_form_u8(bytes)
}

fn main() {
    Terminal::setup();

    let args = env::args().collect::<Vec<_>>();
    assert!(args.len() >= 2, "Expected file path to program");

    let program = load_program_from_file(args[1].as_str());
 
    let mut vm = VM::new().with_program(program);
    vm.run();

    Terminal::shutdown();
}
