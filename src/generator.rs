use crate::assembler::{Asm, Value, Variable};
use crate::parser::{Statement, AST};
use anyhow::{anyhow, Result};

impl<'a> From<&'a AST<'a>> for Vec<Asm<'a>> {
    fn from(ast: &'a AST) -> Self {
        ast.statements().iter().map(|stmt| stmt.into()).collect()
    }
}

impl<'a> From<&'a Statement<'a>> for Asm<'a> {
    fn from(stmt: &'a Statement) -> Self {
        match stmt {
            Statement::Input(var) => Asm::Read(Variable::new(var)),
            Statement::Output(var) => Asm::Write(Variable::new(var)),
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
    fn compile(program: &str) -> Result<AST> {
        let tokens = TokenStream::try_from(program)?;
        let tokens = tokens.into_tokens();
        AST::try_from(&*tokens)
    }
    #[test]
    fn test_input() {
        let program = "input ( x )";
        let ast = compile(program).unwrap();
        let asm = Vec::<Asm>::from(&ast);
        let expect = vec![Asm::Read(Variable::new("x"))];
        assert_eq!(asm, expect);
    }
    #[test]
    fn test_output() {
        let program = "output ( x )";
        let ast = compile(program).unwrap();
        let asm = Vec::<Asm>::from(&ast);
        let expect = vec![Asm::Write(Variable::new("x"))];
        assert_eq!(asm, expect);
    }
}
