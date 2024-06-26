use crate::assembler::{Value, Variable};
use crate::parser::{self, Bool, Compare, Direction, Function, Statement, AST};
use anyhow::{anyhow, Result};
use std::collections::HashSet;
use std::fmt::Display;

const TEMP_VAR: &str = "tmp";
const IF_FLAG: &str = "IF";
const ELSE_FLAG: &str = "ELSE";
const IS_EQ: &str = "IS_EQ";
const WHILE_FLAG: &str = "WHILE";
const RESERVED_VARIABLES: [&str; 5] = [TEMP_VAR, IF_FLAG, ELSE_FLAG, IS_EQ, WHILE_FLAG];

#[derive(Debug, PartialEq, Clone)]
pub enum Asm {
    Define(Variable, Value),
    Add(Variable, Value),
    Sub(Variable, Value),
    Set(Variable, Value),
    Rs(Value),
    Ls(Value),
    Loop(Variable),
    End(Variable),
    Copy(Variable, Vec<Variable>),
    Read(Variable),
    Write(Variable),
}

impl Display for Asm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Asm::*;
        match self {
            Define(var, val) => write!(f, "#define {} {}", var, val),
            Add(var, val) => write!(f, "add {} {}", var, val),
            Sub(var, val) => write!(f, "sub {} {}", var, val),
            Set(var, val) => write!(f, "set {} {}", var, val),
            Rs(val) => write!(f, "rs {}", val),
            Ls(val) => write!(f, "ls {}", val),
            Loop(var) => write!(f, "loop {}", var),
            End(var) => write!(f, "end {}", var),
            Copy(var, vars) => {
                let vars = vars
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(" ");
                write!(f, "copy {} {}", var, vars)
            }
            Read(var) => write!(f, "read {}", var),
            Write(var) => write!(f, "write {}", var),
        }
    }
}

impl From<&AST<'_>> for Vec<Asm> {
    fn from(ast: &AST) -> Self {
        statements_to_asm(ast.statements())
    }
}

fn statements_to_asm(statements: &[Statement]) -> Vec<Asm> {
    statements.iter().flat_map(Vec::<Asm>::from).collect()
}

fn generate_set_ne(var: &parser::Variable, val: &parser::Num, flag: &str) -> Vec<Asm> {
    let flag = Variable::new(flag);
    let val = Value::new_num(val.into());
    vec![
        Asm::Copy(
            Variable::new(var),
            vec![Variable::new(TEMP_VAR), flag.clone()],
        ),
        Asm::Copy(Variable::new(TEMP_VAR), vec![Variable::new(var)]),
        Asm::Sub(flag.clone(), val.clone()),
    ]
}

fn generate_if_flag(flag: &str, func: Vec<Asm>, set: Vec<(&str, u8)>) -> Vec<Asm> {
    let flag = Variable::new(flag);
    [
        vec![Asm::Loop(flag.clone())],
        func,
        set.iter()
            .map(|(var, val)| Asm::Set(Variable::new(var), Value::new_num(*val)))
            .collect(),
        vec![
            Asm::Set(flag.clone(), Value::new_num(0)),
            Asm::End(flag.clone()),
        ],
    ]
    .concat()
}

fn generate_flag_setup(setups: Vec<(&str, u8)>) -> Vec<Asm> {
    setups
        .iter()
        .map(|(var, val)| Asm::Set(Variable::new(var), Value::new_num(*val)))
        .collect()
}

fn generate_if(conditions: &[Compare], func_if: Vec<Asm>, flag: Vec<(&str, u8)>) -> Vec<Asm> {
    match conditions {
        [Compare::NE(var, val)] => [
            generate_set_ne(var, val, IF_FLAG),
            generate_if_flag(IF_FLAG, func_if, flag),
        ]
        .concat(),
        [Compare::EQ(var, val)] => [
            generate_flag_setup(vec![(IS_EQ, 1)]),
            generate_set_ne(var, val, IF_FLAG),
            generate_if_flag(IF_FLAG, vec![], vec![(IS_EQ, 0)]),
            generate_if_flag(IS_EQ, func_if, flag),
        ]
        .concat(),
        [Compare::NE(var, val), rest @ ..] => [
            generate_set_ne(var, val, IF_FLAG),
            generate_if_flag(IF_FLAG, generate_if(rest, func_if, flag), vec![]),
        ]
        .concat(),
        [Compare::EQ(var, val), rest @ ..] => [
            generate_flag_setup(vec![(IS_EQ, 1)]),
            generate_set_ne(var, val, IF_FLAG),
            generate_if_flag(IF_FLAG, vec![], vec![(IS_EQ, 0)]),
            generate_if_flag(IS_EQ, generate_if(rest, func_if, flag), vec![]),
        ]
        .concat(),
        [] => vec![],
    }
}

fn generate_if_else(
    condition: &Bool,
    func_if: &Function,
    func_else: &Option<Function>,
    flag: Vec<(&str, u8)>,
) -> Vec<Asm> {
    let func_if = statements_to_asm(func_if.statements());
    let flag = if func_else.is_some() {
        [flag, vec![(ELSE_FLAG, 0)]].concat()
    } else {
        flag
    };
    let setup_asm = if func_else.is_some() {
        vec![Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(1))]
    } else {
        vec![]
    };
    let if_asm = generate_if(condition.compares(), func_if, flag);
    let else_asm = if let Some(func_else) = func_else {
        let else_asm = statements_to_asm(func_else.statements());
        generate_if_flag(ELSE_FLAG, else_asm, vec![])
    } else {
        vec![]
    };
    [setup_asm, if_asm, else_asm].concat()
}

fn generate_while(condition: &Bool, func: &Function) -> Vec<Asm> {
    [
        vec![
            Asm::Set(Variable::new(WHILE_FLAG), Value::new_num(1)),
            Asm::Loop(Variable::new(WHILE_FLAG)),
            Asm::Set(Variable::new(WHILE_FLAG), Value::new_num(0)),
        ],
        generate_if_else(condition, func, &None, vec![(WHILE_FLAG, 1)]),
        vec![Asm::End(Variable::new(WHILE_FLAG))],
    ]
    .concat()
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
            Statement::WHILE(condition, func) => generate_while(condition, func),
            Statement::IF(condition, func_if, func_else) => {
                generate_if_else(condition, func_if, func_else, vec![])
            }
        }
    }
}

fn list_variables_bool(b: &Bool) -> HashSet<String> {
    b.compares()
        .iter()
        .map(|c| match c {
            Compare::EQ(var, _) => var.to_string(),
            Compare::NE(var, _) => var.to_string(),
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
            let mut variables = list_variables_bool(cond);
            for stmt in stmt.statements() {
                variables.extend(list_variables_statement(stmt, if_level, while_level + 1))
            }
            variables
        }
        Statement::IF(cond, if_func, else_func) => {
            let mut variables = list_variables_bool(cond);
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

fn check_reserved_variables(variables: &HashSet<String>) -> Result<()> {
    for var in RESERVED_VARIABLES.iter() {
        if variables.contains(*var) {
            return Err(anyhow!("Reserved variable name found: {}", var));
        }
    }
    Ok(())
}

pub fn code_gen(ast: &AST) -> Result<String> {
    let variables = list_variables(ast);
    check_reserved_variables(&variables)?;
    let mut variables = variables.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
    variables.extend_from_slice(&RESERVED_VARIABLES);
    let variable_define = variables
        .iter()
        .enumerate()
        .map(|(i, s)| format!("#define {} {}", s, i))
        .collect::<Vec<String>>()
        .join("\n");
    let cell_size = format!("#define __cell_size {}", variables.len());
    let asm = Vec::<Asm>::from(ast);
    let asm = asm
        .iter()
        .map(|a| a.to_string())
        .collect::<Vec<String>>()
        .join("\n");
    Ok(format!("{}\n{}\n{}", variable_define, cell_size, asm))
}

#[cfg(test)]
mod generator {
    use super::*;
    use crate::{assembler::Variable, scanner::TokenStream};
    fn compile(program: &str) -> Result<Vec<Asm>> {
        let tokens = TokenStream::try_from(program)?;
        let tokens = tokens.into_tokens();
        let ast = AST::try_from(&*tokens)?;
        let asm = Vec::<Asm>::from(&ast);
        Ok(asm.clone())
    }
    fn test_list_variables(testcases: &[(&str, HashSet<&str>)]) {
        for (program, expect) in testcases {
            let tokens = TokenStream::try_from(*program).unwrap();
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
            ("if  x == 1 { input ( y ) }", HashSet::from(["x", "y"])),
            (
                "if x == 1 {
                    if y == 1 {
                        input ( y )
                    }
                } else {
                    input ( z )
                }",
                HashSet::from(["x", "y", "z"]),
            ),
            (
                "if x == 1 {
                    input ( y ) 
                } else { 
                    if y == 1 { 
                        input ( z ) 
                    } 
                }",
                HashSet::from(["x", "y", "z"]),
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
            Asm::Copy(
                Variable::new("a"),
                vec![Variable::new(TEMP_VAR), Variable::new(IF_FLAG)],
            ),
            Asm::Copy(Variable::new(TEMP_VAR), vec![Variable::new("a")]),
            Asm::Sub(Variable::new(IF_FLAG), Value::new_num(10)),
            Asm::Loop(Variable::new(IF_FLAG)),
            Asm::Read(Variable::new("x")),
            Asm::Set(Variable::new(IF_FLAG), Value::new_num(0)),
            Asm::End(Variable::new(IF_FLAG)),
        ];
        assert_eq!(asm, expect);
    }
    #[test]
    fn test_single_eq_condition_if() {
        let program = "if a == 10 { input ( x )  }";
        let asm = compile(program).unwrap();
        let expect = vec![
            Asm::Set(Variable::new(IS_EQ), Value::new_num(1)),
            Asm::Copy(
                Variable::new("a"),
                vec![Variable::new(TEMP_VAR), Variable::new(IF_FLAG)],
            ),
            Asm::Copy(Variable::new(TEMP_VAR), vec![Variable::new("a")]),
            Asm::Sub(Variable::new(IF_FLAG), Value::new_num(10)),
            Asm::Loop(Variable::new(IF_FLAG)),
            Asm::Set(Variable::new(IS_EQ), Value::new_num(0)),
            Asm::Set(Variable::new(IF_FLAG), Value::new_num(0)),
            Asm::End(Variable::new(IF_FLAG)),
            Asm::Loop(Variable::new(IS_EQ)),
            Asm::Read(Variable::new("x")),
            Asm::Set(Variable::new(IS_EQ), Value::new_num(0)),
            Asm::End(Variable::new(IS_EQ)),
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
            Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(0)),
            Asm::Set(Variable::new(IF_FLAG), Value::new_num(0)),
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
            Asm::Set(Variable::new(IS_EQ), Value::new_num(1)),
            Asm::Copy(
                Variable::new("a"),
                vec![Variable::new(TEMP_VAR), Variable::new(IF_FLAG)],
            ),
            Asm::Copy(Variable::new(TEMP_VAR), vec![Variable::new("a")]),
            Asm::Sub(Variable::new(IF_FLAG), Value::new_num(10)),
            Asm::Loop(Variable::new(IF_FLAG)),
            Asm::Set(Variable::new(IS_EQ), Value::new_num(0)),
            Asm::Set(Variable::new(IF_FLAG), Value::new_num(0)),
            Asm::End(Variable::new(IF_FLAG)),
            Asm::Loop(Variable::new(IS_EQ)),
            Asm::Read(Variable::new("x")),
            Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(0)),
            Asm::Set(Variable::new(IS_EQ), Value::new_num(0)),
            Asm::End(Variable::new(IS_EQ)),
            Asm::Loop(Variable::new(ELSE_FLAG)),
            Asm::Read(Variable::new("y")),
            Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(0)),
            Asm::End(Variable::new(ELSE_FLAG)),
        ];
        assert_eq!(asm, expect);
    }
    #[test]
    fn test_multi_condition_if_eq() {
        let program = "if a == 10 && b != 11 { input ( x ) }";
        let asm = compile(program).unwrap();
        let expect = vec![
            Asm::Set(Variable::new(IS_EQ), Value::new_num(1)),
            Asm::Copy(
                Variable::new("a"),
                vec![Variable::new(TEMP_VAR), Variable::new(IF_FLAG)],
            ),
            Asm::Copy(Variable::new(TEMP_VAR), vec![Variable::new("a")]),
            Asm::Sub(Variable::new(IF_FLAG), Value::new_num(10)),
            Asm::Loop(Variable::new(IF_FLAG)),
            Asm::Set(Variable::new(IS_EQ), Value::new_num(0)),
            Asm::Set(Variable::new(IF_FLAG), Value::new_num(0)),
            Asm::End(Variable::new(IF_FLAG)),
            Asm::Loop(Variable::new(IS_EQ)),
            Asm::Copy(
                Variable::new("b"),
                vec![Variable::new(TEMP_VAR), Variable::new(IF_FLAG)],
            ),
            Asm::Copy(Variable::new(TEMP_VAR), vec![Variable::new("b")]),
            Asm::Sub(Variable::new(IF_FLAG), Value::new_num(11)),
            Asm::Loop(Variable::new(IF_FLAG)),
            Asm::Read(Variable::new("x")),
            Asm::Set(Variable::new(IF_FLAG), Value::new_num(0)),
            Asm::End(Variable::new(IF_FLAG)),
            Asm::Set(Variable::new(IS_EQ), Value::new_num(0)),
            Asm::End(Variable::new(IS_EQ)),
        ];
        assert_eq!(asm, expect);
    }
    #[test]
    fn test_multi_condition_if_ne() {
        let program = "if a != 10 && b == 11 { input ( x ) }";
        let asm = compile(program).unwrap();
        let expect = vec![
            Asm::Copy(
                Variable::new("a"),
                vec![Variable::new(TEMP_VAR), Variable::new(IF_FLAG)],
            ),
            Asm::Copy(Variable::new(TEMP_VAR), vec![Variable::new("a")]),
            Asm::Sub(Variable::new(IF_FLAG), Value::new_num(10)),
            Asm::Loop(Variable::new(IF_FLAG)),
            Asm::Set(Variable::new(IS_EQ), Value::new_num(1)),
            Asm::Copy(
                Variable::new("b"),
                vec![Variable::new(TEMP_VAR), Variable::new(IF_FLAG)],
            ),
            Asm::Copy(Variable::new(TEMP_VAR), vec![Variable::new("b")]),
            Asm::Sub(Variable::new(IF_FLAG), Value::new_num(11)),
            Asm::Loop(Variable::new(IF_FLAG)),
            Asm::Set(Variable::new(IS_EQ), Value::new_num(0)),
            Asm::Set(Variable::new(IF_FLAG), Value::new_num(0)),
            Asm::End(Variable::new(IF_FLAG)),
            Asm::Loop(Variable::new(IS_EQ)),
            Asm::Read(Variable::new("x")),
            Asm::Set(Variable::new(IS_EQ), Value::new_num(0)),
            Asm::End(Variable::new(IS_EQ)),
            Asm::Set(Variable::new(IF_FLAG), Value::new_num(0)),
            Asm::End(Variable::new(IF_FLAG)),
        ];
        assert_eq!(asm, expect);
    }
    #[test]
    fn test_multi_condition_if_else_eq() {
        let program = "if a == 10 && b != 11 { input ( x ) } else { input ( y ) }";
        let asm = compile(program).unwrap();
        let expect = vec![
            Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(1)),
            Asm::Set(Variable::new(IS_EQ), Value::new_num(1)),
            Asm::Copy(
                Variable::new("a"),
                vec![Variable::new(TEMP_VAR), Variable::new(IF_FLAG)],
            ),
            Asm::Copy(Variable::new(TEMP_VAR), vec![Variable::new("a")]),
            Asm::Sub(Variable::new(IF_FLAG), Value::new_num(10)),
            Asm::Loop(Variable::new(IF_FLAG)),
            Asm::Set(Variable::new(IS_EQ), Value::new_num(0)),
            Asm::Set(Variable::new(IF_FLAG), Value::new_num(0)),
            Asm::End(Variable::new(IF_FLAG)),
            Asm::Loop(Variable::new(IS_EQ)),
            Asm::Copy(
                Variable::new("b"),
                vec![Variable::new(TEMP_VAR), Variable::new(IF_FLAG)],
            ),
            Asm::Copy(Variable::new(TEMP_VAR), vec![Variable::new("b")]),
            Asm::Sub(Variable::new(IF_FLAG), Value::new_num(11)),
            Asm::Loop(Variable::new(IF_FLAG)),
            Asm::Read(Variable::new("x")),
            Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(0)),
            Asm::Set(Variable::new(IF_FLAG), Value::new_num(0)),
            Asm::End(Variable::new(IF_FLAG)),
            Asm::Set(Variable::new(IS_EQ), Value::new_num(0)),
            Asm::End(Variable::new(IS_EQ)),
            Asm::Loop(Variable::new(ELSE_FLAG)),
            Asm::Read(Variable::new("y")),
            Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(0)),
            Asm::End(Variable::new(ELSE_FLAG)),
        ];
        assert_eq!(asm, expect);
    }
    #[test]
    fn test_multi_condition_if_else_ne() {
        let program = "if a != 10 && b == 11 { input ( x ) } else { input ( y ) }";
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
            Asm::Set(Variable::new(IS_EQ), Value::new_num(1)),
            Asm::Copy(
                Variable::new("b"),
                vec![Variable::new(TEMP_VAR), Variable::new(IF_FLAG)],
            ),
            Asm::Copy(Variable::new(TEMP_VAR), vec![Variable::new("b")]),
            Asm::Sub(Variable::new(IF_FLAG), Value::new_num(11)),
            Asm::Loop(Variable::new(IF_FLAG)),
            Asm::Set(Variable::new(IS_EQ), Value::new_num(0)),
            Asm::Set(Variable::new(IF_FLAG), Value::new_num(0)),
            Asm::End(Variable::new(IF_FLAG)),
            Asm::Loop(Variable::new(IS_EQ)),
            Asm::Read(Variable::new("x")),
            Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(0)),
            Asm::Set(Variable::new(IS_EQ), Value::new_num(0)),
            Asm::End(Variable::new(IS_EQ)),
            Asm::Set(Variable::new(IF_FLAG), Value::new_num(0)),
            Asm::End(Variable::new(IF_FLAG)),
            Asm::Loop(Variable::new(ELSE_FLAG)),
            Asm::Read(Variable::new("y")),
            Asm::Set(Variable::new(ELSE_FLAG), Value::new_num(0)),
            Asm::End(Variable::new(ELSE_FLAG)),
        ];
        assert_eq!(asm, expect);
    }
    #[test]
    fn test_while() {
        let program = "while a != 10 { input ( x ) }";
        let asm = compile(program).unwrap();
        let expect = vec![
            Asm::Set(Variable::new(WHILE_FLAG), Value::new_num(1)),
            Asm::Loop(Variable::new(WHILE_FLAG)),
            Asm::Set(Variable::new(WHILE_FLAG), Value::new_num(0)),
            Asm::Copy(
                Variable::new("a"),
                vec![Variable::new(TEMP_VAR), Variable::new(IF_FLAG)],
            ),
            Asm::Copy(Variable::new(TEMP_VAR), vec![Variable::new("a")]),
            Asm::Sub(Variable::new(IF_FLAG), Value::new_num(10)),
            Asm::Loop(Variable::new(IF_FLAG)),
            Asm::Read(Variable::new("x")),
            Asm::Set(Variable::new(WHILE_FLAG), Value::new_num(1)),
            Asm::Set(Variable::new(IF_FLAG), Value::new_num(0)),
            Asm::End(Variable::new(IF_FLAG)),
            Asm::End(Variable::new(WHILE_FLAG)),
        ];
        assert_eq!(asm, expect);
    }
}
