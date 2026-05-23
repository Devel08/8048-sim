#[derive(Debug)]
pub enum I8048Error {
    InvalidOperand(String),
    InvalidInstruction(String),
    InvalidRegister(String),
    InvalidAddress(String),
}

//return the instruction and its operands. (mov) and (r0,#$24)
fn get_instruction(instruction: &str) -> (String, String) {
    let mut parts = instruction.splitn(2, ' ');

    let ins = parts.next().unwrap_or("");
    let operands = parts.next().unwrap_or("").replace(" ", "");
    (ins.to_string(), operands)
}

//split the operands and return them splitted. (r0) (#$24)
fn get_operands(operands: &str) -> (&str, &str) {
    match operands.trim().split_once(',') {
        Some((a, b)) => (a.trim(), b.trim()),
        None => (operands.trim(), ""),
    }
}

fn get_instruction_size(instruction: &str) -> usize {
    if get_operands(&get_instruction(instruction).1)
        .1
        .contains('#')
    {
        2
    } else {
        1
    }
}

// extracts the register from string and returns it as u8
// "r5" -> 5 (u8)
fn get_register(register: &str) -> Result<usize, I8048Error> {
    let num_str: String = register.chars().filter(|c| c.is_ascii_digit()).collect();
    let reg: usize = num_str
        .parse()
        .map_err(|_| I8048Error::InvalidRegister(register.to_string()))?;
    if reg < 8 {
        Ok(reg)
    } else {
        Err(I8048Error::InvalidRegister(reg.to_string()))
    }
}

// parses data
// #F8H -> F8
// #13 -> 13
// #0101B -> 5
fn parse_data(data: &str) -> Result<u8, I8048Error> {
    if data.len() < 2 || !data.starts_with('#') {
        return Err(I8048Error::InvalidOperand(data.to_string()));
    }

    match data.chars().last() {
        Some('h') | Some('H') => {
            let trimmed = &data[1..data.len() - 1];
            u8::from_str_radix(trimmed, 16)
                .map_err(|_| I8048Error::InvalidOperand(data.to_string()))
        }
        Some('b') | Some('B') => {
            let trimmed = &data[1..data.len() - 1];
            u8::from_str_radix(trimmed, 2).map_err(|_| I8048Error::InvalidOperand(data.to_string()))
        }
        Some(c) if c.is_ascii_digit() => {
            let trimmed = &data[1..];
            trimmed
                .parse::<u8>()
                .map_err(|_| I8048Error::InvalidOperand(data.to_string()))
        }
        _ => Err(I8048Error::InvalidOperand(data.to_string())),
    }
}

struct I8048instruction {
    instruction: String,
    size: usize,
}

impl I8048cpu {
    pub fn new() -> Self {
        Self {
            registers1: [0; 8],
            registers2: [0; 8],
            accumulator: 0,
            active_register_bank: 0, // 0 - bank 1, 1 - bank 2
            instructions: Vec::new(),
            pc: 0,
        }
    }

    pub fn get_active_registers_bank(&mut self) -> &mut [u8; 8] {
        if self.active_register_bank == 0 {
            &mut self.registers1
        } else {
            &mut self.registers2
        }
    }

    // 7 address -> fourth instruction
    fn get_instruction_address(&self, add: usize) -> Result<usize, I8048Error> {
        let mut a: usize = 0;

        for i in 0..self.instructions.len() {
            if a == add {
                return Ok(i);
            }
            a += self.instructions[i].size;
        }
        Err(I8048Error::InvalidAddress(add.to_string()))
    }

    fn ins_add(&mut self, operands: (&str, &str)) -> Result<(), I8048Error> {
        if operands.0.to_lowercase().as_str() == "a" {
            if operands.1.to_lowercase().starts_with('r') {
                // ADD A, Rx
                self.accumulator += self.get_active_registers_bank()[get_register(operands.1)?];
                Ok(())
            } else {
                // ADD A, #4
                self.accumulator += parse_data(operands.1)?;
                Ok(())
            }
        } else {
            Err(I8048Error::InvalidOperand(operands.0.to_string()))
        }
    }

    fn ins_mov(&mut self, operands: (&str, &str)) -> Result<(), I8048Error> {
        match operands.0 {
            //if op1 is accumulator
            "a" | "A" => {
                //if we copy a register to the accumulator.

                if operands.1.starts_with('r') || operands.1.starts_with('R') {
                    // MOV A, Rx
                    self.accumulator = self.get_active_registers_bank()[get_register(operands.1)?];
                    Ok(())
                } else if operands.1.starts_with('#') {
                    // MOV A, #4
                    self.accumulator = parse_data(operands.1)?;
                    Ok(())
                } else if operands.1.starts_with('@') {
                    Ok(())
                    // TODO implement memory address operand
                } else {
                    Err(I8048Error::InvalidOperand(operands.1.to_string()))
                }
            }
            //if op1 is a register
            s if s.starts_with('r') || s.starts_with('R') => {
                //if we copy the accumulator to a register.
                if operands.1 == "a" || operands.1 == "A" {
                    //MOV Rx, A
                    self.get_active_registers_bank()[get_register(operands.0)?] = self.accumulator;
                    Ok(())
                } else if operands.1.starts_with('#') {
                    // if it's not an accumulator, then it is data.
                    self.get_active_registers_bank()[get_register(operands.0)?] =
                        parse_data(operands.1)?;
                    Ok(())
                } else if operands.1.starts_with('@') {
                    Ok(())
                    // TODO implement memory address operand
                } else {
                    Err(I8048Error::InvalidOperand(operands.1.to_string()))
                }
            }

            _ => Err(I8048Error::InvalidOperand(operands.0.to_string())),
        }
    }

    fn ins_sel(&mut self, operands: (&str, &str)) -> Result<(), I8048Error> {
        match operands.0.to_lowercase().as_str() {
            "rb0" => {
                self.active_register_bank = 0;
                Ok(())
            }
            "rb1" => {
                self.active_register_bank = 1;
                Ok(())
            }
            "mb0" => {
                // TODO to be implemented
                Ok(())
            }
            "mb1" => {
                // TODO to be implemented
                Ok(())
            }
            _ => Err(I8048Error::InvalidOperand(operands.0.to_string())),
        }
    }
    // TODO ^ add SEL MB0 and SEL MB1 for memory banks

    fn ins_inc(&mut self, operands: (&str, &str)) -> Result<(), I8048Error> {
        if operands.0.to_lowercase().as_str().starts_with('r') {
            self.get_active_registers_bank()[get_register(operands.0)?] += 1;
            Ok(())
        } else if operands.0.to_lowercase().as_str() == "a" {
            self.accumulator += 1;
            Ok(())
        } else {
            Err(I8048Error::InvalidOperand(operands.0.to_string()))
        }
    }

    fn ins_dec(&mut self, operands: (&str, &str)) -> Result<(), I8048Error> {
        if operands.0.starts_with('r') || operands.0.starts_with('R') {
            self.get_active_registers_bank()[get_register(operands.0)?] -= 1;
            Ok(())
        } else if operands.0 == "A" || operands.0 == "a" {
            self.accumulator -= 1;
            Ok(())
        } else {
            Err(I8048Error::InvalidOperand(operands.0.to_string()))
        }
    }

    fn ins_jmp(&mut self, operands: (&str, &str)) -> Result<(), I8048Error> {
        let address = self.get_instruction_address(
            operands
                .0
                .parse()
                .map_err(|_| I8048Error::InvalidOperand(operands.0.to_string()))?,
        );
        match address {
            Err(err) => return Err(err),
            Ok(_) => {
                self.pc = operands
                    .0
                    .parse()
                    .map_err(|_| I8048Error::InvalidOperand(operands.0.to_string()))?;
            }
        }
        Ok(())
    }

    fn parse_instruction(&mut self, instruction: String) -> Result<(), I8048Error> {
        let (ins, operands) = get_instruction(&instruction);
        // let (op1, op2) = get_operands(&operands);
        // println!("{0} {1},{2}", ins, op1, op2);
        match ins.to_lowercase().as_str() {
            "mov" => {
                self.ins_mov(get_operands(&operands))?;
                Ok(())
            }
            "add" => {
                self.ins_add(get_operands(&operands))?;
                Ok(())
            }
            "sel" => {
                self.ins_sel(get_operands(&operands))?;
                Ok(())
            }
            "inc" => {
                self.ins_inc(get_operands(&operands))?;
                Ok(())
            }
            "dec" => {
                self.ins_dec(get_operands(&operands))?;
                Ok(())
            }
            "jmp" => {
                self.ins_jmp(get_operands(&operands))?;
                Ok(())
            }
            _ => Err(I8048Error::InvalidInstruction(ins.to_string())),
        }
    }

    fn push_instruction(&mut self, instruction: String) {
        let ins = I8048instruction {
            instruction: instruction.clone(),
            size: get_instruction_size(instruction.as_str()),
        };
        self.instructions.push(ins);
    }

    // 10 instructions -> 28 bytes
    fn get_taken_memory(&mut self) -> usize {
        let mut mem: usize = 0;

        for i in 0..self.instructions.len() {
            mem += self.instructions[i].size;
        }
        mem
    }

    fn run(&mut self) -> Result<(), I8048Error> {
        while self.pc < self.get_taken_memory() {
            let current_address = self.get_instruction_address(self.pc)?;
            let val = self.instructions[current_address].instruction.clone();



            let size = self.instructions[current_address].size;

            self.pc += size;

                        self.parse_instruction(val.clone())?;
            dbg!(self.pc, current_address, val.clone());
            dbg!("-------------------------------------");
        }
        Ok(())
    }
}

struct I8048cpu {
    registers1: [u8; 8], // bank of registers 1
    registers2: [u8; 8], // bank of registers 2
    active_register_bank: u8,
    accumulator: u8,
    instructions: Vec<I8048instruction>,
    pc: usize,
}

use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
fn main() -> Result<(), I8048Error> {
    let mut i8048 = I8048cpu::new();
    println!("MCS-48 emulator");

    let mut input = String::new();
    let mut option = String::new();
    println!(
        "Enter 'manual' to enter instructions by hand or a file name to parse instructions from file."
    );
    let _ = io::stdin().read_line(&mut option);
    let cleaned_input = option.trim();
    if cleaned_input == "manual" {
        //if choosed to manually enter instructions
        loop {
            print!(">");
            let _ = io::stdout().flush();
            input.clear();

            let _ = io::stdin().read_line(&mut input);
            let cleaned_input = input.trim();

            if input.trim() == "end" {
                break;
            }

            i8048.push_instruction(cleaned_input.to_string());
        }
    } else {
        //reading instructions from file
        let file = File::open(cleaned_input).expect("Failed to open the file");

        let reader = BufReader::new(file);

        for line_result in reader.lines() {
            let line = line_result.expect("Failed to read line");
            i8048.push_instruction(line);
        }
    }

    i8048.run()?;

    println!("Registers bank 1 are: {:?}", i8048.registers1);
    println!("Registers bank 2 are: {:?}", i8048.registers2);
    println!("Accumulator is: {:?}", i8048.accumulator);
    println!("Program counter is: {:?}", i8048.pc);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mov_immediate_to_accumulator() {
        let mut cpu = I8048cpu::new();

        cpu.push_instruction("MOV A, #9".to_string());

        _ = cpu.run();
        assert_eq!(cpu.accumulator, 9);
    }

    #[test]
    fn test_mov_accumulator_to_register() {
        let mut cpu = I8048cpu::new();

        cpu.push_instruction("MOV A, #42".to_string());
        cpu.push_instruction("MOV R7, A".to_string());
        _ = cpu.run();
        assert_eq!(cpu.get_active_registers_bank()[7], 42);
    }

    #[test]
    fn test_mov_register_to_accumulator() {
        let mut cpu = I8048cpu::new();
        cpu.push_instruction("MOV R4, #15".to_string());
        cpu.push_instruction("MOV A, R4".to_string());
        _ = cpu.run();
        assert_eq!(cpu.accumulator, 15);
    }

    #[test]
    fn test_mov_decimal_data() {
        let mut cpu = I8048cpu::new();

        cpu.push_instruction("MOV A, #14".to_string());
        _ = cpu.run();
        assert_eq!(cpu.accumulator, 14);
    }

    #[test]
    fn test_mov_hex_data() {
        let mut cpu = I8048cpu::new();

        cpu.push_instruction("MOV R4, #FAH".to_string());
        _ = cpu.run();
        assert_eq!(cpu.get_active_registers_bank()[4], 0xFA);
    }

    #[test]
    fn test_mov_binary_data() {
        let mut cpu = I8048cpu::new();

        cpu.push_instruction("MOV A, #0101B".to_string());
        _ = cpu.run();
        assert_eq!(cpu.accumulator, 5);
    }

    #[test]
    fn test_multiple_instructions() {
        let mut cpu = I8048cpu::new();

        cpu.push_instruction("MOV A, #15".to_string());
        cpu.push_instruction("MOV R1, A".to_string());
        _ = cpu.run();
        assert_eq!(cpu.accumulator, 15);
        assert_eq!(cpu.get_active_registers_bank()[1], 15);
    }

    #[test]
    fn test_register_banks() {
        let mut cpu = I8048cpu::new();

        // bank register 1, R1=15
        cpu.push_instruction("MOV A, #15".to_string());
        cpu.push_instruction("MOV R1, A".to_string());
        _ = cpu.run();
        assert_eq!(cpu.get_active_registers_bank()[1], 15);
        // bank register 2, R1 = 16
        cpu.push_instruction("SEL RB1".to_string());
        cpu.push_instruction("MOV A, #16".to_string());
        cpu.push_instruction("MOV R1, A".to_string());
        _ = cpu.run();
        assert_eq!(cpu.get_active_registers_bank()[1], 16);
    }

    #[test]
    fn test_add_register_to_accumulator() {
        let mut cpu = I8048cpu::new();

        // A = 15, add it to R1
        cpu.push_instruction("MOV A, #15".to_string());
        cpu.push_instruction("MOV R1, A".to_string());
        // reset accumulator to 0
        cpu.push_instruction("MOV A, #0".to_string());
        // A = 16
        cpu.push_instruction("MOV A, #16".to_string());
        // A = A (16) + R1 (15)
        cpu.push_instruction("ADD A, R1".to_string());
        _ = cpu.run();
        assert_eq!(cpu.accumulator, 31);
    }

    #[test]
    fn test_add_value_to_accumulator() {
        let mut cpu = I8048cpu::new();

        // add 0xFA to accumulator
        cpu.push_instruction("ADD A, #FAH".to_string());
        _ = cpu.run();
        assert_eq!(cpu.accumulator, 0xFA);
    }

    #[test]
    fn test_increment_accumulator() {
        let mut cpu = I8048cpu::new();

        // increment accumulator
        cpu.push_instruction("INC A".to_string());
        _ = cpu.run();
        assert_eq!(cpu.accumulator, 1);
    }

    #[test]
    fn test_increment_register() {
        let mut cpu = I8048cpu::new();

        // increment r7
        cpu.push_instruction("INC R7".to_string());
        _ = cpu.run();
        assert_eq!(cpu.get_active_registers_bank()[7], 1);
    }

    #[test]
    fn test_decrement_accumulator() {
        let mut cpu = I8048cpu::new();
        //set A to 16
        cpu.push_instruction("MOV A, #16".to_string());
        // decrement A
        cpu.push_instruction("DEC A".to_string());
        _ = cpu.run();
        assert_eq!(cpu.accumulator, 15);
    }

    #[test]
    fn test_decrement_register() {
        let mut cpu = I8048cpu::new();
        //set R7 to 16
        cpu.push_instruction("MOV R7, #16".to_string());
        // decrement r7
        cpu.push_instruction("DEC R7".to_string());
        _ = cpu.run();
        assert_eq!(cpu.get_active_registers_bank()[7], 15);
    }
}
