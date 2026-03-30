# BROT-V0

## Overview

Brot is a general purpose RISC style ISA. It differentiates itself through several key points.

1. Hardware level type checking
2. Hardware level bounds checking of pointers
3. Non-power of two based widths.
4. Hardware support for loops.
5. DSP and math style support.
6. Saturating and Modular Arithmetic by default.

The goal is to provide a hardware environment that is highly robust for embedded purposes either making it outright impossible or very unlikely for many classes of bugs to occur. V0 focuses on the core features and as such does not have the following features

1. Hardware loops
2. Kernel/user mode switching — V0 runs in kernel mode at all times (kernel bit hardwired to 1). Kernel-only instruction restrictions are specified for future versions but do not trap in V0.

We focus purely on the tag system and on the pointer system

## Benefits
TODO
## Word width & Bytes

Brot words are currently defined as being **46** bits. We split each 46 bit word into 5×8 subwords with a 6 bit tag at the front denoting its data type. The tag field is 6 bits, encoding values 0–63. Tags 0x00–0x0D name the 14 defined types; any tag value above `0x0D` is treated as **Inert**. The 8 bit subwords in BROT are called **bytes**.

| [45:40]     | [39:32]   | [31:24]   | [23:16]   | [15:8]    | [7:0]     |
|-------------|-----------|-----------|-----------|-----------|-----------|
| Tag Field   | Byte 1    | Byte 2    | Byte 3    | Byte 4    | Byte 5    |

### Types

| Tag (hex)       | Tag (dec) | Name        | Group    | Mutable |
|-----------------|-----------|-------------|----------|---------|
| `0x00`          | 0         | Free        | Special  | Yes     |
| `0x01`          | 1         | U.Int       | Unsigned | Yes     |
| `0x02`          | 2         | ModU        | Unsigned | Yes     |
| `0x03`          | 3         | PkU         | Unsigned | Yes     |
| `0x04`          | 4         | Bool        | Unsigned | Yes     |
| `0x05`          | 5         | S.Int       | Signed   | Yes     |
| `0x06`          | 6         | ModS        | Signed   | Yes     |
| `0x07`          | 7         | Q32.8       | Signed   | Yes     |
| `0x08`          | 8         | Q24.16      | Signed   | Yes     |
| `0x09`          | 9         | Q8.32       | Signed   | Yes     |
| `0x0A`          | 10        | PkS         | Signed   | Yes     |
| `0x0B`          | 11        | Instruction | System   | No      |
| `0x0C`          | 12        | Ptr         | System   | No      |
| `0x0D`          | 13        | Inert       | System   | No      |
| `0x0E`–`0x3F`   | 14–63     | (reserved)  | —        | → Inert |

All types are saturating unless specified otherwise.

**System types** (Inert, Instruction, Ptr) are **immutable** — stores (SWI/SWP) to memory tagged with a system type trap. To reuse a system-type memory location, first convert it to a mutable type via `MCONV`.

**Mutable types** (Free, U.Int, S.Int, ModU, ModS, Q32.8, Q24.16, Q8.32, Bool, PkU, PkS) can be written to by stores.

**PkU and PkS** are defined as types with entries in the tag table and conversion matrix, but **no dedicated packed-processing instructions are defined in V0**. Arithmetic, shift, and logical operations on PkU/PkS are reserved for future extensions. In V0, PkU/PkS values can be stored, loaded, and converted per the conversion matrix.

**Free** (tag 0) is the mutable default. All-zero memory is Free-tagged. Free is writable but traps on all arithmetic and logical operations with any type (including itself). It serves as a "blank slate" — memory must be converted via `CONVERT` or `MCONV` to a data type before use. Converting Free to a data type **zeros the data bits** (the Zero operation) to prevent information leakage from previously freed memory.

### Convert
Conversion is done through a `CONVERT` instruction. 

`CONVERT` operates on GPR values only (46-bit data types). Pointer type (Ptr) is 92-bit and lives exclusively in pointer registers, so it is excluded from this matrix. For Ptr-tagged memory conversions, see MCONV.

`CONVERT` will attempt to faithfully convert the values in a reasonable way. For example Q32.8 to integer will truncate the fractional bits. Integer to Q32.8 will scale the integer bits up. We provide the following conversion matrix.

| From \ To | Free  | U.Int | ModU  | PkU   | Bool  | S.Int | ModS  | Q32.8 | Q24.16 | Q8.32 | PkS   | Instr | Inert |
|-----------|-------|-------|-------|-------|-------|-------|-------|-------|--------|-------|-------|-------|-------|
| Free      | Id    | Zero  | Zero  | Zero  | Zero  | Zero  | Zero  | Zero  | Zero   | Zero  | Zero  | Zero  | Zero  |
| U.Int     | Retag | Id    | Retag | —     | ≠0    | Clamp | Clamp | Scale | Scale  | Scale | —     | —     | Retag |
| ModU      | Retag | Retag | Id    | —     | ≠0    | Clamp | Clamp | Scale | Scale  | Scale | —     | —     | Retag |
| PkU       | Retag | —     | —     | Id    | —     | —     | —     | —     | —      | —     | Retag | —     | Retag |
| Bool      | Retag | 0/1   | 0/1   | —     | Id    | 0/1   | 0/1   | 0/1   | 0/1    | 0/1   | —     | —     | Retag |
| S.Int     | Retag | Clamp | Clamp | —     | ≠0    | Id    | Retag | Scale | Scale  | Scale | —     | —     | Retag |
| ModS      | Retag | Clamp | Clamp | —     | ≠0    | Retag | Id    | Scale | Scale  | Scale | —     | —     | Retag |
| Q32.8     | Retag | Trunc | Trunc | —     | ≠0    | Trunc | Trunc | Id    | Scale  | Scale | —     | —     | Retag |
| Q24.16    | Retag | Trunc | Trunc | —     | ≠0    | Trunc | Trunc | Scale | Id     | Scale | —     | —     | Retag |
| Q8.32     | Retag | Trunc | Trunc | —     | ≠0    | Trunc | Trunc | Scale | Scale  | Id    | —     | —     | Retag |
| PkS       | Retag | —     | —     | Retag | —     | —     | —     | —     | —      | —     | Id    | —     | Retag |
| Instr     | Retag | Retag | Retag | Retag | Retag | Retag | Retag | Retag | Retag  | Retag | Retag | Id    | Retag |
| Inert     | Retag | Retag | Retag | Retag | Retag | Retag | Retag | Retag | Retag  | Retag | Retag | Retag | Id    |

- **Id** — identity, no-op
- **Retag** — retag field change only, data bits unchanged
- **Zero** — data bits cleared to 0, target tag applied. Used when converting from Free to prevent information leakage from freed memory.
- **Scale** — mathematical rescaling / bit shifting; saturates on overflow and sets the Saturation Hit flag
- **Clamp** — range/sign check with saturation clamping, no bit shifting
- **Trunc** — truncate fractional bits (fixed point → integer)
- **≠0** — true if non-zero, false otherwise
- **0/1** — false → 0, true → 1 (mathematical value in the target type's representation)
- **—** — invalid / trap

## Register File

BROT provides **23 general-purpose registers**, each 46 bits wide. Registers are addressed with a 5-bit field.

| Register | Alias | Description                           |
|----------|-------|---------------------------------------|
| r0       | zero  | Hardwired to U.Int(0) (tag=1, data=0) |
| r1–r22   |       | General purpose                       |

It has also **13 Dedicated Pointer Registers**, each 92 bits wide. Pointers can ONLY be stored with PR.

| Register | Alias | Description     |
|----------|-------|-----------------|
| p0       | Stack | Stack Pointer, |
| p1       | Link. | Link Pointer,  |
| p2–p13   |       | General purpose |

### Pointer Register Format

Each pointer register is 92 bits wide, organized as two 46-bit words:

```
91        86 85          46 45        40 39 38            0
┌──────────┬──────────────┬────────────┬──┬───────────────┐
│ Tag (6)  │  Size (40)   │  Tag (6)   │RW│  Base (39)    │
│ upper    │ upper data   │  lower     │  │  lower data   │
└──────────┴──────────────┴────────────┴──┴───────────────┘
 ←───── Upper word (46) ─────→←───── Lower word (46) ────→
```

All pointers use the same format. A singleton is a pointer with size = 1.

#### Pointer — memory format (2 consecutive words)

```
Word 1 — base word (46 bits):
[45:40]  tag = Ptr (12)
[39]     RW bit (1 = read-write, 0 = read-only)
[38:0]   base address (39 bits)

Word 2 — size word (46 bits):
[45:40]  tag = Ptr (12)
[39:0]   size (40 bits, in words)
```

#### Pointer — register format (92 bits)

```
[91:86]  tag = Ptr (12)
[85:46]  size (40 bits)
[45:40]  tag = Ptr (12)
[39]     RW bit
[38:0]   base address (39 bits)
```

LPRI loads: reads 2 consecutive Ptr-tagged words — base word into lower half, size word into upper half.
SPRI stores: writes both halves to 2 consecutive memory words — lower half as word 1, upper half as word 2.

Base addresses are 39 bits (512 Gwords addressable). The size field is 40 bits (up to 1 Twords). The RW bit is at data bit [39] of the base word — it lives in the data portion, not the tag.

**Memory round-trip**: SPRI writes both 46-bit words to memory as-is (tag + data). LPRI reads them back. No special encoding.

### Status Register

The status register (46 bits) is accessed through special instructions.

| Bit | Flag                 | Sticky | Description                    |
|-----|----------------------|--------|--------------------------------|
| 0   | Modular loop taken   | Yes    | A modular wrap occurred        |
| 1   | Saturation hit       | Yes    | A saturating clamp occurred    |
| 2   | Hardware loop active | No     | A hardware loop is in progress |
| 3   | Kernel               | No     | Processor is in kernel mode    |
| 4   | Interrupt            | No     | Reserved for future use        |

### Cycle Counter

A dedicated cycle counter register increments every clock cycle, readable for profiling.

## Instruction encoding.

We define 10 groups of instructions:

1. R-type: Register to Register
2. S-type: Store to memory
3. B-type: Branch instructions
4. I-type: Immediate instructions
5. U-type: Upper immediate loading.
6. J-type: Jump instructions.
7. C-type: Convert instructions.
8. L-type: Loop instructions.
9. P-type: Pointer instructions (MKPTR, PNARROW, pointer store/load)
10. Q-type: System / query instructions (RDSR, PLEN, MTAG)

**Register field widths**: GPR fields are 5 bits (addressing 23 registers). PR fields are 4 bits (addressing 13 pointer registers). The opcode occupies the first full data byte [39:32]. GPR and PR register fields do not align to byte boundaries.

**Immediate encoding**: All immediate fields across all instruction formats are **two's complement signed values**, sign-extended to the operand width before use.

**Type matching**: All arithmetic (ADD, SUB, MUL, MULH, MAC, etc.), logical (AND, OR, XOR), shift (SLL, SRL, SRA), and comparison/branch (BEQ, BNE, BLT, BGE) operations require **all operand register types to match**. A type mismatch traps. Use `CONVERT` for explicit type coercion before the operation. For immediate instructions (ADDI, ANDI, ORI, XORI, SLLI, SRLI, SRAI), the immediate value is interpreted in the same representation as the source register's type. **BTAG** is exempt — it compares a register's tag against a literal tag field, not two register values.

### R-type

We encode R-type as follows:

| [45:40]   | [39:32] | [31:27] | [26:22] | [21:17] | [16:0]           |
|-----------|---------|---------|---------|---------|------------------|
| Tag Field | Opcode  | rd (5)  | rs1 (5) | rs2 (5) | subop (17)       |

We implement the following R-type instructions.

| Instruction | Opcode | Description                  | Notes                  |
|-------------|--------|------------------------------|------------------------|
| ADD         | 0x00   | rd = rs1 + rs2               |                        |
| SUB         | 0x01   | rd = rs1 - rs2               |                        |
| XOR         | 0x02   | rd = rs1 ^ rs2               |                        |
| OR          | 0x03   | rd = rs1 \| rs2              |                        |
| AND         | 0x04   | rd = rs1 & rs2               |                        |
| SLL         | 0x05   | rd = rs1 << rs2              | Integral types only    |
| SHR         | 0x06   | rd = rs1 >> rs2              | Integral types only; logical if rs1 unsigned type, arithmetic if signed type |
| MUL         | 0x07   | rd = rs1 * rs2               | Saturates/wraps per type. Fixed-point auto-scaled. |
| MULH        | 0x08   | rd = upper(rs1 * rs2)        | Upper bits of true (unsaturated/unwrapped) product. Fixed-point: before rescaling. |
| MAC         | 0x09   | rd = rd + (rs1 * rs2)        | Multiply auto-scaled, then accumulated. Both steps saturate/wrap. All three regs must match type. |

**Saturation**: All arithmetic operations on saturating types (U.Int, S.Int, Q32.8, Q24.16, Q8.32, Bool) clamp the result to the representable range and set the **Saturation Hit** status flag on overflow. Modular types (ModU, ModS) wrap on overflow and set the **Modular Loop Taken** flag.

**Fixed-point multiply**: For fixed-point types, the ALU computes the full product and automatically rescales by right-shifting by the fractional bit count (Q32.8: 8, Q24.16: 16, Q8.32: 32), then saturates. This produces a correctly-scaled result in the source type. No manual shifting is needed.

### S-type

S-type instructions store data to memory through a pointer register. Before executing, the hardware checks: (1) the computed address is within the pointer register's bounds, (2) the pointer register's RW bit is set, (3) the target memory location's tag is a mutable type (Free, U.Int, S.Int, ModU, ModS, Q32.8, Q24.16, Q8.32, Bool, PkU, PkS), and (4) the source GPR's tag is a mutable type. System types (Inert, Instruction, Ptr) are immutable — a system type at the target memory location or in the source GPR causes a trap. This prevents stores from creating system-tagged memory; only kernel raw stores (KSWI/KSWG) can write system-typed values. A violation of any check causes a trap.

SWI uses an immediate offset:

| [45:40]   | [39:32] | [31:28] | [27:23] | [22:0]   |
|-----------|---------|---------|---------|----------|
| Tag Field | Opcode  | pr (4)  | rs (5)  | imm (23) |

SWP uses a GPR as the offset:

| [45:40]   | [39:32] | [31:28] | [27:23] | [22:18]       | [17:0]        |
|-----------|---------|---------|---------|---------------|---------------|
| Tag Field | Opcode  | pr (4)  | rs (5)  | rs_offset (5) | reserved (18) |

The `pr` field selects one of the 13 pointer registers. The `rs` field is the source data GPR. For SWP, `rs_offset` is the GPR containing the offset.

| Instruction | Opcode | Description                              | Notes                  |
|-------------|--------|------------------------------------------|------------------------|
| SWI         | 0x0A   | Store word: mem[pr + imm] = rs           | Immediate offset       |
| SWP         | 0x0B   | Store word: mem[pr + rs_offset] = rs     | Register offset        |

### Pointer Store

Pointer store instructions save a pointer register to memory through another pointer register. Before executing, the hardware checks: (1) the computed address is within the addressing pointer register's bounds, (2) the addressing pointer register's RW bit is set, and (3) the target memory location's tag is a mutable type. Writes 2 consecutive words atomically (base word then size word).

**SPRI** (Store Pointer, Immediate offset): `mem[pr_addr + imm] = pr_src`

| [45:40]   | [39:32] | [31:28]      | [27:24]     | [23:0]   |
|-----------|---------|--------------|-------------|----------|
| Tag Field | Opcode  | pr_addr (4)  | pr_src (4)  | imm (24) |

**SPRP** (Store Pointer, Register offset): `mem[pr_addr + rs_offset] = pr_src`

| [45:40]   | [39:32] | [31:28]      | [27:24]     | [23:19]        | [18:0]        |
|-----------|---------|--------------|-------------|----------------|---------------|
| Tag Field | Opcode  | pr_addr (4)  | pr_src (4)  | rs_offset (5)  | reserved (19) |

The `pr_addr` field selects the addressing pointer register. The `pr_src` field selects the source pointer register to store. For SPRP, `rs_offset` is the GPR containing the offset.

| Instruction | Opcode | Description                              | Notes                  |
|-------------|--------|------------------------------------------|------------------------|
| SPRI        | 0x1F   | Store pointer: mem[pr_addr + imm] = pr_src       | Immediate offset       |
| SPRP        | 0x20   | Store pointer: mem[pr_addr + rs_offset] = pr_src | Register offset        |

### B-type

We encode B-type as follows:

| [45:40]   | [39:32] | [31:27]  | [26:22]  | [21:0]   |
|-----------|---------|----------|----------|----------|
| Tag Field | Opcode  | rs1 (5)  | rs2 (5)  | imm (22) |

Except for BTAG which uses the rs2 and imm fields differently:

| [45:40]   | [39:32] | [31:27]  | [26]     | [25:20] | [19:0]   |
|-----------|---------|----------|----------|---------|----------|
| Tag Field | Opcode  | rs1 (5)  | reserved | tag (6) | imm (20) |

The tag field is mapped to the tag type by the assembler.

We implement the following B-type instructions. Branch target is PC + imm.

| Instruction | Opcode | Description                | Notes              |
|-------------|--------|----------------------------|--------------------|
| BEQ         | 0x0C   | Branch if rs1 == rs2       |                    |
| BNE         | 0x0D   | Branch if rs1 != rs2       |                    |
| BLT         | 0x0E   | Branch if rs1 < rs2        |                    |
| BGE         | 0x0F   | Branch if rs1 >= rs2       |                    |
| BTAG        | 0x10   | Branch if tag(rs1) == tag  | Compares tags only |

### I-type

We encode I-type as follows:

| [45:40]   | [39:32] | [31:27] | [26:22]  | [21:0]   |
|-----------|---------|---------|----------|----------|
| Tag Field | Opcode  | rd (5)  | rs1 (5)  | imm (22) |

We implement the following I-type instructions.

| Instruction | Opcode | Description                      | Notes               |
|-------------|--------|----------------------------------|---------------------|
| ADDI        | 0x11   | rd = rs1 + imm                   |                     |
| XORI        | 0x12   | rd = rs1 ^ imm                   |                     |
| ORI         | 0x13   | rd = rs1 \| imm                  |                     |
| ANDI        | 0x14   | rd = rs1 & imm                   |                     |
| SLLI        | 0x15   | rd = rs1 << imm                  | Integral types only |
| SHRI        | 0x16   | rd = rs1 >> imm                  | Integral types only; logical if rs1 unsigned type, arithmetic if signed type |

### Load instructions

Load instructions read data from memory through a pointer register. Before executing, the hardware checks that the computed address is within the pointer register's bounds. A violation causes a trap.

LWI uses an immediate offset:

| [45:40]   | [39:32] | [31:27] | [26:23] | [22:0]   |
|-----------|---------|---------|---------|----------|
| Tag Field | Opcode  | rd (5)  | pr (4)  | imm (23) |

LWP uses a GPR as the offset:

| [45:40]   | [39:32] | [31:27] | [26:23] | [22:18]        | [17:0]        |
|-----------|---------|---------|---------|----------------|---------------|
| Tag Field | Opcode  | rd (5)  | pr (4)  | rs_offset (5)  | reserved (18) |

The `pr` field selects one of the 13 pointer registers. The `rd` field is the destination GPR. For LWP, `rs_offset` is the GPR containing the offset.

| Instruction | Opcode | Description                      | Notes               |
|-------------|--------|----------------------------------|---------------------|
| LWI         | 0x17   | rd = mem[pr + imm]               | Immediate offset    |
| LWP         | 0x18   | rd = mem[pr + rs_offset]         | Register offset     |

If the memory word at the target address has tag Ptr, the load traps. Use LPRI/LPRP to load Ptr-tagged memory.

### Pointer Load

Pointer load instructions read a pointer from memory into a pointer register through an addressing pointer register. Before executing, the hardware checks: (1) the computed address is within the addressing pointer register's bounds, and (2) the memory word at the target address has tag Ptr. Reads 2 consecutive Ptr-tagged words atomically (base word into lower half, size word into upper half).

**LPRI** (Load Pointer, Immediate offset): `pr_dst = mem[pr_addr + imm]`

| [45:40]   | [39:32] | [31:28]      | [27:24]      | [23:0]   |
|-----------|---------|--------------|--------------|----------|
| Tag Field | Opcode  | pr_dst (4)   | pr_addr (4)  | imm (24) |

**LPRP** (Load Pointer, Register offset): `pr_dst = mem[pr_addr + rs_offset]`

| [45:40]   | [39:32] | [31:28]      | [27:24]      | [23:19]        | [18:0]        |
|-----------|---------|--------------|--------------|----------------|---------------|
| Tag Field | Opcode  | pr_dst (4)   | pr_addr (4)  | rs_offset (5)  | reserved (19) |

The `pr_dst` field selects the destination pointer register. The `pr_addr` field selects the addressing pointer register. For LPRP, `rs_offset` is the GPR containing the offset.

| Instruction | Opcode | Description                              | Notes                  |
|-------------|--------|------------------------------------------|------------------------|
| LPRI        | 0x21   | pr_dst = mem[pr_addr + imm]              | Immediate offset       |
| LPRP        | 0x22   | pr_dst = mem[pr_addr + rs_offset]        | Register offset        |

### U-type

We encode U-type as follows:

| [45:40]   | [39:32] | [31:27] | [26:0]   |
|-----------|---------|---------|----------|
| Tag Field | Opcode  | rd (5)  | imm (27) |

We implement the following U-type instructions.

| Instruction | Opcode | Description                      | Notes              |
|-------------|--------|----------------------------------|--------------------|
| LUI         | 0x19   | rd = imm << 13                   | Always sets U.Int  |
| AUIPC       | 0x1A   | rd = PC + (imm << 13)            | Always sets U.Int  |

LUI and AUIPC always set the destination register's tag to U.Int, regardless of input. This ensures immediate-loaded values are usable in arithmetic and as MKPTR source data without requiring a separate CONVERT step.

The shift amount is 13, which pairs with the 22-bit I-type immediate: LUI places the 27-bit immediate at data bits [39:13], and a following ADDI fills bits [21:0] with its 22-bit immediate. The 9-bit overlap at [21:13] is resolved using RISC-V-style sign correction: if the ADDI immediate is negative, add 1 to the LUI immediate to compensate. Together, 27 + 13 = 40 covers all data bits.

### J-type

JAL uses the same encoding as U-type with a 27-bit immediate:

| [45:40]   | [39:32] | [31:27] | [26:0]   |
|-----------|---------|---------|----------|
| Tag Field | Opcode  | rd (5)  | imm (27) |

JALR uses the same encoding as I-type:

| [45:40]   | [39:32] | [31:27] | [26:22]  | [21:0]   |
|-----------|---------|---------|----------|----------|
| Tag Field | Opcode  | rd (5)  | rs1 (5)  | imm (22) |

We implement the following J-type instructions.

| Instruction | Opcode | Description                         | Notes         |
|-------------|--------|-------------------------------------|---------------|
| JAL         | 0x1B   | rd = PC + 1, PC = PC + imm          | U-type format |
| JALR        | 0x1C   | rd = PC + 1, PC = (rs1 + imm) & ~1  | I-type format |

### C-type

We encode C-type as follows:

| [45:40]   | [39:32] | [31:27] | [26:22]  | [21:14]        | [13:0]        |
|-----------|---------|---------|----------|----------------|---------------|
| Tag Field | Opcode  | rd (5)  | rs1 (5)  | target_tag (8) | reserved (14) |

The target_tag field specifies the destination type. The source type is read from the tag of rs1. Conversion follows the conversion matrix defined in the Typing section.

We implement the following C-type instructions.

| Instruction | Opcode | Description                          | Notes                 |
|-------------|--------|--------------------------------------|-----------------------|
| CONVERT     | 0x1D   | rd = convert(rs1, target_tag)        | See conversion matrix |

CONVERT looks up the conversion matrix to determine the operation. If the matrix entry is — (trap), the instruction traps.

**MCONV** (Memory Convert) — converts the tag of a memory word in-place. Used for memory initialization (Free → data type), deallocation (data type → Free), and pointer memory management (Ptr ↔ Free). Data bits are preserved except when converting from Free to a data type, which **zeros the data bits** (the Zero operation) to prevent information leakage. Four variants in two pairs:

**User pair** — pointer register base, bounds-checked. Requires RW bit set. **Cannot target Ptr or Inert — traps if target_tag is Ptr or Inert.**

**MCONV** — pointer register + immediate offset:

| [45:40]   | [39:32] | [31:28] | [27:14]  | [13:6]         | [5:0]        |
|-----------|---------|---------|----------|----------------|--------------|
| Tag Field | Opcode  | pr (4)  | imm (14) | target_tag (8) | reserved (6) |

**MCONVR** — pointer register + GPR offset:

| [45:40]   | [39:32] | [31:28] | [27:23]        | [22:15]        | [14:0]        |
|-----------|---------|---------|----------------|----------------|---------------|
| Tag Field | Opcode  | pr (4)  | rs_offset (5)  | target_tag (8) | reserved (15) |

The `pr` field selects a pointer register. The `target_tag` field specifies the destination type tag.

**Kernel pair** — GPR base address, no bounds check. Requires kernel bit = 1. Can target any type including pointer types.

**MCONVI** — GPR base + immediate offset:

| [45:40]   | [39:32] | [31:27]      | [26:14]  | [13:6]         | [5:0]        |
|-----------|---------|--------------|----------|----------------|--------------|
| Tag Field | Opcode  | rs_base (5)  | imm (13) | target_tag (8) | reserved (6) |

**MCONVG** — GPR base + GPR offset:

| [45:40]   | [39:32] | [31:27]      | [26:22]        | [21:14]        | [13:0]        |
|-----------|---------|--------------|----------------|----------------|---------------|
| Tag Field | Opcode  | rs_base (5)  | rs_offset (5)  | target_tag (8) | reserved (14) |

| Instruction | Opcode | Description                          | Notes                                  |
|-------------|--------|--------------------------------------|----------------------------------------|
| MCONV       | 0x1E   | tag(mem[pr + imm]) = target_tag      | User, no pointer types                 |
| MCONVR      | 0x27   | tag(mem[pr + rs_offset]) = target_tag | User, no pointer types                 |
| MCONVI      | 0x28   | tag(mem[rs_base + imm]) = target_tag | Kernel, any type                       |
| MCONVG      | 0x29   | tag(mem[rs_base + rs_offset]) = target_tag | Kernel, any type                 |

Semantics: data bits are preserved except when converting from Free, which always zeros the data bits (the Zero operation from the conversion matrix) regardless of target type. Only Retag and Zero conversions from the conversion matrix are valid; all other conversion types (Scale, Trunc, Clamp, etc.) trap. User variants are bounds-checked through the pointer register with the RW bit required. Kernel variants use GPR addressing with no bounds check.

**MCONV pointer conversions** — in addition to the Retag/Zero entries from the CONVERT matrix above, MCONV supports these pointer-type conversions:

- **User mode** (MCONV, MCONVR): Ptr → Free only (Zero — data bits cleared, preventing address leakage). Operates one word at a time; both words of a Ptr pair must be converted separately to fully deallocate. Cannot target Inert (Inert is kernel-managed).
- **Kernel mode** (MCONVI, MCONVG): Full Retag in both directions (data bits preserved) — Free/U.Int ↔ Ptr. Also supports ↔ Inert conversions.

### Kernel Store (KSWI/KSWG)

Kernel GPR-addressed stores. These write a full 46-bit word (tag + data) from a source GPR to a memory address computed from a GPR base. **Requires kernel bit = 1** (always on in V0). No bounds check, no mutability check. These exist for bootstrapping: writing initial data to memory before any pointer registers exist.

**KSWI** — GPR base + immediate offset: `mem[rs_base + imm] = rs_src`

| [45:40]   | [39:32] | [31:27]      | [26:22]     | [21:7]   | [6:0]        |
|-----------|---------|--------------|-------------|----------|--------------|
| Tag Field | Opcode  | rs_base (5)  | rs_src (5)  | imm (15) | reserved (7) |

**KSWG** — GPR base + GPR offset: `mem[rs_base + rs_offset] = rs_src`

| [45:40]   | [39:32] | [31:27]      | [26:22]     | [21:17]        | [16:0]        |
|-----------|---------|--------------|-------------|----------------|---------------|
| Tag Field | Opcode  | rs_base (5)  | rs_src (5)  | rs_offset (5)  | reserved (17) |

| Instruction | Opcode | Description                              | Notes                  |
|-------------|--------|------------------------------------------|------------------------|
| KSWI        | 0x2B   | mem[rs_base + imm] = rs_src              | Kernel, GPR + imm      |
| KSWG        | 0x2C   | mem[rs_base + rs_offset] = rs_src        | Kernel, GPR + GPR      |

### P-type

#### MKPTR

**MKPTR** (Make Pointer) — creates a pointer from 2 contiguous U.Int words in memory. Retags them Ptr and loads the result into a destination pointer register. **All variants require kernel bit = 1** (always on in V0).

Four variants in two pairs:

**GPR-addressed pair** — GPR base address, no bounds check.

**MKPTRI** — GPR base + immediate offset:

| [45:40]   | [39:32] | [31:28]      | [27:23]      | [22:7]   | [6:0]     |
|-----------|---------|--------------|--------------|----------|-----------|
| Tag Field | Opcode  | pr_dst (4)   | rs_base (5)  | imm (16) | subop (7) |

**MKPTRG** — GPR base + GPR offset:

| [45:40]   | [39:32] | [31:28]      | [27:23]      | [22:18]        | [17:11]   | [10:0]        |
|-----------|---------|--------------|--------------|----------------|-----------|---------------|
| Tag Field | Opcode  | pr_dst (4)   | rs_base (5)  | rs_offset (5)  | subop (7) | reserved (11) |

**PR-addressed pair** — pointer register base, bounds-checked.

**MKPTRP** — pointer register + immediate offset:

| [45:40]   | [39:32] | [31:28]      | [27:24]      | [23:7]   | [6:0]     |
|-----------|---------|--------------|--------------|----------|-----------|
| Tag Field | Opcode  | pr_dst (4)   | pr_addr (4)  | imm (17) | subop (7) |

**MKPTRR** — pointer register + GPR offset:

| [45:40]   | [39:32] | [31:28]      | [27:24]      | [23:19]        | [18:12]   | [11:0]        |
|-----------|---------|--------------|--------------|----------------|-----------|---------------|
| Tag Field | Opcode  | pr_dst (4)   | pr_addr (4)  | rs_offset (5)  | subop (7) | reserved (12) |

The `pr_dst` field selects the destination pointer register. The `subop` field encodes permissions: bit 0 = reserved, bit 1 = RO(0)/RW(1), remaining bits reserved. Source memory must contain 2 consecutive U.Int-tagged words. Reads both words and retags them Ptr: word 1 becomes the base word (RW bit taken from subop bit 1), word 2 becomes the size word. A singleton is created by placing size = 1 in word 2 before calling MKPTR.

| Instruction | Opcode | Description                                | Notes                  |
|-------------|--------|--------------------------------------------|------------------------|
| MKPTRI      | 0x23   | Create pointer from mem[rs_base + imm]     | GPR + immediate, kernel |
| MKPTRG      | 0x24   | Create pointer from mem[rs_base + rs_offset] | GPR + GPR offset, kernel |
| MKPTRP      | 0x25   | Create pointer from mem[pr_addr + imm]     | PR + immediate, kernel |
| MKPTRR      | 0x26   | Create pointer from mem[pr_addr + rs_offset] | PR + GPR offset, kernel |

#### PNARROW

**PNARROW** (Pointer Narrow) — creates a new pointer register by narrowing an existing one. User-mode safe. Hardware enforces monotonicity: new bounds must be a subset of parent bounds, permissions can only drop (RW→RO fine, RO→RW traps).

| [45:40]   | [39:32] | [31:28]      | [27:24]     | [23:19]       | [18:14]       | [13] | [12:0]        |
|-----------|---------|--------------|-------------|---------------|---------------|------|---------------|
| Tag Field | Opcode  | pr_dst (4)   | pr_src (4)  | rs_offset (5) | rs_length (5) | perm | reserved (13) |

- `pr_dst`: destination pointer register
- `pr_src`: source (parent) pointer register
- `rs_offset`: GPR containing offset from parent base (new_base = pr_src.base + rs_offset)
- `rs_length`: GPR containing size of new region
- Hardware checks: rs_offset >= 0 AND rs_offset + rs_length <= pr_src.size; traps on violation
- Permission bit (1 bit in encoding): 0 = inherit parent permissions, 1 = read-only
- If parent is already read-only, permission bit 0 inherits read-only (cannot escalate)

| Instruction | Opcode | Description                                     | Notes                          |
|-------------|--------|-------------------------------------------------|--------------------------------|
| PNARROW     | 0x2A   | pr_dst = Ptr(pr_src.base + rs_offset, rs_length) | Monotonic bounds + permissions |

### System / Query Instructions

System and query instructions read processor state or pointer metadata into GPRs.

**RDSR** (Read Status Register) — reads the status register into a GPR, tagged U.Int. Read-only: hardware manages all flag setting/clearing transparently.

| [45:40]   | [39:32] | [31:27] | [26:0]               |
|-----------|---------|---------|----------------------|
| Tag Field | Opcode  | rd (5)  | reserved (27 bits)   |

**PLEN** (Pointer Length) — returns the size field of a pointer register into a GPR, tagged U.Int. Does not leak absolute addresses — only returns relative size.

| [45:40]   | [39:32] | [31:27] | [26:23]     | [22:0]               |
|-----------|---------|---------|-------------|----------------------|
| Tag Field | Opcode  | rd (5)  | pr_src (4)  | reserved (23 bits)   |

The `pr_src` field selects the source pointer register.

**MTAG** (Memory Tag) — reads the tag of a memory word into a GPR, tagged U.Int. Bounds-checked through the pointer register. Does **not** require the RW bit (read-only operation, not reading data). Does **not** trap on any memory tag — works on all 14 types including Ptr. Result has the 6-bit tag value in the low data bits ([5:0]), upper data bits zeroed.

**MTAG** — pointer register + immediate offset: `rd = U.Int(tag(mem[pr + imm]))`

| [45:40]   | [39:32] | [31:27] | [26:23] | [22:0]   |
|-----------|---------|---------|---------|----------|
| Tag Field | Opcode  | rd (5)  | pr (4)  | imm (23) |

**MTAGR** — pointer register + GPR offset: `rd = U.Int(tag(mem[pr + rs_offset]))`

| [45:40]   | [39:32] | [31:27] | [26:23] | [22:18]        | [17:0]        |
|-----------|---------|---------|---------|----------------|---------------|
| Tag Field | Opcode  | rd (5)  | pr (4)  | rs_offset (5)  | reserved (18) |

The `pr` field selects the addressing pointer register. Use with BTAG or BEQ to branch on memory tag before choosing LWP vs LPRI.

| Instruction | Opcode | Description                    | Notes                             |
|-------------|--------|--------------------------------|-----------------------------------|
| RDSR        | 0x2D   | rd = status_register           | U.Int tagged, read-only           |
| PLEN        | 0x2E   | rd = pr_src.size               | Returns size field                |
| MTAG        | 0x2F   | rd = U.Int(tag(mem[pr + imm])) | Bounds-checked, no RW required    |
| MTAGR       | 0x30   | rd = U.Int(tag(mem[pr + rs_offset])) | Bounds-checked, no RW required |
