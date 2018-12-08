III. Every Rule Has An Exception...

> This post is kind of an information dump; it'll be needed for the next chapter when we actually
> write some exception handlers.

When something sufficiently unusual happens, a processor will raise an exception for the kernel to
deal with. Each architecture handles them differently; Philipp Opperman has an
[excellent post](https://os.phil-opp.com/cpu-exceptions/) about how x86 handles exceptions through
its Interrupt Descriptor Table, and the Embedded Rust Book has a
[section](https://rust-embedded.github.io/book/start/exceptions.html) about using the `cortex-m`
crate family for handling ARM exceptions.

Both x86 and ARM use tables of function pointers at a fixed location in memory, with additional
bells and whistles for x86 such as interrupt stacks. MIPS takes a different approach, which
simplifies processor exception handling, but makes software a bit more complex.

In MIPS, you get a 32 instruction area to store an exception handler. The specific location and
handler types depend on the processor, but they are located near the beginning of ROM or RAM,
depending on a configuration bit.

This limited space means your handler will usually use a jump table to handle exceptions, and the
coprocessor registers are designed with this in mind: the exception code in COP0 register 13
occupies bits 7 to 2, which makes loading the relevant offset from a table of addresses a simple
AND instruction.

Additionally, MIPS uses a dedicated handler for a commonly occurring exception - "TLB Miss",
where the processor doesn't know how to map a virtual memory address to physical memory address -
which speeds up exception handling in that situation.

## Exception handling process

When an exception occurs:
- The processor switches to kernel mode.
- The exception code is written to part of COP0 register 13 (the exception cause register, or
Cause).
- The current program counter is written to COP0 register 14 (the exception program counter, or
EPC). If the exception happened in a branch delay slot (very rare), the previous instruction
program counter is written instead and a branch delay bit is set in Cause.
- If the exception is related to memory, the address that caused it is written to COP0 register
8 (the bad virtual address register, or BadVAddr).
- The processor then jumps to a fixed address in memory that depends on the exception and chip,
and starts executing code there.

Additionally:
- The EE sets an exception indicator bit in COP0 register 12 (processor status, or Status).
- The EE has multiple levels of exceptions: "level 1 exceptions" are the ones we're going to talk
about, but there are also "level 2 exceptions", which include processor reset, non-maskable
interrupts, performance counter overflow and debug exceptions.
- The IOP has a 3-level stack of interrupt/mode state. When an exception occurs, the current state
is pushed to the stack, and a kernel mode, interrupt disabled state is pushed. At the end of a 
exception handler, the interrupt/mode state is popped from the stack, restoring it to the state
before the exception.

> The only level 2 exception you need to care about is the Reset exception, and that's simply when
> your code starts executing, so you handle it anyway. The other three are reserved mostly for
> the PlayStation 2 development console, called the TOOL, where it would be useful to examine
> memory at a particular point in the program.

> Note that the processor does *not* save register state for you; you must do this yourself. For
> this purpose, MIPS ABIs reserve registers `$k0` and `$k1` for kernel exception bootstrapping.
> I suggest putting the kernel stack pointer in `$k0`, and using `$k1` as a scratch register.

Got all that? No? I'll keep going then.

## Exception codes

Speaking of those exception codes, here they are (for both CPUs):

- 0: Processor Interrupt (we'll cover these next chapter)
- 1: TLB Modified (\*)
- 2: TLB Miss (Load) / TLB Invalid (Load) (\*/\*\*)
- 3: TLB Miss (Store) / TLB Invalid (Store) (\*/\*\*)
- 4: Address Error (Load)
- 5: Address Error (Store)
- 6: Bus Error (Instruction)
- 7: Bus Error (Data)
- 8: System Call (SYSCALL instruction)
- 9: Breakpoint (BREAK instruction)
- 10: Reserved Instruction
- 11: Coprocessor Unusable
- 12: Arithmetic Overflow
- 13: Trap

\*: The TLB is not emulated by either PCSX2 or DobieStation, so you can safely stub them.
\*\*: TLB Miss exceptions go in their own handler to differentiate them from the others.

> Unlike x86, MIPS - at least the versions of MIPS we're using - has no double fault handler, so if
> you cause an exception in an exception handler, the processor will invoke the relevant exception
> handler again. If that's because you caused a bus error in the bus error exception handler, your
> code will infinitely loop. Be careful.

> Other MIPS processors would have an exception code for floating point exceptions, but the IOP
> does not have a floating point unit, and the EE's floating point unit does not raise exceptions.

## Exception handler addresses

Where these exception handlers go depends on the processor, and on a bit in Status called
"Bootstrap Exception Vectors" (BEV) which is used for exception handlers in the ROM.

> I will use the physical address conventions for these memory addresses. Remember that
> `0000'0000` is the start of RAM, and `1FC0'0000` is the start of ROM.

For the IOP:
- TLB Miss exceptions go to `1FC0'0100` in BEV mode, or `0000'0000` normally.
- All other exceptions go to `1FC0'0180` in BEV mode, or `0000'0080` normally.

For the EE:
- TLB Miss exceptions go to `1FC0'0200` in BEV mode, or `0000'0000` normally.
- Performance Counter Overflow exceptions go to `1FC0'0280` in BEV mode, or `0000'0080` normally.
- Debug exceptions go to `1FC0'0300` in BEV mode, or `0000'0100` normally.
- Interrupt exceptions go to `1FC0'0400` in BEV mode, or `0000'0200` normally.
- All other exceptions go to `1FC0'0380` in BEV mode, or `0000'0180` normally.

> You may note that the EE's ROM exception handlers conveniently occur after the IOP's ROM
> exception handlers. It's one of the (few) advantages of the EE being a custom CPU.
