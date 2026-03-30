# BROT ISA V0 Stress-Test Programs

## Conventions

**Register state**: `r1 = U.Int(0x100)` means GPR r1 has tag=U.Int, data=0x100.
**Memory state**: `M[0x100] = (Free, 0x0)` means address 0x100 has tag=Free, data=0x0.
**Pointer register**: `p2 = Array(RW, 0x100, 0x110)` means 82-bit register with PtrA base=0x100, PtrE end=0x110, RW=1.
**Singleton pointer**: `p2 = Singleton(RO, 0x200)` means PtrS target=0x200, RW=0.
All addresses are **word addresses**. Immediates are signed two's complement.
Instruction tags omitted from assembly (hardware sets tag=Instruction on fetch in the encoding, but memory tag is irrelevant in V0).

---

## Program 1: Boot from Reset to First SWI

**Purpose**: Demonstrate the complete boot sequence from all-zero state to a bounded store through a created pointer.

**Pre-conditions**: All memory `(Free, 0x0)`. `r0 = U.Int(0)`. r1–r22 assumed uninitialized (treated as unknown tag). Kernel bit = 1. PC = 0x0 (assumed).

### Assembly

```
; === BOOT: Reset to first SWI ===
; Goal: create an RW array pointer p2 covering [0x100, 0x108),
;       then SWI a value through it, then verify with LWI.
;
; Memory map:
;   0x000–0x00F  Code (pre-loaded, tag irrelevant in V0)
;   0x010–0x011  Scratch area for MKPTR source words
;   0x100–0x107  Data region (target of pointer)

; --- Step 1: Build address values in GPRs ---
; r0 = U.Int(0) [hardwired]

; 0x000: ADDI r1, r0, 0x10       ; r1 = U.Int(0x10) — scratch address
```
| After | r1 = U.Int(0x10) | Note: ADDI source is r0 (U.Int), result inherits U.Int tag |
|-------|------------------|-----|

```
; 0x001: ADDI r2, r0, 0x100      ; r2 = U.Int(0x100) — base address for pointer
```
| After | r2 = U.Int(0x100) | |
|-------|-------------------|---|

```
; 0x002: ADDI r3, r0, 0x108      ; r3 = U.Int(0x108) — end address for pointer
```
| After | r3 = U.Int(0x108) | |
|-------|-------------------|---|

```
; --- Step 2: Write pointer source data to scratch area via KSWI ---
; KSWI writes full 41-bit word (tag+data) from source GPR.
; KSWI imm field is 7 bits [13:7], range -64..+63.

; 0x003: KSWI r1, r2, 0          ; mem[r1 + 0] = r2 → mem[0x10] = U.Int(0x100)
```
| Before | M[0x10] = (Free, 0x0) |
|--------|----------------------|
| After  | M[0x10] = (U.Int, 0x100) |
| Check  | Kernel-only: kernel=1 ✓. No bounds check. No mutability check. |

```
; 0x004: KSWI r1, r3, 1          ; mem[r1 + 1] = r3 → mem[0x11] = U.Int(0x108)
```
| Before | M[0x11] = (Free, 0x0) |
|--------|----------------------|
| After  | M[0x11] = (U.Int, 0x108) |

```
; --- Step 3: Create pointer from the two U.Int words ---
; MKPTRI pr_dst, rs_base, imm, subop
; subop = 0b11: bit0=1 (array), bit1=1 (RW)
; Reads mem[r1+0] and mem[r1+1], retags to PtrA+PtrE, loads into p2.

; 0x005: MKPTRI p2, r1, 0, 0b11  ; p2 = Array(RW, 0x100, 0x108)
```
| Before | M[0x10] = (U.Int, 0x100), M[0x11] = (U.Int, 0x108) |
|--------|-----------------------------------------------------|
| After  | M[0x10] = (PtrA, RW=1, base=0x100), M[0x11] = (PtrE, end=0x108) |
| Register | p2 = Array(RW, 0x100, 0x108) |
| Check | Kernel-only: kernel=1 ✓. Source tags both U.Int ✓. |
| Note | Memory is retagged to PtrA/PtrE (now immutable system types). Scratch area 0x10–0x11 cannot be reused via SWI without MCONV. |

```
; --- Step 4: Prepare a value to store ---

; 0x006: ADDI r4, r0, 42         ; r4 = U.Int(42) — the value to store
```
| After | r4 = U.Int(42) | |
|-------|----------------|---|

```
; --- Step 5: First SWI through the created pointer ---

; 0x007: SWI p2, r4, 0           ; mem[p2.base + 0] = r4 → mem[0x100] = U.Int(42)
```
| Before | M[0x100] = (Free, 0x0) |
|--------|------------------------|
| After  | M[0x100] = (U.Int, 42) |
| Check | Bounds: 0x100 ∈ [0x100, 0x108) ✓. RW: p2.RW=1 ✓. Target mutable: Free ✓. Source mutable: U.Int ✓. |
| **This is the first bounded store.** |

```
; --- Step 6: Verify the store with LWI ---

; 0x008: LWI r5, p2, 0           ; r5 = mem[p2.base + 0] = mem[0x100]
```
| After | r5 = U.Int(42) |
|-------|----------------|
| Check | Bounds: 0x100 ∈ [0x100, 0x108) ✓. Tag at M[0x100] = U.Int (not pointer) ✓. |

```
; 0x009: BNE r5, r4, 0x00F       ; if r5 ≠ r4, jump to FAIL (0x009 + 6 = 0x00F)
```
| Check | Type match: both U.Int ✓. r5=42, r4=42, equal → not taken ✓. |

```
; --- Step 7: Store a second value at a different offset ---

; 0x00A: ADDI r6, r0, 99         ; r6 = U.Int(99)
; 0x00B: SWI p2, r6, 7           ; mem[0x100 + 7] = mem[0x107] = U.Int(99)
```
| Before | M[0x107] = (Free, 0x0) |
|--------|------------------------|
| After  | M[0x107] = (U.Int, 99) |
| Check | Bounds: 0x107 ∈ [0x100, 0x108) ✓. |

```
; --- DONE ---
; 0x00C: JAL r0, 0               ; halt (infinite loop at PC=0x00C + 0 = 0x00C)
```

### Issues

| ID | Severity | Issue | Fix Proposal |
|----|----------|-------|-------------|
| 1.1 | Significant | **Initial r1–r22 state unspecified.** If r1 starts as Free-tagged, ADDI with r1 as *destination* still works (only rs1 tag matters, not rd). But any instruction reading an uninitialized GPR as rs1/rs2 would have unpredictable behavior. The spec should define initial GPR state. | Specify r1–r22 = U.Int(0) at reset, matching r0. |
| 1.2 | Minor | **KSWI immediate is only 7 bits** (range -64..+63), while LWI/SWI have 14-bit immediates. For boot code writing to addresses far from the GPR base, this requires extra ADDI instructions to adjust the base. | Accept as encoding constraint; boot code can compute larger bases via ADDI. |
| 1.3 | Minor | **Instruction fetch tag checking unspecified.** At boot, code memory is Free-tagged (or whatever the loader sets). The spec doesn't say whether the CPU checks for tag=Instruction during fetch. In V0 it presumably doesn't, but this should be explicit. | Add to spec: "V0 does not check memory tags during instruction fetch. Future versions may require Instruction-tagged memory." |
| 1.4 | Minor | **MKPTRI retags scratch memory to PtrA/PtrE**, making it immutable. The scratch area at 0x10–0x11 is now unusable for SWI without MCONV cleanup. This is correct behavior but means every MKPTR call "consumes" its source memory. | Document: MKPTR source memory becomes pointer-tagged (immutable). Plan scratch areas as single-use or budget MCONV cleanup. |

---

## Program 2: Memory Lifecycle — Allocate, Write, Free, Reallocate, Confirm Zeroing

**Purpose**: Exercise the full memory lifecycle and confirm the Zero operation prevents information leakage from freed memory.

**Pre-conditions**: Boot prologue from Program 1 completed. p2 = Array(RW, 0x100, 0x108). Kernel bit = 1.

### Assembly

```
; === MEMORY LIFECYCLE ===
; Goal: PNARROW a sub-region, write data, free (MCONV→Free),
;       reallocate as different type, confirm zeroing.
;       Then test the (now-fixed) Free→Inert path.
;
; Setup: p2 = Array(RW, 0x100, 0x108) from boot prologue.
; We'll narrow p3 to [0x100, 0x104) — 4-word sub-region.

; --- Step 1: PNARROW to allocate sub-region ---
; Need offset=0 and length=4 in GPRs.
; r0 = U.Int(0) [hardwired], use as offset.

; 0x010: ADDI r7, r0, 4          ; r7 = U.Int(4) — length
```
| After | r7 = U.Int(4) | |
|-------|---------------|---|

```
; 0x011: PNARROW p3, p2, r0, r7, perm=0
;        new_base = p2.base + r0 = 0x100 + 0 = 0x100
;        new_end  = 0x100 + r7 = 0x100 + 4 = 0x104
;        perm=0: inherit parent RW → RW
```
| After | p3 = Array(RW, 0x100, 0x104) |
|-------|-------------------------------|
| Check | Source is array ✓. new_base(0x100) >= p2.base(0x100) ✓. new_end(0x104) <= p2.end(0x108) ✓. Perm=0, parent RW → child RW ✓. |
| Note | rs_offset=r0 must be U.Int for PNARROW. r0 = U.Int(0) ✓. rs_length=r7 = U.Int(4) ✓. |

```
; --- Step 2: Write U.Int data through the narrowed pointer ---

; 0x012: ADDI r8, r0, 0x55       ; r8 = U.Int(0x55) — test pattern
; 0x013: SWI p3, r8, 0           ; mem[0x100] = U.Int(0x55)
```
| Before | M[0x100] = (U.Int, 42) | Note: leftover from Program 1 |
|--------|------------------------|------|
| After  | M[0x100] = (U.Int, 0x55) |
| Check | Bounds: 0x100 ∈ [0x100, 0x104) ✓. RW ✓. Target mutable: U.Int ✓. Source mutable: U.Int ✓. |

```
; 0x014: SWI p3, r8, 1           ; mem[0x101] = U.Int(0x55)
```
| Before | M[0x101] = (Free, 0x0) |
|--------|------------------------|
| After  | M[0x101] = (U.Int, 0x55) |

```
; 0x015: SWI p3, r8, 2           ; mem[0x102] = U.Int(0x55)
; 0x016: SWI p3, r8, 3           ; mem[0x103] = U.Int(0x55)
```
| After | M[0x102] = (U.Int, 0x55), M[0x103] = (U.Int, 0x55) |

```
; --- Step 3: Verify writes ---

; 0x017: LWI r9, p3, 0           ; r9 = U.Int(0x55)
; 0x018: BNE r9, r8, FAIL        ; both U.Int, equal → not taken ✓
```

```
; --- Step 4: Free the memory — MCONV to Free ---
; U.Int → Free in CONVERT matrix = Retag (data preserved).
; MCONV allows Retag operations.
; MCONV imm field is 7 bits [20:14].

; 0x019: MCONV p3, 0, Free       ; tag(mem[0x100]) = Free
```
| Before | M[0x100] = (U.Int, 0x55) |
|--------|--------------------------|
| After  | M[0x100] = (Free, 0x55) |
| Check | Bounds: 0x100 ∈ [0x100, 0x104) ✓. RW ✓. Matrix: U.Int→Free = Retag ✓. Target Free is not ptr/Inert ✓. |
| Note | **Data bits preserved!** The freed memory still contains 0x55. |

```
; 0x01A: MCONV p3, 1, Free       ; tag(mem[0x101]) = Free
; 0x01B: MCONV p3, 2, Free       ; tag(mem[0x102]) = Free
; 0x01C: MCONV p3, 3, Free       ; tag(mem[0x103]) = Free
```
| After | M[0x100..0x103] = (Free, 0x55) each — data preserved in all |

```
; --- Step 5: Reallocate as S.Int — MCONV Free → S.Int ---
; Free → S.Int in CONVERT matrix = Zero (data bits cleared).
; This is the information leakage prevention.

; 0x01D: MCONV p3, 0, S.Int      ; tag(mem[0x100]) = S.Int
```
| Before | M[0x100] = (Free, 0x55) |
|--------|-------------------------|
| After  | M[0x100] = (S.Int, 0x0) |
| Check | Matrix: Free→S.Int = Zero ✓. Data bits cleared to 0. |
| **The old value 0x55 is gone. Zero operation prevents leakage.** |

```
; 0x01E: MCONV p3, 1, S.Int      ; mem[0x101] = (S.Int, 0x0)
; 0x01F: MCONV p3, 2, S.Int      ; mem[0x102] = (S.Int, 0x0)
; 0x020: MCONV p3, 3, S.Int      ; mem[0x103] = (S.Int, 0x0)
```

```
; --- Step 6: Verify zeroing ---

; 0x021: LWI r10, p3, 0          ; r10 = S.Int(0x0)
```
| After | r10 = S.Int(0x0) |
|-------|------------------|

```
; To compare with zero, need S.Int(0). r0 is U.Int(0) — type mismatch!
; Must CONVERT r0 to S.Int first.

; 0x022: CONVERT r11, r0, S.Int   ; r11 = S.Int(0)
```
| After | r11 = S.Int(0) |
|-------|----------------|
| Check | U.Int(0) → S.Int: Clamp operation. 0 is in S.Int range → S.Int(0). |

```
; 0x023: BNE r10, r11, FAIL      ; S.Int(0) == S.Int(0) → not taken ✓
```
| Check | Type match: both S.Int ✓. Equal ✓. |

```
; --- Step 7: Test the (fixed) Free→Inert path ---
; Before the spec fix, Free→Inert was Retag (data preserved).
; After the fix, Free→Inert is Zero (data cleared).
; Demonstrate that the bypass no longer works.

; First, write a "secret" value and free it.
; 0x024: MCONV p3, 0, Free       ; mem[0x100]: S.Int(0) → Free(0) [Retag]
; 0x025: ADDI r12, r0, 0x7F      ; r12 = U.Int(0x7F) — "secret"
; 0x026: SWI p3, r12, 0          ; mem[0x100] = U.Int(0x7F)
```
| After | M[0x100] = (U.Int, 0x7F) |

```
; Now free it — data preserved in Free-tagged memory.
; 0x027: MCONV p3, 0, Free       ; mem[0x100]: U.Int(0x7F) → Free(0x7F) [Retag]
```
| After | M[0x100] = (Free, 0x7F) | Data 0x7F preserved — Free memory has residual data. |

```
; Load the freed memory into a GPR.
; 0x028: LWI r13, p3, 0          ; r13 = Free(0x7F)
```
| After | r13 = Free(0x7F) |
|-------|-------------------|
| Note | LWI on Free-tagged memory succeeds (Free is not a pointer type). |

```
; Attempt the bypass: CONVERT Free → Inert.
; BEFORE FIX: This was Retag (data preserved) — bypass succeeded.
; AFTER FIX: This is Zero (data cleared) — bypass blocked.

; 0x029: CONVERT r14, r13, Inert  ; r14 = Inert(0x0)
```
| After | r14 = Inert(0x0) |
|-------|-------------------|
| Note | **Free→Inert = Zero (after fix).** Data bits cleared to 0. The "secret" 0x7F is gone. |

```
; Even if we continue: CONVERT Inert → U.Int is Retag, but data is already 0.
; 0x02A: CONVERT r15, r14, U.Int  ; r15 = U.Int(0x0)
```
| After | r15 = U.Int(0x0) |
|-------|-------------------|
| Note | Inert(0x0) → U.Int(0x0) via Retag. Data is 0, not 0x7F. Bypass defeated. |

```
; Verify: r15 should be 0, not the secret 0x7F.
; 0x02B: BNE r15, r0, FAIL       ; U.Int(0) == U.Int(0) → not taken ✓
;                                 ; If bypass worked, r15 would be U.Int(0x7F) and branch would be taken.

; --- Also test Free → Instruction path (same fix) ---
; 0x02C: CONVERT r16, r13, Instruction  ; r16 = Instruction(0x0)
```
| After | r16 = Instruction(0x0) |
|-------|------------------------|
| Note | Free→Instruction = Zero (after fix). Data cleared. |

```
; 0x02D: CONVERT r17, r16, U.Int  ; r17 = U.Int(0x0) — Retag, but data already 0.
; 0x02E: BNE r17, r0, FAIL        ; U.Int(0) == U.Int(0) → not taken ✓

; --- DONE ---
; 0x02F: JAL r0, 0                ; halt
```

### Issues

| ID | Severity | Issue | Fix Proposal |
|----|----------|-------|-------------|
| 2.1 | Significant | **MCONV data→Free preserves data bits** (Retag). Freed memory retains old values. While the Zero operation on reallocation (Free→data type) clears them, this creates a window where freed data is readable via LWI. After the spec fix (Free→Inert/Instruction = Zero), the bypass is closed, but Free(secret_data) can still be loaded into a GPR. It just can't be converted to a usable type without zeroing. | Consider changing data→Free from Retag to Zero in MCONV to clear data at free time (defense in depth). |
| 2.2 | Minor | **Type matching on branch comparisons requires explicit CONVERT.** Comparing S.Int result against r0 (U.Int) requires a CONVERT step. Every type boundary in control flow costs an instruction. | Accept as designed — explicit type coercion is a feature. |
| 2.3 | Minor | **PNARROW always produces array pointers.** Even a 1-word PNARROW gives PtrA+PtrE, not PtrS. This is fine functionally but means singletons can only be created via MKPTR. | Accept. |
| 2.4 | Minor | **MCONV 7-bit immediate** limits direct offset to ±63. For freeing/reallocating arrays of >64 elements, must use MCONVR with a GPR offset. | Accept as encoding constraint. |

---

## Program 3: Function Call — Save/Restore 4 GPRs + 2 Pointer Registers, Nested Call

**Purpose**: Exercise a calling convention with register spill/restore, pointer register spill via SPRI/LPRI, and stack cleanup of immutable pointer tags.

**Pre-conditions**: Boot prologue completed. p0 = Array(RW, 0x200, 0x240) (64-word stack). r22 = U.Int(0) (frame pointer, offset from p0.base). Kernel bit = 1.

### Setup (extends boot prologue)

```
; --- Stack setup ---
; Create p0 = Array(RW, 0x200, 0x240) via MKPTRI.

; 0x030: ADDI r1, r0, 0x20       ; r1 = U.Int(0x20) — scratch for stack ptr words
; 0x031: ADDI r2, r0, 0x200      ; r2 = U.Int(0x200) — stack base
; 0x032: ADDI r3, r0, 0x240      ; r3 = U.Int(0x240) — stack end (64 words)
; 0x033: KSWI r1, r2, 0          ; mem[0x20] = U.Int(0x200)
; 0x034: KSWI r1, r3, 1          ; mem[0x21] = U.Int(0x240)
; 0x035: MKPTRI p0, r1, 0, 0b11  ; p0 = Array(RW, 0x200, 0x240)
```
| After | p0 = Array(RW, 0x200, 0x240) |
|-------|-------------------------------|
| | M[0x20] = (PtrA, RW=1, 0x200), M[0x21] = (PtrE, 0x240) |

```
; Initialize frame pointer.
; 0x036: ADDI r22, r0, 0         ; r22 = U.Int(0) — current stack offset

; Set up some callee-saved registers with known values for testing.
; 0x037: ADDI r11, r0, 0xAA      ; r11 = U.Int(0xAA) — callee-saved
; 0x038: ADDI r12, r0, 0xBB      ; r12 = U.Int(0xBB) — callee-saved
; 0x039: ADDI r13, r0, 0xCC      ; r13 = U.Int(0xCC) — callee-saved
; 0x03A: ADDI r14, r0, 0xDD      ; r14 = U.Int(0xDD) — callee-saved

; p2 = Array(RW, 0x100, 0x108) from boot prologue — callee-saved
; p3 = Array(RW, 0x100, 0x104) from Program 2 — callee-saved

; --- Call func_a ---
; 0x03B: JAL r21, func_a         ; r21 = U.Int(PC+1 = 0x03C), PC = func_a
```
| After | r21 = U.Int(0x03C) | Return address stored as U.Int in GPR |

```
; --- Return here after func_a ---
; 0x03C: (verify r11–r14 and p2, p3 are restored)
; 0x03C: ADDI r1, r0, 0xAA      ; r1 = U.Int(0xAA)
; 0x03D: BNE r11, r1, FAIL      ; r11 should be 0xAA ✓
; 0x03E: ADDI r1, r0, 0xBB
; 0x03F: BNE r12, r1, FAIL      ; r12 should be 0xBB ✓
; 0x040: JAL r0, 0               ; halt — SUCCESS


; ============================================================
; func_a: saves r11–r14, p2, p3, then calls func_b
; ============================================================
; Convention:
;   r22 = frame offset from p0.base
;   r21 = return address (U.Int)
;   Frame layout (10 words):
;     [r22+0] = saved r21 (return addr)
;     [r22+1] = saved r11
;     [r22+2] = saved r12
;     [r22+3] = saved r13
;     [r22+4] = saved r14
;     [r22+5..6] = saved p2 (array = 2 words: PtrA + PtrE)
;     [r22+7..8] = saved p3 (array = 2 words: PtrA + PtrE)
;     [r22+9] = (padding/unused)

func_a:
; --- Prologue: save registers ---
; 0x050: SWP p0, r21, r22        ; mem[0x200 + 0] = U.Int(0x03C) — save return addr
```
| Before | M[0x200] = (Free, 0x0) |
|--------|------------------------|
| After  | M[0x200] = (U.Int, 0x03C) |
| Check | Bounds: 0x200 ∈ [0x200, 0x240) ✓. RW ✓. Target Free (mutable) ✓. Source U.Int (mutable) ✓. |

```
; 0x051: ADDI r22, r22, 1        ; r22 = U.Int(1)
; 0x052: SWP p0, r11, r22        ; mem[0x201] = U.Int(0xAA) — save r11
; 0x053: ADDI r22, r22, 1        ; r22 = U.Int(2)
; 0x054: SWP p0, r12, r22        ; mem[0x202] = U.Int(0xBB) — save r12
; 0x055: ADDI r22, r22, 1        ; r22 = U.Int(3)
; 0x056: SWP p0, r13, r22        ; mem[0x203] = U.Int(0xCC) — save r13
; 0x057: ADDI r22, r22, 1        ; r22 = U.Int(4)
; 0x058: SWP p0, r14, r22        ; mem[0x204] = U.Int(0xDD) — save r14
```
| After | M[0x200..0x204] = U.Int(0x03C, 0xAA, 0xBB, 0xCC, 0xDD) |

```
; 0x059: ADDI r22, r22, 1        ; r22 = U.Int(5)
; 0x05A: SPRP p0, p2, r22        ; mem[0x205..0x206] = p2 (PtrA + PtrE)
```
| Before | M[0x205] = (Free, 0x0), M[0x206] = (Free, 0x0) |
|--------|------------------------------------------------|
| After  | M[0x205] = (PtrA, RW=1, 0x100), M[0x206] = (PtrE, 0x108) |
| Check | Bounds: 0x205, 0x206 ∈ [0x200, 0x240) ✓. RW ✓. Target Free (mutable) ✓. |
| **Note: Stack memory at 0x205–0x206 is now PtrA/PtrE tagged (immutable system types).** |

```
; 0x05B: ADDI r22, r22, 2        ; r22 = U.Int(7)
; 0x05C: SPRP p0, p3, r22        ; mem[0x207..0x208] = p3 (PtrA + PtrE)
```
| After | M[0x207] = (PtrA, RW=1, 0x100), M[0x208] = (PtrE, 0x104) |

```
; 0x05D: ADDI r22, r22, 2        ; r22 = U.Int(9)

; --- func_a body: trash the callee-saved registers ---
; 0x05E: ADDI r11, r0, 0x11      ; r11 = U.Int(0x11) — clobbered
; 0x05F: ADDI r12, r0, 0x22      ; r12 = U.Int(0x22) — clobbered
; 0x060: ADDI r13, r0, 0x33      ; r13 = U.Int(0x33) — clobbered
; 0x061: ADDI r14, r0, 0x44      ; r14 = U.Int(0x44) — clobbered

; --- Call func_b (nested call) ---
; 0x062: JAL r21, func_b         ; r21 = U.Int(0x063), PC = func_b
```
| After | r21 = U.Int(0x063) | Old return address was saved on stack at M[0x200]. |

```
; --- Return from func_b ---
; 0x063: (continue with epilogue)

; --- Epilogue: restore registers (reverse order) ---
; First restore pointer registers BEFORE cleaning up their stack slots.
; LPRI/LPRP requires the memory to still have pointer tags.

; 0x063: ADDI r22, r22, -2       ; r22 = U.Int(7)
; 0x064: LPRP p3, p0, r22        ; p3 = mem[0x207..0x208] = Array(RW, 0x100, 0x104)
```
| Check | Bounds ✓. M[0x207] tag = PtrA ✓ (first word must be PtrS/PtrA). Loads 2 words. |
| After | p3 = Array(RW, 0x100, 0x104) — restored |

```
; Now clean up the PtrA/PtrE tags so the stack can be reused.
; 0x065: MCONV p0, 7, Free       ; mem[0x207]: PtrA → Free (Zero, data cleared)
; 0x066: ADDI r1, r0, 8
; 0x067: MCONVR p0, r1, Free     ; mem[0x208]: PtrE → Free (Zero, data cleared)
```
| After | M[0x207] = (Free, 0x0), M[0x208] = (Free, 0x0) |
|-------|------------------------------------------------|
| Note | User-mode MCONV PtrA/PtrE → Free = Zero (data bits cleared, prevents address leakage). |
| **Without this cleanup, future SWP to offsets 7–8 would trap (immutable target).** |

```
; 0x068: ADDI r22, r22, -2       ; r22 = U.Int(5)
; 0x069: LPRP p2, p0, r22        ; p2 = mem[0x205..0x206] = Array(RW, 0x100, 0x108)
; 0x06A: MCONV p0, 5, Free       ; mem[0x205]: PtrA → Free (Zero)
; 0x06B: ADDI r1, r0, 6
; 0x06C: MCONVR p0, r1, Free     ; mem[0x206]: PtrE → Free (Zero)

; Now restore GPRs (their stack slots have mutable U.Int tags, no cleanup needed).
; 0x06D: ADDI r22, r22, -1       ; r22 = U.Int(4)
; 0x06E: LWP r14, p0, r22        ; r14 = U.Int(0xDD) — restored
; 0x06F: ADDI r22, r22, -1       ; r22 = U.Int(3)
; 0x070: LWP r13, p0, r22        ; r13 = U.Int(0xCC)
; 0x071: ADDI r22, r22, -1       ; r22 = U.Int(2)
; 0x072: LWP r12, p0, r22        ; r12 = U.Int(0xBB)
; 0x073: ADDI r22, r22, -1       ; r22 = U.Int(1)
; 0x074: LWP r11, p0, r22        ; r11 = U.Int(0xAA)
; 0x075: ADDI r22, r22, -1       ; r22 = U.Int(0)
; 0x076: LWP r21, p0, r22        ; r21 = U.Int(0x03C) — return address restored
; 0x077: JALR r0, r21, 0         ; PC = 0x03C, r0 discarded
```
| Check | JALR: PC = (r21 + 0) & ~1 = 0x03C & ~1 = 0x03C. Returns to caller. |


```
; ============================================================
; func_b: minimal nested function, saves/restores r11 only
; ============================================================
; Frame at r22 = 9 (continuing from func_a's frame end).
; Frame layout: [r22+0] = saved r21

func_b:
; 0x080: SWP p0, r21, r22        ; mem[0x209] = U.Int(0x063)
; 0x081: ADDI r22, r22, 1        ; r22 = U.Int(10)
; 0x082: SWP p0, r11, r22        ; mem[0x20A] = U.Int(0x11) — func_a's clobbered value
```
| After | M[0x209] = (U.Int, 0x063), M[0x20A] = (U.Int, 0x11) |

```
; --- func_b body: trash r11 ---
; 0x083: ADDI r11, r0, 0xFF      ; r11 = U.Int(0xFF) — clobbered by func_b

; --- func_b epilogue ---
; 0x084: LWP r11, p0, r22        ; r11 = U.Int(0x11) — restored to func_a's value
; 0x085: ADDI r22, r22, -1       ; r22 = U.Int(9)
; 0x086: LWP r21, p0, r22        ; r21 = U.Int(0x063) — return to func_a
; 0x087: JALR r0, r21, 0         ; PC = 0x063
```

### Issues

| ID | Severity | Issue | Fix Proposal |
|----|----------|-------|-------------|
| 3.1 | Significant | **Pointer spill leaves PtrA/PtrE on stack (immutable).** Must MCONV each word back to Free before that stack space can be reused. For array pointers, that's 2 MCONV calls per pointer register. A function saving 4 pointer registers needs 8 extra MCONV instructions in the epilogue. | This is inherent to the type system. Document in calling convention: "After LPRP restore, MCONV the stack slots back to Free." |
| 3.2 | Significant | **Restore-before-cleanup ordering constraint.** LPRP requires the stack memory to still have PtrA/PtrE tags. MCONV removes those tags. So the order MUST be: LPRP first, then MCONV. If cleanup happens before restore, the pointer data is lost (MCONV PtrA→Free = Zero clears data bits). | Document: "Always restore pointer registers before cleaning up their stack slots." |
| 3.3 | Significant | **JAL/JALR store return address in GPR (U.Int), not pointer register.** The spec designates p1 as "Link Pointer" but no instruction writes to it. Return addresses are unbounded integers — a corrupted return address in r21 leads to arbitrary code execution. | Add a JALP variant that stores PC+1 as a bounded singleton pointer in p1, and JALRP that reads from p1. |
| 3.4 | Significant | **No stack pointer adjustment.** The stack pointer p0 always covers the full stack. Frame management relies on a GPR offset (r22) with SWP/LWP. There's no way to narrow p0 for the callee (PNARROW could do it, but then the callee can't restore the parent's wider p0). | This GPR-offset approach works but means the callee has access to the caller's stack frame. For isolation, the caller would need to PNARROW a sub-stack and pass it, but that requires the callee to receive the narrowed pointer as an argument. |
| 3.5 | Minor | **10 instructions for pointer register save/restore** (SWP × 1 + ADDI × 1 + SPRP × 1 + LPRP × 1 + MCONV × 2 + MCONVR × 0..1 + ADDI × several). This is expensive compared to GPR save/restore (SWP + LWP, 2 instructions per register). | Accept as the cost of type safety. |

---

## Program 4: FIR Filter Using Q28.7 MAC

**Purpose**: Implement a 4-tap FIR filter using fixed-point multiply-accumulate, exercising Q28.7 arithmetic, MAC auto-rescaling, and bounded array traversal.

**Pre-conditions**: Boot prologue completed. Kernel bit = 1. We set up:
- p4 = Array(RW, 0x300, 0x304) — 4 coefficient slots (Q28.7)
- p5 = Array(RW, 0x310, 0x318) — 8 sample slots (Q28.7)
- p6 = Array(RW, 0x320, 0x321) — 1 output slot (Q28.7)

### Why Q28.7 instead of Q18.17

Q18.17 has 17 fractional bits. A 14-bit signed ADDI immediate on Q18.17 covers raw values -8192..+8191, representing -0.0625..+0.0624 in Q18.17 — far too small for filter coefficients. SLL/SRL are integral-only and trap on fixed-point. MCONV can't do U.Int→Q18.17 (Scale, not Retag/Zero). There is **no clean path to construct arbitrary fractional Q18.17 constants**.

Q28.7 has 7 fractional bits. ADDI's 14-bit immediate covers raw values -8192..+8191, representing -64.0..+63.9921875 in Q28.7 steps of 1/128. This is sufficient for filter coefficients.

### Setup

```
; === FIR FILTER SETUP ===
; Create coefficient array p4 = Array(RW, 0x300, 0x304).

; 0x090: ADDI r1, r0, 0x30       ; r1 = U.Int(0x30) — scratch
; 0x091: ADDI r2, r0, 0x300      ; r2 = U.Int(0x300) — coeff base
; 0x092: ADDI r3, r0, 0x304      ; r3 = U.Int(0x304) — coeff end
; 0x093: KSWI r1, r2, 0          ; mem[0x30] = U.Int(0x300)
; 0x094: KSWI r1, r3, 1          ; mem[0x31] = U.Int(0x304)
; 0x095: MKPTRI p4, r1, 0, 0b11  ; p4 = Array(RW, 0x300, 0x304)
```
| After | p4 = Array(RW, 0x300, 0x304) |

```
; Create sample array p5 = Array(RW, 0x310, 0x318).
; 0x096: ADDI r1, r0, 0x32       ; r1 = U.Int(0x32) — different scratch
; 0x097: ADDI r2, r0, 0x310
; 0x098: ADDI r3, r0, 0x318
; 0x099: KSWI r1, r2, 0          ; mem[0x32] = U.Int(0x310)
; 0x09A: KSWI r1, r3, 1          ; mem[0x33] = U.Int(0x318)
; 0x09B: MKPTRI p5, r1, 0, 0b11  ; p5 = Array(RW, 0x310, 0x318)

; Create output slot p6 = Array(RW, 0x320, 0x321).
; 0x09C: ADDI r1, r0, 0x34
; 0x09D: ADDI r2, r0, 0x320
; 0x09E: ADDI r3, r0, 0x321
; 0x09F: KSWI r1, r2, 0          ; mem[0x34] = U.Int(0x320)
; 0x0A0: KSWI r1, r3, 1          ; mem[0x35] = U.Int(0x321)
; 0x0A1: MKPTRI p6, r1, 0, 0b11  ; p6 = Array(RW, 0x320, 0x321)
```

```
; --- Initialize coefficients as Q28.7 ---
; Coefficients: h[0]=0.25, h[1]=0.5, h[2]=0.5, h[3]=0.25
; In Q28.7: 0.25 = 0.25 * 128 = 32, 0.5 = 0.5 * 128 = 64
;
; First, convert coefficient memory Free → Q28.7 via MCONV.
; 0x0A2: MCONV p4, 0, Q28.7      ; mem[0x300]: Free(0) → Q28.7(0) [Zero]
; 0x0A3: MCONV p4, 1, Q28.7      ; mem[0x301]: Free(0) → Q28.7(0) [Zero]
; 0x0A4: MCONV p4, 2, Q28.7      ; mem[0x302]: Free(0) → Q28.7(0) [Zero]
; 0x0A5: MCONV p4, 3, Q28.7      ; mem[0x303]: Free(0) → Q28.7(0) [Zero]

; Build Q28.7 coefficients in GPRs.
; First need a Q28.7(0) register as base.
; 0x0A6: CONVERT r1, r0, Q28.7   ; r1 = Q28.7(0.0)
```
| After | r1 = Q28.7(0.0) | raw data = 0 |
|-------|------------------|--------------|
| Check | U.Int(0) → Q28.7: Scale. 0 * 2^7 = 0. Q28.7(0.0). |

```
; 0x0A7: ADDI r2, r1, 32         ; r2 = Q28.7(0.25)
```
| After | r2 = Q28.7(0.25) | raw = 0 + 32 = 32. 32/128 = 0.25 ✓ |
|-------|-------------------|------|
| Note | ADDI imm=32 interpreted as Q28.7 raw value. 32/128 = 0.25. |

```
; 0x0A8: ADDI r3, r1, 64         ; r3 = Q28.7(0.5)
```
| After | r3 = Q28.7(0.5) | raw = 64. 64/128 = 0.5 ✓ |

```
; Store coefficients: h = [0.25, 0.5, 0.5, 0.25]
; 0x0A9: SWI p4, r2, 0           ; mem[0x300] = Q28.7(0.25)
```
| Before | M[0x300] = (Q28.7, 0) |
|--------|----------------------|
| After  | M[0x300] = (Q28.7, 32) |
| Check | Target Q28.7 (mutable) ✓. Source Q28.7 (mutable) ✓. |

```
; 0x0AA: SWI p4, r3, 1           ; mem[0x301] = Q28.7(0.5)
; 0x0AB: SWI p4, r3, 2           ; mem[0x302] = Q28.7(0.5)
; 0x0AC: SWI p4, r2, 3           ; mem[0x303] = Q28.7(0.25)
```

```
; --- Initialize samples as Q28.7 ---
; Samples: x = [1.0, 2.0, 3.0, 4.0, 0, 0, 0, 0]
; Q28.7(1.0) raw = 128, Q28.7(2.0) raw = 256, etc.
; These need larger values than ADDI from Q28.7(0).
; CONVERT U.Int(1) → Q28.7 via Scale: 1 * 2^7 = 128. ✓

; 0x0AD: ADDI r4, r0, 1          ; r4 = U.Int(1)
; 0x0AE: CONVERT r4, r4, Q28.7   ; r4 = Q28.7(1.0)
```
| After | r4 = Q28.7(1.0) | raw = 128 |
|-------|------------------|-----------|
| Check | U.Int(1) → Q28.7: Scale. 1 * 128 = 128. ✓ |

```
; 0x0AF: ADDI r5, r0, 2
; 0x0B0: CONVERT r5, r5, Q28.7   ; r5 = Q28.7(2.0) — raw = 256
; 0x0B1: ADDI r6, r0, 3
; 0x0B2: CONVERT r6, r6, Q28.7   ; r6 = Q28.7(3.0) — raw = 384
; 0x0B3: ADDI r7, r0, 4
; 0x0B4: CONVERT r7, r7, Q28.7   ; r7 = Q28.7(4.0) — raw = 512

; Convert sample memory and store.
; 0x0B5: MCONV p5, 0, Q28.7      ; mem[0x310]: Free → Q28.7 [Zero]
; 0x0B6: MCONV p5, 1, Q28.7
; 0x0B7: MCONV p5, 2, Q28.7
; 0x0B8: MCONV p5, 3, Q28.7
; (slots 4–7 left as Free for now — will MCONV when needed)
; 0x0B9: SWI p5, r4, 0           ; mem[0x310] = Q28.7(1.0)
; 0x0BA: SWI p5, r5, 1           ; mem[0x311] = Q28.7(2.0)
; 0x0BB: SWI p5, r6, 2           ; mem[0x312] = Q28.7(3.0)
; 0x0BC: SWI p5, r7, 3           ; mem[0x313] = Q28.7(4.0)
```

### FIR Computation

```
; === FIR FILTER: y = sum(h[k] * x[k]) for k=0..3 ===
; MAC: rd = rd + (rs1 * rs2). All three must be Q28.7.
; MUL auto-rescales: product >> 7.
;
; Initialize accumulator to Q28.7(0).

; 0x0BD: CONVERT r8, r0, Q28.7   ; r8 = Q28.7(0.0) — accumulator
```
| After | r8 = Q28.7(0.0) | raw = 0 |

```
; Tap 0: acc += h[0] * x[0]
; 0x0BE: LWI r9, p4, 0           ; r9 = Q28.7(0.25) — h[0]
; 0x0BF: LWI r10, p5, 0          ; r10 = Q28.7(1.0) — x[0]
; 0x0C0: MAC r8, r9, r10         ; r8 = Q28.7(0) + Q28.7(0.25) * Q28.7(1.0)
```
| MUL step | 0.25 * 1.0 = 0.25. Raw: 32 * 128 = 4096. Auto-rescale >> 7 = 32. Q28.7(0.25). |
|----------|-----------|
| ADD step | 0 + 0.25 = 0.25. Raw: 0 + 32 = 32. |
| After | r8 = Q28.7(0.25) | raw = 32 |
| Check | All three (r8, r9, r10) are Q28.7 ✓. No saturation. |

```
; Tap 1: acc += h[1] * x[1]
; 0x0C1: LWI r9, p4, 1           ; r9 = Q28.7(0.5) — h[1]
; 0x0C2: LWI r10, p5, 1          ; r10 = Q28.7(2.0) — x[1]
; 0x0C3: MAC r8, r9, r10         ; r8 += 0.5 * 2.0 = 1.0
```
| MUL step | 0.5 * 2.0 = 1.0. Raw: 64 * 256 = 16384 >> 7 = 128. |
|----------|-------------|
| ADD step | 0.25 + 1.0 = 1.25. Raw: 32 + 128 = 160. |
| After | r8 = Q28.7(1.25) | raw = 160 |

```
; Tap 2: acc += h[2] * x[2]
; 0x0C4: LWI r9, p4, 2           ; r9 = Q28.7(0.5) — h[2]
; 0x0C5: LWI r10, p5, 2          ; r10 = Q28.7(3.0) — x[2]
; 0x0C6: MAC r8, r9, r10         ; r8 += 0.5 * 3.0 = 1.5
```
| MUL step | 0.5 * 3.0 = 1.5. Raw: 64 * 384 = 24576 >> 7 = 192. |
|----------|-------------|
| ADD step | 1.25 + 1.5 = 2.75. Raw: 160 + 192 = 352. |
| After | r8 = Q28.7(2.75) | raw = 352 |

```
; Tap 3: acc += h[3] * x[3]
; 0x0C7: LWI r9, p4, 3           ; r9 = Q28.7(0.25) — h[3]
; 0x0C8: LWI r10, p5, 3          ; r10 = Q28.7(4.0) — x[3]
; 0x0C9: MAC r8, r9, r10         ; r8 += 0.25 * 4.0 = 1.0
```
| MUL step | 0.25 * 4.0 = 1.0. Raw: 32 * 512 = 16384 >> 7 = 128. |
|----------|-------------|
| ADD step | 2.75 + 1.0 = 3.75. Raw: 352 + 128 = 480. |
| After | r8 = Q28.7(3.75) | raw = 480 |

```
; Store result.
; 0x0CA: MCONV p6, 0, Q28.7      ; mem[0x320]: Free → Q28.7 [Zero]
; 0x0CB: SWI p6, r8, 0           ; mem[0x320] = Q28.7(3.75)
```
| After | M[0x320] = (Q28.7, 480) | 480/128 = 3.75 ✓ |

```
; Verify: y = 0.25*1 + 0.5*2 + 0.5*3 + 0.25*4 = 0.25 + 1.0 + 1.5 + 1.0 = 3.75 ✓

; 0x0CC: JAL r0, 0               ; halt
```

### Saturation Demonstration

```
; --- Show what happens when MAC overflows ---
; Q28.7 max positive: raw = 2^34 - 1 = 17179869183. Value = 134217727.9921875.
; If we accumulate huge values, saturation kicks in.

; 0x0D0: LUI r11, 0x7FFFF        ; r11 = U.Int(0x7FFFF << 14) = U.Int(0x1FFFFC000)
```
| After | r11 = U.Int(0x1FFFFC000) | Note: LUI always U.Int |

```
; 0x0D1: CONVERT r11, r11, Q28.7 ; Scale: huge value, will saturate
```
| After | r11 = Q28.7(MAX) | Saturates to max Q28.7. Saturation Hit flag set. |

```
; 0x0D2: RDSR r12                ; r12 = U.Int(status)
```
| After | r12 = U.Int(0b...10) | Bit 1 (Saturation Hit) = 1 |
| **No way to clear this flag. All subsequent RDSR reads will show it set.** |

```
; 0x0D3: JAL r0, 0               ; halt
```

### Issues

| ID | Severity | Issue | Fix Proposal |
|----|----------|-------|-------------|
| 4.1 | Critical | **No practical way to construct fractional Q18.17 or Q7.28 constants.** Q18.17 ADDI range is ±0.0625. SLL/SRL trap on fixed-point. MCONV can't do Scale (U.Int→Q18.17 traps). The only pre-fix workaround was the Free→Inert→U.Int bypass (now closed). Q28.7 works because ADDI covers ±64.0. | Add a BITCAST instruction (force-retag preserving data bits, no conversion logic) or extend MCONV to support a "force retag" mode for kernel. |
| 4.2 | Significant | **Sticky saturation flag cannot be cleared.** RDSR is read-only (no WRSR). Once Saturation Hit is set, it stays forever. DSP code cannot detect per-operation saturation or reset between filter iterations. | Add WRSR instruction or clear-on-read semantics for sticky flags. |
| 4.3 | Significant | **MCONV cannot convert between non-Free data types** if the matrix entry isn't Retag or Zero. U.Int → Q28.7 is Scale → MCONV traps. Only path is: U.Int → Free (Retag) → Q28.7 (Zero, data cleared). This destroys the data. | Same as 4.1 — need BITCAST or kernel force-retag. |
| 4.4 | Minor | **Memory must be MCONV'd before SWI if types differ.** Storing Q28.7 to Free-tagged memory works (SWI writes full word including tag). But if memory was previously S.Int and we want Q28.7, SWI overwrites both tag and data — this actually works fine. The target tag just needs to be mutable. | Not a real issue — SWI writes full word. No MCONV needed before SWI to mutable memory. |

---

## Program 5: Typed Memcpy with MTAG Dispatch

**Purpose**: Copy a heterogeneous struct containing U.Int fields and a spilled array pointer, using MTAG to detect each word's type and dispatch to the correct copy path.

**Pre-conditions**: Boot prologue completed.
- p7 = Array(RW, 0x400, 0x406) — source struct (6 words)
- p8 = Array(RW, 0x410, 0x416) — destination struct (6 words, all Free)

Source struct layout at 0x400–0x405:
| Offset | Content | Tag |
|--------|---------|-----|
| 0 | U.Int(100) | U.Int |
| 1 | U.Int(200) | U.Int |
| 2 | PtrA(RW, 0x500) | PtrA |
| 3 | PtrE(0x510) | PtrE |
| 4 | U.Int(300) | U.Int |
| 5 | Bool(1) | Bool |

### Setup

```
; === TYPED MEMCPY SETUP ===
; Create source and destination pointers.

; 0x0E0: ADDI r1, r0, 0x40       ; scratch for p7
; 0x0E1: ADDI r2, r0, 0x400
; 0x0E2: ADDI r3, r0, 0x406
; 0x0E3: KSWI r1, r2, 0
; 0x0E4: KSWI r1, r3, 1
; 0x0E5: MKPTRI p7, r1, 0, 0b11  ; p7 = Array(RW, 0x400, 0x406)

; 0x0E6: ADDI r1, r0, 0x42       ; scratch for p8
; 0x0E7: ADDI r2, r0, 0x410
; 0x0E8: ADDI r3, r0, 0x416
; 0x0E9: KSWI r1, r2, 0
; 0x0EA: KSWI r1, r3, 1
; 0x0EB: MKPTRI p8, r1, 0, 0b11  ; p8 = Array(RW, 0x410, 0x416)

; Populate source struct.
; 0x0EC: ADDI r1, r0, 100
; 0x0ED: SWI p7, r1, 0           ; M[0x400] = U.Int(100)
; 0x0EE: ADDI r1, r0, 200
; 0x0EF: SWI p7, r1, 1           ; M[0x401] = U.Int(200)

; Write a pointer into the struct at offset 2–3.
; Need to create an array pointer, spill it, then it occupies 2 words.
; Create a temp pointer p9 = Array(RW, 0x500, 0x510).
; 0x0F0: ADDI r1, r0, 0x44       ; scratch
; 0x0F1: ADDI r2, r0, 0x500
; 0x0F2: ADDI r3, r0, 0x510
; 0x0F3: KSWI r1, r2, 0
; 0x0F4: KSWI r1, r3, 1
; 0x0F5: MKPTRI p9, r1, 0, 0b11  ; p9 = Array(RW, 0x500, 0x510)

; Spill p9 to source struct offset 2.
; First ensure M[0x402..0x403] are mutable (Free by default ✓).
; 0x0F6: ADDI r1, r0, 2
; 0x0F7: SPRP p7, p9, r1         ; mem[0x402] = PtrA(RW, 0x500), mem[0x403] = PtrE(0x510)
```
| After | M[0x402] = (PtrA, RW=1, 0x500), M[0x403] = (PtrE, 0x510) |
| Note | 2-word atomic write for array pointer. |

```
; 0x0F8: ADDI r1, r0, 300
; 0x0F9: SWI p7, r1, 4           ; M[0x404] = U.Int(300)

; Store Bool(1) at offset 5.
; 0x0FA: ADDI r1, r0, 1
; 0x0FB: CONVERT r1, r1, Bool    ; r1 = Bool(1) — U.Int(1) → Bool: ≠0 → true
; 0x0FC: SWI p7, r1, 5           ; M[0x405] = Bool(1)
```
| Check | Source tag = Bool (mutable) ✓. |

### Memcpy Loop

```
; === TYPED MEMCPY ===
; Loop over 6 words. For each word:
;   MTAG to read tag → dispatch:
;     PtrA (9): LPRP + SPRP (copies 2 words), advance index by 2
;     PtrE (10): skip (already copied as part of PtrA)
;     Other: LWP + SWP (copy 1 word), advance index by 1
;
; r15 = loop index (U.Int), r16 = limit (6)

; 0x100: ADDI r15, r0, 0         ; r15 = U.Int(0) — index
; 0x101: ADDI r16, r0, 6         ; r16 = U.Int(6) — limit

loop_top:
; 0x102: BGE r15, r16, loop_done ; if index >= 6, exit loop
```
| Check | Both U.Int ✓. |

```
; Read the tag of the current source word.
; 0x103: MTAGR r17, p7, r15      ; r17 = U.Int(tag(mem[p7.base + r15]))
```
| Iteration 0 | r17 = U.Int(1) | tag(M[0x400]) = U.Int = 1 |
|-------------|----------------|------------|

```
; Check if PtrA (tag 9).
; 0x104: ADDI r18, r0, 9         ; r18 = U.Int(9) — PtrA tag value
; 0x105: BEQ r17, r18, copy_ptr  ; if tag == PtrA, handle pointer copy
```
| Iteration 0 | 1 ≠ 9 → not taken |

```
; Check if PtrE (tag 10) — skip it.
; 0x106: ADDI r18, r0, 10        ; r18 = U.Int(10) — PtrE tag value
; 0x107: BEQ r17, r18, skip_ptre ; if tag == PtrE, skip

; Default path: copy data word via LWP/SWP.
; 0x108: LWP r19, p7, r15        ; r19 = mem[src + index]
```
| Iteration 0 | r19 = U.Int(100) | Loaded from M[0x400] |
| Check | Bounds ✓. Tag U.Int (not pointer) ✓. |

```
; 0x109: SWP p8, r19, r15        ; mem[dst + index] = r19
```
| Iteration 0 | M[0x410] = (U.Int, 100) |
| Before | M[0x410] = (Free, 0x0) |
| Check | Target Free (mutable) ✓. Source U.Int (mutable) ✓. |

```
; 0x10A: ADDI r15, r15, 1        ; index++
; 0x10B: JAL r0, loop_top        ; back to top (PC-relative: loop_top - 0x10C)

copy_ptr:
; Copy array pointer: LPRP + SPRP (2 words atomically).
; 0x10C: LPRP p10, p7, r15       ; p10 = pointer from mem[src + index]
```
| Iteration 2 | p10 = Array(RW, 0x500, 0x510) |
| Check | M[0x402] tag = PtrA ✓. Loads PtrA + PtrE (2 words). |

```
; 0x10D: SPRP p8, p10, r15       ; mem[dst + index] = p10 (2 words)
```
| Iteration 2 | M[0x412] = (PtrA, RW=1, 0x500), M[0x413] = (PtrE, 0x510) |
| Before | M[0x412] = (Free, 0), M[0x413] = (Free, 0) |
| Check | Both destinations Free (mutable) ✓. RW on p8 ✓. |

```
; Advance index by 2 (PtrA + PtrE already copied).
; 0x10E: ADDI r15, r15, 2
; 0x10F: JAL r0, loop_top

skip_ptre:
; PtrE at an unexpected position (orphaned). Skip it.
; 0x110: ADDI r15, r15, 1
; 0x111: JAL r0, loop_top

loop_done:
; 0x112: JAL r0, 0               ; halt
```

### Execution Trace (All Iterations)

| Iter | Index | Src Addr | Src Tag | Action | Dst Addr | Dst After |
|------|-------|----------|---------|--------|----------|-----------|
| 0 | 0 | 0x400 | U.Int | LWP+SWP | 0x410 | U.Int(100) |
| 1 | 1 | 0x401 | U.Int | LWP+SWP | 0x411 | U.Int(200) |
| 2 | 2 | 0x402 | PtrA | LPRP+SPRP | 0x412-3 | PtrA(0x500)+PtrE(0x510) |
| 3 | 4 | 0x404 | U.Int | LWP+SWP | 0x414 | U.Int(300) |
| 4 | 5 | 0x405 | Bool | LWP+SWP | 0x415 | Bool(1) |
| 5 | 6 | — | — | BGE exits | — | — |

Note: Index jumps from 2→4 because the PtrA handler advances by 2. The PtrE at offset 3 is never visited (correct — it was copied as part of the PtrA).

### Issues

| ID | Severity | Issue | Fix Proposal |
|----|----------|-------|-------------|
| 5.1 | Significant | **BTAG is useless for tag-value dispatch.** BTAG checks `tag(rs1) == tag_literal`. Since MTAG returns U.Int, BTAG on the MTAG result always matches U.Int (the container tag), not the tag value. Must use BEQ against U.Int constants instead. | BTAG serves a different purpose (checking a register's own type). For memory tag dispatch, MTAG + BEQ is the correct pattern. Consider a combined BTAG-memory instruction for future versions. |
| 5.2 | Significant | **Copying pointer-tagged words to destination makes those slots immutable.** After SPRP writes PtrA/PtrE to destination, those slots can't be overwritten by SWP. If the destination struct needs to be updated later, MCONV cleanup is required first. | This is inherent to the type system. The destination mirrors the source's type structure. |
| 5.3 | Minor | **LWP on Bool-tagged memory works fine.** Bool is not a pointer type, so LWP succeeds. SWP stores Bool(1) to destination, overwriting Free. This demonstrates that SWI/SWP write the full word including tag. | Not an issue — working as designed. |
| 5.4 | Minor | **No LPRP through singleton for singleton pointers.** If the struct contained a PtrS instead of PtrA, the copy would still work: LPRP reads 1 word (PtrS), SPRP writes 1 word. The copy logic would need to check for PtrS (tag 8) separately from PtrA (tag 9). | Add PtrS handling in the dispatch (another BEQ branch). |

---

## Program 6: PID Controller in Q28.7

**Purpose**: Implement a PID control loop reading a sensor and writing an actuator through separate bounded singleton pointers, exercising fixed-point arithmetic and pointer isolation.

**Pre-conditions**: Boot prologue completed. Kernel bit = 1. We set up:
- p2 = Singleton(RO, 0x600) — sensor input (Q28.7)
- p3 = Singleton(RW, 0x601) — actuator output (Q28.7)
- p4 = Array(RW, 0x610, 0x614) — PID state: [integral, prev_error, setpoint, unused]

### Setup

```
; === PID CONTROLLER SETUP ===

; Create sensor pointer: Singleton(RO, 0x600).
; Need 1 U.Int word at scratch with the target address.
; 0x120: ADDI r1, r0, 0x50       ; scratch
; 0x121: ADDI r2, r0, 0x600      ; sensor addr
; 0x122: KSWI r1, r2, 0          ; mem[0x50] = U.Int(0x600)
; 0x123: MKPTRI p2, r1, 0, 0b00  ; subop=0b00: singleton(0), RO(0)
;                                 ; p2 = Singleton(RO, 0x600)
```
| After | p2 = Singleton(RO, 0x600) |
|-------|---------------------------|
| | M[0x50] = (PtrS, RO, 0x600) — retagged |

```
; Create actuator pointer: Singleton(RW, 0x601).
; 0x124: ADDI r1, r0, 0x52       ; different scratch
; 0x125: ADDI r2, r0, 0x601
; 0x126: KSWI r1, r2, 0          ; mem[0x52] = U.Int(0x601)
; 0x127: MKPTRI p3, r1, 0, 0b10  ; subop=0b10: singleton(0), RW(1)
;                                 ; p3 = Singleton(RW, 0x601)

; Create PID state array: Array(RW, 0x610, 0x614).
; 0x128: ADDI r1, r0, 0x54
; 0x129: ADDI r2, r0, 0x610
; 0x12A: ADDI r3, r0, 0x614
; 0x12B: KSWI r1, r2, 0
; 0x12C: KSWI r1, r3, 1
; 0x12D: MKPTRI p4, r1, 0, 0b11  ; p4 = Array(RW, 0x610, 0x614)

; Initialize PID state to Q28.7(0).
; 0x12E: MCONV p4, 0, Q28.7      ; integral = Q28.7(0)
; 0x12F: MCONV p4, 1, Q28.7      ; prev_error = Q28.7(0)
; 0x130: MCONV p4, 2, Q28.7      ; setpoint slot

; Set setpoint = Q28.7(10.0). 10 * 128 = 1280 raw.
; 0x131: ADDI r1, r0, 10
; 0x132: CONVERT r1, r1, Q28.7   ; r1 = Q28.7(10.0) — raw 1280
; 0x133: SWI p4, r1, 2           ; mem[0x612] = Q28.7(10.0)

; Set PID gains in GPRs (persistent across loop).
; Kp = Q28.7(2.0), Ki = Q28.7(0.5), Kd = Q28.7(0.25)
; 0x134: ADDI r1, r0, 2
; 0x135: CONVERT r11, r1, Q28.7  ; r11 = Q28.7(2.0) — Kp. raw = 256
; 0x136: CONVERT r1, r0, Q28.7   ; r1 = Q28.7(0.0)
; 0x137: ADDI r12, r1, 64        ; r12 = Q28.7(0.5) — Ki. raw = 64
; 0x138: ADDI r13, r1, 32        ; r13 = Q28.7(0.25) — Kd. raw = 32

; Prepare simulated sensor values at M[0x600].
; The "sensor" is memory-mapped I/O. We preload a value for testing.
; 0x139: MCONVI r0, 0x600, Q28.7 ; mem[0x600]: Free → Q28.7 [Zero] (kernel, GPR addr)
```
| Check | MCONVI: kernel ✓. GPR r0=U.Int(0), offset computed as 0+0x600. Wait — MCONVI encoding: rs_base is r0 = U.Int(0), imm is 7 bits. 0x600 = 1536, exceeds 7-bit range (-64..+63). |
| **PROBLEM**: Cannot reach address 0x600 with MCONVI from r0 and 7-bit immediate. |

```
; WORKAROUND: Use a GPR with the target address as base.
; 0x139: ADDI r1, r0, 0x600      ; r1 = U.Int(0x600) — within 14-bit ADDI range ✓ (1536 < 8192)
; 0x13A: MCONVI r1, 0, Q28.7     ; mem[r1 + 0] = mem[0x600]: Free → Q28.7 [Zero]
; 0x13B: ADDI r2, r0, 7
; 0x13C: CONVERT r2, r2, Q28.7   ; r2 = Q28.7(7.0) — simulated sensor reading
; 0x13D: KSWI r1, r2, 0          ; mem[0x600] = Q28.7(7.0)
```
| After | M[0x600] = (Q28.7, 896) | 7.0 * 128 = 896 |

```
; Initialize actuator output memory.
; 0x13E: ADDI r1, r0, 0x601
; 0x13F: MCONVI r1, 0, Q28.7     ; mem[0x601]: Free → Q28.7 [Zero]
```

### PID Loop (3 iterations)

```
; === PID LOOP ===
; Register allocation:
;   r11 = Kp = Q28.7(2.0)
;   r12 = Ki = Q28.7(0.5)
;   r13 = Kd = Q28.7(0.25)
;   r14 = loop counter (U.Int)
;   r15 = loop limit (U.Int)
;   r1–r10 = temporaries (Q28.7 or U.Int as needed)

; 0x140: ADDI r14, r0, 0         ; r14 = U.Int(0) — counter
; 0x141: ADDI r15, r0, 3         ; r15 = U.Int(3) — 3 iterations

pid_top:
; 0x142: BGE r14, r15, pid_done  ; exit if counter >= 3

; Read sensor via singleton pointer (offset must be 0).
; 0x143: LWI r1, p2, 0           ; r1 = Q28.7(sensor_value) = Q28.7(7.0)
```
| Check | Bounds: singleton target 0x600. Offset 0 → addr 0x600 ∈ {0x600} ✓. Tag Q28.7 (not pointer) ✓. |
| Iter 0 | r1 = Q28.7(7.0) | raw = 896 |

```
; Read setpoint.
; 0x144: LWI r2, p4, 2           ; r2 = Q28.7(10.0) — setpoint

; Compute error = setpoint - sensor.
; 0x145: SUB r3, r2, r1          ; r3 = Q28.7(10.0 - 7.0) = Q28.7(3.0)
```
| Iter 0 | r3 = Q28.7(3.0) | raw = 1280 - 896 = 384 |
| Check | Both Q28.7 ✓. |

```
; Read integral and prev_error from state.
; 0x146: LWI r4, p4, 0           ; r4 = Q28.7(integral)
; 0x147: LWI r5, p4, 1           ; r5 = Q28.7(prev_error)
```
| Iter 0 | r4 = Q28.7(0.0), r5 = Q28.7(0.0) |

```
; Update integral: integral += error.
; 0x148: ADD r4, r4, r3          ; r4 = integral + error
```
| Iter 0 | r4 = Q28.7(0.0 + 3.0) = Q28.7(3.0) | raw = 0 + 384 = 384 |

```
; Compute derivative: derivative = error - prev_error.
; 0x149: SUB r6, r3, r5          ; r6 = error - prev_error
```
| Iter 0 | r6 = Q28.7(3.0 - 0.0) = Q28.7(3.0) | raw = 384 |

```
; Compute output = Kp * error + Ki * integral + Kd * derivative.
; Use MUL + ADD (not MAC, to show the separate steps).

; 0x14A: MUL r7, r11, r3         ; r7 = Kp * error = 2.0 * 3.0 = 6.0
```
| Iter 0 | r7 = Q28.7(6.0) | raw: 256*384=98304, >>7 = 768. 768/128=6.0 ✓ |

```
; 0x14B: MUL r8, r12, r4         ; r8 = Ki * integral = 0.5 * 3.0 = 1.5
```
| Iter 0 | r8 = Q28.7(1.5) | raw: 64*384=24576, >>7 = 192. 192/128=1.5 ✓ |

```
; 0x14C: MUL r9, r13, r6         ; r9 = Kd * derivative = 0.25 * 3.0 = 0.75
```
| Iter 0 | r9 = Q28.7(0.75) | raw: 32*384=12288, >>7 = 96. 96/128=0.75 ✓ |

```
; 0x14D: ADD r10, r7, r8         ; r10 = P + I = 6.0 + 1.5 = 7.5
; 0x14E: ADD r10, r10, r9        ; r10 = P + I + D = 7.5 + 0.75 = 8.25
```
| Iter 0 | r10 = Q28.7(8.25) | raw = 768 + 192 + 96 = 1056. 1056/128 = 8.25 ✓ |

```
; Write output to actuator via singleton pointer.
; 0x14F: SWI p3, r10, 0          ; mem[0x601] = Q28.7(8.25)
```
| Before | M[0x601] = (Q28.7, 0) |
|--------|----------------------|
| After  | M[0x601] = (Q28.7, 1056) |
| Check | Singleton offset 0 ✓. RW ✓. Target Q28.7 (mutable) ✓. Source Q28.7 (mutable) ✓. |

```
; Save updated state.
; 0x150: SWI p4, r4, 0           ; integral = Q28.7(3.0)
; 0x151: SWI p4, r3, 1           ; prev_error = error = Q28.7(3.0)

; Increment loop counter.
; 0x152: ADDI r14, r14, 1
; 0x153: JAL r0, pid_top         ; back to loop top

pid_done:
; 0x154: JAL r0, 0               ; halt
```

### Full Iteration Trace

| Iter | Sensor | Error | Integral | Derivative | P | I | D | Output |
|------|--------|-------|----------|------------|---|---|---|--------|
| 0 | 7.0 | 3.0 | 3.0 | 3.0 | 6.0 | 1.5 | 0.75 | 8.25 |
| 1 | 7.0 | 3.0 | 6.0 | 0.0 | 6.0 | 3.0 | 0.0 | 9.0 |
| 2 | 7.0 | 3.0 | 9.0 | 0.0 | 6.0 | 4.5 | 0.0 | 10.5 |

Note: Sensor is constant (7.0) in this test. In real use, the sensor would change based on actuator feedback.

### Issues

| ID | Severity | Issue | Fix Proposal |
|----|----------|-------|-------------|
| 6.1 | Significant | **No DIV instruction.** PID controllers often need dt scaling (output *= dt). Without DIV, the only option is to pre-scale gains by 1/dt before the loop. For dt=0.01 in Q28.7, raw = 1.28 ≈ 1 (coarse). | Add DIV or reciprocal instruction in V1. |
| 6.2 | Significant | **Cannot clear saturation flag between iterations.** If the integral saturates on iteration N, the saturation flag stays set. The controller can't distinguish old saturation from new. | Add WRSR or per-iteration flag clear. |
| 6.3 | Minor | **Singleton pointer offset must be 0.** LWI p2, r1, 1 would trap (out of bounds). This forces memory-mapped I/O to use one pointer per register, increasing pointer register pressure. | Accept — this is the design intent for singletons. |
| 6.4 | Minor | **MCONVI 7-bit immediate cannot reach far addresses.** Had to use GPR base with ADDI to reach 0x600. MCONVI's 7-bit imm is the most limiting of all immediate fields. | Widen MCONVI imm or accept that kernel code uses GPR bases. |
| 6.5 | Minor | **Loop counter (U.Int) and PID values (Q28.7) in different registers.** No type confusion risk — they never interact. But register pressure is high: 3 gains + counter + limit + 6 temporaries = 11 GPRs. Only 12 remaining (r11–r22 minus r22 if frame pointer). | Adequate for V0. |

---

## Program 7: Bubble Sort on U.Int Array

**Purpose**: Sort a U.Int array using bubble sort, including the empty-array edge case, exercising comparison branches and the swap pattern.

**Pre-conditions**: Boot prologue completed. Kernel bit = 1.
- p5 = Array(RW, 0x500, 0x505) — 5-element U.Int array
- Also test p6 = Array(RW, 0x510, 0x510) — 0-element array (base == end)

### Setup

```
; === BUBBLE SORT SETUP ===

; Create 5-element array.
; 0x160: ADDI r1, r0, 0x60       ; scratch
; 0x161: ADDI r2, r0, 0x500
; 0x162: ADDI r3, r0, 0x505
; 0x163: KSWI r1, r2, 0
; 0x164: KSWI r1, r3, 1
; 0x165: MKPTRI p5, r1, 0, 0b11  ; p5 = Array(RW, 0x500, 0x505)

; Initialize array: [5, 3, 1, 4, 2] (unsorted).
; 0x166: ADDI r1, r0, 5
; 0x167: SWI p5, r1, 0           ; M[0x500] = U.Int(5)
; 0x168: ADDI r1, r0, 3
; 0x169: SWI p5, r1, 1           ; M[0x501] = U.Int(3)
; 0x16A: ADDI r1, r0, 1
; 0x16B: SWI p5, r1, 2           ; M[0x502] = U.Int(1)
; 0x16C: ADDI r1, r0, 4
; 0x16D: SWI p5, r1, 3           ; M[0x503] = U.Int(4)
; 0x16E: ADDI r1, r0, 2
; 0x16F: SWI p5, r1, 4           ; M[0x504] = U.Int(2)

; Create empty array (0 elements).
; 0x170: ADDI r1, r0, 0x62       ; scratch
; 0x171: ADDI r2, r0, 0x510
; 0x172: ADDI r3, r0, 0x510      ; end == base → 0 elements
; 0x173: KSWI r1, r2, 0
; 0x174: KSWI r1, r3, 1
; 0x175: MKPTRI p6, r1, 0, 0b11  ; p6 = Array(RW, 0x510, 0x510)
```
| After | p6 = Array(RW, 0x510, 0x510) | PLEN = 0x510 - 0x510 = 0 |

### Empty Array Edge Case

```
; --- Test empty array ---
; 0x176: PLEN r1, p6             ; r1 = U.Int(0) — length of empty array
```
| After | r1 = U.Int(0) |
| Check | p6 is array ✓. end - base = 0. |

```
; 0x177: BEQ r1, r0, skip_sort   ; if length == 0, skip sort
```
| Check | Both U.Int ✓. 0 == 0 → taken ✓. Jumps to skip_sort. |
| **Empty array handled correctly — no access, no trap.** |

### Bubble Sort on 5-Element Array

```
; --- Sort p5 (5 elements) ---
; 0x178: PLEN r1, p5             ; r1 = U.Int(5) — length
; 0x179: BEQ r1, r0, skip_sort   ; 5 ≠ 0 → not taken

; Bubble sort: outer loop i from n-1 down to 1.
; Inner loop j from 0 to i-1.
; If arr[j] > arr[j+1], swap.

; r1 = n = 5 (U.Int)
; r2 = i (outer loop counter, starts at n-1 = 4)
; r3 = j (inner loop counter)
; r4 = j+1
; r5 = arr[j]
; r6 = arr[j+1]

; 0x17A: ADDI r2, r1, -1         ; r2 = U.Int(4) — i = n-1

outer_loop:
; 0x17B: BEQ r2, r0, sort_done   ; if i == 0, done
; 0x17C: ADDI r3, r0, 0          ; r3 = U.Int(0) — j = 0

inner_loop:
; 0x17D: BGE r3, r2, outer_dec   ; if j >= i, go to outer decrement

; Load arr[j] and arr[j+1].
; 0x17E: LWP r5, p5, r3          ; r5 = arr[j]
; 0x17F: ADDI r4, r3, 1          ; r4 = U.Int(j+1)
; 0x180: LWP r6, p5, r4          ; r6 = arr[j+1]

; Compare: if arr[j] > arr[j+1], swap.
; BLT checks rs1 < rs2. We want to swap if arr[j] > arr[j+1],
; i.e., if NOT (arr[j] <= arr[j+1]).
; Use BGE r5, r6 to check arr[j] >= arr[j+1]... but that includes equal.
; For stability, only swap if arr[j] > arr[j+1].
; There is no BGT instruction. Workaround: BEQ to skip, then BGE.
; Or: swap when arr[j+1] < arr[j] → BLT r6, r5, do_swap.

; 0x181: BLT r6, r5, do_swap     ; if arr[j+1] < arr[j], swap
; 0x182: JAL r0, no_swap         ; skip swap

do_swap:
; Swap arr[j] and arr[j+1].
; Both values already in r5 and r6.
; 0x183: SWP p5, r6, r3          ; arr[j] = old arr[j+1]
; 0x184: SWP p5, r5, r4          ; arr[j+1] = old arr[j]
```
| Check | Both SWP: target is U.Int (mutable) ✓. Source U.Int (mutable) ✓. RW ✓. Bounds ✓. |

```
no_swap:
; 0x185: ADDI r3, r3, 1          ; j++
; 0x186: JAL r0, inner_loop

outer_dec:
; 0x187: ADDI r2, r2, -1         ; i--
; 0x188: JAL r0, outer_loop

sort_done:
skip_sort:
; 0x189: JAL r0, 0               ; halt
```

### Execution Trace

**Pass 1 (i=4):**

| j | arr[j] | arr[j+1] | Swap? | Array after |
|---|--------|----------|-------|-------------|
| 0 | 5 | 3 | Yes (3<5) | [3, 5, 1, 4, 2] |
| 1 | 5 | 1 | Yes (1<5) | [3, 1, 5, 4, 2] |
| 2 | 5 | 4 | Yes (4<5) | [3, 1, 4, 5, 2] |
| 3 | 5 | 2 | Yes (2<5) | [3, 1, 4, 2, 5] |

**Pass 2 (i=3):**

| j | arr[j] | arr[j+1] | Swap? | Array after |
|---|--------|----------|-------|-------------|
| 0 | 3 | 1 | Yes | [1, 3, 4, 2, 5] |
| 1 | 3 | 4 | No | [1, 3, 4, 2, 5] |
| 2 | 4 | 2 | Yes | [1, 3, 2, 4, 5] |

**Pass 3 (i=2):**

| j | arr[j] | arr[j+1] | Swap? | Array after |
|---|--------|----------|-------|-------------|
| 0 | 1 | 3 | No | [1, 3, 2, 4, 5] |
| 1 | 3 | 2 | Yes | [1, 2, 3, 4, 5] |

**Pass 4 (i=1):**

| j | arr[j] | arr[j+1] | Swap? | Array after |
|---|--------|----------|-------|-------------|
| 0 | 1 | 2 | No | [1, 2, 3, 4, 5] |

**Final**: `[1, 2, 3, 4, 5]` — sorted ✓

### Memory State After Sort

| Addr | Before | After |
|------|--------|-------|
| 0x500 | U.Int(5) | U.Int(1) |
| 0x501 | U.Int(3) | U.Int(2) |
| 0x502 | U.Int(1) | U.Int(3) |
| 0x503 | U.Int(4) | U.Int(4) |
| 0x504 | U.Int(2) | U.Int(5) |

### Issues

| ID | Severity | Issue | Fix Proposal |
|----|----------|-------|-------------|
| 7.1 | Minor | **No SLT/SLTI instruction.** Cannot compute a comparison result into a register. All comparisons must be done via branches. This makes conditional expressions verbose (need branch + store patterns). | Add SLT rd, rs1, rs2: rd = U.Int(1) if rs1 < rs2, else U.Int(0). |
| 7.2 | Minor | **No BGT instruction.** Must use `BLT r6, r5` (reversed operands) to test arr[j] > arr[j+1]. This works but is unintuitive. | BGT is redundant with BLT (swap operands). Accept. |
| 7.3 | Minor | **No atomic swap.** Each swap requires 4 instructions (2 loads + 2 stores). Both values must be in GPRs before any store to avoid read-after-write hazard. | Accept for V0. Add XCHG in future if needed. |
| 7.4 | Minor | **PLEN correctly returns 0 for empty array.** base == end → PLEN = 0. BEQ catches it. No issues. | Working as designed. |

---

## Program 8: Security Audit — Attempted Attacks

**Purpose**: Systematically attempt every known attack path to forge a pointer, escalate permissions, leak freed data, or get pointer bits into a GPR. Every attempt should trap (or be defanged). Any success is a security model failure.

**Pre-conditions**: Boot prologue completed. Kernel bit = 1 (V0 hardwired). We set up:
- p2 = Array(RW, 0x700, 0x710) — general-purpose RW region
- p3 = Array(RO, 0x700, 0x710) — same region, read-only view (via PNARROW or MKPTR)
- M[0x720–0x721] = spilled pointer (PtrA + PtrE from SPRI)

### Setup

```
; === SECURITY AUDIT SETUP ===

; Create RW array pointer.
; 0x190: ADDI r1, r0, 0x70       ; scratch
; 0x191: ADDI r2, r0, 0x700
; 0x192: ADDI r3, r0, 0x710
; 0x193: KSWI r1, r2, 0
; 0x194: KSWI r1, r3, 1
; 0x195: MKPTRI p2, r1, 0, 0b11  ; p2 = Array(RW, 0x700, 0x710)

; Create RO view of same region.
; 0x196: ADDI r1, r0, 0x72
; 0x197: KSWI r1, r2, 0
; 0x198: KSWI r1, r3, 1
; 0x199: MKPTRI p3, r1, 0, 0b01  ; p3 = Array(RO, 0x700, 0x710) — subop: array(1), RO(0)

; Write some data to the region via p2 (RW).
; 0x19A: ADDI r1, r0, 0xDEAD
; 0x19B: SWI p2, r1, 0           ; M[0x700] = U.Int(0xDEAD)

; Create a spill area and spill a pointer.
; 0x19C: ADDI r1, r0, 0x74
; 0x19D: ADDI r2, r0, 0x720
; 0x19E: ADDI r3, r0, 0x724
; 0x19F: KSWI r1, r2, 0
; 0x1A0: KSWI r1, r3, 1
; 0x1A1: MKPTRI p4, r1, 0, 0b11  ; p4 = Array(RW, 0x720, 0x724)

; Spill p2 into the spill area.
; 0x1A2: SPRI p4, p2, 0          ; M[0x720] = PtrA(RW, 0x700), M[0x721] = PtrE(0x710)
```

### Attack 1: Forge a Pointer via MKPTR (User-Mode Test)

```
; In V0, kernel bit is hardwired to 1. MKPTR always succeeds.
; This attack CANNOT be tested in V0 — would require V1+ user mode.
; We document what SHOULD happen:
;
; In user mode:
;   ADDI r1, r0, 0x0            ; r1 = U.Int(0) — target address 0
;   KSWI r1, r1, 0              ; SHOULD TRAP: kernel-only instruction
;   MKPTRI p5, r1, 0, 0b10     ; SHOULD TRAP: kernel-only instruction
;
; Result in V0: NOT TESTABLE. Both instructions succeed because kernel=1.
```
| Verdict | **N/A** — V0 cannot test user-mode restriction. |

### Attack 2: Escalate RO → RW via PNARROW

```
; p3 = Array(RO, 0x700, 0x710) — read-only.
; Attempt to create an RW child via PNARROW.

; 0x1A3: ADDI r1, r0, 0          ; offset = 0
; 0x1A4: ADDI r2, r0, 8          ; length = 8

; PNARROW perm=0 means "inherit parent permissions."
; Parent is RO → child must be RO.
; 0x1A5: PNARROW p5, p3, r1, r2, perm=0
```
| After | p5 = Array(RO, 0x700, 0x708) |
|-------|-------------------------------|
| Check | Source is array ✓. Bounds ✓. perm=0 → inherit RO from parent. |
| **RO correctly inherited. No escalation.** |

```
; Try to write through the RO pointer.
; 0x1A6: ADDI r3, r0, 0x1234
; 0x1A7: SWI p5, r3, 0           ; TRAP — RW check fails (p5 is RO)
```
| **TRAP**: SWI check (2): pointer register RW bit = 0. Store requires RW. |
| Verdict | **TRAP ✓** — RO enforced, write blocked. |

```
; PNARROW perm=1 means "force RO." Already RO, so no change.
; 0x1A8: PNARROW p5, p3, r1, r2, perm=1
```
| After | p5 = Array(RO, 0x700, 0x708) | Still RO. |

```
; There is no perm value that produces RW from an RO parent.
```
| Verdict | **SECURE ✓** — No RO→RW escalation path exists. |

### Attack 3: Read Pointer Bits into GPR via LWI

```
; M[0x720] = PtrA(RW, 0x700) — spilled pointer data.
; Attempt to load it into a GPR to extract the address.

; 0x1A9: LWI r3, p4, 0           ; attempt load from M[0x720]
```
| **TRAP**: LWI check: target memory tag = PtrA. LWI traps on pointer-tagged memory (PtrS/PtrA/PtrE). |
| Verdict | **TRAP ✓** — Pointer bits blocked from entering GPR. |

### Attack 4: Create Pointer-Tagged Memory via User MCONV

```
; Attempt to retag U.Int data as PtrS via user-mode MCONV.
; M[0x700] = U.Int(0xDEAD). If we could retag to PtrS, we'd have
; a pointer to address 0xDEAD.

; 0x1AA: MCONV p2, 0, PtrS       ; target_tag = PtrS (8)
```
| **TRAP**: User-mode MCONV check: target_tag is PtrS (system type). MCONV/MCONVR trap on pointer target. |
| Verdict | **TRAP ✓** — User cannot create pointer-tagged memory. |

### Attack 5: Leak Freed Data via Free→Inert→U.Int (Fixed)

```
; Place a secret in memory, free it, then try to recover via the
; Free→Inert→U.Int conversion chain.

; 0x1AB: ADDI r1, r0, 0x5EC3     ; r1 = U.Int(0x5EC3) — "secret"
; 0x1AC: SWI p2, r1, 1           ; M[0x701] = U.Int(0x5EC3)
; 0x1AD: MCONV p2, 1, Free       ; M[0x701]: U.Int(0x5EC3) → Free(0x5EC3) [Retag, data preserved]
```
| After | M[0x701] = (Free, 0x5EC3) | Secret data still in memory as Free-tagged. |

```
; Load the freed memory into a GPR.
; 0x1AE: LWI r2, p2, 1           ; r2 = Free(0x5EC3)
```
| After | r2 = Free(0x5EC3) | LWI on Free memory succeeds. |

```
; Attempt the bypass: CONVERT Free → Inert.
; BEFORE FIX: Retag (data preserved) — ATTACK SUCCEEDS.
; AFTER FIX: Zero (data cleared) — attack defeated.

; 0x1AF: CONVERT r3, r2, Inert   ; r3 = Inert(0x0)
```
| After | r3 = Inert(0x0) | **Free→Inert = Zero. Data bits cleared.** |

```
; Continue the chain: Inert → U.Int.
; 0x1B0: CONVERT r4, r3, U.Int   ; r4 = U.Int(0x0) — Retag, but data already 0.
```
| After | r4 = U.Int(0x0) | The secret 0x5EC3 is NOT recoverable. |

```
; Verify the secret is gone.
; 0x1B1: BNE r4, r0, FAIL        ; U.Int(0) == U.Int(0) → not taken ✓
```
| Verdict | **SECURE ✓ (after fix)** — Free→Inert now zeros data. Bypass closed. |

```
; Also test Free → Instruction → U.Int path.
; 0x1B2: CONVERT r5, r2, Instruction  ; r5 = Instruction(0x0) — Free→Instruction = Zero
; 0x1B3: CONVERT r6, r5, U.Int        ; r6 = U.Int(0x0) — Retag on zero data
; 0x1B4: BNE r6, r0, FAIL             ; 0 == 0 → not taken ✓
```
| Verdict | **SECURE ✓** — Free→Instruction also zeros. Both bypass paths closed. |

### Attack 6: Write to Immutable Memory via SWI

```
; M[0x720] = PtrA (immutable system type from pointer spill).
; Attempt to overwrite it with SWI.

; 0x1B5: ADDI r1, r0, 0xBAD
; 0x1B6: SWI p4, r1, 0           ; attempt to write to M[0x720]
```
| **TRAP**: SWI check (3): target memory tag = PtrA (system type, immutable). SWI only writes to mutable types. |
| Verdict | **TRAP ✓** — Cannot overwrite pointer-tagged memory via SWI. |

### Attack 7: Write System-Tagged GPR Value to Memory (Fixed)

```
; Attempt to make memory immutable by storing an Inert-tagged GPR value.
; BEFORE FIX: SWI only checked target tag, not source tag. Attack succeeded.
; AFTER FIX: SWI traps if source GPR tag is a system type.

; 0x1B7: CONVERT r1, r0, Inert   ; r1 = Inert(0) — U.Int(0) → Inert via Retag
```
| After | r1 = Inert(0) | Successfully created Inert-tagged value in GPR. |

```
; Attempt to store it to mutable memory.
; 0x1B8: SWI p2, r1, 2           ; attempt: mem[0x702] = Inert(0)
```
| **TRAP**: SWI check (4): source GPR tag = Inert (system type). SWI traps on system-type source. |
| Verdict | **TRAP ✓ (after fix)** — Cannot create immutable memory via SWI. |

```
; Also test Instruction-tagged source.
; 0x1B9: CONVERT r1, r0, Instruction  ; r1 = Instruction(0)
; 0x1BA: SWI p2, r1, 2                ; attempt to store
```
| **TRAP**: SWI check (4): source GPR tag = Instruction (system type). |
| Verdict | **TRAP ✓** — Instruction-tagged source also blocked. |

### Attack 8: Out-of-Bounds Access via LWI/SWI

```
; p2 = Array(RW, 0x700, 0x710) — valid range [0x700, 0x710), 16 words.
; Attempt access at offset -1 (below base) and offset 16 (at end, exclusive).

; 0x1BB: LWI r1, p2, -1          ; addr = 0x700 + (-1) = 0x6FF
```
| **TRAP**: Bounds check: 0x6FF < 0x700 (below base). Out of bounds. |

```
; 0x1BC: LWI r1, p2, 16          ; addr = 0x700 + 16 = 0x710
```
| **TRAP**: Bounds check: 0x710 ≥ 0x710 (at or past end). End is exclusive. |

```
; Verify valid boundary accesses work.
; 0x1BD: LWI r1, p2, 0           ; addr = 0x700 — at base, valid
```
| Check | 0x700 ∈ [0x700, 0x710) ✓. |

```
; 0x1BE: LWI r1, p2, 15          ; addr = 0x70F — last valid word
```
| Check | 0x70F ∈ [0x700, 0x710) ✓. |

| Verdict | **TRAP ✓** — Bounds checking enforced at both boundaries. |

### Attack 9: Read Freed Pointer Data

```
; M[0x720] = PtrA(RW, 0x700) — spilled pointer with address bits.
; Free it via MCONV and check if address data is recoverable.

; 0x1BF: MCONV p4, 0, Free       ; M[0x720]: PtrA → Free
```
| Before | M[0x720] = (PtrA, RW=1, base=0x700) |
|--------|-------------------------------------|
| After  | M[0x720] = (Free, 0x0) |
| Check | User-mode MCONV: PtrA → Free = **Zero** (data bits cleared). Address 0x700 is gone. |

```
; Load the freed slot.
; 0x1C0: LWI r1, p4, 0           ; r1 = Free(0x0)
```
| After | r1 = Free(0x0) | Address data completely erased. |

```
; Even the Free→Inert path recovers nothing.
; 0x1C1: CONVERT r2, r1, Inert   ; r2 = Inert(0x0) — Zero from Free
; 0x1C2: CONVERT r3, r2, U.Int   ; r3 = U.Int(0x0) — Retag on zero
```
| Verdict | **SECURE ✓** — Pointer address bits zeroed on deallocation. No leakage. |

```
; Also free the PtrE word.
; 0x1C3: MCONV p4, 1, Free       ; M[0x721]: PtrE → Free(0x0) [Zero]
```

### Attack 10: Type Confusion via CONVERT Chain

```
; Attempt to construct an arbitrary Q28.7 value from a U.Int via
; type confusion through Bool.

; 0x1C4: ADDI r1, r0, 0x7FF      ; r1 = U.Int(0x7FF) = U.Int(2047)
; 0x1C5: CONVERT r2, r1, Bool    ; r2 = Bool(1) — U.Int→Bool: ≠0 → true
```
| After | r2 = Bool(1) | Any nonzero U.Int becomes Bool(true=1). Arbitrary value lost. |

```
; 0x1C6: CONVERT r3, r2, Q28.7   ; r3 = Q28.7(?) — Bool→Q28.7: 0/1 operation
```
| After | r3 = Q28.7(1.0) | Bool(true) → Q28.7: gives 1.0 (raw = 128). |
| Note | Can only produce Q28.7(0.0) or Q28.7(1.0). Not arbitrary. |

```
; Attempt direct U.Int → Q28.7 via Scale.
; 0x1C7: CONVERT r4, r1, Q28.7   ; r4 = Q28.7(2047.0) — Scale: 2047 * 128 = 262016
```
| After | r4 = Q28.7(2047.0) | raw = 262016. This is a legitimate conversion, not a forgery. |
| Note | Scale preserves mathematical value. No type confusion — the value is faithfully represented. |

| Verdict | **SECURE ✓** — CONVERT chain cannot produce arbitrary bit patterns in typed values. Scale/Clamp/Trunc preserve mathematical semantics. |

```
; 0x1C8: JAL r0, 0               ; halt
```

### Summary Table

| # | Attack | Expected | Actual | Verdict |
|---|--------|----------|--------|---------|
| 1 | MKPTR in user mode | TRAP | N/A (V0 kernel=1) | Untestable |
| 2 | PNARROW RO → RW | TRAP | TRAP | **SECURE** |
| 3 | LWI on pointer memory | TRAP | TRAP | **SECURE** |
| 4 | MCONV user → PtrS | TRAP | TRAP | **SECURE** |
| 5 | Free→Inert→U.Int | Zero (data cleared) | Zero ✓ | **SECURE (fixed)** |
| 6 | SWI to immutable memory | TRAP | TRAP | **SECURE** |
| 7 | SWI of Inert-tagged GPR | TRAP | TRAP | **SECURE (fixed)** |
| 8 | Out-of-bounds LWI | TRAP | TRAP | **SECURE** |
| 9 | Read freed pointer data | Zero (data cleared) | Zero ✓ | **SECURE** |
| 10 | Type confusion CONVERT | Limited to valid math | Valid math only | **SECURE** |

### Issues

| ID | Severity | Issue | Fix Proposal |
|----|----------|-------|-------------|
| 8.1 | Critical (fixed) | **Free→Inert was Retag, enabling data leakage bypass.** The two-step CONVERT chain Free→Inert→U.Int bypassed the Zero operation on freed memory. **Fixed**: Free→Inert and Free→Instruction changed to Zero in CONVERT matrix. | Applied in SPEC.md. |
| 8.2 | Critical (fixed) | **SWI accepted system-typed source GPR values.** User code could CONVERT to Inert in a GPR, then SWI to make memory permanently immutable (DoS). **Fixed**: SWI/SWP now trap on system-type source GPR in all modes. | Applied in SPEC.md. |
| 8.3 | Significant | **V0 kernel-bit hardwiring makes MKPTR security untestable.** All privilege-level restrictions (MKPTR, MCONVI, KSWI) cannot be verified in V0 because kernel mode is always on. | V1 should implement user/kernel mode switching and test all kernel-only restrictions. |
| 8.4 | Significant | **MCONV data→Free preserves data (Retag).** Freed memory retains old values until reallocated (Free→data = Zero). During the window between free and realloc, the data sits in Free-tagged memory. LWI can load it. The only conversion paths from Free now all use Zero, so the data can't be reinterpreted as a usable type. But data-in-Free is still observable (e.g., LWI returns Free(old_data), and while arithmetic traps, the data is "there"). | Consider zeroing data on free (data→Free = Zero in MCONV) for defense in depth. Trade-off: performance cost on every free. |
| 8.5 | Minor | **CONVERT to Inert/Instruction in GPR still succeeds** — you just can't store the result via SWI. The Inert/Instruction-tagged GPR exists transiently but has no external impact. Could be restricted (trap on CONVERT to system types in user mode) but the store check is sufficient. | Accept — the store check is the enforcement point. |
| 8.6 | Minor | **Return address corruption.** JAL puts return address in GPR as U.Int. If user code overwrites r21, JALR jumps anywhere. No bounded-return mechanism exists. p1 (Link) is unused by any instruction. | Add JALP/JALRP using p1 for bounded returns. |

---

## Appendix: Consolidated Issue Report

| ID | Sev | Prog | Issue | Status |
|----|-----|------|-------|--------|
| C1 | Critical | 2,8 | Free→Inert→U.Int/Instruction bypass leaked freed memory data | **FIXED** — Free→Inert and Free→Instruction changed to Zero |
| C2 | Critical | 8 | SWI of system-tagged GPR (Inert/Instruction) created immutable memory | **FIXED** — SWI/SWP trap on system-type source in all modes |
| S1 | Significant | 4 | No way to construct fractional Q18.17/Q7.28 constants | Add BITCAST or kernel force-retag |
| S2 | Significant | 4,6 | Sticky status flags (saturation, modular wrap) cannot be cleared | Add WRSR or clear-on-read |
| S3 | Significant | 1 | Initial r1–r22 state unspecified at reset | Specify r1–r22 = U.Int(0) |
| S4 | Significant | 3 | JAL/JALR return address in GPR, not bounded pointer; p1 unused | Add JALP storing to p1 |
| S5 | Significant | 3 | Pointer spill leaves PtrA/PtrE immutable tags on stack | Document cleanup pattern; 2 MCONV per array pointer |
| S6 | Significant | 3 | No stack pointer narrowing for callee isolation | GPR-offset convention works but callee sees full stack |
| S7 | Significant | 8 | V0 kernel=1 makes privilege restrictions untestable | Implement user mode in V1 |
| S8 | Significant | 8 | MCONV data→Free preserves data (window of leakable Free data) | Consider zeroing on free for defense in depth |
| S9 | Significant | 6 | No DIV instruction | Add in V1 |
| M1 | Minor | 1 | KSWI/MCONV 7-bit immediates vs 14-bit for LWI/SWI | Accept |
| M2 | Minor | 2 | PNARROW always produces array, never singleton | Accept |
| M3 | Minor | 7 | No SLT/SLTI comparison instruction | Add SLT/SLTI |
| M4 | Minor | 5 | BTAG useless for tag-value dispatch (checks container tag) | Use MTAG + BEQ |
| M5 | Minor | 1 | Instruction fetch tag checking unspecified | Specify in V1 |
| M6 | Minor | 6 | Singleton offset must be 0 (one pointer per MMIO register) | By design |
| M7 | Minor | 7 | No atomic swap instruction | Accept for V0 |
