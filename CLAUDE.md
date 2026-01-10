# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is **rcc** (Rust C Compiler), a C11-compliant C compiler written in Rust that generates x86-64 assembly. The compiler implements a traditional three-phase architecture: lexical analysis → parsing/semantic analysis → code generation.

**Current Status**: ~4636 lines of code, supporting most C11 features including structs, unions, enums, typedef, pointers, arrays, and C11 escape sequences.

## Build and Test Commands

### Building
```bash
cargo build              # Debug build
cargo build --release    # Release build
```

### Running the Compiler
```bash
# Compile from inline code
./target/debug/rcc -i "int main() { return 42; }" > output.s

# Compile from file
./target/debug/rcc -f test/programs/01_basic_ops.c > output.s

# With debug output (shows symbol tables and function info)
./target/debug/rcc --debug -i "int main() { return 42; }" > output.s

# Disable optimization (enabled by default)
./target/debug/rcc --optimize=false -i "int main() { return 42; }" > output.s
```

### Testing
```bash
# Run all tests (recommended - runs inline, file, and function tests)
bash test/inline.sh && bash test/file.sh && bash test/func.sh

# Run individual test suites
bash test/inline.sh      # quick unit tests for language features
bash test/file.sh        # integration tests
bash test/func.sh        # External function linking tests

# Compile and run a single test manually
./target/debug/rcc -i "int main() { return 42; }" > test/bin/tmp.s
cc -g -o test/bin/tmp test/bin/tmp.s
./test/bin/tmp
echo $?  # Should print 42
```

**Test Structure**: The test suite uses `test/common.sh` which provides three assert functions:
- `assert_inline <expected> <code>` - Wraps code in `int main() { ... }` and tests return value
- `assert_file <expected> <file>` - Compiles a full C file and tests return value
- `assert_func <expected> <code>` - Links with `test/bin/func.o` for external function tests

## Architecture

### Compilation Pipeline
```
Source Code
    ↓
[Lexer] (lexer.rs) - Tokenization with escape sequence support
    ↓
Token Stream (token.rs)
    ↓
[AST Parser] (ast/*.rs) - Parsing + semantic analysis + type checking
    ↓
Abstract Syntax Tree (node.rs) with type information
    ↓
[Generator] (x86/*.rs) - Code generation for x86-64
    ↓
[AsmBuilder] (asm_builder.rs) - Optimization (removes redundant push/pop)
    ↓
x86-64 Assembly Output
```

### Type System Architecture

The type system uses a **global type table** to ensure type uniqueness and prevent duplication:

- **`types/table.rs`**: Global type table that stores all types with unique IDs
- **`types/type_ref.rs`**: `TypeRef` - lightweight reference to a type via ID (used throughout the codebase)
- **`types/kind.rs`**: `TypeKind` - the actual type data (Int, Char, Ptr, Array, Struct, etc.)
- **`types/spec.rs`**: Type specifiers and qualifiers (const, static, etc.)

**Key Pattern**: When creating or looking up types, always go through the type table to get a `TypeRef`. Never create raw `TypeKind` instances directly in AST nodes - use `TypeRef` instead.

**Self-Referential Structs**: Handled by registering the struct as an incomplete type when parsing begins, then updating to a complete type after parsing members. This allows `struct Node { struct Node *next; }`.

**Nested Structs**: Naturally handled by the recursive descent parser's function call stack.

### Symbol Table and Scoping

- **`symbol/table.rs`**: Scope-aware symbol table for name resolution
- Supports block scoping, shadowing, and both local/global variables
- Symbols store variable declarations, function declarations, and enum constants

### Code Generation Details

- **x86-64 ABI Compliance**: 16-byte stack alignment for function calls
- **Register Usage**: Prioritizes registers over stack to avoid stack leaks
- **Pointer Arithmetic**: Add/subtract operations handle pointer scaling automatically
- **Assembly Format**: Intel syntax, `.string` directives require proper escaping (handled by `escape_string_for_asm`)

### Escape Sequence Implementation

The lexer fully supports C11 escape sequences (added in latest implementation):
- Simple escapes: `\'`, `\"`, `\?`, `\\`, `\a`, `\b`, `\f`, `\n`, `\r`, `\t`, `\v`
- Octal: `\0` through `\377` (1-3 digits, value 0-255)
- Hexadecimal: `\xHH` (variable length hex digits, value 0-255)

**Two-Stage Escaping**: Lexer parses source escapes into internal representation, then `x86.rs::escape_string_for_asm()` re-escapes for assembly `.string` directives.

## Common Abbreviations

The codebase uses consistent C grammar terminology:

| Abbreviation | Full Term | Meaning |
|--------------|-----------|---------|
| `decl` | declaration | Variable/function declarations |
| `spec` | specifier | Type specifiers, storage class specifiers |
| `qual` | qualifier | Type qualifiers (const, volatile) |
| `param` | parameter | Function parameters |
| `func` | function | Function |
| `ptr` | pointer | Pointer type |
| `abst` | abstract | Abstract declarator |
| `expr` | expression | Expression |
| `stmt` | statement | Statement |

**Note**: `declarator` and `initializer` are not abbreviated as they are critical grammar concepts.

## Code Modification Guidelines

### When Adding Features

1. **Read existing code first** - Understand the pattern before modifying
2. **Update tests** - Add test cases to `test/inline.sh`, `test/file.sh`, or `test/func.sh`
3. **Follow C11 spec** - Reference the C11 specification for correctness
4. **Avoid over-engineering** - Keep implementations minimal and focused

### Common Pitfalls

- **Left Recursion in BNF**: Parser may not follow BNF exactly due to simultaneous semantic analysis
- **Stack Alignment**: Function calls require 16-byte alignment (allocate stack space in multiples of 16)
- **32-bit Register Behavior**: Only 32-bit register operations zero-clear upper bits
- **Pointer Arithmetic**: Remember to scale by pointed-to type size in add/subtract

### Type System Usage

```rust
// CORRECT: Use type table to get TypeRef
let int_type = type_table.get_int_type();

// INCORRECT: Don't create TypeKind directly
// let my_type = TypeKind::Int;  // Never do this in AST nodes
```

### Debugging

```bash
# Enable debug mode to see symbol tables and function info
./target/debug/rcc --debug -i "int main() { return 42; }" > output.s

# Use gdb to debug generated assembly
gdb ./test/bin/tmp
(gdb) b 11              # Set breakpoint at assembly line 11
(gdb) run               # Run to breakpoint
(gdb) info registers    # Check register state
```

## References

- [C11 Specification (PDF)](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf)
- [C11 Specification (HTML)](https://port70.net/~nsz/c/c11/n1570.html)
- [Online Assembly Compiler (Godbolt)](https://godbolt.org/)
- [低レイヤを知りたい人のためのCコンパイラ作成入門](https://www.sigbus.info/compilerbook)
