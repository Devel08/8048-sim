# 8048-sim
Minimal Rust prototype emulator for the 8048 instruction set.

**Status**

    Partial 8048 emulator (subset). Work-in-progress;
    I have currently implemented:
    Program counter
    Two 8-register banks and an 8-bit accumulator.
    
**Implemented instructions**
| Instruction   | Status    | Notes       |
| ------ | ----- | ------- |
| MOV | Partially implemented | No memory address operands |
| ADD | Partially implemented | No memory address operands |
| SEL | Partially implemented | No memory banks selection |
| INC | Partially implemented | No memory address operands |
| DEC | Partially implemented | No memory address operands |
| JMP | Partially implemented | Only to program memory locations, no labels |

**Example program, that would cause an infinite loop :-D**

    MOV A, #10
    MOV R0, A
    INC R0
    ADD A, R0
    JMP 0
