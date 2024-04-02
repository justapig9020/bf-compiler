// Reserved words:
// - IF: if
// - ELSE: else
// - NC: next_cell
// - PC: prev_cell
//
// Semantic:
// - AST: Function EOF
// - Function: [Statement]*
// - Statement: If | While | Assign | Move | Input | Output
// - If: if Bool { Function } [else { Function }]!
// - While: while Bool { Function }
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
pub struct AST<'a>(Function<'a>);

impl<'a> TryFrom<&[Token<'a>]> for AST<'a> {
    type Error = anyhow::Error;
    fn try_from(tokens: &[Token<'a>]) -> std::prelude::v1::Result<Self, Self::Error> {
        let function = Function::try_from(tokens)?;
        let last_token = &tokens[function.len()];
        if *last_token != Token::EOF {
            Err(anyhow!("Expected EOF, found {:?}", last_token))
        } else {
            Ok(AST(function))
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Function<'a>(Vec<Statement<'a>>);

impl Function<'_> {
    fn len(&self) -> usize {
        self.0.iter().map(Statement::len).sum()
    }
}

impl<'a> TryFrom<&[Token<'a>]> for Function<'a> {
    type Error = anyhow::Error;
    fn try_from(mut value: &[Token<'a>]) -> Result<Self> {
        let mut statements = vec![];
        while !value.is_empty() {
            let Ok(statement) = Statement::try_from(value) else {
                break;
            };
            let consumed = statement.len();
            statements.push(statement);
            value = &value[consumed..];
        }
        Ok(Self(statements))
    }
}

#[derive(Debug, PartialEq, Clone)]
enum Statement<'a> {
    IF(Bool<'a>, Function<'a>, Option<Function<'a>>),
    WHILE(Bool<'a>, Function<'a>),
    Assign(Variable<'a>, Num),
    Move(Direction),
    Input(Variable<'a>),
    Output(Variable<'a>),
}

impl Statement<'_> {
    fn len(&self) -> usize {
        match self {
            Self::IF(bool, if_func, else_func) => {
                3 + bool.len() + if_func.len() + else_func.as_ref().map_or(0, |f| 2 + f.len())
            }
            Self::WHILE(bool, func) => 3 + bool.len() + func.len(),
            Self::Assign(_, _) => 3,
            Self::Move(_) => 1,
            Self::Input(_) => 4,
            Self::Output(_) => 4,
        }
    }
}

fn try_parse_input<'a>(tokens: &[Token<'a>]) -> Result<Statement<'a>> {
    if tokens.len() < 4 {
        return Err(anyhow!(
            "Expected at least 4 tokens, found {:?}",
            tokens.len()
        ));
    }
    match &tokens[..4] {
        [Token::ID("input"), Token::LP, variable, Token::RP] => {
            let variable = Variable::try_from(variable)?;
            Ok(Statement::Input(variable))
        }
        _ => Err(anyhow!("Expected input ( Variable ), found {:?}", tokens)),
    }
}

fn try_parse_output<'a>(tokens: &[Token<'a>]) -> Result<Statement<'a>> {
    if tokens.len() < 4 {
        return Err(anyhow!(
            "Expected at least 4 tokens, found {:?}",
            tokens.len()
        ));
    }
    match &tokens[..4] {
        [Token::ID("output"), Token::LP, variable, Token::RP] => {
            let variable = Variable::try_from(variable)?;
            Ok(Statement::Output(variable))
        }
        _ => Err(anyhow!("Expected input ( Variable ), found {:?}", tokens)),
    }
}

fn try_parse_move<'a>(tokens: &[Token<'a>]) -> Result<Statement<'a>> {
    if tokens.len() < 1 {
        return Err(anyhow!(
            "Expected at least 1 token, found {:?}",
            tokens.len()
        ));
    }
    match &tokens[0] {
        Token::ID("move_right") => Ok(Statement::Move(Direction::Right)),
        Token::ID("move_left") => Ok(Statement::Move(Direction::Left)),
        _ => Err(anyhow!(
            "Expected move_right or move_left, found {:?}",
            tokens
        )),
    }
}

fn try_parse_assign<'a>(tokens: &[Token<'a>]) -> Result<Statement<'a>> {
    if tokens.len() < 3 {
        return Err(anyhow!(
            "Expected at least 3 tokens, found {:?}",
            tokens.len()
        ));
    }
    match &tokens[..3] {
        [id, Token::ASSIGN, num] => {
            let variable = Variable::try_from(id)?;
            let num = Num::try_from(num)?;
            Ok(Statement::Assign(variable, num))
        }
        _ => Err(anyhow!("Expected Variable = NUMBER, found {:?}", tokens)),
    }
}

fn try_parse_while<'a>(tokens: &[Token<'a>]) -> Result<Statement<'a>> {
    if tokens.len() < 6 {
        return Err(anyhow!(
            "Expected at least 6 tokens, found {:?}",
            tokens.len()
        ));
    }
    match &tokens {
        [Token::ID("while"), rest @ ..] => {
            let bools = Bool::try_from(rest)?;
            let rest = &rest[bools.len()..];
            match rest {
                [Token::LB, statements @ ..] => {
                    let statements = Function::try_from(statements)?;
                    let consumed = statements.len() + 1; // +1 for LB
                    if rest[consumed] != Token::RB {
                        Err(anyhow!("Expected }}, found {:?}", rest[statements.len()]))
                    } else {
                        Ok(Statement::WHILE(bools, statements))
                    }
                }
                _ => Err(anyhow!(
                    "Expected while Bool {{ statements }}, found {:?}",
                    tokens
                )),
            }
        }
        _ => Err(anyhow!(
            "Expected while Bool {{ statements }}, found {:?}",
            tokens
        )),
    }
}

fn try_parse_if<'a>(tokens: &[Token<'a>]) -> Result<Statement<'a>> {
    if tokens.len() < 6 {
        return Err(anyhow!(
            "Expected at least 6 tokens, found {:?}",
            tokens.len()
        ));
    }
    match &tokens {
        [Token::ID("if"), rest @ ..] => {
            let bools = Bool::try_from(rest)?;
            let rest = &rest[bools.len()..];
            match rest {
                [Token::LB, statements @ ..] => {
                    let if_func = Function::try_from(statements)?;
                    let last_token = &statements[if_func.len()];
                    if *last_token != Token::RB {
                        Err(anyhow!("Expected }}, found {:?}", last_token))
                    } else {
                        Ok(Statement::IF(bools, if_func, None))
                    }
                }
                _ => Err(anyhow!(
                    "Expected while Bool {{ statements }}, found {:?}",
                    tokens
                )),
            }
        }
        _ => Err(anyhow!(
            "Expected while Bool {{ statements }}, found {:?}",
            tokens
        )),
    }
}

fn try_parse_else<'a>(tokens: &[Token<'a>]) -> Option<Function<'a>> {
    match tokens {
        [Token::ID("else"), Token::LB, statements @ ..] => {
            let else_func = Function::try_from(statements).ok()?;
            let last_token = &statements[else_func.len()];
            if *last_token != Token::RB {
                None
            } else {
                Some(else_func)
            }
        }
        _ => None,
    }
}

fn try_parse_if_else<'a>(tokens: &[Token<'a>]) -> Result<Statement<'a>> {
    let if_statement = try_parse_if(tokens)?;
    let else_func = try_parse_else(&tokens[if_statement.len()..]);
    let Statement::IF(bools, if_func, _) = if_statement else {
        panic!("Expected IF statement, found {:?}", if_statement)
    };
    Ok(Statement::IF(bools, if_func, else_func))
}

impl<'a> TryFrom<&[Token<'a>]> for Statement<'a> {
    type Error = anyhow::Error;
    fn try_from(value: &[Token<'a>]) -> Result<Self> {
        let try_matches = [
            try_parse_input,
            try_parse_output,
            try_parse_move,
            try_parse_assign,
            try_parse_while,
            try_parse_if_else,
        ];
        for try_match in try_matches {
            if let Ok(statement) = try_match(value) {
                return Ok(statement);
            }
        }
        Err(anyhow!("No match found for {:?}", value))
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Bool<'a> {
    compares: Vec<Compare<'a>>,
}

impl Bool<'_> {
    fn len(&self) -> usize {
        if self.compares.is_empty() {
            return 0;
        } else {
            3 + 4 * (self.compares.len() - 1)
        }
    }
}

impl<'a> TryFrom<&[Token<'a>]> for Bool<'a> {
    type Error = anyhow::Error;
    fn try_from(value: &[Token<'a>]) -> Result<Self> {
        if value.len() < 3 {
            return Err(anyhow!(
                "Expected at least 3 tokens, found {:?}",
                value.len()
            ));
        }
        let (first_compare, rest_compares) = value.split_at(3);
        let Ok(compare) = Compare::try_from(first_compare) else {
            return Err(anyhow!("Expected a compare, found {:?}", first_compare));
        };
        let mut compares = vec![compare];
        let mut rest_compares = rest_compares.chunks_exact(4);
        loop {
            match rest_compares.next() {
                Some([Token::AND, compare @ ..]) => {
                    let Ok(compare) = Compare::try_from(compare) else {
                        break;
                    };
                    compares.push(compare);
                }
                _ => break,
            }
        }
        Ok(Self { compares })
    }
}

#[derive(Debug, PartialEq, Clone)]
enum Compare<'a> {
    EQ(Variable<'a>, Num),
    NE(Variable<'a>, Num),
}

impl<'a> TryFrom<&[Token<'a>]> for Compare<'a> {
    type Error = anyhow::Error;
    fn try_from(tokens: &[Token<'a>]) -> Result<Self> {
        if tokens.len() < 3 {
            return Err(anyhow!("Expected at least 3 tokens, found {:?}", tokens));
        }
        let variable = Variable::try_from(&tokens[0])?;
        let num = Num::try_from(&tokens[2])?;
        match tokens[1] {
            Token::EQ => Ok(Self::EQ(variable, num)),
            Token::NE => Ok(Self::NE(variable, num)),
            _ => Err(anyhow!("Expected == or !=, found {:?}", tokens[1])),
        }
    }
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

pub fn parse(tokens: TokenStream) -> Result<Function<'_>> {
    todo!();
}

#[cfg(test)]
mod parser {
    use super::*;
    macro_rules! test_all_cases {
        ($testcases: expr, $type: ty) => {
            for (token, expect) in $testcases.iter() {
                let output = <$type>::try_from(token);
                if let Ok(expect) = expect {
                    assert_eq!(output.unwrap(), *expect);
                } else {
                    assert!(output.is_err());
                }
            }
        };
    }
    macro_rules! test_all_cases_vec {
        ($testcases: expr, $type: ty) => {
            for (token, expect) in $testcases.iter() {
                let output = <$type>::try_from(&**token);
                if let Ok(expect) = expect {
                    assert_eq!(output.unwrap(), *expect);
                } else {
                    assert!(output.is_err());
                }
            }
        };
    }
    #[test]
    fn test_parse_direction() {
        let testcase = [
            (Token::ID("move_right"), Ok(Direction::Right)),
            (Token::ID("move_left"), Ok(Direction::Left)),
            (Token::ID("abcd"), Err(())),
        ];
        test_all_cases!(testcase, Direction);
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
        let testcase = testcase
            .iter()
            .chain(reserved_words.iter())
            .collect::<Vec<_>>();
        test_all_cases!(testcase, Variable);
    }
    #[test]
    fn test_parse_num() {
        let testcase = [
            (Token::NUM("123"), Ok(Num(123))),
            (Token::ID("hello"), Err(())),
        ];
        test_all_cases!(testcase, Num);
    }
    #[test]
    fn test_parse_compare() {
        let testcase = [
            (
                vec![Token::ID("hello"), Token::EQ, Token::NUM("123")],
                Ok(Compare::EQ(Variable("hello"), Num(123))),
            ),
            (
                vec![Token::NUM("123"), Token::EQ, Token::ID("hello")],
                Err(()),
            ),
            (
                vec![Token::ID("hello"), Token::NE, Token::NUM("123")],
                Ok(Compare::NE(Variable("hello"), Num(123))),
            ),
        ];
        test_all_cases_vec!(testcase, Compare);
    }
    #[test]
    fn test_parse_bool() {
        let testcase = [
            (
                vec![Token::ID("hello"), Token::EQ, Token::NUM("123")],
                Ok(Bool {
                    compares: vec![Compare::EQ(Variable("hello"), Num(123))],
                }),
            ),
            (
                vec![Token::ID("hello"), Token::EQ, Token::NUM("123"), Token::LB],
                Ok(Bool {
                    compares: vec![Compare::EQ(Variable("hello"), Num(123))],
                }),
            ),
            (
                vec![Token::ID("hello"), Token::EQ, Token::ID("123")],
                Err(()),
            ),
            (
                vec![
                    Token::ID("hello"),
                    Token::EQ,
                    Token::NUM("123"),
                    Token::AND,
                    Token::ID("world"),
                    Token::EQ,
                    Token::NUM("124"),
                ],
                Ok(Bool {
                    compares: vec![
                        Compare::EQ(Variable("hello"), Num(123)),
                        Compare::EQ(Variable("world"), Num(124)),
                    ],
                }),
            ),
        ];
        test_all_cases_vec!(testcase, Bool);
    }
    #[test]
    fn test_parse_input() {
        let testcase = [
            (
                vec![Token::ID("input"), Token::LP, Token::ID("hello"), Token::RP],
                Ok(Statement::Input(Variable("hello"))),
            ),
            (
                vec![Token::ID("input"), Token::LP, Token::NUM("123"), Token::RP],
                Err(()),
            ),
        ];
        test_all_cases_vec!(testcase, Statement);
    }
    #[test]
    fn test_parse_output() {
        let testcase = [
            (
                vec![
                    Token::ID("output"),
                    Token::LP,
                    Token::ID("hello"),
                    Token::RP,
                ],
                Ok(Statement::Output(Variable("hello"))),
            ),
            (
                vec![Token::ID("output"), Token::LP, Token::NUM("123"), Token::RP],
                Err(()),
            ),
        ];
        test_all_cases_vec!(testcase, Statement);
    }
    #[test]
    fn test_parse_move_statement() {
        let testcase = [
            (
                vec![Token::ID("move_right")],
                Ok(Statement::Move(Direction::Right)),
            ),
            (
                vec![Token::ID("move_left")],
                Ok(Statement::Move(Direction::Left)),
            ),
            (vec![Token::ID("abcd")], Err(())),
        ];
        test_all_cases_vec!(testcase, Statement);
    }
    #[test]
    fn test_parse_assign() {
        let testcase = [
            (
                vec![Token::ID("hello"), Token::ASSIGN, Token::NUM("123")],
                Ok(Statement::Assign(Variable("hello"), Num(123))),
            ),
            (
                vec![Token::NUM("123"), Token::ASSIGN, Token::ID("hello")],
                Err(()),
            ),
            (
                vec![Token::ID("Hello"), Token::ASSIGN, Token::ID("hello")],
                Err(()),
            ),
        ];
        test_all_cases_vec!(testcase, Statement);
    }
    #[test]
    fn test_parse_while() {
        let testcase = [
            (
                "while abc == 123 { input ( cde ) }",
                Ok((
                    vec![Compare::EQ(Variable("abc"), Num(123))],
                    vec![Statement::Input(Variable("cde"))],
                )),
            ),
            ("while abc == 123 input ( cde ) }", Err(())),
            ("while { input ( cde ) }", Err(())),
            (
                "while abc == 123 && efg != 124 { input ( hij ) }",
                Ok((
                    vec![
                        Compare::EQ(Variable("abc"), Num(123)),
                        Compare::NE(Variable("efg"), Num(124)),
                    ],
                    vec![Statement::Input(Variable("hij"))],
                )),
            ),
        ]
        .into_iter()
        .map(|(s, r)| {
            let tokens = TokenStream::try_from(s).unwrap().into_tokens();
            let expect = r.map(|(compares, statement)| {
                Statement::WHILE(Bool { compares }, Function(statement))
            });
            (tokens, expect)
        })
        .collect::<Vec<_>>();

        test_all_cases_vec!(testcase, Statement);
    }
    #[test]
    fn test_parse_if() {
        let testcase = [
            (
                "if abc == 123 { input ( cde ) }",
                Ok((
                    vec![Compare::EQ(Variable("abc"), Num(123))],
                    vec![Statement::Input(Variable("cde"))],
                )),
            ),
            ("if abc == 123 input ( cde ) }", Err(())),
        ]
        .into_iter()
        .map(|(s, r)| {
            let tokens = TokenStream::try_from(s).unwrap().into_tokens();
            let expect = r.map(|(compares, statement)| {
                Statement::IF(Bool { compares }, Function(statement), None)
            });
            (tokens, expect)
        })
        .collect::<Vec<_>>();
        test_all_cases_vec!(testcase, Statement);
    }
    #[test]
    fn test_parse_if_else() {
        let testcase = [
            (
                "if abc == 123 { input ( cde ) } else { output ( fgh ) }",
                Ok((
                    vec![Compare::EQ(Variable("abc"), Num(123))],
                    vec![Statement::Input(Variable("cde"))],
                    vec![Statement::Output(Variable("fgh"))],
                )),
            ),
            (
                "if abc == 123 input ( cde ) } else output ( efg ) }",
                Err(()),
            ),
        ]
        .into_iter()
        .map(|(s, r)| {
            let tokens = TokenStream::try_from(s).unwrap().into_tokens();
            let expect = r.map(|(compares, statement_if, statement_else)| {
                Statement::IF(
                    Bool { compares },
                    Function(statement_if),
                    Some(Function(statement_else)),
                )
            });
            (tokens, expect)
        })
        .collect::<Vec<_>>();
        test_all_cases_vec!(testcase, Statement);
    }
    #[test]
    fn test_parse_ast() {
        let testcase: Vec<(_, Result<AST>)> = vec![(
            "
while state != 0 {
    if state == 1 {
        if symbol == 0 {
            symbol = 1
            move_left
            state = 0
        } else {
            if symbol == 1 {
                symbol = 0
                move_right
                state = 0
            }
        }
    }
",
            Ok(AST(Function(vec![Statement::WHILE(
                Bool {
                    compares: vec![Compare::NE(Variable("state"), Num(0))],
                },
                Function(vec![Statement::IF(
                    Bool {
                        compares: vec![Compare::EQ(Variable("state"), Num(1))],
                    },
                    Function(vec![Statement::IF(
                        Bool {
                            compares: vec![Compare::EQ(Variable("symbol"), Num(0))],
                        },
                        Function(vec![
                            Statement::Assign(Variable("symbol"), Num(1)),
                            Statement::Move(Direction::Left),
                            Statement::Assign(Variable("state"), Num(0)),
                        ]),
                        Some(Function(vec![Statement::IF(
                            Bool {
                                compares: vec![Compare::EQ(Variable("symbol"), Num(1))],
                            },
                            Function(vec![
                                Statement::Assign(Variable("symbol"), Num(0)),
                                Statement::Move(Direction::Right),
                                Statement::Assign(Variable("state"), Num(0)),
                            ]),
                            None,
                        )])),
                    )]),
                    None,
                )]),
            )]))),
        )]
        .into_iter()
        .map(|(s, expect)| {
            let tokens = TokenStream::try_from(s).unwrap().into_tokens();
            (tokens, expect)
        })
        .collect::<Vec<_>>();

        test_all_cases_vec!(testcase, AST);
    }
}
