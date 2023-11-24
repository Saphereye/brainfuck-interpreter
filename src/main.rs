use clap::{App, Arg};
use color_eyre::eyre::Result;

use std::fs;
use std::io;

struct Cpu {
    feed_tape: String,
    data_pointer: usize,
    output: String,
    tape_size: usize,
}

impl Cpu {
    fn new(feed_tape: String, tape_size: usize) -> Self {
        Self {
            feed_tape,
            data_pointer: 0,
            output: String::new(),
            tape_size,
        }
    }

    fn run(&mut self, pre_defined_input: Option<String>) -> Result<()> {
        let mut tape: Vec<u8> = vec![0; self.tape_size];
        let mut instruction_pointer = 0;
        let mut input_index: usize = 0;

        while instruction_pointer < self.feed_tape.len() {
            match self.feed_tape.chars().nth(instruction_pointer).unwrap() {
                '>' => self.data_pointer += 1,
                '<' => self.data_pointer -= 1,
                '+' => tape[self.data_pointer] = tape[self.data_pointer].wrapping_add(1),
                '-' => tape[self.data_pointer] = tape[self.data_pointer].wrapping_sub(1),
                '.' => {
                    self.output.push(tape[self.data_pointer] as char);

                    #[cfg(debug_assertions)]
                    if (tape[self.data_pointer] as char).is_ascii_graphic() {
                        println!("Pushing char: {}", tape[self.data_pointer] as char);
                    } else {
                        println!("Pushing char(u8): {:?}", tape[self.data_pointer]);
                    }
                }
                ',' => match pre_defined_input {
                    Some(ref input) => {
                        tape[self.data_pointer] = input.as_bytes()[input_index];
                        input_index += 1;
                    }
                    None => {
                        let mut input = String::new();
                        io::stdin().read_line(&mut input)?;
                        tape[self.data_pointer] = input.as_bytes()[0];
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
                // Debugging
                // '#' => println!("Tape: {:?}", tape),
                // '^' => println!("Data Pointer: {}", self.data_pointer),
                // '!' => println!("Output: {}", self.output),
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(debug_assertions))]
    color_eyre::install()?;

    // Usage
    // cargo run -- -i examples/hello_world.bf
    // cargo run -- -i examples/hello_world.bf -s 2048
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
        .get_matches();

    let mut input: String = String::new();
    let mut tape_size: usize = 2048;

    if let Some(input_file) = matches.value_of("input") {
        let filename = input_file;
        input = fs::read_to_string(filename)?;
    }

    if let Some(input_string) = matches.value_of("size") {
        tape_size = input_string.parse()?;
    }

    let mut cpu = Cpu::new(input, tape_size);

    cpu.run(None)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cpu() {
        let cpu = Cpu::new(String::from("++++++++++"), 2048);
        assert_eq!(cpu.feed_tape, "++++++++++");
        assert_eq!(cpu.data_pointer, 0);
    }

    #[test]
    fn test_run_cpu() {
        let mut cpu = Cpu::new(String::from("++++++++++"), 2048);
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
        );
        let result = cpu.run(None);
        assert!(result.is_ok());
        assert_eq!(cpu.output, "Hello World!\n");
    }

    #[test]
    fn test_hello_world_file() {
        let program = r#"
        [ This program prints "Hello World!" and a newline to the screen, its
        length is 106 active command characters. [It is not the shortest.]

        This loop is an "initial comment loop", a simple way of adding a comment
        to a BF program such that you don't have to worry about any command
        characters. Any ".", ",", "+", "-", "<" and ">" characters are simply
        ignored, the "[" and "]" characters just have to be balanced. This
        loop and the commands it contains are ignored because the current cell
        defaults to a value of 0; the 0 value causes this loop to be skipped.
        ]
        ++++++++               Set Cell #0 to 8
        [
            >++++               Add 4 to Cell #1; this will always set Cell #1 to 4
            [                   as the cell will be cleared by the loop
                >++             Add 2 to Cell #2
                >+++            Add 3 to Cell #3
                >+++            Add 3 to Cell #4
                >+              Add 1 to Cell #5
                <<<<-           Decrement the loop counter in Cell #1
            ]                   Loop until Cell #1 is zero; number of iterations is 4
            >+                  Add 1 to Cell #2
            >+                  Add 1 to Cell #3
            >-                  Subtract 1 from Cell #4
            >>+                 Add 1 to Cell #6
            [<]                 Move back to the first zero cell you find; this will
                                be Cell #1 which was cleared by the previous loop
            <-                  Decrement the loop Counter in Cell #0
        ]                       Loop until Cell #0 is zero; number of iterations is 8

        The result of this is:
        Cell no :   0   1   2   3   4   5   6
        Contents:   0   0  72 104  88  32   8
        Pointer :   ^

        >>.                     Cell #2 has value 72 which is 'H'
        >---.                   Subtract 3 from Cell #3 to get 101 which is 'e'
        +++++++..+++.           Likewise for 'llo' from Cell #3
        >>.                     Cell #5 is 32 for the space
        <-.                     Subtract 1 from Cell #4 for 87 to give a 'W'
        <.                      Cell #3 was set to 'o' from the end of 'Hello'
        +++.------.--------.    Cell #3 for 'rl' and 'd'
        >>+.                    Add 1 to Cell #5 gives us an exclamation point
        >++.                    And finally a newline from Cell #6
        "#;
        let mut cpu = Cpu::new(String::from(program), 2048);
        let result = cpu.run(None);
        assert!(result.is_ok());
        assert_eq!(cpu.output, "Hello World!\n");
    }

    #[test]
    fn test_add_numbers() {
        let program = r#"
        ++       Cell c0 = 2
        > +++++  Cell c1 = 5

        [        Start your loops with your cell pointer on the loop counter (c1 in our case)
        < +      Add 1 to c0
        > -      Subtract 1 from c1
        ]        End your loops with the cell pointer on the loop counter

        At this point our program has added 5 to 2 leaving 7 in c0 and 0 in c1
        but we cannot output this value to the terminal since it is not ASCII encoded

        To display the ASCII character "7" we must add 48 to the value 7
        We use a loop to compute 48 = 6 * 8

        ++++ ++++  c1 = 8 and this will be our loop counter again
        [
        < +++ +++  Add 6 to c0
        > -        Subtract 1 from c1
        ]
        < .        Print out c0 which has the value 55 which translates to "7"!
        "#;
        let mut cpu = Cpu::new(String::from(program), 2048);
        let result = cpu.run(None);
        assert!(result.is_ok());
        assert_eq!(cpu.output, "7");
    }

    #[test]
    fn test_input() {
        let program = r#"
        ,.
        "#;
        let mut cpu = Cpu::new(String::from(program), 2048);
        let result = cpu.run(Some("1".to_string()));
        assert!(result.is_ok());
        assert_eq!(cpu.output, "1");
    }
}
