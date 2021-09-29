# jovis
Youtube video explanation: (will be linked here in future)

the jovis programming language can be best described as a thin metaprogramming wrapper on top of assembly; the language itself provides no control structures, no memory management, nothing that you would expect a programming language to provide. what it instead provides is a versatile message-based type system and direct, but managed, access to assembly, which allows you to code any control structures, paradigms, or high-level features in yourself. many useful implementations can be found in the standard and core libraries (currently WIP)

## Usage
 - `<compiler binary> <source file name>`

pre-compiled binaries will be available very soon.

## Manual Compilation
### Dependencies
 - Rust compiler
 - The compiler is written in Rust, but the linker is written in C. The Rust build file is set up to compile the C files itself, but you still need to have a C compiler installed.
 - libbfd, or all of GNU binutils
 - Keystone

### Commands
 - `cargo build`

Man, I love the Rust build system.