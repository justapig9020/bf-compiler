// Reserved words:
// - IF: if
// - ELSE: else
// - NC: next_cell
// - PC: prev_cell
//
// Semantic:
// - Function: [Statement]*
// - Statement: If | While | Assign | Move
// - If: if Flag { Function } [else { Function }]!
// - While: while Flag { Function }
// - Assign: ID = NUMBER
// - Flag: Bool [&& Flag]*
// - Bool: ID != NUMBER
// - Move: ID("move_right") | ID("move_left")

use crate::scanner::{Token, TokenStream};
use anyhow::{anyhow, Result};

#[derive(Debug)]
struct AST<'a> {
    function: Vec<Statement<'a>>,
}

#[derive(Debug)]
enum Statement<'a> {
    IF(Flag<'a>, Vec<Statement<'a>>, Option<Vec<Statement<'a>>>),
    WHILE(Flag<'a>, Vec<Statement<'a>>),
    Assign(Token<'a>, Token<'a>),
    Move(Direction),
}

#[derive(Debug)]
struct Flag<'a> {
    flags: Vec<Bool<'a>>,
}

#[derive(Debug)]
struct Bool<'a> {
    id: Token<'a>,
    num: Token<'a>,
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum Direction {
    Left,
    Right,
}

impl TryFrom<Token<'_>> for Direction {
    type Error = anyhow::Error;
    fn try_from(token: Token<'_>) -> Result<Self> {
        let Token::ID(id) = token else {
            return Err(anyhow!("Expected ID, found {:?}", token));
        };
        match id {
            "move_right" => Ok(Self::Right),
            "move_left" => Ok(Self::Left),
            _ => Err(anyhow!("Expected move_right or move_left, found {}", id)),
        }
    }
}

const RESERVED_WORDS: [&str; 5] = ["if", "else", "while", "next_cell", "prev_cell"];

pub fn parse(tokens: TokenStream) -> Result<AST> {
    todo!();
}

#[cfg(test)]
mod parser {
    use super::*;

    #[test]
    fn test_parse_direction() {
        let testcase = [
            (Token::ID("move_right"), Ok(Direction::Right)),
            (Token::ID("move_left"), Ok(Direction::Left)),
            (Token::ID("abcd"), Err(())),
        ];
        for (token, expect) in testcase.into_iter() {
            let output = Direction::try_from(token);
            if let Ok(expect) = expect {
                assert_eq!(output.unwrap(), expect);
            } else {
                assert!(output.is_err());
            }
        }
    }
}
