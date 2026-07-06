# Contributing to MorganOS

Thank you for your interest in contributing to MorganOS!

## Getting Started

### Prerequisites
- Rust nightly (for `#![no_std]` support)
- `x86_64-unknown-none` target installed
- QEMU for testing (optional)

### Setup
```bash
# Clone the repository
git clone https://github.com/mason00398/MorganOS.git
cd MorganOS

# Install cross-compilation target
rustup target add x86_64-unknown-none

# Build
cd kernel && cargo build --release
cd ../bootloader && cargo build --release
```

## How to Contribute

### Reporting Bugs
- Use the **Bug Report** template
- Include steps to reproduce
- Mention kernel version and command used

### Suggesting Features
- Use the **Feature Request** template
- Explain the problem it solves
- Provide implementation ideas if possible

### Submitting Changes
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-fix`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-fix`)
5. Open a Pull Request

## Coding Standards

### Rust Guidelines
- Use `#![no_std]` for kernel code
- No `println!` — use `console::print!`
- All allocations must be in `alloc` modules
- Use `spin::Mutex` for synchronization
- Document all public functions

### File Organization
```
kernel/src/
├── main.rs          # Entry point
├── drivers/         # Hardware drivers
├── commands/        # CLI commands
└── ...
```

## Code Review

All submissions require review. We reserve the right to reject any contribution.

## Questions?
Open an issue or start a discussion.
