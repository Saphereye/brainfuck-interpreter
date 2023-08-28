use clap::{App, Arg};
use color_eyre::eyre::Result;

use std::fs;
use std::io;

struct Cpu {
    instruction_pointer: usize,
    loop_stack: Vec<usize>,
    tape: Vec<u8>,
    data_pointer: usize,
    debug: bool,
    output: String,
}

impl Cpu {
    fn default(tape_size: usize, debug: bool) -> Self {
        Self {
            instruction_pointer: 0,
            loop_stack: Vec::new(),
            tape: vec![0; tape_size],
            data_pointer: 0,
            debug,
            output: "".to_string(),
        }
    }

    fn interpret(&mut self, input: String) {
        self.instruction_pointer = 0;
        while self.instruction_pointer < input.len() {
            if let Some(x) = input.chars().nth(self.instruction_pointer) {
                if self.debug {
                    println!("Current Char: {x} Current ip: {} Current dp: {} Loop Stack: {:?} Tape: {:?}", self.instruction_pointer, self.data_pointer, self.loop_stack, self.tape);
                }

                match x {
                    '[' => {
                        if self.tape[self.data_pointer] == 0 {
                            // Track the number of '[' characters encountered
                            let mut nested_brackets: u32 = 1; // Start with 1 to account for the initial '['
                            let mut current_instruction = self.instruction_pointer + 1; // Move past the current '['

                            while nested_brackets > 0 {
                                if let Some(y) = input.chars().nth(current_instruction) {
                                    if y == '[' {
                                        nested_brackets += 1;
                                    } else if y == ']' {
                                        nested_brackets -= 1;
                                    }
                                    current_instruction += 1;
                                } else {
                                    // Handle error: Missing matching ']'
                                    break;
                                }
                            }

                            // If matching ']' was found, update instruction_pointer
                            if nested_brackets == 0 {
                                self.instruction_pointer = current_instruction;
                            } else {
                                // Handle error: Missing matching ']'
                            }
                        } else {
                            self.loop_stack.push(self.instruction_pointer);
                        }
                    }
                    ']' => {
                        if let Some(x) = self.loop_stack.pop() {
                            if self.tape[self.data_pointer] == 0 {
                                self.instruction_pointer += 1;
                            } else {
                                self.instruction_pointer = x;
                                self.loop_stack.push(x);
                            }
                        }
                    }
                    '>' => self.data_pointer += 1,
                    '<' => {
                        if self.data_pointer != 0 {
                            self.data_pointer -= 1
                        }
                    }
                    '+' => {
                        if self.tape[self.data_pointer] == 255 {
                            self.tape[self.data_pointer] = 0;
                        } else {
                            self.tape[self.data_pointer] += 1
                        }
                    }
                    '-' => {
                        if self.tape[self.data_pointer] == 0 {
                            self.tape[self.data_pointer] = 255;
                        } else {
                            self.tape[self.data_pointer] -= 1
                        }
                    }
                    '.' => {
                        self.output.push(self.tape[self.data_pointer] as char);
                        if self.debug {
                            println!(
                                "Printing character: {} Character code: {}",
                                self.tape[self.data_pointer] as char, self.tape[self.data_pointer]
                            )
                        }
                    }
                    ',' => {
                        let mut input = String::new();
                        io::stdin()
                            .read_line(&mut input)
                            .expect("Failed to read line");
                        if let Some(x) = input.as_bytes().get(0).copied() {
                            self.tape[self.data_pointer] = x;
                        }
                    }
                    _ => (),
                }
            } else {
                println!("Illegal instruction pointer value");
                break;
            }

            self.instruction_pointer += 1;
        }
        println!("\nExecution over");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_eyre::install()?;

    let matches = App::new("Brainfuck Interpreter")
        .version("0.1.0")
        .author("Saphereye")
        .about("Brainf*ck interpreter written in rust")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("FILE")
                .help("Sets the input file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("debug")
                .short("d")
                .long("debug")
                .help("Enables debug mode (default FALSE)"),
        )
        .arg(
            Arg::with_name("raw")
                .short("r")
                .long("raw")
                .help("Enables raw mode")
                .value_name("RAW_INPUT")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("size")
                .short("s")
                .long("size")
                .help("Specify tape length (default 10)")
                .value_name("TAPE_LEN")
                .takes_value(true),
        )
        .get_matches();

    let filename: &str;
    let mut input: String = String::new();
    let debug: bool = matches.is_present("debug");
    let mut tape_size: usize = 10;

    if let Some(input_file) = matches.value_of("input") {
        println!("Input file: {}", input_file);
        filename = input_file;
        input = fs::read_to_string(filename)?;
    }

    if let Some(input_string) = matches.value_of("raw") {
        println!("Raw input: {}", input_string);
        input = input_string.to_string();
    }

    if let Some(input_string) = matches.value_of("size") {
        println!("Tape size: {}", input_string);
        tape_size = input_string.parse()?;
    }

    let mut cpu = Cpu::default(tape_size, debug);
    cpu.interpret(input);
    println!("Program output: {}", cpu.output);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Cpu;

    #[test]
    fn test_increment_tape() {
        let mut cpu = Cpu::default(10, false);
        cpu.tape[cpu.data_pointer] = 42;
        cpu.interpret("+".to_string());
        assert_eq!(cpu.tape[cpu.data_pointer], 43);
    }

    #[test]
    fn test_decrement_tape() {
        let mut cpu = Cpu::default(10, false);
        cpu.tape[cpu.data_pointer] = 5;
        cpu.interpret("-".to_string());
        assert_eq!(cpu.tape[cpu.data_pointer], 4);
    }

    #[test]
    fn test_move_data_pointer() {
        let mut cpu = Cpu::default(10, false);
        cpu.interpret(">>".to_string());
        assert_eq!(cpu.data_pointer, 2);
        cpu.interpret("<".to_string());
        assert_eq!(cpu.data_pointer, 1);
    }

    #[test]
    fn test_ascii_output() {
        let mut cpu = Cpu::default(10, false);
        cpu.interpret(
            "+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.".to_string(),
        );
        assert_eq!(cpu.output, "A".to_string());
    }

    #[test]
    fn test_hello_world() {
        let input = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.".to_string();
        let mut cpu = Cpu::default(10, false);
        cpu.interpret(input);
        let output = cpu.output;
        assert_eq!(output, "Hello, World!");
    }
}
