# BROT ASM Synatx

* Each line contains exactly one instruction.
* Instructions are in all caps
* Each instruction is seperated from a space by its args. The args may have any amount of white space between them but also must have commas between each.
* Anything after `;` is ignored. Any line *without* a `;` is ignored
* Other files are included with a #include at the top. You may either specify a path, or the path is impliclity understood to be the main file.
* The main file is the only compilition target. All other files are recursively included via includes. There is no linking. BROT ASM programs are simply giant single files. 
* Immediate values *must* be annonated with a type. `ADDI r2, r1, 56:uint` No types means no compilation. Labels are case-insensitive (`56:UINT` is the same as `56:uint`). Every label except `ptr` can be used as the target type of `CONVERT`.
  * Bools are written with TRUE and FALSE
* Branches follow labels or line numbers. Line numbering ignores blank spaces.
* Immediate values outside of the 22 bit signed/unsigned range error.

  ```
  ADDI r2, r3, 45.6:Q24.16  

  ADD r4, r2, r3
  ```

  these are one word spaces apart.

  * Loops are written with `LABEL;`


## Type Labels

  | Label    | Type        | Description                                       | Usable on immediates |
  |----------|-------------|---------------------------------------------------|----------------------|
  | `free`   | Free        | Blank memory; must be converted before use        | No                   |
  | `uint`   | U.Int       | Unsigned integer, saturates on overflow           | Yes                  |
  | `umod`   | ModU        | Unsigned integer, wraps on overflow               | Yes                  |
  | `pku`    | PkU         | Packed unsigned (no packed operations in V0)      | No                   |
  | `bool`   | Bool        | 0 or 1; any nonzero immediate becomes 1           | Yes                  |
  | `sint`   | S.Int       | Signed integer, saturates on overflow             | Yes                  |
  | `smod`   | ModS        | Signed integer, wraps on overflow                 | Yes                  |
  | `q32.8`  | Q32.8       | Signed fixed point, 8 fractional bits             | Yes                  |
  | `q24.16` | Q24.16      | Signed fixed point, 16 fractional bits            | Yes                  |
  | `q8.32`  | Q8.32       | Signed fixed point, 32 fractional bits            | Yes                  |
  | `pks`    | PkS         | Packed signed (no packed operations in V0)        | No                   |
  | `instr`  | Instruction | Executable instruction word; immutable            | No                   |
  | `ptr`    | Ptr         | Bounds-checked pointer, held in pointer registers | No                   |
  | `inert`  | Inert       | Immutable, kernel-managed; unknown tags are Inert | No                   |
