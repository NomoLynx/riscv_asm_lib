# Compact optimization audit

Tracks [NomoLynx/test_code_data#19](https://github.com/NomoLynx/test_code_data/issues/19).
The inspected upstream baseline is `f2b6e8c9bc34e3a40d698095951fead77ac4c353`.
Changes belong to this library, not the compiler snapshot in `test_code_data/tooling`.

Normative sources:

- [C version 2.0, edition v20260120](https://docs.riscv.org/reference/isa/v20260120/unpriv/c-st-ext.html)
- [Zc version 1.0.0, edition v20260120](https://docs.riscv.org/reference/isa/v20260120/unpriv/zc.html)

## Metadata contract

`has_compact_equ()` is generated from `compact_equ` in
[`opcode.csv`](../src/r5asm/opcode/opcode.csv), using
[`opcode_col.csv`](../src/r5asm/opcode/opcode_col.csv). It describes a potential
architectural equivalent, not permission to compress every operand or proof
that a missing target instruction has been implemented.

Only existing CSV values are corrected. No rows are added, removed or reordered.
Instruction sizes and opcode/funct fields are unchanged.

## Existing C equivalents

All target rows below already exist. Integer C instructions require C/Zca;
single-precision transfers require RV32 F+C/Zcf; double-precision transfers
require D+C/Zcd. A target's legal operands and width must still be checked.

| Base row | Existing compressed equivalents | Width | Baseline flag |
| --- | --- | --- | --- |
| LUI | C.LUI | RV32/RV64 | FALSE |
| ADDI | C.ADDI, C.LI, C.NOP, C.MV, C.ADDI16SP, C.ADDI4SPN | RV32/RV64 | FALSE |
| ANDI | C.ANDI | RV32/RV64 | FALSE |
| SLLI | C.SLLI | RV32/RV64 | FALSE |
| SRLI | C.SRLI | RV32/RV64 | FALSE |
| SRAI | C.SRAI | RV32/RV64 | FALSE |
| ADD | C.ADD, C.MV | RV32/RV64 | FALSE |
| SUB | C.SUB | RV32/RV64 | FALSE |
| XOR | C.XOR | RV32/RV64 | FALSE |
| OR | C.OR | RV32/RV64 | FALSE |
| AND | C.AND | RV32/RV64 | FALSE |
| EBREAK | C.EBREAK | RV32/RV64 | FALSE |
| LW | C.LW, C.LWSP | RV32/RV64 | TRUE |
| SW | C.SW, C.SWSP | RV32/RV64 | FALSE |
| JAL | C.J; C.JAL with rd=x1 | both; C.JAL RV32 only | FALSE |
| JALR | C.JR, C.JALR | RV32/RV64 | FALSE |
| BEQ | C.BEQZ | RV32/RV64 | FALSE |
| BNE | C.BNEZ | RV32/RV64 | FALSE |
| ADDIW | C.ADDIW | RV64 | FALSE |
| ADDW | C.ADDW | RV64 | FALSE |
| SUBW | C.SUBW | RV64 | FALSE |
| LD | C.LD, C.LDSP | RV64 | TRUE |
| SD | C.SD, C.SDSP | RV64 | FALSE |
| FLW | C.FLW, C.FLWSP | RV32 | TRUE |
| FSW | C.FSW, C.FSWSP | RV32 | FALSE |
| FLD | C.FLD, C.FLDSP | RV32/RV64 | TRUE |
| FSD | C.FSD, C.FSDSP | RV32/RV64 | FALSE |

The 23 FALSE values above become TRUE. FLH becomes FALSE: neither audited
standard defines C.FLH. The four valid pre-existing TRUE values remain TRUE.

## Missing entries: human review only

These are architectural gaps, not additions authorized by this repair.
Existing base rows have their potential-equivalent flag corrected, but the
optimizer must leave these unsupported forms uncompressed.

| Missing mnemonic | Missing CSV key | Base instruction / expansion | Requirements |
| --- | --- | --- | --- |
| c.lbu | CLBU | LBU | Zcb, compact GPRs, byte offset 0..3 |
| c.lh | CLH | LH | Zcb, compact GPRs, offset 0 or 2 |
| c.lhu | CLHU | LHU | Zcb, compact GPRs, offset 0 or 2 |
| c.sb | CSB | SB | Zcb, compact GPRs, byte offset 0..3 |
| c.sh | CSH | SH | Zcb, compact GPRs, offset 0 or 2 |
| c.zext.b | CZEXTB | ANDI rd,rd,255 | Zcb, compact GPR |
| c.sext.b | CSEXTB | SEXTB | Zcb+Zbb, compact GPR |
| c.zext.h | CZEXTH | ZEXTH | Zcb+Zbb, compact GPR |
| c.sext.h | CSEXTH | SEXTH | Zcb+Zbb, compact GPR |
| c.zext.w | CZEXTW | ADDUW rd,rd,x0 | RV64 Zcb+Zba, compact GPR |
| c.mul | CMUL | MUL rd,rd,rs2 | Zcb+M/Zmmul, compact GPRs |
| c.not | CNOT | XORI rd,rd,-1 | Zcb, compact GPR |
| cm.push | CMPUSH | stack adjustment and register stores | Zcmp, register-list/stack contract |
| cm.pop | CMPOP | register loads and stack adjustment | Zcmp, register-list/stack contract |
| cm.popret | CMPOPRET | pop and return sequence | Zcmp, return/fault contract |
| cm.popretz | CMPOPRETZ | pop, clear a0 and return | Zcmp, return/fault contract |
| cm.mva01s | CMMVA01S | two saved-register-to-argument moves | Zcmp, saved-register mapping |
| cm.mvsa01 | CMMVSA01 | two argument-to-saved-register moves | Zcmp, distinct destinations |
| cm.jt | CMJT | table-based jump | Zcmt+Zicsr, configured executable JVT |
| cm.jalt | CMJALT | table-based jump and link | Zcmt+Zicsr, configured executable JVT |

The eleven additional existing base flags are LBU, LH, LHU, SB, SH, SEXTB,
SEXTH, ZEXTH, ADDUW, MUL and XORI. ANDI already has a C equivalent.
Zcmp/Zcmt are not single-instruction mnemonic substitutions and conflict with
Zcd encodings. They require separately reviewed implementation and profiles.

Legacy CLQ/CLQSP/CSQ/CSQSP and shift-by-64 rows are not a claim of current
RV32/RV64 support. This audit does not add RV128, Zcmop or Zclsd support.

## Safety and scope

- The process-global `OPTIMIZE_TO_COMPACT_CODE` opt-in remains FALSE by default.
  Enable it only for a target supporting the required C/F/D instructions.
- Conversion uses the configured `build_target`: 4 for RV32, 8 for RV64.
  The original public no-target conversion helper uses the RV64 default.
- Check register classes and numeric IDs, not spelling equality between ABI
  names and x-register names. Preserve source locations when operands move.
- Preserve signed immediates and byte offsets. Scale/scatter only during
  encoding, and emit exactly two bytes for supported compressed instructions.
- Keep relocation-bearing instructions and symbolic control transfers in
  their original form. This repair does not add branch relaxation.
- Do not introduce reserved encodings or substitute microarchitectural HINTs
  for ordinary instructions. Explicit legal HINT encodings remain distinct
  from optimization eligibility.
- Preserve mixed-width instruction/label offsets and mark actual compressed
  ELF output with EF_RISCV_RVC.

## Validation boundaries

Host regressions must exercise actual conversion and emission, including
all 42 current C spellings, signed/scaled boundaries, reserved controls,
width restrictions, ABI aliases, pseudo expansion and labels.

Native acceptance additionally requires library-built ELF executed in QEMU
with observed exit 42 or 142. GNU assembly is not library acceptance.

The baseline's wildcard dependency resolution selects Syn 3, incompatible
with the sibling macro source. Validation uses a compatible **local-only**
Pest/Syn lock; no dependency-crate source or upstream lockfile fix is included.
The baseline ELF writer emits ELF64 only. RV32 checks are encoding/host
evidence, not RV32 native acceptance. ELF32 support is a separate follow-up.

## Observed validation

Validated on 2026-11-28 UTC with the installed Rust 1.99.1 toolchain
(`cargo +stable`), the local compatible dependency resolution, and QEMU 8.2.2
user-mode `qemu-riscv64 -cpu rv64`. This does not claim a run under the
repository's separately pinned Rust 1.88 toolchain.

| Check | Observed result |
| --- | --- |
| CSV integrity | 318 unchanged rows/order; 35 compact-equivalent flag corrections; no other fields changed |
| Focused host regressions | 8 passed; all 42 current C spellings round-trip through conversion/parser/emitter |
| Signed CI fields | 512 exact register/immediate combinations checked across RV32/RV64 |
| Unchanged upstream host suite with staged fixtures | 80 passed, 3 BF16 failures |
| Repaired host suite with the identical fixture inputs | 88 passed, the same 3 BF16 failures |
| General RV64 fixture, optimization on | Library-built ELF64, 75 compressed instructions, 362 text bytes, guest exit 42 |
| Identical general RV64 fixture, optimization off | Library-built ELF64, 128 ordinary instructions, 512 text bytes, guest exit 42 |
| Literal forward/backward control fixture, optimization on | Library-built ELF64, 11 compressed instructions, 74 text bytes, guest exit 42 |

The general fixture initializes its memory arena and poisons load destinations
so skipped transfers cannot inherit expected values. Actual ELF inspection
confirmed two-/four-byte instruction boundaries and EF_RISCV_RVC only in the
compressed outputs. The measured general-fixture text reduction is 29.30%;
it is not a workload-wide compression guarantee.

These unchanged baseline failures remain outside this repair:

- `assembler::tests::build_bfloat16_support_unit_test`
- `meta::instruction_register_meta::tests::test_half_precision_conversion_selectors`
- `tests::test_bfloat16_conversion_encoding`

The exact BF16 expected/actual encodings and selector mismatch reproduce on
the unchanged baseline. No BF16 expectation, input or opcode field is changed.
Earlier attempts, actual ELF/source/builder hashes and raw exits are retained
in the local validation evidence, not embedded in immutable fixtures.
