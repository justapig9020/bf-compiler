use crate::assembler::{Asm, Value, Variable};
use crate::parser::{Bool, Compare, Direction, Function, Statement, AST};
use anyhow::{anyhow, Result};
use std::collections::HashSet;

const TEMP_VAR: &str = "tmp";
const IF_FLAG: &str = "IF";
const ELSE_FLAG: &str = "ELSE";

impl From<AST<'_>> for Vec<Asm> {
    fn from(ast: AST) -> Self {
        ast.statements()
            .into_iter()
            .map(|stmt| Vec::<Asm>::from(stmt))
            .flatten()
            .collect()
    }
}

fn generate_if_else_inner(
    conditions: &[Compare],
    func_if: &Function,
    func_else: Option<&Function>,
) -> Vec<Asm> {
    match conditions {
        [first, rest @ ..] => match first {
            Compare::EQ(var, num) => [
                vec![
                    Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(1)),
                    Asm::Copy(
                        Variable::new(var),
                        vec![Variable::new(TEMP_VAR), Variable::new(IF_FLAG)],
                    ),
                    Asm::Copy(Variable::new(TEMP_VAR), vec![Variable::new(var)]),
                    Asm::Sub(Variable::new(IF_FLAG), Value::new_num(num.into())),
                    Asm::Loop(Variable::new(IF_FLAG)),
                ],
                if let Some(func) = func_else {
                    func.statements()
                        .into_iter()
                        .map(|stmt| Vec::<Asm>::from(stmt))
                        .flatten()
                        .collect()
                } else {
                    vec![]
                },
                vec![
                    Asm::Set(Variable::new(IF_FLAG), Value::new_num(0)),
                    Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(0)),
                    Asm::End(Variable::new(IF_FLAG)),
                    Asm::Loop(Variable::new(ELSE_FLAG)),
                ],
                generate_if_else_inner(rest, func_if, None),
                vec![
                    Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(0)),
                    Asm::End(Variable::new(ELSE_FLAG)),
                ],
            ]
            .concat(),
            Compare::NE(var, num) => [
                vec![
                    Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(1)),
                    Asm::Copy(
                        Variable::new(var),
                        vec![Variable::new(TEMP_VAR), Variable::new(IF_FLAG)],
                    ),
                    Asm::Copy(Variable::new(TEMP_VAR), vec![Variable::new(var)]),
                    Asm::Sub(Variable::new(IF_FLAG), Value::new_num(num.into())),
                    Asm::Loop(Variable::new(IF_FLAG)),
                ],
                generate_if_else_inner(rest, func_if, None),
                vec![
                    Asm::Set(Variable::new(IF_FLAG), Value::new_num(0)),
                    Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(0)),
                    Asm::End(Variable::new(IF_FLAG)),
                ],
                if let Some(func) = func_else {
                    [
                        vec![Asm::Loop(Variable::new(ELSE_FLAG))],
                        func.statements()
                            .into_iter()
                            .map(|stmt| Vec::<Asm>::from(stmt))
                            .flatten()
                            .collect(),
                        vec![
                            Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(0)),
                            Asm::End(Variable::new(ELSE_FLAG)),
                        ],
                    ]
                    .concat()
                } else {
                    vec![]
                },
            ]
            .concat(),
        },
        [] => func_if
            .statements()
            .into_iter()
            .map(|stmt| Vec::<Asm>::from(stmt))
            .flatten()
            .collect(),
    }
}

fn generate_if_else(
    condition: &Bool,
    func_if: &Function,
    func_else: &Option<Function>,
) -> Vec<Asm> {
    generate_if_else_inner(condition.compares(), func_if, func_else.as_ref())
}

impl From<&Statement<'_>> for Vec<Asm> {
    fn from(stmt: &Statement) -> Self {
        match stmt {
            Statement::Input(var) => vec![Asm::Read(Variable::new(var))],
            Statement::Output(var) => vec![Asm::Write(Variable::new(var))],
            Statement::Assign(var, val) => {
                vec![Asm::Set(Variable::new(var), Value::new_num(val.into()))]
            }
            Statement::Move(direction) => match direction {
                Direction::Right => vec![Asm::Rs(Value::new_const("__cell_size"))],
                Direction::Left => vec![Asm::Ls(Value::new_const("__cell_size"))],
            },
            Statement::WHILE(condition, func) => todo!(),
            Statement::IF(condition, func_if, func_else) => {
                generate_if_else(condition, func_if, func_else)
            }
        }
    }
}

fn list_variables_bool(b: &Bool) -> HashSet<String> {
    b.compares()
        .iter()
        .map(|c| {
            use crate::parser::Compare;
            match c {
                Compare::EQ(var, _) => var.to_string(),
                Compare::NE(var, _) => var.to_string(),
            }
        })
        .collect()
}

fn list_variables_statement(
    stmt: &Statement,
    if_level: usize,
    while_level: usize,
) -> HashSet<String> {
    match stmt {
        Statement::Input(var) => HashSet::from([var.to_string()]),
        Statement::Output(var) => HashSet::from([var.to_string()]),
        Statement::Assign(var, _) => HashSet::from([var.to_string()]),
        Statement::WHILE(cond, stmt) => {
            let mut variables = HashSet::from([format!("__while_{}", while_level)]);
            variables.extend(list_variables_bool(cond));
            for stmt in stmt.statements() {
                variables.extend(list_variables_statement(stmt, if_level, while_level + 1))
            }
            variables
        }
        Statement::IF(cond, if_func, else_func) => {
            let mut variables = HashSet::from([format!("__if_{}", if_level)]);
            variables.extend(list_variables_bool(cond));
            for stmt in if_func.statements() {
                variables.extend(list_variables_statement(stmt, if_level + 1, while_level))
            }
            if let Some(else_func) = else_func {
                for stmt in else_func.statements() {
                    variables.extend(list_variables_statement(stmt, if_level, while_level));
                }
            }
            variables
        }
        Statement::Move(_) => HashSet::new(),
    }
}

fn list_variables(ast: &AST) -> HashSet<String> {
    let mut variables = HashSet::new();
    for stmt in ast.statements() {
        variables.extend(list_variables_statement(stmt, 0, 0));
    }
    variables
}

pub fn code_gen(ast: &AST) -> Result<String> {
    let variables = list_variables(ast);
    todo!();
}

#[cfg(test)]
mod generator {
    use super::*;
    use crate::{assembler::Variable, scanner::TokenStream};
    fn compile(program: &str) -> Result<Vec<Asm>> {
        let tokens = TokenStream::try_from(program)?;
        let tokens = tokens.into_tokens();
        let ast = AST::try_from(&*tokens)?;
        let asm = Vec::<Asm>::from(ast);
        Ok(asm.clone())
    }
    fn test_list_variables(testcases: &[(&str, HashSet<&str>)]) {
        for (program, expect) in testcases {
            println!("{program:?}");
            let tokens = TokenStream::try_from(*program).unwrap();
            println!("{tokens:?}");
            let ast = AST::try_from(&*tokens.into_tokens()).unwrap();
            let variables = list_variables(&ast);
            let expect: HashSet<String> = expect.iter().map(|s| s.to_string()).collect();
            assert_eq!(variables, expect);
        }
    }
    #[test]
    fn test_list_pure_variables() {
        let testcases = [
            ("input ( x )", HashSet::from(["x"])),
            (
                "output ( y )\nx = 1\noutput ( x )",
                HashSet::from(["y", "x"]),
            ),
        ];
        test_list_variables(&testcases);
    }
    #[test]
    fn test_list_if_variables() {
        let testcases = [
            (
                "if  x == 1 { input ( y ) }",
                HashSet::from(["x", "y", "__if_0"]),
            ),
            (
                "if x == 1 {
                    if y == 1 {
                        input ( y )
                    }
                } else {
                    input ( z )
                }",
                HashSet::from(["x", "y", "z", "__if_0", "__if_1"]),
            ),
            (
                "if x == 1 {
                    input ( y ) 
                } else { 
                    if y == 1 { 
                        input ( z ) 
                    } 
                }",
                HashSet::from(["x", "y", "z", "__if_0"]),
            ),
        ];
        test_list_variables(&testcases);
    }
    #[test]
    fn test_input() {
        let program = "input ( x )";
        let asm = compile(program).unwrap();
        let expect = vec![Asm::Read(Variable::new("x"))];
        assert_eq!(asm, expect);
    }
    #[test]
    fn test_output() {
        let program = "output ( x )";
        let asm = compile(program).unwrap();
        let expect = vec![Asm::Write(Variable::new("x"))];
        assert_eq!(asm, expect);
    }
    #[test]
    fn test_assign() {
        let program = "x = 1";
        let asm = compile(program).unwrap();
        let expect = vec![Asm::Set(Variable::new("x"), Value::new_num(1))];
        assert_eq!(asm, expect);
    }
    #[test]
    fn test_move_right() {
        let program = "move_right";
        let asm = compile(program).unwrap();
        let expect = vec![Asm::Rs(Value::new_const("__cell_size"))];
        assert_eq!(asm, expect);
    }
    #[test]
    fn test_move_left() {
        let program = "move_left";
        let asm = compile(program).unwrap();
        let expect = vec![Asm::Ls(Value::new_const("__cell_size"))];
        assert_eq!(asm, expect);
    }
    #[test]
    fn test_single_ne_condition_if() {
        let program = "if a != 10 { input ( x ) }";
        let asm = compile(program).unwrap();
        let expect = vec![
            Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(1)),
            Asm::Copy(
                Variable::new("a"),
                vec![Variable::new(TEMP_VAR), Variable::new(IF_FLAG)],
            ),
            Asm::Copy(Variable::new(TEMP_VAR), vec![Variable::new("a")]),
            Asm::Sub(Variable::new(IF_FLAG), Value::new_num(10)),
            Asm::Loop(Variable::new(IF_FLAG)),
            Asm::Read(Variable::new("x")),
            Asm::Set(Variable::new(IF_FLAG), Value::new_num(0)),
            Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(0)),
            Asm::End(Variable::new(IF_FLAG)),
        ];
        assert_eq!(asm, expect);
    }
    #[test]
    fn test_single_eq_condition_if() {
        let program = "if a == 10 { input ( x )  }";
        let asm = compile(program).unwrap();
        let expect = vec![
            Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(1)),
            Asm::Copy(
                Variable::new("a"),
                vec![Variable::new(TEMP_VAR), Variable::new(IF_FLAG)],
            ),
            Asm::Copy(Variable::new(TEMP_VAR), vec![Variable::new("a")]),
            Asm::Sub(Variable::new(IF_FLAG), Value::new_num(10)),
            Asm::Loop(Variable::new(IF_FLAG)),
            Asm::Set(Variable::new(IF_FLAG), Value::new_num(0)),
            Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(0)),
            Asm::End(Variable::new(IF_FLAG)),
            Asm::Loop(Variable::new(ELSE_FLAG)),
            Asm::Read(Variable::new("x")),
            Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(0)),
            Asm::End(Variable::new(ELSE_FLAG)),
        ];
        assert_eq!(asm, expect);
    }
    #[test]
    fn test_single_ne_condition_if_else() {
        let program = "if a != 10 { input ( x ) } else { input ( y ) }";
        let asm = compile(program).unwrap();
        let expect = vec![
            Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(1)),
            Asm::Copy(
                Variable::new("a"),
                vec![Variable::new(TEMP_VAR), Variable::new(IF_FLAG)],
            ),
            Asm::Copy(Variable::new(TEMP_VAR), vec![Variable::new("a")]),
            Asm::Sub(Variable::new(IF_FLAG), Value::new_num(10)),
            Asm::Loop(Variable::new(IF_FLAG)),
            Asm::Read(Variable::new("x")),
            Asm::Set(Variable::new(IF_FLAG), Value::new_num(0)),
            Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(0)),
            Asm::End(Variable::new(IF_FLAG)),
            Asm::Loop(Variable::new(ELSE_FLAG)),
            Asm::Read(Variable::new("y")),
            Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(0)),
            Asm::End(Variable::new(ELSE_FLAG)),
        ];
        assert_eq!(asm, expect);
    }
    #[test]
    fn test_single_eq_condition_if_else() {
        let program = "if a == 10 { input ( x ) } else { input ( y ) }";
        let asm = compile(program).unwrap();
        let expect = vec![
            Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(1)),
            Asm::Copy(
                Variable::new("a"),
                vec![Variable::new(TEMP_VAR), Variable::new(IF_FLAG)],
            ),
            Asm::Copy(Variable::new(TEMP_VAR), vec![Variable::new("a")]),
            Asm::Sub(Variable::new(IF_FLAG), Value::new_num(10)),
            Asm::Loop(Variable::new(IF_FLAG)),
            Asm::Read(Variable::new("y")),
            Asm::Set(Variable::new(IF_FLAG), Value::new_num(0)),
            Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(0)),
            Asm::End(Variable::new(IF_FLAG)),
            Asm::Loop(Variable::new(ELSE_FLAG)),
            Asm::Read(Variable::new("x")),
            Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(0)),
            Asm::End(Variable::new(ELSE_FLAG)),
        ];
        assert_eq!(asm, expect);
    }
    #[test]
    fn test_multi_condition_if() {
        let program = "if a == 10 && b != 11 { input ( x ) }";
        let asm = compile(program).unwrap();
        let expect = vec![
            Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(1)),
            Asm::Copy(
                Variable::new("a"),
                vec![Variable::new(TEMP_VAR), Variable::new(IF_FLAG)],
            ),
            Asm::Copy(Variable::new(TEMP_VAR), vec![Variable::new("a")]),
            Asm::Sub(Variable::new(IF_FLAG), Value::new_num(10)),
            Asm::Loop(Variable::new(IF_FLAG)),
            Asm::Set(Variable::new(IF_FLAG), Value::new_num(0)),
            Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(0)),
            Asm::End(Variable::new(IF_FLAG)),
            Asm::Loop(Variable::new(ELSE_FLAG)),
            Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(1)),
            Asm::Copy(
                Variable::new("b"),
                vec![Variable::new(TEMP_VAR), Variable::new(IF_FLAG)],
            ),
            Asm::Copy(Variable::new(TEMP_VAR), vec![Variable::new("b")]),
            Asm::Sub(Variable::new(IF_FLAG), Value::new_num(11)),
            Asm::Loop(Variable::new(IF_FLAG)),
            Asm::Read(Variable::new("x")),
            Asm::Set(Variable::new(IF_FLAG), Value::new_num(0)),
            Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(0)),
            Asm::End(Variable::new(IF_FLAG)),
            Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(0)),
            Asm::End(Variable::new(ELSE_FLAG)),
        ];
        assert_eq!(asm, expect);
    }
}
