# II. EE Booting: An Emotional Experience

The EE kernel gets a megabyte of reserved memory - specifically from `0000'0000` to `0010'0000`. In
that megabyte, you need to store the library of system calls that the kernel provides, and act on
outside events.

To start the EE kernel we need to first load it from ROM, putting specific parts of the ROM in
particular places in RAM.

If only there was a convenient, standard format for loading data into a specific location in
memory.

## Magical elves

Fortunately, there is: the Executable and Linkable Format, or ELF.

> The following paragraph is a lie that I will change at some point: I will need to discuss how to
> parse ELF, because the IOP kernel depends on it.

I'm not going to go into much detail about how to implement ELF loading, because the
[specification](https://refspecs.linuxfoundation.org/elf/elf.pdf) is easy enough to understand,
plus there are [guides](https://wiki.osdev.org/ELF#Loading_ELF_Binaries) for writing homebrew ELF
loaders, and even [crates](https://crates.io/search?q=elf) for it.

But all of these must get their ELF data from somewhere.

The naive solution is to append the ELF to the ROM at a fixed address. A more efficient format
called "ROMDIR" will be discussed later; PCSX2 requires that format to recognise your ROM as valid.
Fortunately, DobieStation is not as picky, and we will use that for testing.

## Stick to the script

The ELF that you compile needs certain functions at specific addresses to handle MIPS exceptions.
I will explain them later, but for now, your kernel should leave the area from `0000'0000` until
`0000'0280` clean. To do this, we need to tell the linker not to put data there through a linker
script.

> We also need to do this for the bootloader, before you ask.

A linker script contains two main parts: we need to tell the linker where we start executing code
from, and where to put code/data.

We tell the linker where to execute code by telling it which symbol to treat as the start of the
program. This is the `START(<symbol>)` command.

We tell the linker where to put code/data using the `SECTIONS` command. `SECTIONS` is a block of,
well, program sections, such as `.text` (your code), `.data` (global variables) and `.bss` (zeroed
global variables, taking up no binary space).

The easiest solution is to just tell the linker to offset your code by `0x280` bytes.

```ld
/* Set the start point to _start */
START(_start);

/* The sections of the program */
SECTIONS {
    /* "section : address" means "start section at address" */
    .text : 0x00000000 {
        /* 
         * "." refers to the current memory pointer. In this case, ". = foo" sets the current
         * memory pointer to `foo + address` (see above).
         */
        . = 0x280;

        /* Then include all symbols in .text and its subsections. */
        *(.text .text.*);
    }

    /* Without the address, the linker just aligns it after the end of the previous section. */
    .data : {
        *(.data .data.*);
    }

    .bss : {
        *(.bss .bss.*);
    }
}
```

And then we can use this for a very, very simplistic program.
{{#playpen ../rs/2_1_ee_boot.rs}}

