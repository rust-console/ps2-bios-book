# II. Wellington Bootloader

> This chapter to written to show you what *would* be possible with compiler support. However, it
> does talk about the architecture of MIPS, and you'll need it later.

Remember the [steps to boot the console](/ps2-bios-book/md/1_fundamentals.md#the-boot-process)? Let's put
those into action.

## "Both CPUs start from the same BIOS ROM"

Both CPUs start at the same fixed address in the virtual memory space: `BFC0'0000`, the start of
BIOS ROM.

> Both CPUs effectively mask the virtual address with `1FFF'FFFF`. This results in `BFC0'0000`
> being mapped to `1FC0'0000` by both chips; the two addresses are interchangeable and reference the
> same data. As you will see later, `BFC0'0000` seems to have practical problems in binutils (it
> seems like the pointer is treated as signed, leading to strange behaviour), so I will often refer
> to `1FC0'0000`, which does not have these issues.

> Also worth mentioning is that in MIPS, everything with the virtual `8000'0000` bit set is kernel
> memory space and everything without it set is user memory. This is a mostly theoretical
> distinction when working with a bare metal MIPS target like the PlayStation 2 though, because the
> console is almost always in kernel mode when playing a game.

## "Figure out if you are the EE or IOP"

MIPS has a 4 coprocessor interface baked into the ISA; this provides a standard method of accessing
custom features of each chip without needing a new assembler for each of them. These coprocessors
were left defined but unspecified, however MIPS has conventions for them:

- Coprocessor 0 (COP0) is the system control coprocessor; essentially a set of registers containing
processor state. Because it contains such important information, it is mandatory and found in all
MIPS processors.
- Coprocessor 1 (COP1) is the floating point unit, and all floating point math goes through it. The
IOP does not have a floating point unit, and the EE's floating point unit is very nonstandard, which
is why our code doesn't use them.
- Coprocessor 2 (COP2) is left for custom accelerators, and both the EE and IOP use them.
- Coprocessor 3 was originally for more custom accelerators, but it got repurposed into more
floating point operations. Neither CPU has this coprocessor.

We will need COP0 for this, and it too has conventions for register names and contents, although
not specifically what the register contains. The specific register we need is COP0 register 15,
which has the mnemonic "PRid" for "Processor Identification".

The PRid register looks like ths in both CPUs:

_[fancy diagram showing the least significant byte being marked "revision number" and the second
least significant byte being marked "model number"; the other bytes are "reserved"]_

On the EE, the model number is `0x2E`, while on the IOP the model number is `0x00` (it was a much
earlier core), which means we just need to check what the model number field is and jump to the
appropriate function.

To get a register from coprocessor 0, we use `mfc0 <dest reg> <cop0 reg>`.

So we could write a function that looks like this:

{{#playpen ../rs/2_bootloader.rs}}

> If we could compile code for the IOP, anyway. Note that it won't compile on the Rust Playground
> because the Playground runs on x86.

## "Load and run the EE/IOP kernel"

Here's where things become a little painful.
