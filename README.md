# bf-compiler

## Introduction

`bf-compiler` compiles a custom language `brainfuck-c` (bfc) into Brainfuck, allowing for more complex and readable program creation in a higher-level syntax before being converted to Brainfuck.

## Build

### Requirements

- `git`
- `cargo`

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/justapig9020/bf-compiler
   ```
2. Build the project:
   ```bash
   cd bf-compiler
   cargo build
   ```

## Usage

To compile a `brainfuck-c` source file to Brainfuck, use the following command:

```bash
bf-compiler [OPTIONS] <SOURCE>

Arguments:
  <SOURCE>

Options:
  -o, --output <OUTPUT>
  -h, --help             Print help
  -V, --version          Print version
```

## Brainfuck-c (bfc)

### Syntax

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

## Related Projects

This project is part of a series aimed at building a compiler to prove that Brainfuck is Turing complete. You can find the other related projects here:

- [tm-compiler](https://github.com/justapig9020/tm-compiler): Converts Turing machines into a custom C-like IR (bf-c).
- [bf-compiler](https://github.com/justapig9020/bf-compiler): Compiles bf-c programs into Brainfuck.
- [rubf](https://github.com/justapig9020/rubf): A Brainfuck virtual machine.

## License

`bf-compiler` is open-source and available under the MIT License. For more details, see the LICENSE file in the repository.
