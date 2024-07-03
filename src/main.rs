#[doc = include_str!("../README.md")]
use anyhow::Context;
use anyhow::Error;
use anyhow::Ok;
use clap::{App, Arg};

use pretty_env_logger::env_logger;
use std::fs;
use std::io;
use std::io::Read;

struct Cpu {
    feed_tape: String,
    data_pointer: usize,
    output: String,
    tape_size: usize,

    /// The output is printed all at once at the end of the program
    one_shot_output: bool,

    /// Extended commands level
    level: u8,

    // Extended commands I helper
    storage: u8,
}

impl Cpu {
    fn new(feed_tape: String, tape_size: usize, level: u8) -> Self {
        Self {
            feed_tape,
            data_pointer: 0,
            output: String::new(),
            tape_size,
            one_shot_output: false,
            storage: 0,
            level,
        }
    }

    /// Main core of the program
    ///
    /// The brainfuck instruction can be summarised as follows:
    ///
    /// | Instruction | Description                                           |
    /// |-------------|-------------------------------------------------------|
    /// | `<`         | head0 = head0 - 1                                     |
    /// | `>`         | head0 = head0 + 1                                     |
    /// | `{`         | head1 = head1 - 1                                     |
    /// | `}`         | head1 = head1 + 1                                     |
    /// | `-`         | tape[head0] = tape[head0] - 1                         |
    /// | `+`         | tape[head0] = tape[head0] + 1                         |
    /// | `.`         | tape[head1] = tape[head0]                             |
    /// | `,`         | tape[head0] = tape[head1]                             |
    /// | `[`         | if (tape[head0] == 0): jump forwards to matching `]`  |
    /// | `]`         | if (tape[head0] != 0): jump backwards to matching `[` |
    ///
    /// > source: arXiv:2406.19108
    fn run(&mut self, input: Option<String>) -> Result<(), Error> {
        let mut tape: Vec<u8> = vec![0; self.tape_size];
        let mut instruction_pointer = 0;
        let mut input_index: usize = 0;

        while instruction_pointer < self.feed_tape.len() {
            #[cfg(debug_assertions)]
            log::debug!(
                "Instruction Pointer: {}, Data Pointer: {}, Current Char: {}",
                instruction_pointer,
                self.data_pointer,
                self.feed_tape.chars().nth(instruction_pointer).unwrap()
            );

            match self.feed_tape.chars().nth(instruction_pointer).unwrap() {
                // Basic Commands
                '>' => {
                    if self.data_pointer < self.tape_size - 1 {
                        self.data_pointer += 1
                    }
                }
                '<' => {
                    if self.data_pointer != 0 {
                        self.data_pointer -= 1
                    }
                }
                '+' => tape[self.data_pointer] = tape[self.data_pointer].wrapping_add(1),
                '-' => tape[self.data_pointer] = tape[self.data_pointer].wrapping_sub(1),
                '.' => {
                    if self.one_shot_output {
                        self.output.push(tape[self.data_pointer] as char);
                    } else {
                        print!("{}", tape[self.data_pointer] as char);
                    }

                    // log::debug!("Output: '{}'", tape[self.data_pointer] as char);

                    // #[cfg(debug_assertions)]
                    // if (tape[self.data_pointer] as char).is_ascii_graphic() {
                    //     println!("Pushing char: {}", tape[self.data_pointer] as char);
                    // } else {
                    //     println!("Pushing char(u8): {:?}", tape[self.data_pointer]);
                    // }
                }
                ',' => match input {
                    Some(ref input) => {
                        tape[self.data_pointer] = input.as_bytes()[input_index];
                        input_index += 1;
                    }
                    None => {
                        let mut input = [0];
                        // read a single character
                        io::stdin().read_exact(&mut input)?;

                        // log::debug!("Input: {:?}", input);

                        tape[self.data_pointer] = input[0];
                    }
                },
                '[' => {
                    if tape[self.data_pointer] == 0 {
                        let mut loop_count = 1;
                        while loop_count > 0 {
                            instruction_pointer += 1;
                            match self.feed_tape.chars().nth(instruction_pointer).unwrap() {
                                '[' => loop_count += 1,
                                ']' => loop_count -= 1,
                                _ => (),
                            }
                        }
                    }
                }
                ']' => {
                    if tape[self.data_pointer] != 0 {
                        let mut loop_count = 1;
                        while loop_count > 0 {
                            instruction_pointer -= 1;
                            match self.feed_tape.chars().nth(instruction_pointer).unwrap() {
                                '[' => loop_count -= 1,
                                ']' => loop_count += 1,
                                _ => (),
                            }
                        }
                    }
                }

                // Extended commands I (https://esolangs.org/wiki/Extended_Brainfuck#Extended_Type_I)
                '@' | '$' | '!' | '}' | '{' | '~' | '^' | '&' | '|' => {
                    if self.level < 1 {
                        instruction_pointer += 1;
                        continue;
                    }

                    match self.feed_tape.chars().nth(instruction_pointer).unwrap() {
                        '@' => break,
                        '$' => self.storage = tape[self.data_pointer],
                        '!' => tape[self.data_pointer] = self.storage,
                        '}' => tape[self.data_pointer] >>= 1,
                        '{' => tape[self.data_pointer] <<= 1,
                        '~' => tape[self.data_pointer] = !tape[self.data_pointer],
                        '^' => tape[self.data_pointer] ^= self.storage,
                        '&' => tape[self.data_pointer] &= self.storage,
                        '|' => tape[self.data_pointer] |= self.storage,
                        _ => unreachable!("The level 1 commands have already been matched"),
                    }
                }

                // Extended commands II (https://esolangs.org/wiki/Extended_Brainfuck#Extended_Type_II)
                '?' | '(' | ')' | '*' | '/' | '=' | '_' | '%' => {
                    if self.level < 2 {
                        instruction_pointer += 1;
                        continue;
                    }

                    match self.feed_tape.chars().nth(instruction_pointer).unwrap() {
                        '?' => todo!(),
                        '(' => todo!(),
                        ')' => todo!(),
                        '*' => {
                            tape[self.data_pointer] =
                                tape[self.data_pointer].wrapping_mul(self.storage)
                        }
                        '/' => {
                            if self.storage != 0 {
                                tape[self.data_pointer] =
                                    tape[self.data_pointer].wrapping_div(self.storage)
                            } else {
                                log::error!(
                                    "Division by zero, instruction pointer: {}, current char: /",
                                    instruction_pointer
                                );
                            }
                        }
                        '=' => {
                            tape[self.data_pointer] =
                                tape[self.data_pointer].wrapping_add(self.storage)
                        }
                        '_' => {
                            tape[self.data_pointer] =
                                tape[self.data_pointer].wrapping_sub(self.storage)
                        }
                        '%' => {
                            if self.storage != 0 {
                                tape[self.data_pointer] =
                                    tape[self.data_pointer].wrapping_rem(self.storage)
                            } else {
                                log::error!(
                                    "Division by zero, instruction pointer: {}, current char: %",
                                    instruction_pointer
                                );
                            }
                        }
                        _ => unreachable!("The level 2 commands have already been matched"),
                    }
                }
                // Extended commands III (https://esolangs.org/wiki/Extended_Brainfuck#Extended_Type_III)
                'X' | 'x' | 'M' | 'm' | 'L' | 'l' | ':' | '0'..='9' | 'A'..='F' | '#' => {
                    if self.level < 3 {
                        instruction_pointer += 1;
                        continue;
                    }

                    match self.feed_tape.chars().nth(instruction_pointer).unwrap() {
                        'X' => todo!(),
                        'x' => todo!(),
                        'M' => todo!(),
                        'm' => todo!(),
                        'L' => todo!(),
                        'l' => todo!(),
                        ':' => todo!(),
                        '0'..='9' => todo!(),
                        'A'..='F' => todo!(),
                        '#' => todo!(),
                        _ => unreachable!("The level 3 commands have already been matched"),
                    }
                }
                _ => (),
            }
            instruction_pointer += 1;
        }

        #[cfg(debug_assertions)]
        for character in self.output.chars() {
            if character.is_ascii_graphic() {
                print!("{}", character);
            } else {
                print!("{:?}", character);
            }
        }
        #[cfg(debug_assertions)]
        println!();

        #[cfg(not(debug_assertions))]
        println!("{}", self.output);

        Ok(())
    }
}

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
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
            Arg::with_name("size")
                .short("s")
                .long("size")
                .help("Specify tape length (default 2048)")
                .value_name("TAPE_LEN")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("level")
                .short("l")
                .long("level")
                .help("Specfies the extended brainfuk level. 1-3 is extended, 0 is normal bf")
                .value_name("LEVEL")
                .takes_value(true)
                .default_value("0"),
        )
        .get_matches();

    let mut input: String = String::new();
    let mut tape_size: usize = 2048;

    let level: u8 = matches
        .value_of("level")
        .unwrap()
        .parse()
        .context("Invalid level command level")?;

    if let Some(input_file) = matches.value_of("input") {
        log::trace!("Reading {}.bf file", input_file.to_string());
        let filename = input_file;
        input = fs::read_to_string(filename)
            .with_context(|| format!("Failed to read file: {}", filename))?;
    }

    if let Some(input_string) = matches.value_of("size") {
        log::trace!("Setting tape size to {}", input_string.to_string());
        tape_size = input_string.parse()?;
    }

    let mut cpu = Cpu::new(input, tape_size, level);

    log::trace!("Running program");
    cpu.run(None)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cpu() {
        let cpu = Cpu::new(String::from("++++++++++"), 2048, 0);
        assert_eq!(cpu.feed_tape, "++++++++++");
        assert_eq!(cpu.data_pointer, 0);
    }

    #[test]
    fn test_run_cpu() {
        let mut cpu = Cpu::new(String::from("++++++++++"), 2048, 0);
        let result = cpu.run(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_hello_world() {
        let mut cpu = Cpu::new(
            String::from(
                "++++++++++[>+++++++>++++++++++>+++>+<<<<-]>++.>+.+++++++\
            ..+++.>++.<<+++++++++++++++.>.+++.------.--------.>+.>.",
            ),
            2048,
            0,
        );
        cpu.one_shot_output = true;
        let result = cpu.run(None);
        assert!(result.is_ok());
        assert_eq!(cpu.output, "Hello World!\n");
    }

    #[test]
    fn test_hello_world_file() {
        let program = include_str!("../examples/hello_world_big.bf");
        let mut cpu = Cpu::new(String::from(program), 2048, 0);
        cpu.one_shot_output = true;
        let result = cpu.run(None);
        assert!(result.is_ok());
        assert_eq!(cpu.output, "Hello World!\n");
    }

    #[test]
    fn test_add_numbers() {
        let program = include_str!("../examples/add_numbers.bf");
        let mut cpu = Cpu::new(String::from(program), 2048, 0);
        cpu.one_shot_output = true;
        let result = cpu.run(None);
        assert!(result.is_ok());
        assert_eq!(cpu.output, "7");
    }

    #[test]
    fn test_input() {
        let program = r#"
        ,.
        "#;
        let mut cpu = Cpu::new(String::from(program), 2048, 0);
        cpu.one_shot_output = true;
        let result = cpu.run(Some("1".to_string()));
        assert!(result.is_ok());
        assert_eq!(cpu.output, "1");
    }
}
