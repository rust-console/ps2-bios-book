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
so I will sometimes call them the "EE" and "IOP" respectively.

[very high level diagram here]

These CPUs both use the MIPS instruction set, though the R3051 uses 32-bit MIPS I, and the R5900
uses 64-bit MIPS III.

The R5900 is significantly faster than the R3051, so the R3051 is used to offload slow tasks
like input/output, and then notify the R5900 when something has happened through a communication
link.

## The boot process

The PS2 BIOS boot process at a very high level like this:

- Figure out if you are the R5900 (Emotion Engine CPU) or the R3051 (Input/Output Processor CPU).
- If you are the R5900:
  - Load and run the EE kernel.
  - Set up the processor and memory.
  - Set up the EE side of the communication link.
  - Synchronise with the R3051 through it.
- If you are the R3051:
  - Load and run the IOP kernel.
  - Set up the processor and memory.
  - Set up the IOP side of the communication link.
  - Synchronise with the R5900 through it.
- When both CPUs are set up and ready:
  - Play a pretty logo.
  - Check if there is a disc in the drive.
  - If there is, do something reasonable about it:
    - Run a PlayStation 2 game on the R5900.
    - Run a PlayStation 1 game on the R3051.
    - Play a DVD or CD.
    - Complain about an unrecognised disc.
  - If there isn't, load the BIOS interface.

And all of this in 4 megabytes of ROM. Quite impressive, isn't it?

Now, since we are running on emulators, we can remove parts of this: people will watch DVDs
and CDs with their media player of choice, and use a dedicated PlayStation 1 emulator for PS1
games. That gives us a little extra room for debugging or fancy graphics if we desire.

### Some bad news

As of time of writing, LLVM - the code generator behind `rustc` - does not support the MIPS I
instruction set, which the R3051 uses. This means you can't use Rust on the IOP at present,
unless you use MIPS II, which is a superset of the MIPS I instruction set. This carries risks of
your code randomly breaking because LLVM decided to use an instruction not supported by the R3051,
which I decided not to bother with. Still, I will document what the IOP Rust code *would* look
like, if it had native support.

Equally, the R5900 is a *quirky* chip which LLVM does not support directly, because it uses 64-bit
pointers, while the R5900 only has a 32-bit address space. Fortunately, we can pretend that the
R5900 is a 32-bit MIPS II CPU, which *is* supported by LLVM, and this is what we will do.
