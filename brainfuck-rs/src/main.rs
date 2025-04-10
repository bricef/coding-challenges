mod vm;

use std::io::Read;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the Brainfuck program file
    #[arg(short, long)]
    program: String,
}

fn main() {
    let args = Args::parse();
    let program: Vec<char> = std::fs::read_to_string(args.program).unwrap().chars().collect::<Vec<_>>();
    
    let mut tape: Vec<u8> = vec![0u8; 30000];
    let mut data_pointer: usize = 0;
    let mut program_pointer: usize = 0;

    while program_pointer < program.len() {
        let c = program[program_pointer];
        match c {
            // Move the pointer to the right
            '>' => data_pointer += 1,

            // Move the pointer to the left
            '<' => data_pointer -= 1,

            // Increment the memory cell at the pointer
            '+' => tape[data_pointer] = tape[data_pointer].wrapping_add(1),

            // Decrement the memory cell at the pointer
            '-' => tape[data_pointer] = tape[data_pointer].wrapping_sub(1),

            // Output the character signified by the cell at the pointer
            '.' => print!("{}", tape[data_pointer] as char),

            // Input a character and store it in the cell at the pointer
            ',' => {
                let mut input = [0u8];
                std::io::stdin().read_exact(&mut input).unwrap();
                tape[data_pointer] = input[0];
            }
            // Jump past the matching ] if the cell at the pointer is 0
            '[' => {
                if tape[data_pointer] == 0 {
                    let mut open_brackets = 1;
                    while open_brackets > 0 {
                        program_pointer += 1;
                        let c = program[program_pointer];
                        if c == '[' {
                            open_brackets += 1;
                        } else if c == ']' {
                            open_brackets -= 1;
                        }
                    }
                }
            }
            // Jump back to the matching [ if the cell at the pointer is nonzero
            ']' => {
                if tape[data_pointer] != 0 {
                    let mut open_brackets = 1;
                    while open_brackets > 0 {
                        program_pointer -= 1;
                        let c = program[program_pointer];
                        if c == '[' {
                            open_brackets -= 1;
                        } else if c == ']' {
                            open_brackets += 1;
                        }
                    }
                }
            }
            _ => (),
        }
        program_pointer += 1;
    }
}

