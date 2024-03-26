use anyhow::Result;
// Assembly
// - #define <var> <val>
// - Add <var>, <val>
// - Sub <var>, <val>
// - Set <var>, <val>
// - Rs <val>
// - Ls <val>
// - Loop <label>
// - End <label>
// - Copy <var>, [<var>]+
//
//

pub fn assemble(asm: &str) -> Result<String> {
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
            _ => todo!(),
        };
        program.push_str(&bf_command);
    }
    Ok(program)
}

#[cfg(test)]
mod asm {
    use super::*;

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
}
