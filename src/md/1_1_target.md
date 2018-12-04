# I. Targets: Ready... Aim...

If we tried to compile something right now, Rust would probably spit out an x86 ELF/Mach-O/PE
executable. It wouldn't run for a few reasons:

- The PlayStation 2 doesn't understand these formats; it'd just try to execute it as a binary blob
and trip up on their magic numbers.
- Even if it did understand these files, they would be targeted for the wrong architecture.

Rust needs to be told how to emit code for the PS2. For that we need to define a *target file*.

## Target files

`rustc` includes its own target files for each architecture; you can look at the available targets
with the following command:

```
rustc --print target-list
```

Each target file is in a JSON format, that you can inspect with the following command:

```
rustc -Z unstable-options --print target-spec-json --target $target
```

For the EE, we'll start off the `mipsel-unknown-linux-gnu` target, which looks like this:

```json
{
  "arch": "mips",
  "cpu": "mips32r2",
  "data-layout": "e-m:m-p:32:32-i8:8:32-i16:16:32-i64:64-n32-S64",
  "dynamic-linking": true,
  "env": "gnu",
  "executables": true,
  "features": "+mips32r2,+fpxx,+nooddspreg",
  "has-elf-tls": true,
  "has-rpath": true,
  "is-builtin": true,
  "linker-flavor": "gcc",
  "linker-is-gnu": true,
  "llvm-target": "mipsel-unknown-linux-gnu",
  "max-atomic-width": 32,
  "os": "linux",
  "position-independent-executables": true,
  "pre-link-args": {
    "gcc": [
      "-Wl,--as-needed",
      "-Wl,-z,noexecstack"
    ]
  },
  "relro-level": "full",
  "target-c-int-width": "32",
  "target-endian": "little",
  "target-family": "unix",
  "target-pointer-width": "32",
  "vendor": "unknown"
}
```

This contains a lot of unnecessary (and inaccurate) things, such as this target being for MIPS32r2.
Let's change it a bit.

```json
{
  "arch": "mips",
  "cpu": "mips2",
  "data-layout": "e-m:m-p:32:32-i8:8:32-i16:16:32-i64:64-n32-S64",
  "dynamic-linking": false,
  "executables": true,
  "features": "+mips2",
  "linker": "mipsel-none-elf-ld",
  "linker-flavor": "ld",
  "llvm-target": "mipsel-none-elf",
  "llvm-args": "-mxgot",
  "max-atomic-width": 32,
  "os": "none",
  "panic-strategy": "abort",
  "position-independent-executables": false,
  "relro-level": "full",
  "soft-float": true,
  "target-c-int-width": "32",
  "target-endian": "little",
  "target-family": "unix",
  "target-pointer-width": "32",
  "vendor": "unknown"
}
```

Some important changes:

- `"cpu": "mips2"` - We need LLVM to target the MIPS II instruction set.
- `"soft-float": true` - The R5900 has a single-float FPU, which LLVM has quite a few bugs with, so
we pretend it doesn't have one to work around them.
- `"linker": "mipsel-none-elf-ld"`/`"linker-flavor": "ld"` - We will need to use the GNU linker to
build this, because LLD seems to have a nasty habit of optimising out our code.

> The correct settings here would be `"cpu": "mips3"` for the R5900 and `"cpu": "mips1"` for the
> R3051, but as mentioned previously, LLVM support for these needs to mature.

I will refer to this target file as `ee.json`, and you should put it in your crate/workspace root.

## Building cross binutils

> This isn't directly Rust related, but we need a linker for our code, and binutils has proven to
> be very reliable in my experiments. One day, I hope LLD is stable enough to use.

You'll need a GNU-compatible host C compiler (`gcc`/`clang` will do fine, but not MSVC++), and a
copy of the [binutils source](https://ftp.gnu.org/gnu/binutils/). I'm using binutils 2.31.

After extracting your source, you can build it with a standard-ish method:

```
mkdir build
cd build
../configure --target="mipsel-none-elf" 
make
sudo make install
```

And then you can test it installed correctly by running `mipsel-none-elf-ld --version`.

## One final thing

To get `rustc` to build for a native target, we use `cargo build`; but Cargo doesn't currently
work well with cross-compilation, because it expects the various libraries to be already installed.

> This may change with std-aware Cargo.

We can get around this through the `cargo-xbuild` wrapper, which you can grab with a simple `cargo
install cargo-xbuild`. This allows you to build your code with `cargo xbuild --target ee.json`, and
also wraps Clippy.

> Don't forget the `.json` for `--target`; I had some problems where it would build your code fine
> without it (i.e. `--target ee`), but fail to build any library crates your code depended on.
