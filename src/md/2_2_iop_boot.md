# II. IOP Booting: Hell on Earth

> This page is a work in progress; I'm not happy with it so far, but the information has to be
> put down somewhere before it can be made pretty.

The IOP kernel has a *unique*, modular architecture, based around relocatable ELF modules called
"IOP Relocatable Executables", or "IRX"es.

So, you could use your ELF parser for the bootloader for the IOP too, right?

Not without accomodating the quirks of the IRX format.

## The `.iopmod` section

First of all, the IRX uses a header that a common sense ELF parser would reject as invalid
(possibly intentionally): it uses the "Processor Specific" region of ELF types, as opposed to the
standardised formats between 1 and 4. This can be used to detect an IRX file.

Each IRX has a specific section - `.iopmod` (section number `0x70000080`) - which contains an IRX's
metadata, which looks like this:

```rust
/// `.iopmod` section
#[repr(C)]
pub struct Metadata {
    /// "module structure" pointer
    module: usize,
    /// Start address
    start: usize,
    /// Heap start
    heap: usize,
    /// Text section size
    text_size: usize,
    /// Data section size
    data_size: usize,
    /// BSS section size
    bss_size: usize,
    /// Major/minor version in binary coded decimal, e.g. 0x0102 for 1.2.
    version: u32,
    /// Module name
    name: [u8; 8],
}

/// The IOP module metadata, mostly filled in after compile.
#[link_section = ".iopmod"]
static IOPMOD: Metadata = Metadata {
    module: 0xDEADBEEF,
    start: 0xDEADBEEF,
    heap: 0xDEADBEEF,
    text_size: 0xDEADBEEF,
    data_size: 0xDEADBEEF,
    bss_size: 0xDEADBEEF,
    version: 0x0100,
    name: *b"Example\0",
};
```

Searching for this data requires combing through the ELF section table until you find an entry with
the name `.iopmod`. If you don't find this entry, it's probably an invalid IRX.

## The IRX export table

IRX modules contain an export table, which lists the functions that the IRX module provides. This
table looks like this:

```rust
/// An IRX export table.
#[repr(C)]
struct Export<N> {
    /// Magic number 0x41c0'0000, used for recognising the export table.
    magic: u32,
    /// Always zero. If this isn't zero, it's possibly a false positive.
    zero: u32,
    /// Version in binary-coded decimal.
    version: u32,
    /// Name of this module.
    name: [u8; 8],
    /// Addresses of exported functions, terminated with a zero reference.
    exports: [usize; N],
}
```

Searching for the export table involves searching for the export table magic number `41C0'0000`
(chosen because it isn't a valid MIPS instruction), and then parsing the table as above.

> I've encoded the export number into the struct, but I'm not sure how to parse a table into this.

## The IRX import table

IRX modules can contain arbitrarily many module import tables, which list the numbered functions
the module requires. This table looks like this:

```rust
/// An IRX function stub.
#[repr(C)]
struct FunctionStub {
    /// Jump instruction.
    jump: u32,
    /// Function number.
    func: u32,
}

/// An IRX import table.
#[repr(C)]
struct Import<N> {
    /// Magic number 0x41e0'0000, used for recognising the import table.
    magic: u32,
    /// Always zero. If this isn't zero, it's possibly a false positive.
    zero: u32,
    /// Version of the module in binary-coded decimal.
    version: u32,
    /// Name of the module.
    name: [u8; 8],
    /// Imported function stub, followed by an all-zero stub.
    stubs: [FunctionStub; N],
}
```

Each stub is a very minimal two-instruction "do nothing" function that looks like this in the
assembly:

```
03e00008        jr      $ra       # Return to caller
240000NN        li      $zero,NN  # Write to an always-zero register the function reference.
```

From there you will need to overwrite the `jr $ra` instruction with a `j <addr>` instruction.
The `j` opcode has its six most significant bits as `000010`, and the rest of the instruction is
the address to jump to. Since each MIPS instruction is four-byte aligned, the address is
right-shifted by two bits, giving a total of a 2^28 byte jump address.

As an example, to jump to the address `0321'1234`, you shift right the address by two bits to get
`000C'848D`, AND the address with `07FF'FFFF`, and then OR in `0800'0000` to produce `080C'848D`.
This is then used to overwrite the `jr $ra`/`03E0'0008` instruction.

The index of the function address is given in the least significant byte of the following
`li $zero, NN` instruction, for the module listed in the import table's module name.

> I'm well aware this is quite messy and possibly explained badly.
