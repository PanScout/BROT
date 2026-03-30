# BROT ISA V0 OS-Level Stress-Test Programs

## Conventions

**Register state**: `r1 = U.Int(0x100)` means GPR r1 has tag=U.Int, data=0x100.
**Memory state**: `M[0x100] = (Free, 0x0)` means address 0x100 has tag=Free, data=0x0.
**Pointer register**: `p2 = Array(RW, 0x100, 0x110)` means 82-bit register with PtrA base=0x100, PtrE end=0x110, RW=1.
**Singleton pointer**: `p2 = Singleton(RO, 0x200)` means PtrS target=0x200, RW=0.
All addresses are **word addresses**. Immediates are signed two's complement.
Instruction tags omitted from assembly (tag=Instruction on fetch).

### Kernel Master Pointer Pattern

OS programs use a recurring pattern: the kernel maintains a "master" array pointer register covering its entire heap/metadata region, plus a shadow GPR tracking the base address. To read any address A within the region:

```
SUB r_offset, r_addr, r_base    ; offset = A - base
LWP r_result, p_master, r_offset  ; read through master pointer
```

This avoids creating temporary pointer registers for every kernel read.

---

## Program 1: Bump Allocator (kmalloc)

**Purpose**: Implement a bump allocator that creates bounded array pointers for callers, exposing MKPTR scratch overhead and the missing KLWI/PBASE instructions.

**Pre-conditions**: All memory `(Free, 0x0)`. `r0 = U.Int(0)`. Kernel bit = 1. PC = 0x0.

**Memory map**:
- `0x000–0x03F` — Code
- `0x010–0x011` — Scratch area for MKPTR source words
- `0x1000–0x10FF` — Heap region (256 words)

**Register allocation**:
- `r20` = heap_base (shadow address)
- `r19` = heap_current (bump pointer, next free address)
- `r18` = heap_end (limit)
- `r17` = scratch base address
- `p12` = master heap pointer (covers entire heap)
- `p2` = return register (convention: kmalloc returns pointer here)

### Assembly

```
; === BOOT PROLOGUE ===
; Build kernel state from reset.

; 0x000: ADDI r20, r0, 0x1000     ; r20 = heap base address
```
| After | r20 = U.Int(0x1000) | Heap base shadow |
|-------|---------------------|------------------|

```
; 0x001: ADDI r18, r0, 0x1100     ; r18 = heap end address
```
| After | r18 = U.Int(0x1100) | |
|-------|---------------------|---|

```
; 0x002: ADDI r19, r0, 0x1000     ; r19 = heap_current (starts at base)
```
| After | r19 = U.Int(0x1000) | Bump pointer |
|-------|---------------------|--------------|

```
; 0x003: ADDI r17, r0, 0x10       ; r17 = scratch base
```
| After | r17 = U.Int(0x10) | |
|-------|-------------------|---|

```
; === CREATE MASTER HEAP POINTER (p12) ===
; Write base and end addresses to scratch, retag, MKPTRI.

; 0x004: KSWI r17, r20, 0         ; mem[0x10] = U.Int(0x1000) — base
```
| Before | M[0x10] = (Free, 0x0) |
|--------|----------------------|
| After  | M[0x10] = (U.Int, 0x1000) |
| Check  | Kernel=1 ✓ |

```
; 0x005: KSWI r17, r18, 1         ; mem[0x11] = U.Int(0x1100) — end
```
| Before | M[0x11] = (Free, 0x0) |
|--------|----------------------|
| After  | M[0x11] = (U.Int, 0x1100) |

```
; 0x006: MKPTRI p12, r17, 0, 0b11 ; p12 = Array(RW, 0x1000, 0x1100)
;        subop 0b11: bit0=1 (array), bit1=1 (RW)
;        Reads mem[0x10] (U.Int base) + mem[0x11] (U.Int end)
;        Retags them PtrA + PtrE, loads into p12
```
| Before | M[0x10] = (U.Int, 0x1000), M[0x11] = (U.Int, 0x1100) |
|--------|------------------------------------------------------|
| After  | M[0x10] = (PtrA, RW=1, base=0x1000), M[0x11] = (PtrE, end=0x1100) |
| Register | p12 = Array(RW, 0x1000, 0x1100) |
| Check | Kernel=1 ✓. Source tags U.Int ✓. |
| Note | Scratch 0x10–0x11 now immutable (PtrA/PtrE). Must MCONVI to reuse. |

```
; === CLEAN UP SCRATCH (must do before next MKPTR call) ===

; 0x007: MCONVI r17, 0, 0         ; tag(mem[0x10]) = Free (0)
;        PtrA → Free: kernel MCONV = Retag (data preserved)
```
| Before | M[0x10] = (PtrA, RW=1, base=0x1000) |
|--------|--------------------------------------|
| After  | M[0x10] = (Free, 0x1000) |
| Note | Data bits preserved (Retag). This is kernel MCONVI — no Zero. |

```
; 0x008: MCONVI r17, 1, 0         ; tag(mem[0x11]) = Free (0)
```
| Before | M[0x11] = (PtrE, end=0x1100) |
|--------|------------------------------|
| After  | M[0x11] = (Free, 0x1100) |

```
; === kmalloc(size=16) — First allocation ===
; Allocate [heap_current, heap_current + 16) = [0x1000, 0x1010)
; Returns Array(RW, 0x1000, 0x1010) in p2.

; Step 1: Compute new end and bounds check.

; 0x009: ADDI r1, r0, 16          ; r1 = U.Int(16) — requested size
```
| After | r1 = U.Int(16) | |
|-------|----------------|---|

```
; 0x00A: ADD r2, r19, r1          ; r2 = heap_current + size = 0x1000 + 16 = 0x1010
```
| After | r2 = U.Int(0x1010) | New end after allocation |
|-------|--------------------|-----------------------|
| Check | Type match: r19=U.Int, r1=U.Int ✓ |

```
; 0x00B: BLT r18, r2, +OOM       ; if heap_end < new_end, out of memory
;        r18=0x1100, r2=0x1010. 0x1100 < 0x1010? No. Fall through.
```
| Check | Type match: r18=U.Int, r2=U.Int ✓. 0x1100 >= 0x1010, no branch. |
|-------|------------------------------------------------------------------|

```
; Step 2: Write base/end to scratch for MKPTR.

; 0x00C: KSWI r17, r19, 0         ; mem[0x10] = U.Int(0x1000) — alloc base
```
| Before | M[0x10] = (Free, 0x1000) |
|--------|--------------------------|
| After  | M[0x10] = (U.Int, 0x1000) |
| Note | KSWI overwrites full word including tag. Previous Free data irrelevant. |

```
; 0x00D: KSWI r17, r2, 1          ; mem[0x11] = U.Int(0x1010) — alloc end
```
| Before | M[0x11] = (Free, 0x1100) |
|--------|--------------------------|
| After  | M[0x11] = (U.Int, 0x1010) |

```
; Step 3: Create pointer and load into return register p2.

; 0x00E: MKPTRI p2, r17, 0, 0b11  ; p2 = Array(RW, 0x1000, 0x1010)
```
| Before | M[0x10] = (U.Int, 0x1000), M[0x11] = (U.Int, 0x1010) |
|--------|------------------------------------------------------|
| After  | M[0x10] = (PtrA, RW=1, base=0x1000), M[0x11] = (PtrE, end=0x1010) |
| Register | p2 = Array(RW, 0x1000, 0x1010) |

```
; Step 4: Clean scratch for next use.

; 0x00F: MCONVI r17, 0, 0         ; mem[0x10] → Free
```
| After | M[0x10] = (Free, 0x1000) |
|-------|--------------------------|

```
; 0x010: MCONVI r17, 1, 0         ; mem[0x11] → Free
```
| After | M[0x11] = (Free, 0x1010) |
|-------|--------------------------|

```
; Step 5: Advance bump pointer.

; 0x011: ADD r19, r19, r1         ; heap_current = 0x1000 + 16 = 0x1010
```
| After | r19 = U.Int(0x1010) | Bump pointer advanced |
|-------|---------------------|-----------------------|

```
; === kmalloc(size=8) — Second allocation ===
; Allocate [0x1010, 0x1018). Returns in p3.
; (Reusing p3 instead of p2 to keep first allocation alive.)

; 0x012: ADDI r1, r0, 8           ; r1 = U.Int(8)
```
| After | r1 = U.Int(8) | |
|-------|---------------|---|

```
; 0x013: ADD r2, r19, r1          ; r2 = 0x1010 + 8 = 0x1018
```
| After | r2 = U.Int(0x1018) | |
|-------|--------------------|-|

```
; 0x014: BLT r18, r2, +OOM       ; 0x1100 < 0x1018? No. Fall through.
```

```
; 0x015: KSWI r17, r19, 0         ; mem[0x10] = U.Int(0x1010)
```
| After | M[0x10] = (U.Int, 0x1010) | |
|-------|---------------------------|-|

```
; 0x016: KSWI r17, r2, 1          ; mem[0x11] = U.Int(0x1018)
```
| After | M[0x11] = (U.Int, 0x1018) | |
|-------|---------------------------|-|

```
; 0x017: MKPTRI p3, r17, 0, 0b11  ; p3 = Array(RW, 0x1010, 0x1018)
```
| Before | M[0x10] = (U.Int, 0x1010), M[0x11] = (U.Int, 0x1018) |
|--------|------------------------------------------------------|
| After  | M[0x10] = (PtrA, RW=1, base=0x1010), M[0x11] = (PtrE, end=0x1018) |
| Register | p3 = Array(RW, 0x1010, 0x1018) |

```
; 0x018: MCONVI r17, 0, 0         ; scratch cleanup
```
| After | M[0x10] = (Free, 0x1010) |
|-------|--------------------------|

```
; 0x019: MCONVI r17, 1, 0         ; scratch cleanup
```
| After | M[0x11] = (Free, 0x1018) |
|-------|--------------------------|

```
; 0x01A: ADD r19, r19, r1         ; heap_current = 0x1010 + 8 = 0x1018
```
| After | r19 = U.Int(0x1018) | |
|-------|---------------------|-|

```
; === VERIFY: Write through allocated pointers, read back ===

; 0x01B: ADDI r4, r0, 0xBEEF     ; r4 = U.Int(0xBEEF) — test value
```
| After | r4 = U.Int(0xBEEF) | Note: 0xBEEF = 48879, fits in 14-bit unsigned? No — 14-bit signed range is -8192..+8191. 0xBEEF exceeds this. |
|-------|--------------------|-|

**Issue 1.1**: ADDI immediate is 14-bit signed (-8192 to +8191). 0xBEEF = 48879 does not fit. Must use LUI + ADDI.

```
; 0x01B: LUI r4, 0x5F7           ; r4 = U.Int(0x5F7 << 14) = U.Int(0x17DC000)
;        Wait — we just want 0xBEEF. LUI sets upper 21 bits.
;        0xBEEF = 0b10111110_11101111
;        LUI loads imm << 14. To get 0xBEEF:
;        upper 21 bits of (0xBEEF << (35-21))? No — LUI sets rd = imm << 14.
;        0xBEEF in bits: 1011_1110_1110_1111 (16 bits)
;        rd = imm << 14. imm = 0xBEEF >> 14 = 0x0002 (only top 2 bits).
;        That gives 0x8000. Then ADDI adds lower 14 bits: 0x3EEF.
;        0x8000 + 0x3EEF = 0xBEEF ✓
;
; Actually: LUI rd = imm << 14 (21-bit immediate).
; 0xBEEF = 0x0002 << 14 | 0x3EEF
; But 0x3EEF as signed 14-bit = 0x3EEF. Max positive 14-bit = 0x1FFF = 8191.
; 0x3EEF = 16111 > 8191, so this is interpreted as negative.
; 0x3EEF in 14-bit signed = 0x3EEF - 0x4000 = -273
; So: LUI r4, 3 gives r4 = 3 << 14 = 0xC000
;     ADDI r4, r4, -273 gives 0xC000 + (-273) = 0xC000 - 0x111 = 0xBEEF ✓

; Fix: Use a simpler test value that fits in 14-bit signed.

; 0x01B: ADDI r4, r0, 42         ; r4 = U.Int(42) — test value
```
| After | r4 = U.Int(42) | |
|-------|----------------|-|

```
; 0x01C: ADDI r5, r0, 99         ; r5 = U.Int(99) — second test value
```
| After | r5 = U.Int(99) | |
|-------|----------------|-|

```
; Write to first allocation (p2: [0x1000, 0x1010))
; 0x01D: SWI p2, r4, 0           ; mem[0x1000] = U.Int(42)
```
| Before | M[0x1000] = (Free, 0x0) |
|--------|------------------------|
| After  | M[0x1000] = (U.Int, 42) |
| Check  | (1) 0x1000 in [0x1000, 0x1010) ✓ (2) RW=1 ✓ (3) Free is mutable ✓ (4) U.Int is mutable ✓ |

```
; Write to second allocation (p3: [0x1010, 0x1018))
; 0x01E: SWI p3, r5, 0           ; mem[0x1010] = U.Int(99)
```
| Before | M[0x1010] = (Free, 0x0) |
|--------|------------------------|
| After  | M[0x1010] = (U.Int, 99) |
| Check  | (1) 0x1010 in [0x1010, 0x1018) ✓ (2) RW=1 ✓ (3) Free mutable ✓ (4) U.Int mutable ✓ |

```
; Read back from first allocation
; 0x01F: LWI r6, p2, 0           ; r6 = mem[0x1000] = U.Int(42)
```
| After | r6 = U.Int(42) | Matches written value ✓ |
|-------|----------------|------------------------|
| Check | 0x1000 in [0x1000, 0x1010) ✓. Tag=U.Int (not pointer) ✓. |

```
; Read back from second allocation
; 0x020: LWI r7, p3, 0           ; r7 = mem[0x1010] = U.Int(99)
```
| After | r7 = U.Int(99) | Matches written value ✓ |
|-------|----------------|------------------------|

```
; === CROSS-ALLOCATION BOUNDS CHECK ===
; Try to access p2 at offset 16 (should be out of bounds — p2 ends at 0x1010)

; 0x021: LWI r8, p2, 16          ; mem[0x1000 + 16] = mem[0x1010] — OUT OF BOUNDS
```
| Check | addr = 0x1000 + 16 = 0x1010. p2 bounds [0x1000, 0x1010). 0x1010 >= 0x1010 → **TRAP** ✓ |
|-------|------------------------------------------------------------------------------------------|
| Note | Allocations are properly isolated — p2 cannot reach p3's memory. |

```
; === END OF PROGRAM 1 ===
; Total: 34 instructions (0x000–0x021), including OOM branch target (not shown)
```

### Issue Report — Program 1

| ID | Severity | Issue | Detail |
|----|----------|-------|--------|
| 1.1 | Minor | Large constant construction is tedious | Building values > 8191 requires LUI+ADDI (2 instructions). Common for heap addresses. |
| 1.2 | Significant | MKPTR scratch overhead | Every pointer creation needs: 2× KSWI + 1× MKPTRI + 2× MCONVI cleanup = **5 instructions** + 2 scratch words. The scratch area becomes immutable after MKPTRI and must be cleaned. |
| 1.3 | Significant | No KLWI/KLWG | Kernel has KSWI/KSWG for raw writes but no corresponding raw reads. To read heap metadata, the kernel must maintain a master pointer register (p12). Asymmetric and forces pointer register consumption for kernel bookkeeping. |
| 1.4 | Significant | Pointer return convention undefined | kmalloc returns a pointer in p2 (or p3). No ABI convention specifies which pointer register is the return register. Must be defined by OS calling convention. |
| 1.5 | Significant | No PBASE instruction | Caller cannot extract the base address from the returned pointer register. If caller later needs to pass this address to kfree, it must have separately tracked it. |
| 1.6 | Minor | Bump allocator cannot free | By design (bump allocator limitation, not ISA), but the ISA makes a proper free-list allocator significantly harder (see Program 2). |

---

## Program 2: Free-List Allocator (kmalloc / kfree)

**Purpose**: Implement a first-fit free-list allocator with explicit deallocation, exposing the critical address-tracking problem (no PBASE) and free-list traversal patterns.

**Pre-conditions**: Boot prologue complete (same as P1). Master heap pointer p12 and shadow r20 already set up. Kernel bit = 1.

**Memory map**:
- `0x010–0x011` — Scratch area for MKPTR
- `0x1000–0x10FF` — Heap region (256 words)
- Free-list metadata: first 2 words of each free block = `[size (U.Int), next_addr (U.Int)]`
- Sentinel value for end-of-list: `0x3FFFFFFFF` (all 1s in 34-bit data field)

**Register allocation**:
- `r20` = U.Int(0x1000) — heap base (shadow for p12)
- `r19` = U.Int(0x1000) — free_list_head (address of first free block)
- `r18` = U.Int(0x1100) — heap end
- `r17` = U.Int(0x10) — scratch base
- `p12` = Array(RW, 0x1000, 0x1100) — master heap pointer
- `p2` = return register for kmalloc

### Assembly

```
; === INITIALIZE FREE LIST ===
; Single free block: [size=256, next=SENTINEL]
; Free-list metadata is stored as U.Int in the heap itself.
; We use master pointer p12 to write (not KSWI — p12 already exists).

; 0x030: ADDI r1, r0, 256         ; r1 = U.Int(256) — total heap size
```
| After | r1 = U.Int(256) | |
|-------|----------------|-|

```
; Problem: heap memory is (Free, 0x0). SWI to Free-tagged memory works
; (Free is mutable), but the written value overwrites tag+data? No —
; SWI writes the source GPR's full 41-bit word (tag+data) to memory.
; Source r1 = U.Int(256). Target M[0x1000] = (Free, 0x0).
; Check (3): target tag Free is mutable ✓
; Check (4): source tag U.Int is mutable ✓
; Result: M[0x1000] = (U.Int, 256)

; 0x031: SWI p12, r1, 0           ; mem[0x1000] = U.Int(256) — block size
```
| Before | M[0x1000] = (Free, 0x0) |
|--------|------------------------|
| After  | M[0x1000] = (U.Int, 256) |
| Check  | (1) 0x1000 in [0x1000, 0x1100) ✓ (2) RW=1 ✓ (3) Free mutable ✓ (4) U.Int mutable ✓ |

```
; 0x032: LUI r2, 0x1FFFFF         ; r2 = U.Int(0x1FFFFF << 14) = U.Int(0x7FFFFC000)
;        Wait — 21-bit imm. 0x1FFFFF = 2097151 (max 21-bit unsigned).
;        But imm is SIGNED. Max positive 21-bit = 0x0FFFFF = 1048575.
;        We need a sentinel. Let's use a simpler value: 0xFFFF (65535).
;        Actually for a sentinel we just need something outside heap range.
;        Heap is 0x1000–0x1100. Use 0x0 as sentinel (address 0 is never a valid heap block).

; 0x032: ADDI r2, r0, 0           ; r2 = U.Int(0) — null/sentinel (no next block)
```
| After | r2 = U.Int(0) | Sentinel = 0 (no valid heap block at address 0) |
|-------|---------------|-------------------------------------------------|

```
; 0x033: SWI p12, r2, 1           ; mem[0x1001] = U.Int(0) — next = sentinel
```
| Before | M[0x1001] = (Free, 0x0) |
|--------|------------------------|
| After  | M[0x1001] = (U.Int, 0) |

```
; Free list state:
;   Head: r19 = 0x1000
;   Block @ 0x1000: [size=256, next=0 (end)]
;   Remaining words 0x1002–0x10FF: (Free, 0x0)

; === kmalloc(32) — First allocation ===
; Traverse free list, find first block with size >= 32+2 (2 words for metadata).
; Split the block: allocate [0x1002, 0x1022), leave remainder.
;
; Strategy: metadata occupies first 2 words of free block.
; Allocated region starts at block_addr + 2.
; Minimum block size = requested_size + 2 (for metadata).

; Step 1: Load current block's size via master pointer.

; 0x034: SUB r3, r19, r20         ; r3 = free_list_head - heap_base = 0x1000 - 0x1000 = 0
```
| After | r3 = U.Int(0) | Offset into master pointer |
|-------|---------------|---------------------------|

```
; 0x035: LWP r4, p12, r3          ; r4 = mem[0x1000] = U.Int(256) — block size
```
| After | r4 = U.Int(256) | |
|-------|----------------|-|
| Check | addr = 0x1000 + 0 = 0x1000. In [0x1000, 0x1100) ✓. Tag=U.Int ✓. |

```
; Step 2: Check if block is large enough: size >= requested + 2 (metadata overhead)

; 0x036: ADDI r5, r0, 34          ; r5 = U.Int(34) — needed = 32 + 2
```
| After | r5 = U.Int(34) | |
|-------|----------------|-|

```
; 0x037: BLT r4, r5, +NEXT_BLOCK  ; if block_size < needed, try next block
;        256 < 34? No. Fall through — block is large enough.
```

```
; Step 3: Split the block.
; Allocate from current block: alloc_base = block_addr + 2 = 0x1002
; alloc_end = alloc_base + 32 = 0x1022
; Remaining block starts at 0x1022, size = 256 - 34 = 222

; 0x038: ADDI r6, r19, 2          ; r6 = U.Int(0x1002) — alloc_base
```
| After | r6 = U.Int(0x1002) | |
|-------|--------------------|-|

```
; 0x039: ADDI r7, r0, 32          ; r7 = U.Int(32) — alloc size
```
| After | r7 = U.Int(32) | |
|-------|----------------|-|

```
; 0x03A: ADD r8, r6, r7           ; r8 = 0x1002 + 32 = 0x1022 — alloc_end
```
| After | r8 = U.Int(0x1022) | Also = new free block address |
|-------|--------------------|-|

```
; Step 4: Update remainder block metadata.
; New block at 0x1022: [size=222, next=old_next(0)]

; 0x03B: SUB r9, r4, r5           ; r9 = 256 - 34 = 222 — remainder size
```
| After | r9 = U.Int(222) | |
|-------|----------------|-|

```
; 0x03C: SUB r10, r8, r20         ; r10 = 0x1022 - 0x1000 = 0x22 — offset for remainder
```
| After | r10 = U.Int(0x22) | |
|-------|-------------------|-|

```
; 0x03D: SWP p12, r9, r10         ; mem[0x1022] = U.Int(222) — remainder size
```
| Before | M[0x1022] = (Free, 0x0) |
|--------|------------------------|
| After  | M[0x1022] = (U.Int, 222) |
| Check  | (1) 0x1022 in [0x1000, 0x1100) ✓ (2) RW=1 ✓ (3) Free mutable ✓ (4) U.Int mutable ✓ |

```
; 0x03E: ADDI r10, r10, 1         ; r10 = 0x23 — offset for remainder's next field
```
| After | r10 = U.Int(0x23) | |
|-------|-------------------|-|

```
; Load old block's next pointer
; 0x03F: ADDI r3, r3, 1           ; r3 = 1 — offset to old block's next field
```
| After | r3 = U.Int(1) | |
|-------|---------------|-|

```
; 0x040: LWP r11, p12, r3         ; r11 = mem[0x1001] = U.Int(0) — old next
```
| After | r11 = U.Int(0) | Sentinel |
|-------|----------------|-|

```
; 0x041: SWP p12, r11, r10        ; mem[0x1023] = U.Int(0) — remainder's next = sentinel
```
| Before | M[0x1023] = (Free, 0x0) |
|--------|------------------------|
| After  | M[0x1023] = (U.Int, 0) |

```
; Step 5: Update free list head to point to remainder.

; 0x042: ADDI r19, r8, 0          ; r19 = 0x1022 — new head (ADD r19, r8, r0 also works)
;        Wait — ADDI r19, r8, 0 works: r19 = r8 + 0 = 0x1022
```
| After | r19 = U.Int(0x1022) | Free list head updated |
|-------|---------------------|-----------------------|

```
; Step 6: Create pointer for allocation [0x1002, 0x1022).
; Write base/end to scratch, MKPTRI into p2.

; 0x043: KSWI r17, r6, 0          ; mem[0x10] = U.Int(0x1002) — alloc base
```
| After | M[0x10] = (U.Int, 0x1002) |
|-------|---------------------------|

```
; 0x044: KSWI r17, r8, 1          ; mem[0x11] = U.Int(0x1022) — alloc end
```
| After | M[0x11] = (U.Int, 0x1022) |
|-------|---------------------------|

```
; 0x045: MKPTRI p2, r17, 0, 0b11  ; p2 = Array(RW, 0x1002, 0x1022)
```
| After | p2 = Array(RW, 0x1002, 0x1022) |
|-------|-------------------------------|
| After | M[0x10] = (PtrA, RW=1, 0x1002), M[0x11] = (PtrE, 0x1022) |

```
; 0x046: MCONVI r17, 0, 0         ; scratch cleanup
; 0x047: MCONVI r17, 1, 0         ; scratch cleanup
```
| After | M[0x10] = (Free, 0x1002), M[0x11] = (Free, 0x1022) |
|-------|-----------------------------------------------------|

```
; kmalloc(32) complete.
; Returned: p2 = Array(RW, 0x1002, 0x1022)
; Caller also knows: alloc_base = r6 = 0x1002 (must save this for kfree!)
; Free list: head=0x1022, block=[size=222, next=0]

; === kmalloc(16) — Second allocation ===
; Same procedure. Allocate from block at 0x1022.
; Need 16+2=18 words. Block has 222 words. 222 >= 18 ✓.
; alloc_base = 0x1022 + 2 = 0x1024
; alloc_end = 0x1024 + 16 = 0x1034
; remainder at 0x1034, size = 222 - 18 = 204

; 0x048: SUB r3, r19, r20         ; r3 = 0x1022 - 0x1000 = 0x22
```
| After | r3 = U.Int(0x22) | |
|-------|------------------|-|

```
; 0x049: LWP r4, p12, r3          ; r4 = mem[0x1022] = U.Int(222)
```
| After | r4 = U.Int(222) | |
|-------|----------------|-|

```
; 0x04A: ADDI r5, r0, 18          ; needed = 16 + 2
; 0x04B: BLT r4, r5, +NEXT       ; 222 < 18? No.
; 0x04C: ADDI r6, r19, 2          ; r6 = 0x1024 — alloc base
; 0x04D: ADDI r7, r0, 16          ; r7 = 16
; 0x04E: ADD r8, r6, r7           ; r8 = 0x1034 — alloc end
; 0x04F: SUB r9, r4, r5           ; r9 = 222 - 18 = 204 — remainder
; 0x050: SUB r10, r8, r20         ; r10 = 0x1034 - 0x1000 = 0x34
; 0x051: SWP p12, r9, r10         ; mem[0x1034] = U.Int(204)
; 0x052: ADDI r10, r10, 1         ; r10 = 0x35
; 0x053: ADDI r3, r3, 1           ; r3 = 0x23
; 0x054: LWP r11, p12, r3         ; r11 = mem[0x1023] = U.Int(0)
; 0x055: SWP p12, r11, r10        ; mem[0x1035] = U.Int(0)
; 0x056: ADDI r19, r8, 0          ; r19 = 0x1034
```
| After | r19 = U.Int(0x1034) | Free list head |
|-------|---------------------|-|

```
; 0x057: KSWI r17, r6, 0          ; mem[0x10] = U.Int(0x1024)
; 0x058: KSWI r17, r8, 1          ; mem[0x11] = U.Int(0x1034)
; 0x059: MKPTRI p3, r17, 0, 0b11  ; p3 = Array(RW, 0x1024, 0x1034)
; 0x05A: MCONVI r17, 0, 0         ; cleanup
; 0x05B: MCONVI r17, 1, 0         ; cleanup
```
| After | p3 = Array(RW, 0x1024, 0x1034) |
|-------|-------------------------------|

```
; State after 2 allocations:
;   p2 = Array(RW, 0x1002, 0x1022) — 32-word alloc #1
;   p3 = Array(RW, 0x1024, 0x1034) — 16-word alloc #2
;   Free list: head=0x1034, block=[size=204, next=0]
;   Metadata: M[0x1000..0x1001] = old metadata (stale)
;             M[0x1022..0x1023] = old metadata (stale)
;             M[0x1034..0x1035] = [U.Int(204), U.Int(0)]

; === kfree(alloc1) — Free the first allocation ===
;
; CRITICAL ISSUE: kfree needs the BASE ADDRESS of the allocation.
; The pointer register p2 contains Array(RW, 0x1002, 0x1022), but
; there is NO instruction to extract 0x1002 from p2 into a GPR.
;
; The caller must have separately saved r6 = U.Int(0x1002) at malloc time.
; kfree signature: kfree(addr_gpr) where addr_gpr = alloc base address.
;
; For this demo, r6 = U.Int(0x1002) still holds the first allocation's base.

; Step 1: Compute block header address = alloc_base - 2

; 0x05C: ADDI r12, r6, -2         ; r12 = 0x1002 - 2 = 0x1000 — block header addr
```
| After | r12 = U.Int(0x1000) | |
|-------|---------------------|-|

```
; Step 2: Convert allocated region to Free.
; The allocation spans [0x1002, 0x1022). We also reclaim metadata [0x1000, 0x1002).
; Total block = [0x1000, 0x1022) = 34 words.
;
; Memory at 0x1000–0x1001 has stale U.Int metadata.
; Memory at 0x1002–0x1021 may have U.Int data written by user.
; All are mutable types (U.Int) — MCONV to Free works.
;
; But wait: we need to WRITE new metadata (size, next) to the freed block.
; If we convert to Free first, then write metadata, the writes work (Free is mutable).
; The data bits in Free aren't zeroed by MCONV (U.Int→Free = Retag, data preserved).

; Actually, we don't need to MCONV at all for kfree!
; The block already contains U.Int data (mutable). We can just overwrite
; the first 2 words with new metadata via SWP through master pointer.

; Step 3: Write new free-block metadata.

; 0x05D: ADDI r13, r0, 34         ; r13 = U.Int(34) — total block size (32 + 2 metadata)
```
| After | r13 = U.Int(34) | |
|-------|----------------|-|

```
; 0x05E: SUB r10, r12, r20        ; r10 = 0x1000 - 0x1000 = 0 — offset in master
```
| After | r10 = U.Int(0) | |
|-------|----------------|-|

```
; 0x05F: SWP p12, r13, r10        ; mem[0x1000] = U.Int(34) — freed block size
```
| Before | M[0x1000] = (U.Int, 256) | Old stale metadata |
|--------|--------------------------|-------------------|
| After  | M[0x1000] = (U.Int, 34) |
| Check  | (1) bounds ✓ (2) RW ✓ (3) U.Int mutable ✓ (4) U.Int mutable ✓ |

```
; 0x060: ADDI r10, r10, 1         ; r10 = 1
; 0x061: SWP p12, r19, r10        ; mem[0x1001] = U.Int(0x1034) — next = old head
```
| Before | M[0x1001] = (U.Int, 0) | Old stale next |
|--------|------------------------|---------------|
| After  | M[0x1001] = (U.Int, 0x1034) |

```
; Step 4: Update free list head.

; 0x062: ADDI r19, r12, 0         ; r19 = 0x1000 — head points to freed block
```
| After | r19 = U.Int(0x1000) | |
|-------|---------------------|-|

```
; Free list after kfree:
;   Head: 0x1000
;   Block @ 0x1000: [size=34, next=0x1034]
;   Block @ 0x1034: [size=204, next=0]
;
; Note: p2 STILL holds Array(RW, 0x1002, 0x1022)!
; The pointer register was NOT invalidated.
; User code can still read/write through p2 — use-after-free!

; === DEMONSTRATE USE-AFTER-FREE ===

; 0x063: SWI p2, r4, 0            ; mem[0x1002] = U.Int(42) — SUCCEEDS!
```
| Before | M[0x1002] = (U.Int, 42) | Or whatever was there |
|--------|-------------------------|----------------------|
| After  | M[0x1002] = (U.Int, 42) |
| Check  | (1) 0x1002 in [0x1002, 0x1022) ✓ (2) RW=1 ✓ (3) U.Int mutable ✓ (4) U.Int mutable ✓ |
| **ISSUE** | **Write succeeds through freed pointer!** p2 is stale but functional. No hardware check prevents use-after-free. |

```
; === kmalloc(20) — Reallocate from freed block ===
; First-fit: try block at 0x1000 (size=34). Need 20+2=22. 34 >= 22 ✓.
; alloc_base = 0x1000 + 2 = 0x1002
; alloc_end = 0x1002 + 20 = 0x1016
; remainder at 0x1016, size = 34 - 22 = 12

; 0x064: SUB r3, r19, r20         ; r3 = 0
; 0x065: LWP r4, p12, r3          ; r4 = U.Int(34)
; 0x066: ADDI r5, r0, 22          ; needed
; 0x067: BLT r4, r5, +NEXT       ; 34 < 22? No.
; 0x068: ADDI r6, r19, 2          ; r6 = 0x1002
; 0x069: ADDI r7, r0, 20          ; alloc size
; 0x06A: ADD r8, r6, r7           ; r8 = 0x1016
; 0x06B: SUB r9, r4, r5           ; r9 = 12 — remainder
; 0x06C: SUB r10, r8, r20         ; r10 = 0x16
; 0x06D: SWP p12, r9, r10         ; mem[0x1016] = U.Int(12)
; 0x06E: ADDI r10, r10, 1         ; r10 = 0x17
; 0x06F: ADDI r3, r3, 1           ; r3 = 1
; 0x070: LWP r11, p12, r3         ; r11 = mem[0x1001] = U.Int(0x1034)
; 0x071: SWP p12, r11, r10        ; mem[0x1017] = U.Int(0x1034)
; 0x072: ADDI r19, r8, 0          ; r19 = 0x1016
```

```
; 0x073: KSWI r17, r6, 0          ; scratch base
; 0x074: KSWI r17, r8, 1          ; scratch end
; 0x075: MKPTRI p4, r17, 0, 0b11  ; p4 = Array(RW, 0x1002, 0x1016)
; 0x076: MCONVI r17, 0, 0         ; cleanup
; 0x077: MCONVI r17, 1, 0         ; cleanup
```
| After | p4 = Array(RW, 0x1002, 0x1016) |
|-------|-------------------------------|

```
; CRITICAL: p2 STILL holds Array(RW, 0x1002, 0x1022)
;           p4 holds Array(RW, 0x1002, 0x1016)
;           p2 and p4 OVERLAP at [0x1002, 0x1016)
;           p2 extends PAST p4 into [0x1016, 0x1022) which contains
;           free-list metadata at 0x1016–0x1017!
;
;           A write through stale p2 at offset 20 would hit mem[0x1016]
;           which is the remainder block's size field = U.Int(12).
;           This CORRUPTS the free list.

; 0x078: ADDI r14, r0, 20         ; r14 = 20
; 0x079: SWP p2, r4, r14          ; mem[0x1002 + 20] = mem[0x1016] = U.Int(34)
;                                  ; OVERWRITES FREE-LIST METADATA!
```
| Before | M[0x1016] = (U.Int, 12) | Free block size |
|--------|-------------------------|----------------|
| After  | M[0x1016] = (U.Int, 34) | **CORRUPTED** |
| Check  | (1) 0x1016 in [0x1002, 0x1022) ✓ (2) RW ✓ (3) U.Int mutable ✓ (4) U.Int mutable ✓ |
| **ISSUE** | **Stale pointer corrupts allocator metadata. No hardware protection.** |

```
; === END OF PROGRAM 2 ===
; Total: ~74 instructions (0x030–0x079)
```

### Issue Report — Program 2

| ID | Severity | Issue | Detail |
|----|----------|-------|--------|
| 2.1 | **Critical** | No pointer revocation | After kfree, the old pointer register (p2) remains valid. Reads/writes through stale pointers succeed. Reallocated memory can be accessed through both old and new pointers, enabling use-after-free and metadata corruption. Hardware provides **zero** protection. |
| 2.2 | **Critical** | kfree requires base address as separate argument | No PBASE instruction to extract base address from pointer register. Caller must separately track the U.Int address returned at malloc time. If lost, memory is permanently leaked — no way to identify which block to free. API must be `kfree(addr_gpr)` not `kfree(ptr_reg)`. |
| 2.3 | Significant | Free-list metadata is accessible through stale pointers | After free, metadata (size, next) is written into the freed block. A stale pointer covering that region can read allocator internals (information disclosure) or corrupt them (heap corruption). |
| 2.4 | Significant | MCONV data→Free preserves data bits | `MCONV U.Int→Free` is Retag (data preserved). If the OS converts freed blocks to Free tag for safety, the data is still readable through any stale pointer until overwritten. No automatic zeroing on free. |
| 2.5 | Significant | No coalescing without O(n) scan | Adjacent free blocks can't be merged without scanning the entire free list. No backward pointers (pointer bits can't enter GPRs for a doubly-linked list). Fragmentation will accumulate. |
| 2.6 | Minor | 2-word metadata overhead per block | Each allocation consumes 2 extra words for free-list metadata. A 3-word allocation costs 5 words (60% overhead). Minimum block size is 3 words (2 metadata + 1 data). |

---

## Program 3: Full Context Switch

**Purpose**: Save one process's full register state, restore another's, counting every instruction and exposing the MCONV cleanup overhead, missing WRSR, and self-save problem.

**Pre-conditions**: Kernel bit = 1. Two processes (A, B) with pre-initialized save areas. Process A is running.

**Memory map**:
- `0x2000–0x203F` — Process A save area (64 words)
- `0x2040–0x207F` — Process B save area (64 words)
- `0x010–0x011` — Scratch for MKPTR

**Save area layout** (per process, 50 words used of 64):
- `[0..21]` — GPR r1–r22 (22 words)
- `[22]` — Status register (1 word)
- `[23..48]` — Pointer registers p0–p12 (26 words max: 13 regs × 2 words each for arrays)
- `[49..63]` — Padding

**Register allocation**:
- `p11` = save area pointer: Array(RW, 0x2000, 0x2080) — covers both save areas
- `r16` = U.Int(0) — Process A save offset (0 for A, 64 for B)
- `r15` = U.Int(64) — Process B save offset

**Note**: p11 is the "save pointer" — it cannot save itself. It must be reconstructed from constants on restore.

### Assembly

```
; === SETUP: Create save area pointer p11 ===

; 0x080: ADDI r1, r0, 0x10        ; r1 = scratch base
; 0x081: ADDI r2, r0, 0x2000      ; r2 = save area start
;        Wait — 0x2000 = 8192. ADDI imm is 14-bit signed: max +8191.
;        0x2000 = 8192 > 8191. Need LUI+ADDI.
;
; 0x081: LUI r2, 1                ; r2 = U.Int(1 << 14) = U.Int(0x4000)
;        That's 16384. Too much. We need 0x2000 = 8192.
;        LUI rd = imm << 14. For 0x2000: 0x2000 / 0x4000 = 0.5. Not integer.
;        0x2000 = 0b10000000000000 = 1 << 13.
;        LUI shifts by 14 bits. So LUI r2, 0 gives 0. Not useful.
;        Actually: 0x2000 in upper+lower:
;        0x2000 = 0x0 << 14 + 0x2000. But 0x2000 = 8192. 14-bit signed max = 8191.
;        0x2000 as signed 14-bit = -8192 (0x2000 = 0b10000000000000, sign bit set).
;        So: ADDI r2, r0, -8192 would give... negative number.
;        r0 = U.Int(0). ADDI r2, r0, -8192. Immediates are signed.
;        But r0 is U.Int. Adding -8192 to U.Int(0) in unsigned saturating:
;        U.Int is UNSIGNED. 0 + (-8192) = negative, which saturates to 0.
;        That doesn't work!
;
;        For U.Int, ADDI immediate is interpreted as unsigned? No — spec says
;        "the immediate value is interpreted in the same representation as the
;        source register's type." U.Int is unsigned. But the immediate field is
;        always two's complement signed. So for U.Int, a negative immediate
;        is interpreted as... hmm, this is ambiguous.
;
;        Actually the spec says immediates are "two's complement signed values,
;        sign-extended to the operand width before use." So -8192 sign-extended
;        to 35 bits is 0x7FFFFE000 (large positive unsigned). Then U.Int(0) + that
;        = 0x7FFFFE000 which is huge and likely saturates at max U.Int.
;
;        Let's use LUI properly:
;        LUI r2, 0 = U.Int(0). No good.
;        LUI r2, 1 = U.Int(1 << 14) = U.Int(16384 = 0x4000).
;        ADDI r2, r2, -8192 = 16384 + (-8192).
;        But wait: -8192 sign-extended to 35 bits for U.Int arithmetic...
;        The immediate is sign-extended to 35 bits: 0x7FFFFE000.
;        U.Int(0x4000) + U.Int(0x7FFFFE000) = saturates.
;        This doesn't work for U.Int with negative immediates!

; ISSUE 3.1: Constructing addresses > 8191 with U.Int is broken.
; ADDI with negative immediate on U.Int produces a massive unsigned number.
; LUI only produces multiples of 16384.
;
; The only way: LUI gives 16384-aligned values, then ADDI adds 0..8191.
; 0x2000 = 8192. Not a multiple of 16384. Not reachable with LUI alone.
; 0x2000 = 0 × 16384 + 8192. But 8192 > max positive immediate (8191).
;
; Workaround: Use SLLI. ADDI r2, r0, 1. SLLI r2, r2, 13. r2 = 1 << 13 = 8192 = 0x2000.
; But SLLI requires integral types and U.Int is integral. ✓

; 0x081: ADDI r2, r0, 1           ; r2 = U.Int(1)
```
| After | r2 = U.Int(1) | |
|-------|---------------|-|

```
; 0x082: SLLI r2, r2, 13          ; r2 = U.Int(1 << 13) = U.Int(0x2000) = U.Int(8192)
```
| After | r2 = U.Int(0x2000) | Save area A base |
|-------|--------------------|-|

```
; 0x083: ADDI r3, r0, 1           ; r3 = U.Int(1)
; 0x084: SLLI r3, r3, 7           ; r3 = U.Int(128) = U.Int(0x80)
; 0x085: ADD r3, r2, r3           ; r3 = U.Int(0x2080) — save area end
```
| After | r3 = U.Int(0x2080) | |
|-------|--------------------|-|

```
; 0x086: KSWI r1, r2, 0           ; mem[0x10] = U.Int(0x2000)
; 0x087: KSWI r1, r3, 1           ; mem[0x11] = U.Int(0x2080)
; 0x088: MKPTRI p11, r1, 0, 0b11  ; p11 = Array(RW, 0x2000, 0x2080)
; 0x089: MCONVI r1, 0, 0          ; scratch cleanup
; 0x08A: MCONVI r1, 1, 0          ; scratch cleanup
```
| After | p11 = Array(RW, 0x2000, 0x2080) |
|-------|--------------------------------|

```
; Setup offsets
; 0x08B: ADDI r16, r0, 0          ; r16 = 0 — Process A offset
; 0x08C: ADDI r15, r0, 64         ; r15 = 64 — Process B offset
```

```
; ============================================================
; === SAVE PROCESS A STATE ===
; ============================================================
; We save r1–r22 (22 GPRs), status register, and p0–p10,p12–p13 (12 pointer regs).
; p11 (save pointer) cannot be saved through itself.
;
; Problem: We need working registers to compute offsets for saving.
; But saving a register destroys our offset tracking.
; Solution: save GPRs in order using an advancing offset register.
; Use r16 (Process A offset = 0) as the base. But we need a temporary
; register for SWP's offset argument.
;
; Actually: SWI has 14-bit immediate offset. p11 base = 0x2000.
; Process A at offset 0, so addresses 0x2000+0..0x2000+21 for GPRs.
; SWI p11, rs, imm where imm = slot index.
; 14-bit signed imm range: -8192..+8191. Slot indices 0..49 fit easily.

; --- Save GPRs r1–r22 (22 SWI instructions) ---

; 0x08D: SWI p11, r1, 0           ; save_area[0] = r1
```
| Before | M[0x2000] = (Free, 0x0) |
|--------|------------------------|
| After  | M[0x2000] = (U.Int, 1) |
| Note   | r1 was set to U.Int(0x10) earlier, but may have been modified. For this trace, assume r1 = U.Int(1) from the ADDI at 0x081. Actually r1 was set at 0x080. Let's track: r1 = U.Int(0x10). |
| Corrected | M[0x2000] = (U.Int, 0x10) — saving r1's current value |
| Check | (1) 0x2000 in [0x2000, 0x2080) ✓ (2) RW ✓ (3) Free mutable ✓ (4) U.Int mutable ✓ |

```
; 0x08E: SWI p11, r2, 1           ; save_area[1] = r2 = U.Int(0x2000)
; 0x08F: SWI p11, r3, 2           ; save_area[2] = r3 = U.Int(0x2080)
; 0x090: SWI p11, r4, 3           ; save_area[3] = r4
; 0x091: SWI p11, r5, 4           ; save_area[4] = r5
; 0x092: SWI p11, r6, 5           ; save_area[5] = r6
; 0x093: SWI p11, r7, 6           ; save_area[6] = r7
; 0x094: SWI p11, r8, 7           ; save_area[7] = r8
; 0x095: SWI p11, r9, 8           ; save_area[8] = r9
; 0x096: SWI p11, r10, 9          ; save_area[9] = r10
; 0x097: SWI p11, r11, 10         ; save_area[10] = r11
; 0x098: SWI p11, r12, 11         ; save_area[11] = r12
; 0x099: SWI p11, r13, 12         ; save_area[12] = r13
; 0x09A: SWI p11, r14, 13         ; save_area[13] = r14
; 0x09B: SWI p11, r15, 14         ; save_area[14] = r15 = U.Int(64)
; 0x09C: SWI p11, r16, 15         ; save_area[15] = r16 = U.Int(0)
; 0x09D: SWI p11, r17, 16         ; save_area[16] = r17
; 0x09E: SWI p11, r18, 17         ; save_area[17] = r18
; 0x09F: SWI p11, r19, 18         ; save_area[18] = r19
; 0x0A0: SWI p11, r20, 19         ; save_area[19] = r20
; 0x0A1: SWI p11, r21, 20         ; save_area[20] = r21
; 0x0A2: SWI p11, r22, 21         ; save_area[21] = r22
```
| Count | **22 SWI instructions** for GPR save |
|-------|--------------------------------------|

```
; --- Save status register ---

; 0x0A3: RDSR r1                   ; r1 = U.Int(status_register)
;        Status bits: [0]=mod_flag, [1]=sat_flag, [2]=hw_loop, [3]=kernel(1), [4]=int
```
| After | r1 = U.Int(0x08) | Assuming kernel=1 only, all flags clear. 0b01000 = 8 |
|-------|------------------|-|

```
; 0x0A4: SWI p11, r1, 22          ; save_area[22] = U.Int(status)
```
| After | M[0x2016] = (U.Int, 0x08) | |
|-------|---------------------------|-|
| Count | **2 instructions** for status save |

```
; --- Save pointer registers p0–p10, p12–p13 (skip p11 — it's the save pointer) ---
; SPRI p11, pN, offset
; Each array pointer writes 2 words (PtrA + PtrE). Singleton writes 1 word.
; We reserve 2 words per register (worst case all arrays).
; Slots 23–48: 13 pointer regs × 2 words = 26 slots. But we skip p11 = 12 regs × 2 = 24.

; 0x0A5: SPRI p11, p0, 23         ; save p0 (stack pointer) at offset 23
```
| Check | (1) 0x2000+23=0x2017 in [0x2000,0x2080) ✓ (2) RW ✓ (3) target mutable ✓ |
|-------|-------------------------------------------------------------------------|
| Note | If p0 is array: writes 2 words (PtrA at 0x2017, PtrE at 0x2018). Both target words must be mutable. If p0 is singleton: writes 1 word (PtrS at 0x2017). |
| After | M[0x2017] = (PtrA, ...), M[0x2018] = (PtrE, ...) — assuming p0 is array |
| **Key** | **These memory words are now immutable** (PtrA/PtrE system types). |

```
; 0x0A6: SPRI p11, p1, 25         ; save p1 (link pointer) at offset 25
; 0x0A7: SPRI p11, p2, 27         ; save p2 at offset 27
; 0x0A8: SPRI p11, p3, 29         ; save p3 at offset 29
; 0x0A9: SPRI p11, p4, 31         ; save p4 at offset 31
; 0x0AA: SPRI p11, p5, 33         ; save p5 at offset 33
; 0x0AB: SPRI p11, p6, 35         ; save p6 at offset 35
; 0x0AC: SPRI p11, p7, 37         ; save p7 at offset 37
; 0x0AD: SPRI p11, p8, 39         ; save p8 at offset 39
; 0x0AE: SPRI p11, p9, 41         ; save p9 at offset 41
; 0x0AF: SPRI p11, p10, 43        ; save p10 at offset 43
; --- skip p11 (save pointer, cannot self-save) ---
; 0x0B0: SPRI p11, p12, 45        ; save p12 at offset 45
; 0x0B1: SPRI p11, p13, 47        ; save p13 at offset 47
```
| Count | **13 SPRI instructions** (12 saved, p11 skipped but still takes a slot count) |
|-------|------------------------------------------------------------------------------|
| Note | Wait — we only saved 12 pointer registers (p0-p10, p12-p13 = 12). But 12 × 2 = 24 words. Slots 23..46. |
| Corrected | p0@23, p1@25, p2@27, p3@29, p4@31, p5@33, p6@35, p7@37, p8@39, p9@41, p10@43, p12@45, p13@47. That's 12 saves using slots 23–48. |

```
; --- MCONV cleanup: convert all PtrA/PtrE words back to Free ---
; Each array pointer wrote 2 immutable words. Must MCONV each to Free
; before the save area can be used for restoring (LPRP needs pointer tags!).
;
; WAIT: We must NOT clean up yet! The pointer data in the save area must
; REMAIN as PtrA/PtrE for LPRP to work on restore. LPRP requires the
; memory to have pointer tags.
;
; But then... when DO we clean up? After restore of Process B.
; Actually: the cleanup happens AFTER LPRP reads the data back.
;
; Flow:
; 1. Save Process A: SPRI writes PtrA/PtrE words (immutable)
; 2. Restore Process B: LPRP reads PtrA/PtrE from B's save area
; 3. Clean up Process B's save area: MCONV PtrA/PtrE → Free
; 4. (Later) When restoring Process A: LPRP reads from A's save area
;    (still has PtrA/PtrE from step 1 — good!)
;
; So we DON'T clean up A's save area now. We clean up B's after restore.

; ============================================================
; === RESTORE PROCESS B STATE ===
; ============================================================
; Process B save area starts at offset 64 in p11 (address 0x2040).
; Assume Process B was previously saved with same layout.
; Pre-condition: B's save area has valid data:
;   M[0x2040..0x205B] = GPR values (U.Int tagged)
;   M[0x2056] = status (U.Int)
;   M[0x2057..0x206E] = pointer data (PtrA/PtrE tagged)

; --- Restore pointer registers FIRST (before cleanup) ---
; Must do this before MCONV because LPRP requires PtrA/PtrE tags.

; 0x0B2: LPRI p0, p11, 87         ; p0 = mem[0x2040 + 23] = mem[0x2057]
;        Wait: offset from p11 base. p11 base = 0x2000.
;        Process B save area = 0x2040. Slot 23 (pointers) = 0x2040 + 23 = 0x2057.
;        Offset from p11 base = 0x2057 - 0x2000 = 0x57 = 87.
```
| Check | addr = 0x2000 + 87 = 0x2057. In [0x2000, 0x2080) ✓ |
|-------|-----------------------------------------------------|
| Note | Memory at 0x2057 must be PtrA or PtrS. Assuming PtrA: reads 2 words (0x2057, 0x2058). |
| After | p0 = restored from B's saved state |

```
; 0x0B3: LPRI p1, p11, 89         ; restore p1 from offset 89
; 0x0B4: LPRI p2, p11, 91         ; restore p2
; 0x0B5: LPRI p3, p11, 93         ; restore p3
; 0x0B6: LPRI p4, p11, 95         ; restore p4
; 0x0B7: LPRI p5, p11, 97         ; restore p5
; 0x0B8: LPRI p6, p11, 99         ; restore p6
; 0x0B9: LPRI p7, p11, 101        ; restore p7
; 0x0BA: LPRI p8, p11, 103        ; restore p8
; 0x0BB: LPRI p9, p11, 105        ; restore p9
; 0x0BC: LPRI p10, p11, 107       ; restore p10
; --- skip p11 ---
; 0x0BD: LPRI p12, p11, 109       ; restore p12
; 0x0BE: LPRI p13, p11, 111       ; restore p13
```
| Count | **12 LPRI instructions** for pointer restore |
|-------|--------------------------------------------|

```
; --- Clean up Process B save area: MCONV PtrA/PtrE → Free ---
; Each pointer slot is 2 words. 12 pointer registers × 2 = 24 MCONV.
; We use MCONV (user variant) through p11. Target tag = Free (0).
;
; Slots at offsets 87, 88, 89, 90, ..., 110, 111 (24 words total).

; 0x0BF: MCONV p11, 87, 0         ; M[0x2057]: PtrA → Free (Zero — clears data ✓)
; 0x0C0: MCONV p11, 88, 0         ; M[0x2058]: PtrE → Free (Zero)
; 0x0C1: MCONV p11, 89, 0
; 0x0C2: MCONV p11, 90, 0
; 0x0C3: MCONV p11, 91, 0
; 0x0C4: MCONV p11, 92, 0
; 0x0C5: MCONV p11, 93, 0
; 0x0C6: MCONV p11, 94, 0
; 0x0C7: MCONV p11, 95, 0
; 0x0C8: MCONV p11, 96, 0
; 0x0C9: MCONV p11, 97, 0
; 0x0CA: MCONV p11, 98, 0
; 0x0CB: MCONV p11, 99, 0
; 0x0CC: MCONV p11, 100, 0
; 0x0CD: MCONV p11, 101, 0
; 0x0CE: MCONV p11, 102, 0
; 0x0CF: MCONV p11, 103, 0
; 0x0D0: MCONV p11, 104, 0
; 0x0D1: MCONV p11, 105, 0
; 0x0D2: MCONV p11, 106, 0
; 0x0D3: MCONV p11, 107, 0
; 0x0D4: MCONV p11, 108, 0
; 0x0D5: MCONV p11, 109, 0
; 0x0D6: MCONV p11, 110, 0
```
| Count | **24 MCONV instructions** for pointer slot cleanup |
|-------|---------------------------------------------------|
| Note | MCONV user variant. PtrA→Free and PtrE→Free both use Zero operation: data bits cleared. This is correct — we don't want stale pointer addresses in Free memory. |
| **ISSUE** | MCONV imm field is 7 bits [20:14], range -64..+63. Offsets 87–111 are **out of range**! Max offset = 63. |

**Issue 3.2**: MCONV immediate field is only 7 bits (-64..+63). Process B's pointer save area starts at offset 87 from p11 base, which exceeds the immediate range. Must use MCONVR with a GPR offset instead.

```
; FIX: Use MCONVR with GPR offset for all cleanup.

; 0x0BF: ADDI r1, r0, 87          ; r1 = U.Int(87) — first pointer slot offset
; 0x0C0: MCONVR p11, r1, 0        ; M[0x2057]: PtrA → Free
; 0x0C1: ADDI r1, r1, 1           ; r1 = 88
; 0x0C2: MCONVR p11, r1, 0        ; M[0x2058]: PtrE → Free
; ... (repeat for all 24 words)
```
| Count | **24 MCONVR + 23 ADDI = 47 instructions** for cleanup. Or use a loop. |
|-------|----------------------------------------------------------------------|

```
; Using a loop instead:
; 0x0BF: ADDI r1, r0, 87          ; r1 = start offset
; 0x0C0: ADDI r2, r0, 111         ; r2 = end offset
; CLEANUP_LOOP:
; 0x0C1: MCONVR p11, r1, 0        ; convert slot to Free
; 0x0C2: ADDI r1, r1, 1           ; next slot
; 0x0C3: BGE r2, r1, CLEANUP_LOOP ; if end >= current, continue
;        (branch to 0x0C1, offset = -2)
; Loop runs 24 times × 3 instructions = 72 instruction executions.
```
| Count | **3 instructions, 72 executions** for cleanup loop |
|-------|--------------------------------------------------|

```
; --- Restore status register ---

; 0x0C4: LWI r1, p11, 86          ; r1 = B's saved status = mem[0x2056]
;        Offset 86 = 64 (B base) + 22 (status slot).
;        0x2000 + 86 = 0x2056.
```
| After | r1 = U.Int(B's_status) | |
|-------|-----------------------|-|
| **ISSUE** | **No WRSR instruction.** We loaded the status value but cannot write it to the status register. Process B's flags are LOST. Saturation and modular flags from Process A bleed through. |

```
; --- Restore GPRs r1–r22 ---
; Process B GPRs at offsets 64..85 from p11 base.

; 0x0C5: LWI r1, p11, 64          ; r1 = B's saved r1
; 0x0C6: LWI r2, p11, 65          ; r2 = B's saved r2
; 0x0C7: LWI r3, p11, 66
; 0x0C8: LWI r4, p11, 67
; 0x0C9: LWI r5, p11, 68
; 0x0CA: LWI r6, p11, 69
; 0x0CB: LWI r7, p11, 70
; 0x0CC: LWI r8, p11, 71
; 0x0CD: LWI r9, p11, 72
; 0x0CE: LWI r10, p11, 73
; 0x0CF: LWI r11, p11, 74
; 0x0D0: LWI r12, p11, 75
; 0x0D1: LWI r13, p11, 76
; 0x0D2: LWI r14, p11, 77
; 0x0D3: LWI r15, p11, 78
; 0x0D4: LWI r16, p11, 79
; 0x0D5: LWI r17, p11, 80
; 0x0D6: LWI r18, p11, 81
; 0x0D7: LWI r19, p11, 82
; 0x0D8: LWI r20, p11, 83
; 0x0D9: LWI r21, p11, 84
; 0x0DA: LWI r22, p11, 85
```
| Count | **22 LWI instructions** for GPR restore |
|-------|-----------------------------------------|
| Note | LWI imm is 14-bit signed. Offsets 64–85 are within range ✓ |

```
; --- Jump to Process B's PC ---
; Process B's saved PC would be in its return address register (r21).
; After restoring r21, we JALR to it.

; 0x0DB: JALR r0, r21, 0          ; PC = r21 (Process B resumes)
```

```
; === END OF PROGRAM 3 ===
```

### Instruction Count Summary

| Phase | Operation | Instructions | Executions |
|-------|-----------|-------------|------------|
| Setup | Create p11, offsets | 11 | 11 |
| Save GPRs | SWI × 22 | 22 | 22 |
| Save status | RDSR + SWI | 2 | 2 |
| Save pointers | SPRI × 12 | 12 | 12 |
| Restore pointers | LPRI × 12 | 12 | 12 |
| Cleanup (loop) | MCONVR + ADDI + BGE | 3 | 72 |
| Restore status | LWI (unusable) | 1 | 1 |
| Restore GPRs | LWI × 22 | 22 | 22 |
| Jump | JALR | 1 | 1 |
| **Total** | | **86 unique** | **155 executed** |

The MCONV cleanup loop dominates: **72 of 155 executed instructions (46%)** are just converting PtrA/PtrE tags back to Free.

### Issue Report — Program 3

| ID | Severity | Issue | Detail |
|----|----------|-------|--------|
| 3.1 | **Critical** | No WRSR — status register not restorable | RDSR is read-only. Saved status flags (saturation, modular wrap) cannot be written back on context restore. Sticky flags bleed between processes. A saturating overflow in Process A permanently affects Process B's flag state. **Correctness bug for any preemptive OS.** |
| 3.2 | Significant | MCONV immediate too small for save area offsets | MCONV imm field is 7 bits (-64..+63). Save area offsets for Process B's pointer slots exceed this. Must use MCONVR with GPR offset, adding instruction overhead. |
| 3.3 | Significant | MCONV cleanup is 46% of context switch | 72 of 155 executed instructions are tag cleanup. Hardware-assisted pointer save/restore (that auto-cleans tags) would halve context switch cost. |
| 3.4 | Significant | Self-save problem — p11 cannot save itself | The save pointer register must be reconstructed from constants on every context switch. Requires knowing the save area address at compile time or storing it in a fixed memory location accessible via another pointer. |
| 3.5 | Significant | Singleton vs array ambiguity at save time | SPRI writes 1 or 2 words depending on pointer type (singleton vs array). The kernel doesn't know which registers hold singletons at save time. Must reserve 2 words per register (worst case). On restore, LPRI auto-detects from PtrS vs PtrA tag. |
| 3.6 | Significant | No interrupt mechanism | Context switches can only be triggered voluntarily (cooperative scheduling). No timer interrupt, no hardware preemption. A misbehaving process cannot be interrupted. |
| 3.7 | Minor | Address construction > 8191 requires SLLI | U.Int addresses above 8191 can't be built with single ADDI (14-bit signed). Need ADDI+SLLI or LUI+ADDI (with sign-extension complexity). Affects all kernel setup code. |

---

## Program 4: Capability Delegation via PNARROW Chain

**Purpose**: Demonstrate kernel creating a region, narrowing it to processes with decreasing permissions, and exposing the fatal pointer revocation gap.

**Pre-conditions**: Kernel bit = 1. Heap and scratch set up as in P1.

**Memory map**:
- `0x3000–0x3040` — Shared region (64 words)
- `0x010–0x011` — Scratch

**Register allocation**:
- `p2` = kernel master pointer (full region RW)
- `p3` = Process A capability (sub-region RW)
- `p4` = Process B capability (sub-region RO)
- `p5` = sub-module capability (further narrowed from p3)

### Assembly

```
; === CREATE MASTER REGION POINTER ===

; 0x100: ADDI r1, r0, 0x10        ; scratch base
; 0x101: ADDI r2, r0, 0x3000      ; region base
;        0x3000 = 12288. > 8191. Need SLLI.
; 0x101: ADDI r2, r0, 3           ; r2 = 3
; 0x102: SLLI r2, r2, 12          ; r2 = 3 << 12 = 0x3000 = U.Int(12288)
```
| After | r2 = U.Int(0x3000) | |
|-------|--------------------|-|

```
; 0x103: ADDI r3, r0, 0x40        ; r3 = U.Int(64) — region size
; 0x104: ADD r3, r2, r3           ; r3 = 0x3000 + 64 = 0x3040 — region end
```
| After | r3 = U.Int(0x3040) | |
|-------|--------------------|-|

```
; 0x105: KSWI r1, r2, 0           ; mem[0x10] = U.Int(0x3000)
; 0x106: KSWI r1, r3, 1           ; mem[0x11] = U.Int(0x3040)
; 0x107: MKPTRI p2, r1, 0, 0b11   ; p2 = Array(RW, 0x3000, 0x3040)
; 0x108: MCONVI r1, 0, 0          ; cleanup
; 0x109: MCONVI r1, 1, 0          ; cleanup
```
| After | p2 = Array(RW, 0x3000, 0x3040) | Master region |
|-------|-------------------------------|---------------|

```
; === INITIALIZE REGION WITH DATA ===
; Write some U.Int values for processes to work with.

; 0x10A: ADDI r4, r0, 0xA         ; test value
; 0x10B: SWI p2, r4, 0            ; mem[0x3000] = U.Int(0xA)
; 0x10C: ADDI r4, r0, 0xB
; 0x10D: SWI p2, r4, 1            ; mem[0x3001] = U.Int(0xB)
; 0x10E: ADDI r4, r0, 0xC
; 0x10F: SWI p2, r4, 32           ; mem[0x3020] = U.Int(0xC)
```

```
; === DELEGATE TO PROCESS A: RW sub-region [0x3000, 0x3020) ===
; PNARROW p3, p2, rs_offset, rs_length, perm
; offset = 0 (from p2 base), length = 32, perm = 0 (inherit RW)

; 0x110: ADDI r5, r0, 0           ; offset = 0
; 0x111: ADDI r6, r0, 32          ; length = 32
; 0x112: PNARROW p3, p2, r5, r6, 0 ; p3 = Array(RW, 0x3000, 0x3020)
```
| After | p3 = Array(RW, 0x3000, 0x3020) |
|-------|-------------------------------|
| Check | Source p2 is array ✓. new_base = 0x3000 + 0 = 0x3000 >= 0x3000 ✓. new_end = 0x3000 + 32 = 0x3020 <= 0x3040 ✓. perm=0: inherit RW ✓. |

```
; === DELEGATE TO PROCESS B: RO sub-region [0x3020, 0x3030) ===
; perm = 1 (force read-only)

; 0x113: ADDI r5, r0, 32          ; offset = 32 (from p2 base 0x3000)
; 0x114: ADDI r6, r0, 16          ; length = 16
; 0x115: PNARROW p4, p2, r5, r6, 1 ; p4 = Array(RO, 0x3020, 0x3030)
```
| After | p4 = Array(RO, 0x3020, 0x3030) |
|-------|-------------------------------|
| Check | new_base = 0x3000+32 = 0x3020 >= 0x3000 ✓. new_end = 0x3020+16 = 0x3030 <= 0x3040 ✓. perm=1: force RO ✓. |

```
; === PROCESS A FURTHER NARROWS TO SUB-MODULE ===
; A gives sub-module access to [0x3008, 0x3018) RW

; 0x116: ADDI r5, r0, 8           ; offset = 8 (from p3 base 0x3000)
; 0x117: ADDI r6, r0, 16          ; length = 16
; 0x118: PNARROW p5, p3, r5, r6, 0 ; p5 = Array(RW, 0x3008, 0x3018)
```
| After | p5 = Array(RW, 0x3008, 0x3018) |
|-------|-------------------------------|
| Check | Source p3 is array ✓. new_base = 0x3000+8 = 0x3008 >= 0x3000 ✓. new_end = 0x3008+16 = 0x3018 <= 0x3020 ✓. perm=0: inherit RW ✓. |

```
; === ATTEMPT WIDENING: p5 → wider than p3 ===
; Sub-module tries to re-widen its capability back to p3's full range.

; 0x119: ADDI r5, r0, 0           ; offset = 0
; 0x11A: ADDI r6, r0, 32          ; length = 32 (p3's full range)
; 0x11B: PNARROW p6, p5, r5, r6, 0 ; ATTEMPT: [0x3008+0, 0x3008+32) = [0x3008, 0x3028)
```
| Check | new_end = 0x3008+32 = 0x3028. p5.end = 0x3018. 0x3028 > 0x3018 → **TRAP** ✓ |
|-------|-----------------------------------------------------------------------------|
| Note | Monotonic narrowing enforced. Cannot widen past parent bounds. |

```
; === PROCESS B ATTEMPTS WRITE THROUGH RO POINTER ===

; 0x11C: ADDI r7, r0, 0xFF
; 0x11D: SWI p4, r7, 0            ; ATTEMPT: write to RO pointer
```
| Check | p4 RW bit = 0 (read-only) → **TRAP** ✓ |
|-------|----------------------------------------|
| Note | SWI check (2): RW bit not set. Permission downgrade is enforced. |

```
; === PROCESS B ATTEMPTS TO ESCALATE PERMISSIONS ===

; 0x11E: ADDI r5, r0, 0
; 0x11F: ADDI r6, r0, 8
; 0x120: PNARROW p6, p4, r5, r6, 0 ; perm=0 inherits parent's RO
```
| After | p6 = Array(RO, 0x3020, 0x3028) |
|-------|-------------------------------|
| Note | perm=0 inherits RO from parent. Even "inherit" can't escalate. ✓ |

```
; === DEMONSTRATE REVOCATION GAP ===
; Kernel decides to revoke Process A's access to [0x3000, 0x3020).
; Strategy: convert region to Free via MCONV.

; 0x121: ADDI r8, r0, 0           ; offset counter
; 0x122: ADDI r9, r0, 32          ; region size
; REVOKE_LOOP:
; 0x123: MCONVR p2, r8, 0         ; tag(mem[0x3000 + r8]) = Free
; 0x124: ADDI r8, r8, 1
; 0x125: BLT r8, r9, REVOKE_LOOP  ; loop 32 times
```
| After | M[0x3000..0x301F] all = (Free, 0x0) | Data zeroed by Ptr→Free? No — these were U.Int, and U.Int→Free = Retag (data preserved). |
|-------|-------------------------------------|------|
| Corrected | M[0x3000] = (Free, 0xA), M[0x3001] = (Free, 0xB), rest = (Free, 0x0) |

```
; Region is now "freed." But Process A's p3 is still valid!

; 0x126: LWI r10, p3, 0           ; r10 = mem[0x3000] = Free(0xA)
```
| Check | 0x3000 in [0x3000, 0x3020) ✓. Tag = Free ✓ (not pointer, LWI allows). |
|-------|---------------------------------------------------------------------|
| After | r10 = Free(0xA) |
| **ISSUE** | **Read succeeds through revoked capability!** LWI loads the Free-tagged word including stale data (0xA). The "revocation" did nothing to prevent access. |

```
; 0x127: SWI p3, r7, 0            ; mem[0x3000] = U.Int(0xFF)
```
| Check | (1) bounds ✓ (2) RW ✓ (3) Free is mutable ✓ (4) U.Int is mutable ✓ |
|-------|-------------------------------------------------------------------|
| After | M[0x3000] = (U.Int, 0xFF) |
| **ISSUE** | **Write succeeds through revoked capability!** Process A can write to memory the kernel intended to reclaim. If this region is reallocated to Process C, Process A can read/write Process C's data. |

```
; === ATTEMPT: Kernel reallocates region to Process C ===
; Kernel creates new pointer for Process C over same range.

; 0x128: ADDI r5, r0, 0
; 0x129: ADDI r6, r0, 32
; 0x12A: PNARROW p6, p2, r5, r6, 0 ; p6 = Array(RW, 0x3000, 0x3020) — Process C
```
| After | p6 = Array(RW, 0x3000, 0x3020) |
|-------|-------------------------------|

```
; Now BOTH p3 (Process A, stale) AND p6 (Process C, new) point to [0x3000, 0x3020).
; Process A writes:
; 0x12B: ADDI r7, r0, 0xDEAD
;        (0xDEAD = 57005, exceeds 14-bit. Use smaller value.)
; 0x12B: ADDI r7, r0, 0x999
;        (0x999 = 2457. Fits in 14-bit signed ✓)
; 0x12C: SWI p3, r7, 5            ; mem[0x3005] = U.Int(0x999) — A corrupts C's data
```
| After | M[0x3005] = (U.Int, 0x999) | |
|-------|----------------------------|-|

```
; Process C reads, expecting its own data:
; 0x12D: LWI r10, p6, 5           ; r10 = mem[0x3005] = U.Int(0x999) — CORRUPTED by A!
```
| After | r10 = U.Int(0x999) | Process C sees Process A's write |
|-------|--------------------|-|
| **ISSUE** | **Complete capability isolation failure.** Stale pointer from revoked Process A can write into newly allocated Process C's region. No hardware mechanism prevents this. |

```
; === END OF PROGRAM 4 ===
; Total: ~46 instructions
```

### Issue Report — Program 4

| ID | Severity | Issue | Detail |
|----|----------|-------|--------|
| 4.1 | **Critical** | No pointer revocation mechanism | Kernel cannot invalidate pointer registers held by other processes. MCONV-ing memory to Free doesn't prevent access through existing pointer registers. Stale capabilities remain fully functional. After reallocation, multiple processes can access the same memory — complete isolation failure. |
| 4.2 | **Critical** | Use-after-free via stale capability | After kernel frees a region (MCONV→Free) and reallocates it (PNARROW to new process), the old process's stale pointer register can still read and write the region. This enables cross-process data corruption without any privilege escalation. |
| 4.3 | Significant | No pointer overlap detection | Kernel cannot determine if two pointer registers have overlapping ranges. The "no overlapping magisteria" invariant from the design must be tracked in a software ledger with O(n) comparisons. No hardware support. |
| 4.4 | Significant | MCONV data→Free preserves data on non-pointer types | When revoking, U.Int→Free is Retag (data preserved). Stale pointer can read old data values through Free-tagged memory. Only PtrS/PtrA/PtrE→Free uses Zero. For defense-in-depth, kernel should explicitly zero data after MCONV. |
| 4.5 | Minor | PNARROW can't produce singleton | Sub-module needing exactly 1 word gets a length-1 array. Functionally equivalent but wastes 1 PtrE word in memory if spilled. |
| 4.6 | Minor | Delegation is one-directional | PNARROW only narrows. A sub-module can't "return" a capability to its parent with wider bounds. Must pass through kernel (which recreates via MKPTR). |

### Proposed Fix for Revocation

**Option A — Epoch-based revocation**: Add an epoch counter to pointer registers. Kernel increments global epoch on revoke. Pointer access checks epoch against global — stale epochs trap. Hardware cost: ~8 bits per pointer register + global register + comparator per access.

**Option B — Fat pointer with revocation bit**: Add a per-region "valid" bit in a hardware revocation table indexed by region ID. Each pointer carries a region ID. Access checks table bit. Hardware cost: region table SRAM + lookup per access.

**Option C — Software-only (current)**: Kernel must track all outstanding capabilities in a software ledger and explicitly overwrite pointer registers on revoke (requires context switch to each holder). Correct but extremely expensive.

---

## Program 5: Process Loader — Code, Data, and Stack Setup

**Purpose**: Kernel loads a 3-instruction program into memory, creates code/data/stack regions, and transfers control. Exposes instruction encoding construction, immutable code memory, and JALR security gap.

**Pre-conditions**: Kernel bit = 1. Scratch at 0x10–0x11.

**Memory map**:
- `0x4000–0x4002` — Code region (3 instruction words)
- `0x4010–0x401F` — Data region (16 words)
- `0x4100–0x413F` — Stack region (64 words)

**Target program** (3 instructions):
```
; Loaded program will:
;   LWI r1, p3, 0       ; load first data word
;   ADDI r1, r1, 1       ; increment it
;   SWI p3, r1, 0        ; store back
```

**Register allocation**:
- `p2` = code pointer (RO)
- `p3` = data pointer (RW)
- `p0` = stack pointer (RW)
- `r21` = entry point address (for JALR)

### Assembly

```
; === STEP 1: Construct instruction words in GPRs ===
;
; Each instruction is 41 bits: [40:35]=tag [34:28]=opcode [27:21]=field2 ...
; The tag field for instructions in memory should be Instruction (tag 12 = 0x0C).
; But we're building the DATA bits in a GPR (which carries U.Int tag).
; We'll write via KSWI (writes full 41-bit word = GPR tag + data).
; But GPR tag is U.Int (1), not Instruction (12).
; So KSWI writes U.Int-tagged words. Then we MCONVI to retag as Instruction.
;
; Actually: what data bits does the instruction need?
; For "LWI r1, p3, 0":
;   [40:35] = tag (set by MCONVI, not encoded in data bits)
;   [34:28] = opcode = 0x19 (LWI)
;   [27:21] = rd = r1 = 1
;   [20:14] = pr = p3 = 3
;   [13:0]  = imm = 0
;
; Data bits [34:0] of the 41-bit word (tag will be set separately):
;   [34:28] = 0x19 = 0b0011001
;   [27:21] = 0b0000001
;   [20:14] = 0b0000011
;   [13:0]  = 0b00000000000000
;
; As a 35-bit value:
;   0b 0011001 0000001 0000011 00000000000000
;   = 0x19 << 28 | 1 << 21 | 3 << 14 | 0
;   = 0x19 * 0x10000000 = 0x190000000
;   + 1 * 0x200000 = 0x200000
;   + 3 * 0x4000 = 0xC000
;   = 0x1902_0C000
;
;   Wait, let me recalculate properly.
;   35-bit data field = bits [34:0]
;   opcode at [34:28]: 0x19 = 25 decimal
;   rd at [27:21]: 1
;   pr at [20:14]: 3
;   imm at [13:0]: 0
;
;   value = (0x19 << 28) | (1 << 21) | (3 << 14) | 0
;         = 25 * 268435456 = 6710886400 = 0x190000000
;         + 1 * 2097152 = 0x200000
;         + 3 * 16384 = 0xC000
;         = 0x19020C000
;
;   This is a 33-bit number (0x19020C000). Fits in 35 bits ✓.
;
; Building 0x19020C000 from ADDI (14-bit) and LUI (21-bit << 14):
;   LUI loads imm << 14 into GPR as U.Int.
;   0x19020C000 >> 14 = 0x19020C000 / 16384 = 0x640830 (approx)
;   Actually: 0x19020C000 = 0x640830 * 0x4000 + 0x0
;   0x640830 as 21-bit: max 21-bit unsigned = 0x1FFFFF = 2097151.
;   0x640830 = 6555696. > 2097151. Doesn't fit in 21-bit LUI!
;
; ISSUE: Instruction word construction requires values > 21-bit LUI range.
; Need multiple shifts and ORs.

; Build LWI instruction: opcode=0x19, rd=1, pr=3, imm=0
; Strategy: build field by field using shifts.

; 0x200: ADDI r1, r0, 0x19        ; r1 = U.Int(25) — opcode
; 0x201: SLLI r1, r1, 7           ; r1 = 25 << 7 = U.Int(3200) — opcode in [34:28] rel to [34:0]?
;
; Wait. Let me reconsider the bit layout.
; The full 41-bit word is: [40:35]=tag(6) [34:28]=opcode(7) [27:21]=rd(7) [20:14]=pr(7) [13:0]=imm(14)
; When stored in memory, the tag is bits [40:35] and data is bits [34:0].
; KSWI writes the GPR's tag into [40:35] and data into [34:0].
; After KSWI, we MCONVI to change [40:35] to Instruction tag.
; So we only need to construct the DATA portion = bits [34:0] = 35 bits.
;
; bits [34:28] = opcode (7 bits), starting at position 28 in the data field
; bits [27:21] = rd (7 bits), starting at position 21
; bits [20:14] = pr (7 bits), starting at position 14
; bits [13:0] = imm (14 bits), starting at position 0
;
; r1 = opcode << 28 | rd << 21 | pr << 14 | imm
;
; opcode << 28: 0x19 << 28. U.Int is 35 bits. Max = 0x7FFFFFFFF.
; 0x19 << 28 = 25 * 268435456 = 6710886400 = 0x190000000.
; 0x190000000 in 35 bits: max 35-bit = 0x7FFFFFFFF = 34359738367.
; 6710886400 < 34359738367 ✓.
;
; Build strategy:
;   ADDI r1, r0, 0x19     ; r1 = 25
;   SLLI r1, r1, 28       ; r1 = 25 << 28 = 0x190000000
;   Problem: SLLI imm is 14-bit signed. 28 fits ✓.
;   But shifting 25 left by 28 on a 35-bit unsigned:
;   25 = 0b11001. Shifted 28 = 0b11001_0000...0 (28 zeros).
;   Total bits needed: 5 + 28 = 33. Fits in 35 ✓.

; 0x200: ADDI r1, r0, 0x19        ; r1 = U.Int(25)
```
| After | r1 = U.Int(25) | LWI opcode |
|-------|----------------|-|

```
; 0x201: SLLI r1, r1, 28          ; r1 = U.Int(0x190000000)
```
| After | r1 = U.Int(0x190000000) | Opcode positioned |
|-------|------------------------|-|

```
; 0x202: ADDI r2, r0, 1           ; r2 = U.Int(1) — rd = r1
; 0x203: SLLI r2, r2, 21          ; r2 = U.Int(0x200000) — rd positioned
; 0x204: OR r1, r1, r2            ; r1 = 0x190000000 | 0x200000 = 0x190200000
```
| After | r1 = U.Int(0x190200000) | opcode + rd |
|-------|------------------------|-|

```
; 0x205: ADDI r2, r0, 3           ; r2 = 3 — pr = p3
; 0x206: SLLI r2, r2, 14          ; r2 = U.Int(0xC000) — pr positioned
; 0x207: OR r1, r1, r2            ; r1 = 0x190200000 | 0xC000 = 0x19020C000
```
| After | r1 = U.Int(0x19020C000) | LWI r1, p3, 0 — complete data bits |
|-------|------------------------|---|

```
; imm = 0, no need to OR. First instruction word ready in r1.

; Build ADDI instruction: opcode=0x12, rd=1, rs1=1, imm=1
; data = 0x12 << 28 | 1 << 21 | 1 << 14 | 1

; 0x208: ADDI r3, r0, 0x12        ; r3 = 18 (ADDI opcode)
; 0x209: SLLI r3, r3, 28          ; r3 = 0x120000000
; 0x20A: ADDI r2, r0, 1
; 0x20B: SLLI r2, r2, 21
; 0x20C: OR r3, r3, r2            ; + rd=1
; 0x20D: ADDI r2, r0, 1
; 0x20E: SLLI r2, r2, 14
; 0x20F: OR r3, r3, r2            ; + rs1=1
; 0x210: ADDI r2, r0, 1
; 0x211: OR r3, r3, r2            ; + imm=1
```
| After | r3 = U.Int(0x120204001) | ADDI r1, r1, 1 — data bits |
|-------|------------------------|-|

```
; Build SWI instruction: opcode=0x0B, pr=p3=3, rs=r1=1, imm=0
; data = 0x0B << 28 | 3 << 21 | 1 << 14 | 0

; 0x212: ADDI r4, r0, 0x0B        ; r4 = 11 (SWI opcode)
; 0x213: SLLI r4, r4, 28          ; r4 = 0x0B0000000
; 0x214: ADDI r2, r0, 3
; 0x215: SLLI r2, r2, 21
; 0x216: OR r4, r4, r2            ; + pr=3
; 0x217: ADDI r2, r0, 1
; 0x218: SLLI r2, r2, 14
; 0x219: OR r4, r4, r2            ; + rs=1
```
| After | r4 = U.Int(0x0B0604000) | SWI p3, r1, 0 — data bits |
|-------|------------------------|-|
| Recheck | 0x0B << 28 = 0x0B0000000. 3 << 21 = 0x600000. 1 << 14 = 0x4000. Total = 0x0B0604000 ✓ |

```
; === STEP 2: Write instruction data to code region via KSWI ===
; Code region: 0x4000–0x4002

; Need r5 = address 0x4000
; 0x21A: ADDI r5, r0, 1
; 0x21B: SLLI r5, r5, 14          ; r5 = U.Int(0x4000) = U.Int(16384)
```
| After | r5 = U.Int(0x4000) | Code region base |
|-------|--------------------|-|

```
; 0x21C: KSWI r5, r1, 0           ; mem[0x4000] = U.Int(0x19020C000)
; 0x21D: KSWI r5, r3, 1           ; mem[0x4001] = U.Int(0x120204001)
; 0x21E: KSWI r5, r4, 2           ; mem[0x4002] = U.Int(0x0B0604000)
```
| After | M[0x4000] = (U.Int, 0x19020C000) |
|-------|----------------------------------|
| After | M[0x4001] = (U.Int, 0x120204001) |
| After | M[0x4002] = (U.Int, 0x0B0604000) |
| Note | Written as U.Int — NOT yet executable. Must retag as Instruction. |

```
; === STEP 3: Retag code region as Instruction ===
; MCONVI with target_tag = Instruction (12 = 0x0C)

; 0x21F: MCONVI r5, 0, 12         ; tag(mem[0x4000]) = Instruction
```
| Before | M[0x4000] = (U.Int, 0x19020C000) |
|--------|----------------------------------|
| After  | M[0x4000] = (Instruction, 0x19020C000) |
| Note | U.Int → Instruction: CONVERT matrix says "—" (trap). But MCONVI is kernel MCONV — it uses Retag/Zero from the matrix. Wait: MCONV only allows Retag and Zero operations. U.Int→Instruction in the matrix is "—" (trap). Does MCONVI trap too? |

**Issue 5.1 — CRITICAL**: The CONVERT matrix entry for U.Int→Instruction is "—" (trap). MCONV spec says "Only Retag and Zero conversions from the conversion matrix are valid; all other conversion types (Scale, Trunc, Clamp, etc.) trap." A "—" entry is not Retag or Zero — it's a trap entry. **MCONVI U.Int→Instruction should trap!**

But wait — the MCONV spec also says the kernel pair "Can target any type including pointer types." Does this override the conversion matrix? Let me re-read:

The kernel MCONVI/MCONVG description says: "Can target any type including pointer types." And for pointer conversions: "Kernel mode (MCONVI, MCONVG): Full Retag in both directions (data bits preserved) — Free/U.Int ↔ PtrS/PtrA/PtrE."

This implies the kernel pair uses Retag for all conversions regardless of the CONVERT matrix. The CONVERT matrix governs the GPR `CONVERT` instruction and the user `MCONV/MCONVR` pair. The kernel pair bypasses the matrix entirely.

```
; Assuming kernel MCONVI does Retag regardless of matrix:

; 0x21F: MCONVI r5, 0, 12         ; tag(mem[0x4000]) = Instruction (Retag, data preserved)
; 0x220: MCONVI r5, 1, 12         ; tag(mem[0x4001]) = Instruction
; 0x221: MCONVI r5, 2, 12         ; tag(mem[0x4002]) = Instruction
```
| After | M[0x4000] = (Instruction, 0x19020C000) |
|-------|--------------------------------------|
| After | M[0x4001] = (Instruction, 0x120204001) |
| After | M[0x4002] = (Instruction, 0x0B0604000) |
| Note | Code region now immutable. SWI to these addresses will trap (system type target). |

```
; === STEP 4: Initialize data region ===

; 0x222: ADDI r6, r0, 0x4010      ; Data region base — 0x4010 = 16400
;        0x4010 > 8191. Need shift.
; 0x222: ADDI r6, r5, 0x10        ; r6 = 0x4000 + 16 = 0x4010
```
| After | r6 = U.Int(0x4010) | Data region base |
|-------|--------------------|-|

```
; 0x223: ADDI r7, r0, 100         ; Initial data value
; 0x224: KSWI r6, r7, 0           ; mem[0x4010] = U.Int(100)
```
| After | M[0x4010] = (U.Int, 100) | |
|-------|-------------------------|-|

```
; === STEP 5: Create code pointer (RO) ===

; 0x225: ADDI r8, r0, 0x10        ; scratch base
; 0x226: ADDI r9, r5, 3           ; r9 = 0x4003 — code end
; 0x227: KSWI r8, r5, 0           ; mem[0x10] = U.Int(0x4000)
; 0x228: KSWI r8, r9, 1           ; mem[0x11] = U.Int(0x4003)
; 0x229: MKPTRI p2, r8, 0, 0b01   ; p2 = Array(RO, 0x4000, 0x4003)
;        subop 0b01: bit0=1 (array), bit1=0 (RO)
; 0x22A: MCONVI r8, 0, 0          ; cleanup
; 0x22B: MCONVI r8, 1, 0          ; cleanup
```
| After | p2 = Array(RO, 0x4000, 0x4003) | Code pointer |
|-------|-------------------------------|-|

```
; === STEP 6: Create data pointer (RW) ===

; 0x22C: ADDI r9, r6, 16          ; r9 = 0x4020 — data end
; 0x22D: KSWI r8, r6, 0           ; mem[0x10] = U.Int(0x4010)
; 0x22E: KSWI r8, r9, 1           ; mem[0x11] = U.Int(0x4020)
; 0x22F: MKPTRI p3, r8, 0, 0b11   ; p3 = Array(RW, 0x4010, 0x4020)
; 0x230: MCONVI r8, 0, 0          ; cleanup
; 0x231: MCONVI r8, 1, 0          ; cleanup
```
| After | p3 = Array(RW, 0x4010, 0x4020) | Data pointer |
|-------|-------------------------------|-|

```
; === STEP 7: Create stack pointer (RW) ===

; 0x232: ADDI r9, r0, 1
; 0x233: SLLI r9, r9, 14          ; r9 = 0x4000
; 0x234: ADDI r9, r9, 0x100       ; r9 = 0x4100 — stack base
; 0x235: ADDI r10, r9, 64         ; r10 = 0x4140 — stack end
; 0x236: KSWI r8, r9, 0           ; mem[0x10] = U.Int(0x4100)
; 0x237: KSWI r8, r10, 1          ; mem[0x11] = U.Int(0x4140)
; 0x238: MKPTRI p0, r8, 0, 0b11   ; p0 = Array(RW, 0x4100, 0x4140)
; 0x239: MCONVI r8, 0, 0          ; cleanup
; 0x23A: MCONVI r8, 1, 0          ; cleanup
```
| After | p0 = Array(RW, 0x4100, 0x4140) | Stack pointer |
|-------|-------------------------------|-|

```
; === STEP 8: Transfer control ===
; Set up r21 = entry point address, r22 = frame offset.

; 0x23B: ADDI r22, r0, 0          ; r22 = U.Int(0) — frame pointer offset
; 0x23C: ADDI r21, r5, 0          ; r21 = U.Int(0x4000) — entry point
; 0x23D: JALR r0, r21, 0          ; PC = r21 + 0 = 0x4000. r0 = discard return addr.
```
| After | PC = 0x4000 |
|-------|-------------|
| Note | JALR target is GPR integer, NOT bounds-checked against p2 (code pointer). |

```
; === EXECUTION OF LOADED PROGRAM ===
;
; PC=0x4000: LWI r1, p3, 0        ; r1 = mem[0x4010] = U.Int(100)
; PC=0x4001: ADDI r1, r1, 1       ; r1 = U.Int(101)
; PC=0x4002: SWI p3, r1, 0        ; mem[0x4010] = U.Int(101)
;
; Program completes. No return address — falls through to PC=0x4003.
; PC=0x4003 contains (Free, 0x0). Instruction fetch of non-Instruction tag: ???
```
| **ISSUE** | PC=0x4003 fetches Free-tagged memory. Behavior UNSPECIFIED. Should trap but no spec rule covers instruction fetch tag checking. |

```
; === SECURITY ISSUE: JALR TARGET NOT BOUNDED ===
; User code (in future with user mode) can:
;   ADDI r1, r0, 0         ; r1 = kernel code address 0x0
;   JALR r0, r1, 0         ; jump to kernel code!
;
; JALR uses a GPR integer as target. No bounds check against any
; code pointer. The instruction fetch may check for Instruction tag
; at the target address, but this is NOT specified.
;
; If kernel code region is tagged Instruction, user jumping there
; executes kernel code at kernel privilege. This is a CRITICAL
; control-flow hijacking vulnerability.

; === END OF PROGRAM 5 ===
; Total: ~62 instructions (0x200–0x23D plus loaded program)
```

### Instruction Count: Building 3 Instructions

| Phase | Instructions | Notes |
|-------|-------------|-------|
| Build LWI word | 8 (ADDI, SLLI, ADDI, SLLI, OR, ADDI, SLLI, OR) | 3 fields + shifts |
| Build ADDI word | 10 (ADDI, SLLI, ADDI, SLLI, OR, ADDI, SLLI, OR, ADDI, OR) | 4 fields |
| Build SWI word | 8 | 3 fields |
| **Total** | **26 instructions** to build 3 instruction words | ~8.7 instructions per encoded word |

### Issue Report — Program 5

| ID | Severity | Issue | Detail |
|----|----------|-------|--------|
| 5.1 | **Critical** | JALR not bounded by code pointer | JALR jumps to any GPR integer address. No hardware bounds check against a code pointer register. User code can jump to kernel code addresses, executing kernel instructions at kernel privilege. Control-flow integrity is broken. |
| 5.2 | **Critical** | Instruction fetch tag checking unspecified | When PC advances to a non-Instruction-tagged word, behavior is undefined. Spec does not state that instruction fetch checks for Instruction tag. Should trap. Without this, user code could execute data or Free memory as instructions. |
| 5.3 | Significant | MCONVI U.Int→Instruction: matrix says trap | CONVERT matrix has "—" for U.Int→Instruction. Whether kernel MCONVI bypasses the matrix for non-pointer types is ambiguous. Spec should explicitly state that kernel MCONVI always does Retag (bypassing matrix) for all type pairs. |
| 5.4 | Significant | Code patching requires 3 kernel operations per word | Once tagged Instruction (immutable), modifying code requires: MCONVI Instruction→U.Int, KSWI new data, MCONVI U.Int→Instruction. No self-modifying code possible in user mode. |
| 5.5 | Significant | No return mechanism from loaded code | Loaded program has no way to return to the loader. JALR stores return address in GPR r0 (discarded). Even if stored in r_N, the program needs to know the return address — which requires the loader to set it up in a GPR before transfer. No hardware call stack. |
| 5.6 | Significant | No ECALL/SYSCALL instruction | Loaded user code has no way to request kernel services. Must rely on JALR to known kernel addresses (which is itself a security issue per 5.1). V1+ needs a proper trap-based syscall mechanism. |
| 5.7 | Minor | Instruction construction is extremely tedious | ~9 instructions per encoded word. A 100-instruction program requires ~900 setup instructions. Real systems would use a bootloader loading from storage, but no storage I/O instructions exist in V0. |
| 5.8 | Minor | No FENCE.I equivalent | After writing and retagging code, no instruction fence guarantees fetch coherence. If processor has an instruction cache (or prefetch buffer), stale data may be fetched. |

---

## Appendix: Consolidated OS-Level Issue Report

### Critical Issues (4)

| ID | Program(s) | Issue | Proposed Fix |
|----|-----------|-------|--------------|
| OS-C1 | P2, P4 | **No pointer revocation** — stale pointer registers bypass all protection after free/realloc. Kernel MCONV-ing memory to Free doesn't invalidate existing pointer registers. After reallocation, multiple processes access the same memory (complete isolation failure). | Add hardware revocation: epoch-based (pointer carries epoch, global counter increments on revoke, stale epochs trap) or revocation table (per-region valid bit, pointer carries region ID). Software-only revocation requires context-switching to every holder. |
| OS-C2 | P5 | **JALR not bounded by code pointer** — JALR jumps to any GPR integer address with no bounds check. User code can jump to kernel code addresses. Combined with lack of user/kernel privilege separation in V0, this enables arbitrary control flow hijacking. | Add JALP variant that jumps through a code pointer register (bounds-checked), or require instruction fetch tag check (OS-C3). |
| OS-C3 | P5 | **Instruction fetch tag checking unspecified** — when PC points to non-Instruction-tagged memory, behavior is undefined. Without this check, data/Free memory could be executed as instructions. | Specify: instruction fetch traps if tag ≠ Instruction. This also partially mitigates OS-C2 (user can only jump to Instruction-tagged memory). |
| OS-C4 | P3 | **No WRSR — status register not restorable** — RDSR is read-only. Sticky flags (saturation hit, modular wrap) from one process bleed into the next on context switch. A saturating overflow in Process A permanently affects Process B's computation correctness. | Add WRSR instruction (kernel-only) to write status register, or add clear-on-read semantics for sticky flags. |

### Significant Issues (10)

| ID | Program(s) | Issue | Proposed Fix |
|----|-----------|-------|--------------|
| OS-S1 | P1, P2 | **No PBASE instruction** — can't extract base address from pointer register into GPR. Kernel must maintain shadow bookkeeping of all addresses. kfree signature forced to be (addr_gpr) instead of (ptr_reg). | Add PBASE rd, pr instruction (kernel-only to prevent address leakage in user mode). |
| OS-S2 | P1, P2 | **MKPTR scratch overhead** — 5 instructions per pointer creation (2× KSWI + MKPTRI + 2× MCONVI cleanup). Scratch words become immutable (PtrA/PtrE) and must be cleaned before reuse. | Consider MKPTR-from-GPR variant: creates pointer directly from 2 GPR values (base, end) without touching memory. Saves all 5 overhead instructions. |
| OS-S3 | P3 | **MCONV cleanup dominates context switch** — 72 of 155 executed instructions (46%) are MCONV tag cleanup converting PtrA/PtrE→Free in save areas. | Add SPRI-clean variant that auto-converts spilled pointer words back to mutable after store. Or add bulk save/restore instructions. |
| OS-S4 | P3 | **Self-save problem** — the pointer register used for saving (p11) cannot save itself through itself. Must be reconstructed from constants after every context switch. | Document as kernel convention. Reserve p11 (or similar) as kernel-only save register with known address. |
| OS-S5 | P2 | **kfree requires address as separate argument** — pointer register alone insufficient for deallocation because PBASE doesn't exist. Forces all user code to maintain parallel address bookkeeping alongside capability pointers. | Consequence of OS-S1. Fix PBASE to fix this. |
| OS-S6 | P5 | **MCONVI and CONVERT matrix ambiguity** — CONVERT matrix has "—" for U.Int→Instruction. Whether kernel MCONVI bypasses the matrix for non-pointer types is not explicitly stated. Code loading depends on this working. | Explicitly specify: kernel MCONVI/MCONVG always performs Retag (or Zero from Free), bypassing the CONVERT matrix entirely. The matrix governs GPR CONVERT and user MCONV only. |
| OS-S7 | P5 | **Code patching requires 3 kernel operations per word** — Instruction tag is immutable system type. Modifying code: MCONVI→U.Int, KSWI new data, MCONVI→Instruction. Self-modifying code impossible in user mode. | By design (security feature). Document the pattern. Consider adding a KPATCH instruction for atomic modify+retag. |
| OS-S8 | P5 | **No ECALL/SYSCALL** — no user→kernel transition instruction. User code has no sanctioned path to request kernel services. Must JALR to known kernel addresses (which is itself a security hole per OS-C2). | Add ECALL instruction: traps to kernel exception handler at a fixed/configured address, passing syscall number in a GPR. |
| OS-S9 | P4 | **No pointer overlap detection** — kernel can't verify two pointer registers don't have overlapping ranges. The "no overlapping magisteria" invariant must be tracked in software with O(n) scanning. | Accept as software responsibility. Consider POVERLAP instruction for hardware-assisted comparison. |
| OS-S10 | P3 | **Singleton vs array save ambiguity** — SPRI writes 1 word for singletons, 2 for arrays. Kernel doesn't know register types at save time. Must pessimistically reserve 2 words per register and MTAG after save to determine actual size. | Always reserve 2 words. Standardize that SPRI of singleton writes PtrS at position N and leaves position N+1 unchanged (must be mutable type). |

### Minor Issues (5)

| ID | Program(s) | Issue | Proposed Fix |
|----|-----------|-------|--------------|
| OS-M1 | P1, P2 | **No KLWI/KLWG** — kernel has raw stores (KSWI/KSWG) but no raw loads. Must maintain master pointer for all kernel reads. Asymmetric. | Add KLWI/KLWG for raw kernel reads, or accept master pointer pattern as convention. |
| OS-M2 | P3 | **MCONV immediate too small** — 7-bit signed range (-64..+63). Save area offsets for Process B exceed this. Must use MCONVR with GPR offset. | Accept. MCONVR exists for this purpose. |
| OS-M3 | P5 | **Instruction construction is extremely tedious** — ~9 host instructions per target instruction word. Real bootloader would load from storage, but no I/O instructions in V0. | Accept for V0. Storage I/O is a V1 concern. |
| OS-M4 | P5 | **No FENCE.I** — no instruction cache coherence barrier. Modified code may not be fetched correctly on cached implementations. | Add FENCE.I for implementations with instruction caches. |
| OS-M5 | P4 | **PNARROW can't produce singleton** — narrowing to 1 word produces length-1 array, not singleton. Functionally equivalent but occupies 2 words if spilled. | Accept. Length-1 array works. |

### Cross-References to Stress-Test Issues

Several OS-level issues reinforce or extend stress-test findings:

| OS Issue | Stress-Test Issue | Relationship |
|----------|-------------------|-------------|
| OS-C4 (No WRSR) | S2 (Sticky flags can't be cleared) | Same root cause. OS context makes it a critical correctness bug. |
| OS-S1 (No PBASE) | — | New issue. Only visible at OS level. |
| OS-S2 (MKPTR scratch) | — | New issue. Per-allocation cost not visible in single-use stress tests. |
| OS-S3 (MCONV cleanup cost) | S5 (Pointer spill leaves immutable tags) | Same mechanism. OS context reveals 46% overhead. |
| OS-C2/C3 (JALR/fetch security) | M5 (Instruction fetch tag unspecified) | Escalated from Minor to Critical at OS level due to privilege boundary implications. |
| OS-S8 (No ECALL) | — | New issue. Only relevant when user/kernel separation exists. |
| OS-C1 (No revocation) | — | New issue. Not visible in single-process stress tests. |

### Summary Statistics

| Severity | Count | Fixed in this document | Requiring spec changes |
|----------|-------|----------------------|----------------------|
| Critical | 4 | 0 | 4 (OS-C1 through OS-C4) |
| Significant | 10 | 0 | 7 (OS-S1,S2,S3,S6,S7,S8,S9) |
| Minor | 5 | 0 | 2 (OS-M1, OS-M4) |
| **Total** | **19** | **0** | **13** |

Combined with the 18 issues from the stress tests (2 critical fixed, 16 remaining), the BROT ISA V0 has **33 total known issues** of which **6 are critical** (2 fixed, 4 new from OS-level analysis).
