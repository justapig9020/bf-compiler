use crate::assembler::{Asm, Value, Variable};
use crate::parser::{Statement, AST};
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
            Statement::Assign(var, val) => Asm::Set(Variable::new(var), Value::new(val.into())),
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
        let expect = vec![Asm::Set(Variable::new("x"), Value::new(1))];
        assert_eq!(asm, expect);
    }
}
