mod assembler;
mod compiler;
mod generator;
mod parser;
mod scanner;
use crate::assembler::assemble;
use anyhow::Result;
use clap::Parser;
use std::fs::File;
use std::io::{stdout, Write};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    source: String,
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let source = std::fs::read_to_string(args.source).unwrap();
    let program = assemble(&source)?;
    let mut output: Box<dyn Write> = if let Some(file_name) = args.output {
        let output_file = File::create(file_name)?;
        Box::new(output_file)
    } else {
        Box::new(stdout())
    };
    write!(output, "{}", program)?;
    Ok(())
}
