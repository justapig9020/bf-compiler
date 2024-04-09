use crate::assembler::{Asm, Value, Variable};
use crate::parser::{Direction, Statement, AST};
use anyhow::{anyhow, Result};

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
            _ => todo!(),
        }
    }
}

pub fn code_gen(ast: &AST) -> Result<String> {
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
}
