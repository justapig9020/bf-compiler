use anyhow::Result;
use regex;
// Token
// - ID: [a-zA-z_][a-zA-Z_0-9]*
// - NUM: [0-9]+
// - EQ: =
// - EXCL: !
// - LB: {
// - RB: }
// - AND: &&
//
// Reserved words:
// - IF: if
// - ELSE: else
// - NC: next_cell
// - PC: prev_cell

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    ID(&'a str),
    NUM(&'a str),
    EQ,
    EXCL,
    LB,
    RB,
    AND,
    EOF,
}

pub struct TokenStream<'a> {
    tokens: Vec<Token<'a>>,
}

fn match_id(program: &str) -> Option<Token> {
    let re = regex::Regex::new(r"^[a-zA-z_][a-zA-Z_0-9]*$").unwrap();
    if re.is_match(program) {
        Some(Token::ID(program))
    } else {
        None
    }
}

impl<'a> TryFrom<&'a str> for Token<'a> {
    type Error = anyhow::Error;
    fn try_from(program: &'a str) -> Result<Self> {
        let match_func = [match_id];
        for func in match_func.iter() {
            if let Some(token) = func(program) {
                return Ok(token);
            }
        }
        Err(anyhow::anyhow!("Invalid token"))
    }
}

impl<'a> TryFrom<&'a str> for TokenStream<'a> {
    type Error = anyhow::Error;
    fn try_from(program: &'a str) -> Result<Self> {
        let tokens = program
            .split_whitespace()
            .map(|token| Token::try_from(token))
            .collect::<Result<Vec<Token>>>()?;
        Ok(Self { tokens })
    }
}

#[cfg(test)]
mod token {
    use super::*;
    macro_rules! test_token {
        ($program: expr, $expect_match: expr, $type: expr) => {
            let output = Token::try_from($program);
            if $expect_match {
                assert_eq!(output.unwrap(), $type($program));
            } else {
                assert!(output.is_err());
            }
        };
    }
    #[test]
    fn test_id() {
        let testcases = [
            ("hello", true),
            ("hello123", true),
            ("_hello", true),
            ("HELLO", true),
            ("1hello", false),
        ];
        for (program, is_match) in testcases.into_iter() {
            test_token!(program, is_match, Token::ID);
        }
    }
}
