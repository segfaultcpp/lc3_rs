# lc3_rs
This project is implementation of LC-3 virtual machine based on [this](https://www.jmeiners.com/lc3-vm/#includes-block-87 "Write your Own Virtual Machine") article on Rust. The project also contains some examples for running on the virtual machine.

# Build
Before compiling the Rust project you need to compile cpp/terminal_setup.cpp as static library.

## clang
> clang -c -o cpp/terminal_setup.o cpp/terminal_setup.cpp \
> llvm-ar rc cpp/terminal_setup.lib cpp/terminal_setup.o
## gcc (on linux)
> gcc -c -o cpp/terminal_setup.o cpp/terminal_setup.cpp \
> ar rcs cpp/terminal_setup.a cpp/terminal_setup.o

Once you've got the static library you can compile the Rust project with cargo.