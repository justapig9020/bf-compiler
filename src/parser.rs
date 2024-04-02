// Reserved words:
// - IF: if
// - ELSE: else
// - NC: next_cell
// - PC: prev_cell
//
// Semantic:
// - Function: [Statement]*
// - Statement: If | While | Assign | Move | Input | Output
// - If: if Flag { Function } [else { Function }]!
// - While: while Flag { Function }
// - Assign: Variable = NUMBER
// - Bool: Compare [&& Compare]*
// - Compare: Equal | NotEqual
// - Equal: Variable == NUMBER
// - NotEqual: Variable != NUMBER
// - Move: ID("move_right") | ID("move_left")
// - Input: ID("input") ( Variable )
// - Output: ID("output") ( Variable )
// - Variable: ID

use crate::scanner::{Token, TokenStream};
use anyhow::{anyhow, Result};

#[derive(Debug, PartialEq, Clone)]
struct AST<'a> {
    function: Vec<Statement<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
enum Statement<'a> {
    IF(Bool<'a>, Vec<Statement<'a>>, Option<Vec<Statement<'a>>>),
    WHILE(Bool<'a>, Vec<Statement<'a>>),
    Assign(Variable<'a>, Num),
    Move(Direction),
    Input(Variable<'a>),
    Output(Variable<'a>),
}

#[derive(Debug, PartialEq, Clone)]
struct Bool<'a> {
    compares: Vec<Compare<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
enum Compare<'a> {
    EQ(Variable<'a>, Num),
    NE(Variable<'a>, Num),
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Num(u8);

impl<'a> TryFrom<&Token<'a>> for Num {
    type Error = anyhow::Error;
    fn try_from(token: &Token<'a>) -> Result<Self> {
        let Token::NUM(num) = token else {
            return Err(anyhow!("Expected NUM, found {:?}", token));
        };
        Ok(Self(num.parse().unwrap()))
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Variable<'a>(&'a str);

impl<'a> TryFrom<&Token<'a>> for Variable<'a> {
    type Error = anyhow::Error;
    fn try_from(token: &Token<'a>) -> Result<Self> {
        let Token::ID(id) = token else {
            return Err(anyhow!("Expected ID, found {:?}", token));
        };
        if RESERVED_WORDS.contains(&id) {
            return Err(anyhow!("{} is a reserved word", id));
        }
        Ok(Self(id))
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum Direction {
    Left,
    Right,
}

impl TryFrom<&Token<'_>> for Direction {
    type Error = anyhow::Error;
    fn try_from(token: &Token<'_>) -> Result<Self> {
        let Token::ID(id) = token else {
            return Err(anyhow!("Expected ID, found {:?}", token));
        };
        match *id {
            "move_right" => Ok(Self::Right),
            "move_left" => Ok(Self::Left),
            _ => Err(anyhow!("Expected move_right or move_left, found {}", id)),
        }
    }
}

const RESERVED_WORDS: [&str; 7] = [
    "if",
    "else",
    "while",
    "next_cell",
    "prev_cell",
    "input",
    "output",
];

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
            let output = Direction::try_from(&token);
            if let Ok(expect) = expect {
                assert_eq!(output.unwrap(), expect);
            } else {
                assert!(output.is_err());
            }
        }
    }

    #[test]
    fn test_parse_variable() {
        let testcase = [
            (Token::ID("hello"), Ok(Variable("hello"))),
            (Token::NUM("123"), Err(())),
        ];
        let reserved_words: Vec<(Token, Result<Variable, ()>)> = RESERVED_WORDS
            .iter()
            .map(|s| (Token::ID(s), Err(())))
            .collect();
        let testcase = testcase.into_iter().chain(reserved_words.into_iter());
        for (token, expect) in testcase {
            let output = Variable::try_from(&token);
            if let Ok(expect) = expect {
                assert_eq!(output.unwrap(), expect);
            } else {
                assert!(output.is_err());
            }
        }
    }
    #[test]
    fn test_parse_num() {
        let testcase = [
            (Token::NUM("123"), Ok(Num(123))),
            (Token::ID("hello"), Err(())),
        ];
        for (token, expect) in testcase.into_iter() {
            let output = Num::try_from(&token);
            if let Ok(expect) = expect {
                assert_eq!(output.unwrap(), expect);
            } else {
                assert!(output.is_err());
            }
        }
    }
}
