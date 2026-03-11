### PLT Relocation Entry

The **PLT Relocation Entry** describes a single relocation record in the `.rela.plt` section of an ELF binary. It tells the dynamic linker exactly how to patch the GOT.PLT at runtime when resolving a lazily-bound external function.

Each entry has three responsibilities:

- **Where** to patch — `r_offset` points to the GOT.PLT slot that needs to be filled with the resolved function address.
- **What** to do — `r_info` encodes both the relocation type (e.g. `R_X86_64_JUMP_SLOT`) and the symbol index in the symbol table, identifying which external function is being resolved.
- **By how much** to adjust — `r_addend` provides a signed offset applied during relocation calculation, allowing the linker to compensate for things like PC-relative addressing.

In typical PLT lazy binding, the first time a function is called the PLT stub jumps through the GOT.PLT slot (which initially points back into the PLT), triggering the dynamic linker to walk the `.rela.plt` entries, find the matching symbol, resolve it, and write the real address into the GOT.PLT slot — so subsequent calls go directly to the function.

| Field | Type | Size | Description |
|-------|------|------|-------------|
| r_offset | u64 | 8 bytes | Offset in the GOT.PLT section where the relocation applies |
| r_info | RelaRInfo | varies | Relocation type and symbol index |
| r_addend | i64 | 8 bytes | Signed addend; can be negative (e.g. PC-relative compensation) |

### ELF PLT Relocation Table

| Field | Type | Size | Description |
|-------|------|------|-------------|
| virtual_address | u64 | 8 bytes | Virtual address of the PLT relocation table in memory |
| offset | u64 | 8 bytes | File offset of the PLT relocation table |
| entries | Vec<[PLTRelocationEntry](#plt-relocation-entry)> | varies | List of PLT relocation entries |
| alignment | Alignment | varies | Alignment requirement for the section |
| link | u32 | 4 bytes | Section index of the associated symbol table |
| info | u32 | 4 bytes | Section index of the section to which relocations apply |

### ELF PLT Section

| Field | Type | Size | Description |
|-------|------|------|-------------|
| code | Vec<u8> | varies | PLT executable code bytes |
| got_plt_base | u64 | 8 bytes | Base address of the .got.plt section |
| plt_base_addr | u64 | 8 bytes | Virtual address where .plt starts in memory |
| next_got_offset | u64 | 8 bytes | Next available GOT slot offset, after the reserved entries |
| offset | u64 | 8 bytes | File offset of the PLT section |
| got_entries | Vec<u64> | varies | GOT entries used for binary generation |
| alignment | Alignment | varies | Alignment requirement for the section |
