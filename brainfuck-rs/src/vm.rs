
#![allow(dead_code)]

pub enum Instruction {
    Increment(u8),
    Push(u8),
    Add,
}

type Program = Vec<Instruction>;

type Brainfuck = Vec<BFCode>;

#[derive(Debug, PartialEq, Eq)]
pub enum BFCode {
    MoveRight,
    MoveLeft,
    Increment,
    Decrement,
    Input,
    Output,
    LoopStart,
    LoopEnd,
}


pub fn instruction_to_brainfuck(program: Program) -> Brainfuck {
    let mut brainfuck = Vec::new();

    for instruction in program{
        match instruction {
            Instruction::Increment(n) => {
                for _ in 0..n {
                    brainfuck.push(BFCode::Increment);
                }
            }
            Instruction::Push(n) => {
                brainfuck.push(BFCode::MoveRight);
                for _ in 0..n {
                    brainfuck.push(BFCode::Increment);
                }
            }
            Instruction::Add => {
                
            }
        }
    }

    brainfuck
}


#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;


    #[test]
    fn empty_program_leads_to_empty_brainfuck() {
        let instructions = vec![];
        let result = instruction_to_brainfuck(instructions);
        assert!(result.is_empty());
    }

    #[rstest]
    #[case(
        vec![Instruction::Increment(1)], 
        vec![BFCode::Increment]
    )]
    #[case(
        vec![Instruction::Increment(3)], 
        vec![BFCode::Increment, BFCode::Increment, BFCode::Increment]
    )]
    #[case(
        vec![Instruction::Increment(0)], 
        vec![]
    )]
    #[case(
        vec![Instruction::Push(1)],
        vec![BFCode::MoveRight, BFCode::Increment]
    )]

    #[case(
        vec![Instruction::Push(4)],
        vec![BFCode::MoveRight, BFCode::Increment, BFCode::Increment, BFCode::Increment, BFCode::Increment]
    )]
    fn test_program_to_brainfuck(#[case] input: Program, #[case] expected: Brainfuck) {
        let result = instruction_to_brainfuck(input);
       
        assert_eq!(result.as_slice(), expected.as_slice());
    }
    

}