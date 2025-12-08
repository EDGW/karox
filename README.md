# karox

karox is a operating system kernel that runs on risc-v and loongarch processors.

It's my experimental project, and it's still in development.

# Build

Make sure you have a rust `nightly` version enabled, and targets `riscv64gc-unknown-none-elf` & `loongarch64-unknown-none` installed before you build.

- `make all` Build kernel for all builds
- `make [target]` Build kernel for a specific build
- `make build` Build kernel for the default build
- `make clean` Clean the workspace

Build properties are configured in `config.mak`.
