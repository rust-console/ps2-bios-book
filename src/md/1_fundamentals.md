# I. Putting the "Mental" in "Fundamentals"

Welcome to this blog about writing a BIOS for PlayStation 2 emulators in Rust.

By studying this process, you should get a greater appreciation of how much effort goes on behind
the scenes to boot your computer.

## Legals

This book is licensed under a Creative Commons Attribution-ShareAlike 4.0 International License.
The source code is licensed under the terms of the GNU General Public License version 3.0 or (at
your option) any later version.

## The PS2 architecture

The PlayStation 2 is an unusually laid out computer compared to the Intel x86 PC or ARM
phone/tablet you're probably reading this on.

It has two CPUs in it, the R5900 contained in the Emotion Engine chip (this chip contains a lot
of other processors, which is why I will call the CPU itself the R5900), and the R3051 contained
in the Input/Output Processor. "Emotion Engine" and "Input/Output Processor" are quite long names,
so I will call them the "EE" and "IOP" respectively.

It also has a custom GPU called the Graphics Synthesizer, which I will call the "GS".

These are connected together like this:

![high level diagram](/ps2-bios-book/svg/1_fundamentals_1.svg)

These CPUs both use the MIPS instruction set, though the IOP uses 32-bit MIPS I, and the EE
uses 64-bit MIPS III.

The EE is significantly faster than the IOP, so the IOP is used to offload slow tasks
like input/output, and then notify the EE when something has happened through a communication
link.

Each of these chips has its own embedded memory; the EE has 32 MiB of system memory, the IOP has
its own 2 MiB of memory, and the GS has 4 MiB of embedded memory.

## The boot process

The PS2 BIOS boot process at a very high level like this:

- Both CPUs start from same BIOS ROM.
- Figure out if you are the EE (Emotion Engine CPU) or the IOP (Input/Output Processor CPU).
- If you are the EE:
  - Load and run the EE kernel.
  - Set up the processor and memory.
  - Set up the EE side of the communication link.
  - Synchronise with the IOP through it.
- If you are the IOP:
  - Load and run the IOP kernel.
  - Set up the processor and memory.
  - Set up the IOP side of the communication link.
  - Synchronise with the EE through it.
- When both CPUs are set up and ready:
  - Play a pretty logo.
  - Check if there is a disc in the drive.
  - If there is, do something reasonable about it:
    - Run a PlayStation 2 game on the EE.
    - Run a PlayStation 1 game on the IOP.
    - Play a DVD or CD.
    - Complain about an unrecognised disc.
  - If there isn't, load the BIOS interface.

And all of this in 4 megabytes of ROM. Quite impressive, isn't it?

Now, since we are running on emulators, we can remove parts of this: people will watch DVDs
and CDs with their media player of choice, and use a dedicated PlayStation 1 emulator for PS1
games. That gives us a little extra room for debugging or fancy graphics if we desire.

### Some bad news

As of time of writing, LLVM - the code generator behind `rustc` - does not support the MIPS I
instruction set, which the IOP uses. This means you can't use Rust on the IOP at present,
unless you use MIPS II, which is a superset of the MIPS I instruction set. This carries risks of
your code randomly breaking because LLVM decided to use an instruction not supported by the IOP,
which I decided not to bother with. Still, I will document what the IOP Rust code *would* look
like, if it had native support.

Equally, the EE is a *quirky* chip which LLVM does not support directly, because it uses 64-bit
pointers, while the EE only has a 32-bit address space. Fortunately, we can pretend that the
EE is a 32-bit MIPS II CPU, which *is* supported by LLVM, and this is what we will do.
