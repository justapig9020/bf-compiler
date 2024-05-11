# bf-compiler

bf-compiler compiles a custom language `brainfuck-c` (bfc) to brainfuck

# Build

```
git clone https://github.com/justapig9020/bf-compiler
cd bf-compiler
cargo build
```

# Usage

```
Usage: bf-compiler [OPTIONS] <SOURCE>

Arguments:
  <SOURCE>

Options:
  -o, --output <OUTPUT>
  -h, --help             Print help
  -V, --version          Print version
```

# brainfuck-c

## Syntax

- AST: Function EOF
- Function: [Statement]\*
- Statement: If | While | Assign | Move | Input | Output
- If: if Bool { Function } [else { Function }]!
- While: while Bool { Function }
- Assign: Variable = NUMBER
- Bool: Compare [&& Compare]\*
- Compare: Equal | NotEqual
- Equal: Variable == NUMBER
- NotEqual: Variable != NUMBER
- Move: move_right | move_left
- Input: input ( Variable )
- Output: output ( Variable )
- Variable: ID

# License

bf-compiler is open-source and available under the MIT License. For more details, see the LICENSE file in the repository.
