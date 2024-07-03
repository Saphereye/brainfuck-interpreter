# Brainf*ck interpreter in Rust ![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white) 
Implemented a [brainf*ck](https://esolangs.org/wiki/Brainfuck) interpreter in Rust.

The logic for the programming language can be summarised as follows:
| Instruction | Description |
|-------------|-------------|
| `<`         | head0 = head0 - 1 |
| `>`         | head0 = head0 + 1 |
| `{`         | head1 = head1 - 1 |
| `}`         | head1 = head1 + 1 |
| `-`         | tape[head0] = tape[head0] - 1 |
| `+`         | tape[head0] = tape[head0] + 1 |
| `.`         | tape[head1] = tape[head0] |
| `,`         | tape[head0] = tape[head1] |
| `[`         | if (tape[head0] == 0): jump forwards to matching `]` command. |
| `]`         | if (tape[head0] != 0): jump backwards to matching `[` command. |

> source: arXiv:2406.19108

## Usage
The program has a bunch of helpful CLI commands. An example usage while using this repo:
```cargo run --release -- --input examples/serpinkski.b```

## Developing
To turn on the debug mode, the program must first be compiled in the `debug` profile.
Then while running the program set the `RUST_LOG` env variable (to `DEBUG`, `TRACE`, etc.).
