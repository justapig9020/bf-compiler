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

pub fn assemble(asm: &str) -> String {
    let commands = asm.split("\n");
    for command in commands {
        let parts: Vec<_> = command.split(" ").collect();
        match parts[0] {
            "add" => {
                let var = parts[1].parse::<usize>().unwrap();
                let val = parts[2].parse::<usize>().unwrap();
                return format!("{}{}{}", ">".repeat(var), "+".repeat(val), "<".repeat(var));
            }
            "sub" => {
                let var = parts[1].parse::<usize>().unwrap();
                let val = parts[2].parse::<usize>().unwrap();
                return format!("{}{}{}", ">".repeat(var), "-".repeat(val), "<".repeat(var));
            }
            _ => todo!(),
        }
    }
    todo!();
}

#[cfg(test)]
mod asm {
    use super::*;

    #[test]
    fn test_add() {
        let asm = "add 1 2";
        let expect = ">++<";
        let output = assemble(asm);
        assert_eq!(output, expect);
    }
    #[test]
    fn test_sub() {
        let asm = "sub 3 5";
        let expect = ">>>-----<<<";
        let output = assemble(asm);
        assert_eq!(output, expect);
    }
}
