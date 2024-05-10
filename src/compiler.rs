use crate::generator::code_gen;
use crate::parser::AST;
use crate::scanner::TokenStream;
use anyhow::{Result};

pub fn compile(program: &str) -> Result<String> {
    let tokens = TokenStream::try_from(program)?;
    let tokens = tokens.into_tokens();
    let ast = AST::try_from(&*tokens)?;
    code_gen(&ast)
}
