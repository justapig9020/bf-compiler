use anyhow::Result;
use std::collections::HashMap;

fn parse_copy(parts: &[&str]) -> Result<String> {
    let src = parts[0].parse::<usize>()?;
    let mut inner = String::new();
    for part in &parts[1..] {
        let dest = part.parse::<usize>()?;
        inner.push_str(&format!("{}+{}", ">".repeat(dest), "<".repeat(dest)));
    }
    let rs_src = ">".repeat(src);
    let ls_src = "<".repeat(src);
    Ok(format!(
        "{}[-{}{}{}]{}",
        rs_src, ls_src, inner, rs_src, ls_src
    ))
}

fn replace(parts: &[&str], map: &HashMap<&str, usize>) -> String {
    parts
        .iter()
        .map(|part| {
            if map.contains_key(part) {
                map[part].to_string()
            } else {
                part.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn preprocess(asm: &str) -> Result<String> {
    let commands = asm.split("\n");
    let mut program = String::new();
    let mut map = HashMap::new();
    for command in commands {
        let parts: Vec<_> = command.split(" ").collect();
        let command = match parts[0] {
            "#define" => {
                let var = parts[1];
                let val = parts[2].parse::<usize>()?;
                map.insert(var, val);
                String::new()
            }
            _ => format!("{}\n", replace(&parts, &map)),
        };
        program.push_str(&command);
    }
    Ok(program)
}

pub fn assemble(asm: &str) -> Result<String> {
    let asm = preprocess(asm)?;
    let commands = asm.split("\n");
    let mut program = String::new();
    for command in commands {
        let parts: Vec<_> = command.split(" ").collect();
        let bf_command = match parts[0] {
            "add" => {
                let var = parts[1].parse::<usize>()?;
                let val = parts[2].parse::<usize>()?;
                format!("{}{}{}", ">".repeat(var), "+".repeat(val), "<".repeat(var))
            }
            "sub" => {
                let var = parts[1].parse::<usize>()?;
                let val = parts[2].parse::<usize>()?;
                format!("{}{}{}", ">".repeat(var), "-".repeat(val), "<".repeat(var))
            }
            "set" => {
                let var = parts[1].parse::<usize>()?;
                let val = parts[2].parse::<usize>()?;
                format!(
                    "{}[-]{}{}",
                    ">".repeat(var),
                    "+".repeat(val),
                    "<".repeat(var)
                )
            }
            "rs" => ">".repeat(parts[1].parse::<usize>()?),
            "ls" => "<".repeat(parts[1].parse::<usize>()?),
            "loop" => "[".to_string(),
            "end" => "]".to_string(),
            "copy" => parse_copy(&parts[1..])?,
            "" => String::new(),
            s => todo!("'{}' not implemented", s),
        };
        program.push_str(&bf_command);
    }
    Ok(program)
}

#[cfg(test)]
mod asm {
    use super::*;

    // Assembly
    // - #define <var> <val>
    // - Add <var>, <val>
    // - Sub <var>, <val>
    // - Set <var>, <val>
    // - Rs <val>
    // - Ls <val>
    // - Loop
    // - End
    // - Copy <var>, [<var>]+
    //

    #[test]
    fn test_add() {
        let asm = "add 1 2";
        let expect = ">++<";
        let output = assemble(asm).unwrap();
        assert_eq!(output, expect);
    }
    #[test]
    fn test_sub() {
        let asm = "sub 3 5";
        let expect = ">>>-----<<<";
        let output = assemble(asm).unwrap();
        assert_eq!(output, expect);
    }
    #[test]
    fn test_set() {
        let asm = "set 3 5";
        let expect = ">>>[-]+++++<<<";
        let output = assemble(asm).unwrap();
        assert_eq!(output, expect);
    }
    #[test]
    fn test_rs() {
        let asm = "rs 3";
        let expect = ">>>";
        let output = assemble(asm).unwrap();
        assert_eq!(output, expect);
    }
    #[test]
    fn test_ls() {
        let asm = "ls 4";
        let expect = "<<<<";
        let output = assemble(asm).unwrap();
        assert_eq!(output, expect);
    }
    #[test]
    fn test_loop() {
        let asm = "loop\nls 3\nend";
        let expect = "[<<<]";
        let ooutput = assemble(asm).unwrap();
        assert_eq!(ooutput, expect);
    }
    #[test]
    fn test_copy_2() {
        let asm = "copy 1 2 3";
        let expect = ">[-<>>+<<>>>+<<<>]<";
        let output = assemble(asm).unwrap();
        assert_eq!(output, expect);
    }
    #[test]
    fn test_copy_3() {
        let asm = "copy 1 2 3 4";
        let expect = ">[-<>>+<<>>>+<<<>>>>+<<<<>]<";
        let output = assemble(asm).unwrap();
        assert_eq!(output, expect);
    }
    #[test]
    fn test_preprocess() {
        let asm = "#define a 3\nadd a 2";
        let expect = "add 3 2\n";
        let output = preprocess(asm).unwrap();
        assert_eq!(output, expect);
    }
}
