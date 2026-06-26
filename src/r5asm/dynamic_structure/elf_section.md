### PLT Relocation Entry

The **PLT Relocation Entry** describes a single relocation record in the `.rela.plt` section of an ELF binary. It tells the dynamic linker exactly how to patch the GOT.PLT at runtime when resolving a lazily-bound external function.

Each entry has three responsibilities:

- **Where** to patch — `r_offset` points to the GOT.PLT slot that needs to be filled with the resolved function address.
- **What** to do — `r_info` encodes both the relocation type (e.g. `R_X86_64_JUMP_SLOT`) and the symbol index in the symbol table, identifying which external function is being resolved.
- **By how much** to adjust — `r_addend` provides a signed offset applied during relocation calculation, allowing the linker to compensate for things like PC-relative addressing.

In typical PLT lazy binding, the first time a function is called the PLT stub jumps through the GOT.PLT slot (which initially points back into the PLT), triggering the dynamic linker to walk the `.rela.plt` entries, find the matching symbol, resolve it, and write the real address into the GOT.PLT slot — so subsequent calls go directly to the function.

| Field | Type | Size | Description |
| ------- | ------ | ------ | ------------- |
| r_offset | u64 | 8 bytes | Offset in the GOT.PLT section where the relocation applies |
| r_info | RelaRInfo | varies | Relocation type and symbol index |
| r_addend | i64 | 8 bytes | Signed addend; can be negative (e.g. PC-relative compensation) |

### ELF PLT Relocation Table

The ELF PLT (Procedure Linkage Table) relocation table is used by the dynamic linker to resolve function calls to external/shared library symbols at runtime. Each entry in this table describes how a PLT slot should be patched so that indirect function calls are correctly redirected to their resolved addresses. This mechanism is essential for supporting lazy binding and dynamic symbol resolution in ELF-based systems.

| Field | Type | Size | Description |
| ------- | ------ | ------ | ------------- |
| virtual_address | u64 | 8 bytes | Virtual address of the PLT relocation table in memory |
| offset | u64 | 8 bytes | File offset of the PLT relocation table |
| entries | Vec<[PLTRelocationEntry](#plt-relocation-entry)> | varies | List of PLT relocation entries |
| alignment | Alignment | varies | Alignment requirement for the section |
| link | u32 | 4 bytes | Section index of the associated symbol table |
| info | u32 | 4 bytes | Section index of the section to which relocations apply |

### ELF PLT Section

The ELF PLT (Procedure Linkage Table) section contains the executable stubs used to perform indirect function calls in dynamically linked binaries. It works together with the Global Offset Table (GOT) to support runtime symbol resolution by the dynamic linker. Each PLT entry typically jumps through a GOT slot, which is updated during lazy binding or eager relocation. This section also stores metadata required for code generation and address patching during linking and loading, such as base addresses and offsets that allow the linker and runtime loader to correctly relocate and patch function call targets.

| Field | Type | Size | Description |
| ------- | ------ | ------ | ------------- |
| code | Vec<u8> | varies | PLT executable code bytes |
| got_plt_base | u64 | 8 bytes | Base address of the .got.plt section |
| plt_base_addr | u64 | 8 bytes | Virtual address where .plt starts in memory |
| next_got_offset | u64 | 8 bytes | Next available GOT slot offset, after the reserved entries |
| offset | u64 | 8 bytes | File offset of the PLT section |
| got_entries | Vec<u64> | varies | GOT entries used for binary generation |
| alignment | Alignment | varies | Alignment requirement for the section, ensuring correct placement in memory for architecture-safe and efficient access. |

### ELF String Table

The ELF string table is a dedicated section that stores null-terminated strings used throughout the ELF file, such as symbol names, section names, and debug labels. Instead of embedding raw strings repeatedly in multiple places, ELF uses offsets into this table to keep binaries compact and easier to relocate. During linking and loading, other sections reference this table to resolve human-readable names from indices.

| Field | Type | Size | Description |
| ------- | ------ | ------ | ------------- |
| virtual_address | u64 | 8 bytes | virtual address of the string table in memory |
| offset | u64 | 8 bytes | virtual address of the string table in memory |
| data | Vec<String> | varies | List of strings stored in this table. Internally these are usually stored as a contiguous null-terminated byte buffer and referenced via offsets rather than direct pointers. |
| alignment | Alignment | varies | Alignment requirement for the section, ensuring correct placement in memory for architecture-safe and efficient access. |

### ELF GNU Hash Section

The ELF GNU hash section is a highly optimized symbol lookup structure used by the dynamic linker to accelerate symbol resolution in shared libraries. It improves lookup performance compared to traditional SysV hash tables by combining a Bloom filter with bucketed hash chains, allowing most non-matching symbols to be rejected quickly before expensive comparisons are performed. This structure is critical for fast dynamic linking in large binaries with many symbols.

| Field | Type | Size | Description |
| ------- | ------ | ------ | ------------- |
| virtual_address | u64 | 8 bytes | virtual address of the string table in memory |
| offset | u64 | 8 bytes | virtual address of the string table in memory |
| nbuckets | u32 | 4 bytes | Number of hash buckets used to group symbols for lookup. |
| symoffset | u32 | 4 bytes | Index of the first symbol in the dynamic symbol table that participates in GNU hash lookup. |
| bloom_size | u32 | 4 bytes | Size of the Bloom filter array used for fast probabilistic rejection of non-matching symbols. See [Bloom Filter (Wikipedia)](https://en.wikipedia.org/wiki/Bloom_filter?utm_source=chatgpt.com). |
| bloom_shift | u32 | 4 bytes | Shift constant used in the Bloom filter hashing algorithm to spread bits across the filter and reduce collisions. Part of the GNU hash optimization scheme [GNU Hash ELF Overview](https://flapenguin.me/elf-dt-hash?utm_source=chatgpt.com). |
| bloom_filter | Vec<u64> | varies | Bit array used as a Bloom filter to quickly rule out symbols before bucket and chain lookup. It provides probabilistic filtering to reduce expensive string comparisons [Bloom Filter Concept](https://en.wikipedia.org/wiki/Bloom_filter?utm_source=chatgpt.com). |
| buckets | Vec<u32> | varies | Array of bucket heads that map hash values to linked symbol chains used during lookup. |
| chains | Vec<u32> | varies | Hash chain entries used to resolve collisions within each bucket during final symbol lookup [ELF GNU Hash Details](https://flapenguin.me/elf-dt-hash?utm_source=chatgpt.com). |
| alignment | Alignment | varies | Alignment requirement for the section, ensuring correct placement in memory for architecture-safe and efficient access. |

### ELF Dynamic Symbol Table

| Field | Type | Size | Description |
| ------- | ------ | ------ | ------------- |
| virtual_address | u64 | 8 bytes | virtual address of the string table in memory |
| offset | u64 | 8 bytes | virtual address of the string table in memory |
| entries | Vec<DynamicSymbolEntry> | varies | store the dynamic symbol entry |
| alignment | Alignment | varies | Alignment requirement for the section |

## ELF GNU Version Section

The ELF GNU version section (.gnu.version) stores version indices for entries in the dynamic symbol table. Each version entry corresponds to one symbol in .dynsym and indicates the version of that symbol required or provided by a shared object. During dynamic linking, the loader uses this information together with the GNU version definition and version requirement sections to ensure that symbols are resolved to compatible library versions, preventing ABI incompatibilities.

| Field | Type | Size | Description |
| ------- | ------ | ------ | ------------- |
| virtual_address | u64 | 8 bytes | virtual address of the string table in memory |
| offset | u64 | 8 bytes | virtual address of the string table in memory |
| entries | Vec<GnuVersionEntry> | varies | List of GNU version entries. Each entry contains a version index corresponding to a symbol in the dynamic symbol table (`.dynsym`). The dynamic linker uses these indices to verify symbol version compatibility during symbol resolution. See [GNU Symbol Versioning Overview](https://maskray.me/blog/2020-11-26-all-about-symbol-versioning?utm_source=chatgpt.com) and [LSB Symbol Versioning Specification](https://refspecs.linuxfoundation.org/LSB_3.1.1/LSB-Core-generic/LSB-Core-generic/symversion.html?utm_source=chatgpt.com). |
| alignment | Alignment | varies | Alignment requirement for the section |

### ELF GNU Version Required Section

The ELF GNU Version Required section (.gnu.version_r) describes the versioned symbols that an ELF object requires from its shared library dependencies. Each record corresponds to one required shared library (such as libc.so.6) and contains one or more auxiliary version entries specifying the exact symbol versions that must be provided. During program loading, the dynamic linker verifies that each dependency exports the required versions before symbol resolution proceeds, helping prevent ABI incompatibilities.

| Field | Type | Size | Description |
| ------- | ------ | ------ | ------------- |
| version | u16 | 2 bytes | Version of the Elfxx_Verneed structure format. This value is currently always 1 as defined by the GNU ELF specification. |
| cnt | u16 | 2 bytes | Number of auxiliary version entries (Elfxx_Vernaux) associated with this required shared library. |
| file_offset | u32 | 4 bytes | Offset into the dynamic string table (.dynstr) containing the name of the required shared library (for example, libc.so.6). |
| aux_offset | u32 | 4 bytes | Offset, relative to the start of this header, to the first auxiliary version entry (Elfxx_Vernaux). This is typically the size of the header itself. |
| vn_next | u32 | 4 bytes | Offset to the next version required header (Elfxx_Verneed). A value of 0 indicates that this is the last required library entry. |
| required_entries | Vec<GnuVersionRequiredAux> | varies | List of auxiliary version entries describing the individual symbol versions required from this shared library. The number of entries is determined by `cnt`. Each entry typically references a version name stored in `.dynstr` and is matched by the dynamic linker during symbol resolution. See [GNU Symbol Versioning Overview](https://maskray.me/blog/2020-11-26-all-about-symbol-versioning?utm_source=chatgpt.com) and [LSB Symbol Versioning Specification](https://refspecs.linuxfoundation.org/LSB_3.1.1/LSB-Core-generic/LSB-Core-generic/symversion.html?utm_source=chatgpt.com). |
