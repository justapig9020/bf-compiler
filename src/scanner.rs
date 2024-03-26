use anyhow::Result;
use regex;

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    ID(&'a str),
    NUM(&'a str),
    ASSIGN,
    EQ,
    NE,
    LB,
    RB,
    AND,
    EOF,
}

pub struct TokenStream<'a> {
    tokens: Vec<Token<'a>>,
}

impl<'a> TokenStream<'a> {
    pub fn into_tokens(self) -> Vec<Token<'a>> {
        self.tokens
    }
}

macro_rules! match_regex {
    ($regex: expr, $type: expr) => {
        |program| -> Option<Token> {
            let re = regex::Regex::new($regex).unwrap();
            if re.is_match(program) {
                Some($type(program))
            } else {
                None
            }
        }
    };
}

macro_rules! match_str {
    ($str: expr, $type: expr) => {
        |program| -> Option<Token> {
            if program == $str {
                Some($type)
            } else {
                None
            }
        }
    };
}

impl<'a> TryFrom<&'a str> for Token<'a> {
    type Error = anyhow::Error;
    fn try_from(program: &'a str) -> Result<Self> {
        let match_func = [
            match_regex!(r"^[a-zA-z_][a-zA-Z_0-9]*$", Token::ID),
            match_regex!(r"^[0-9]+$", Token::NUM),
            match_str!("=", Token::ASSIGN),
            match_str!("==", Token::EQ),
            match_str!("!=", Token::NE),
            match_str!("{", Token::LB),
            match_str!("}", Token::RB),
            match_str!("&&", Token::AND),
        ];
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
            .map(|token| {
                let token = Token::try_from(token);
                println!("{:?}", token);
                token
            })
            .chain(std::iter::once(Ok(Token::EOF)))
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
                if let Ok(token) = output {
                    assert_ne!(token, $type($program));
                }
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
    #[test]
    fn test_num() {
        let testcases = [
            ("123", true),
            ("123hello", false),
            ("hello123", false),
            ("_123", false),
        ];
        for (program, is_match) in testcases.into_iter() {
            test_token!(program, is_match, Token::NUM);
        }
    }
    #[test]
    fn test_assign() {
        let program = "=";
        let expect = Token::ASSIGN;
        let output = Token::try_from(program).unwrap();
        assert_eq!(output, expect);
    }
    #[test]
    fn test_eq() {
        let program = "==";
        let expect = Token::EQ;
        let output = Token::try_from(program).unwrap();
        assert_eq!(output, expect);
    }
    #[test]
    fn test_excl() {
        let program = "!=";
        let expect = Token::NE;
        let output = Token::try_from(program).unwrap();
        assert_eq!(output, expect);
    }
    #[test]
    fn test_lb() {
        let program = "{";
        let expect = Token::LB;
        let output = Token::try_from(program).unwrap();
        assert_eq!(output, expect);
    }
    #[test]
    fn test_rb() {
        let program = "}";
        let expect = Token::RB;
        let output = Token::try_from(program).unwrap();
        assert_eq!(output, expect);
    }
    #[test]
    fn test_and() {
        let program = "&&";
        let expect = Token::AND;
        let output = Token::try_from(program).unwrap();
        assert_eq!(output, expect);
    }
    #[test]
    fn test_token_stream() {
        let program = "
            if hello != 123 && world == 456 {
                abc = 789
            }";
        let expect = vec![
            Token::ID("if"),
            Token::ID("hello"),
            Token::NE,
            Token::NUM("123"),
            Token::AND,
            Token::ID("world"),
            Token::EQ,
            Token::NUM("456"),
            Token::LB,
            Token::ID("abc"),
            Token::ASSIGN,
            Token::NUM("789"),
            Token::RB,
            Token::EOF,
        ];
        let output = TokenStream::try_from(program).unwrap();
        assert_eq!(output.tokens, expect);
    }
}
