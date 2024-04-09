use crate::assembler::{Asm, Value, Variable};
use crate::parser::{Bool, Direction, Statement, AST};
use anyhow::{anyhow, Result};
use std::collections::HashSet;

impl From<AST<'_>> for Vec<Asm> {
    fn from(ast: AST) -> Self {
        ast.statements()
            .into_iter()
            .map(|stmt| stmt.into())
            .collect()
    }
}

impl From<&Statement<'_>> for Asm {
    fn from(stmt: &Statement) -> Self {
        match stmt {
            Statement::Input(var) => Asm::Read(Variable::new(var)),
            Statement::Output(var) => Asm::Write(Variable::new(var)),
            Statement::Assign(var, val) => Asm::Set(Variable::new(var), Value::new_num(val.into())),
            Statement::Move(direction) => match direction {
                Direction::Right => Asm::Rs(Value::new_const("__cell_size")),
                Direction::Left => Asm::Ls(Value::new_const("__cell_size")),
            },
            Statement::WHILE(condition, func) => todo!(),
            _ => todo!(),
        }
    }
}

fn list_variables_bool(b: &Bool) -> HashSet<String> {
    todo!();
}

fn list_variables_statement(stmt: &Statement) -> HashSet<String> {
    match stmt {
        Statement::Input(var) => HashSet::from([var.to_string()]),
        Statement::Output(var) => HashSet::from([var.to_string()]),
        Statement::Assign(var, _) => HashSet::from([var.to_string()]),
        Statement::WHILE(cond, stmt) => {
            let mut variables = list_variables_bool(cond);
            for stmt in stmt.statements() {
                variables.extend(list_variables_statement(stmt));
            }
            variables
        }
        Statement::IF(cond, if_func, else_func) => {
            let mut variables = list_variables_bool(cond);
            for stmt in if_func.statements() {
                variables.extend(list_variables_statement(stmt));
            }
            if let Some(else_func) = else_func {
                for stmt in else_func.statements() {
                    variables.extend(list_variables_statement(stmt));
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
        variables.extend(list_variables_statement(stmt));
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
    use crate::scanner::TokenStream;
    fn compile(program: &str) -> Result<Vec<Asm>> {
        let tokens = TokenStream::try_from(program)?;
        let tokens = tokens.into_tokens();
        let ast = AST::try_from(&*tokens)?;
        let asm = Vec::<Asm>::from(ast);
        Ok(asm.clone())
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
        for (program, expect) in testcases {
            let tokens = TokenStream::try_from(program).unwrap();
            let ast = AST::try_from(&*tokens.into_tokens()).unwrap();
            let variables = list_variables(&ast);
            let expect: HashSet<String> = expect.iter().map(|s| s.to_string()).collect();
            assert_eq!(variables, expect);
        }
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
    fn test_while() {
        let program = "input ( x )\nwhile ( x ) { move_right }";
    }
}
