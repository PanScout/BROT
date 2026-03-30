# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Structure

This is a specification-only repository for the BROT ISA (Instruction Set Architecture) — a RISC-style ISA with hardware-enforced type safety and bounds checking. There is no buildable code, test suite, or linter yet.

- `spec/SPEC.md` — The canonical ISA specification. All instruction encodings, tag definitions, register files, and semantic rules live here. This is the source of truth.
- `spec/BROT-scratchpad.md` — Design notes and working ideas (informal).
- `assembler/` — Placeholder for future assembler implementation.
- `BROT-0/` — Placeholder for future BROT-0 hardware/RTL implementation.
- `BVM/` — Placeholder for future BROT Virtual Machine implementation.

## Key Architecture Concepts

- **46-bit word width**: Every word is 6-bit tag + 5×8-bit bytes. Instruction encodings must sum to exactly 46 bits.
- **14 tag types** (tags 0x00–0x0D; 0x0E–0x3F reserved/Inert): Free, U.Int, ModU, PkU, Bool, S.Int, ModS, Q32.8, Q24.16, Q8.32, PkS, Instruction, Ptr, Inert. Groups: Free | Unsigned (0x01–0x04) | Signed (0x05–0x0A) | System (0x0B–0x0D).
- **Split register file**: 23 GPRs (46-bit, r0 hardwired U.Int(0)) + 13 pointer registers (92-bit). Pointer data never enters GPRs.
- **Two privilege levels**: Kernel (bit 3 in status register) and user mode. Pointer creation and pointer-type memory tagging are kernel-only operations.
- **10 instruction format types**: R, S, B, I, U, J, C, L, P, Q — each with defined bit layouts in SPEC.md.
- **Conversion matrix**: CONVERT instruction uses a defined matrix (in SPEC.md) mapping source×target type pairs to operations (Retag, Scale, Clamp, Trunc, trap). "—" entries trap.

## Core Principle

Every ISA decision exists in a circular constraint graph. Never evaluate an instruction, encoding, or semantic rule in isolation. Always trace its implications through at minimum these axes before proposing or accepting a change:

Every ISA decision exists in a circular constraint graph. Never evaluate an instruction, encoding, or semantic rule in isolation. Always trace its implications through at minimum these axes before proposing or accepting a change:

1. **Bootstrapping**: Can this feature be used before itself exists? If it depends on another feature, does that feature depend on it? Walk the boot sequence from reset with zero initialized state.
2. **Privilege boundary**: Who can execute this? If user-mode, can it be used to forge, widen, or escalate anything? If kernel-mode, is there a user-mode fast path so this doesn't become a syscall bottleneck?
3. **Type system coherence**: Does this instruction respect the tag invariants? Can it move typed data into a context where the type loses meaning (e.g., pointer bits into a GPR)? Can it create typed data from untyped sources without going through the proper creation path?
4. **Encoding budget**: Does this fit in 46 bits? Count the actual bits for every field. If a field is "4 bits used" in an 8-bit byte, state what the remaining bits are for. Never leave spare bits unallocated — they will be squatted on later.
5. **Pipeline impact**: Does this instruction require multi-cycle memory operations (e.g., atomic 2-word pointer load/store on a 46-bit datapath)? Does it introduce new hazards? Is it on a latency-critical path?
6. **Memory model interaction**: What tags can the source/destination memory have? What happens for each possible tag? Enumerate them, don't assume.

## Mandatory Checks

Before proposing any new instruction or modification:

- **Write the boot sequence**. Start from reset (all memory Inert, no pointer registers initialized, kernel bit = 1). Execute your proposed instruction sequence step by step. If any step requires a resource that hasn't been created yet, the design is broken.
- **Write the attack sequence**. Assume user-mode code wants to forge a pointer to address 0x0000 with full RW permissions. Attempt to construct this using every user-mode instruction available. If any path succeeds, the security model is broken.
- **Write the spill/restore sequence**. Your instruction's operands live in registers. Registers get spilled during calls and interrupts. Can every operand type be saved to memory and restored without losing information or creating a forgery vector?
- **Check the tag matrix**. For every instruction that reads memory, enumerate what happens for each of the 15 tag values at the target address. For every instruction that writes memory, enumerate what happens for each tag at the destination. "Trap" is a valid answer. "Undefined" is not.

## BROT-Specific Invariants

These are load-bearing. Violating any of them means the design is broken:

- **Pointer bits never enter a GPR.** LWI/LWP trap on pointer-tagged memory. CONVERT has no pointer rows/columns. The only path for pointer data is: pointer register ↔ pointer-tagged memory via LPRI/LPRP/SPRI/SPRP.
- **Pointer creation is kernel-only.** MKPTR (all variants) requires kernel bit = 1. User code creates pointers only via PNARROW, which enforces monotonic narrowing (bounds shrink, permissions drop, never widen).
- **Memory retagging to pointer types is kernel-only.** User-mode MCONV/MCONVR trap if target_tag is Ptr or Inert. Only kernel MCONVI/MCONVG can create pointer-tagged memory.
- **Immutable memory is enforced.** System types (Inert, Instruction, Ptr) cannot be written by SWI/SWP. MCONV is the only path to change their tags, and only via Retag operations.
- **Monotonicity is hardware-enforced.** PNARROW checks new_bounds ⊆ parent_bounds AND new_permissions ⊆ parent_permissions. This is not a software convention — it is a hardware trap.
- **All memory access goes through pointer registers.** The only exceptions are kernel-mode GPR-addressed instructions (MKPTRI/MKPTRG, MCONVI/MCONVG) which exist solely for bootstrapping and kernel memory management.

## Common Failure Modes

These are the mistakes that will be made repeatedly. Check for them explicitly:

- **Chicken-and-egg**: Instruction X requires resource Y, but resource Y can only be created by instruction X. Example: MCONV requires a pointer register, but the first pointer register requires MCONV to prepare memory.
- **Tag leakage**: A path exists where typed data reaches a context that strips or ignores the tag. Example: LWI loading pointer-tagged memory into a GPR exposes raw address bits.
- **Forgery via data path**: User code can write arbitrary integers to memory, then use an instruction to reinterpret those integers as a privileged type. Example: user-mode MKPTR reading user-written U.Int values to construct arbitrary pointers.
- **Permission escalation**: An instruction allows creating a more-permissive version of a restricted resource. Example: PNARROW producing an RW pointer from an RO parent.
- **Encoding phantom**: A field is described in text but doesn't exist in the bit layout, or the bit layout has unnamed fields that will be interpreted differently by different implementations. Example: PNARROW's permission bit being described but not allocated in the encoding.
- **Implicit assumptions about memory state**: An instruction's semantics assume memory is in a particular state (e.g., "source must be U.Int") without considering what happens at boot when everything is Inert.

## How to Propose Changes

When adding or modifying an instruction:

1. State the instruction's purpose in one sentence.
2. Write the encoding with exact bit positions. Count the bits. They must sum to 46.
3. State the kernel/user mode requirement.
4. For each operand, state whether it's a GPR index, PR index, or immediate, and how many bits it uses.
5. State what memory tags are valid at source/destination and what happens for each invalid tag.
6. Write one example of correct use in a realistic code sequence.
7. Write one example of attempted misuse and confirm it traps.
8. Confirm the boot sequence still works with this change.
