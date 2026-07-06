# RunST X Kernel

<div align="center">

**A Personal Operating System Kernel written in Rust**

[![Language](https://img.shields.io/badge/language-Rust-blue.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Status](https://img.shields.io/badge/status-v0.4-yellow.svg)]()
[![GitHub stars](https://img.shields.io/github/stars/mason00398/runst_x?style=social)]()

</div>

---

## 📋 Overview

RunST X is a personal operating system kernel built from scratch in Rust. It implements core OS components including process scheduling, memory management, networking, storage, and input devices — all in a monolithic kernel architecture.

## 🏗️ Architecture

```
┌─────────────────────────────────────────────┐
│              User Space (Commands)            │
│  ls  cat  mkdir  ping  net  ps  vmstat  ... │
├─────────────────────────────────────────────┤
│              Kernel Core                      │
│  main.rs  panic.rs  console.rs  pci.rs       │
├──────────┬──────────┬───────────┬────────────┤
│ Drivers  │Drivers   │ Drivers   │ Drivers    │
│ AHCI     │ FAT32   │ Keyboard  │ VGA        │
│ RTL8139  │ Memory  │ Process   │ RTC        │
│ TCP/IP   │ VM      │ Net       │            │
├──────────┴──────────┴───────────┴────────────┤
│              Bootloader                       │
│  SHA256 verification  PE/COFF validation      │
└─────────────────────────────────────────────┘
```

## ✨ Features

### Core Systems
- **Memory Management** — 16MB heap allocator with allocation tracking
- **Virtual Memory** — Page table management with page permissions
- **Process Scheduler** — Cooperative multitasking via `fringe` generators
- **Process Control** — fork(), terminate(), parent-child relationships
- **IPC & Sync** — Message queues, semaphores, mutex locks

### Networking
- **RTL8139 Ethernet** — Full NIC driver with MAC reading and ARP cache
- **ICMP Protocol** — Ping command with echo request/reply
- **TCP Protocol** — Socket API, connection state machine
- **UDP Protocol** — Datagram sockets
- **DNS Cache** — Domain name resolution with caching

### Storage & Input
- **FAT32 Filesystem** — Directory reading and file access
- **AHCI Storage** — SATA controller initialization and sector I/O
- **PS/2 Keyboard** — Scan code processing with Shift support

### System Tools
- **PCI Enumeration** — Recursive bus scanning with bridge support
- **Boot Verification** — SHA256 hash and PE/COFF validation
- **CLI Console** — Tab completion, command history, arrow keys

## 📁 Project Structure

```
runst_x/
├── bootloader/
│   ├── Cargo.toml
│   └── src/main.rs          # Bootloader with SHA256 verification
├── kernel/
│   ├── Cargo.toml            # Dependencies: fringe, spin, bytemuck...
│   ├── build.rs
│   └── src/
│       ├── main.rs           # Kernel entry point
│       ├── panic.rs          # Panic handler
│       ├── console.rs        # Console with tab/history
│       ├── pci.rs            # PCI enumeration
│       ├── drivers/
│       │   ├── mod.rs
│       │   ├── ahci.rs       # AHCI storage driver
│       │   ├── fat32.rs      # FAT32 filesystem
│       │   ├── keyboard.rs   # PS/2 keyboard
│       │   ├── memory.rs     # Memory allocator
│       │   ├── net.rs        # Network structures
│       │   ├── process.rs    # Process scheduler
│       │   ├── rtc.rs        # Real-time clock
│       │   ├── rtl8139.rs    # RTL8139 NIC driver
│       │   ├── tcpip.rs      # TCP/IP stack
│       │   ├── vga.rs        # VGA text mode
│       │   └── vm.rs         # Virtual memory manager
│       └── commands/
│           ├── mod.rs
│           ├── cal.rs         # Calendar
│           ├── cat.rs         # File cat
│           ├── clear.rs       # Clear screen
│           ├── echo.rs        # Echo
│           ├── help.rs        # Help
│           ├── kill_cmd.rs    # Kill process
│           ├── ls.rs          # List directory
│           ├── meminfo.rs     # Memory info
│           ├── mkdir.rs       # Make directory
│           ├── net.rs         # Network commands
│           ├── pcilist.rs     # PCI list
│           ├── ping.rs        # ICMP ping
│           ├── ps.rs          # Process list
│           ├── reboot.rs      # Reboot
│           ├── shutdown.rs    # Shutdown
│           ├── uptime.rs      # Uptime
│           ├── ver.rs         # Version
│           ├── vmstat.rs      # VM statistics
│           └── whoami.rs      # Current user
├── Cargo.toml                # Workspace manifest
├── build.sh                  # Build script
└── .cargo/config.toml        # Cross-compilation config
```

## 📊 Audit Reports

| Report | Description |
|--------|-------------|
| [AUDIT_REPORT_v0.4.md](AUDIT_REPORT_v0.4.md) | Initial vulnerability audit |
| [FUNCTIONAL_AUDIT_v0.4.md](FUNCTIONAL_AUDIT_v0.4.md) | Feature completeness analysis |
| [FINAL_AUDIT_v0.4.md](FINAL_AUDIT_v0.4.md) | Comprehensive final audit |
| [FINAL_AUDIT_v0.4_REVIEWED.md](FINAL_AUDIT_v0.4_REVIEWED.md) | Reviewed audit with fixes applied |
| [COMPARISON_AUDIT_v0.4.md](COMPARISON_AUDIT_v0.4.md) | Comparison with Linux/Windows/macOS |

## 🛠️ Build

```bash
# Requires: rustup, x86_64-unknown-none target
rustup target add x86_64-unknown-none

# Build bootloader + kernel
./build.sh

# Or manually:
cd bootloader && cargo build --release
cd ../kernel && cargo build --release
```

## 📈 Roadmap

| Version | Status | Features |
|---------|--------|----------|
| v0.1 | ✅ Released | Basic kernel, VGA, keyboard |
| v0.2 | ✅ Released | Memory allocator, process basics |
| v0.3 | ✅ Released | Network stack, storage drivers |
| **v0.4** | ✅ **Current** | Full audit, IPC, virtual memory, improved scheduler |
| v0.5 | 🔄 Planned | ELF loader, real context switch, TCP handshake |
| v1.0 | 📋 Planned | Multi-level page tables, syscalls, memory protection |
| v2.0 | 📋 Planned | GUI, ext4, USB, ACPI power management |

## 🔧 Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `fringe` | 1.2 | Coroutine scheduler |
| `spin` | 0.9 | Synchronization primitives |
| `bytemuck` | 1.0 | Safe byte manipulation |
| `linked_list_allocator` | 0.10 | Heap allocator |
| `simple-ahci` | 0.1 | AHCI storage controller |
| `fat32` | 0.13 | FAT32 filesystem |
| `pc-keyboard` | 0.5 | PS/2 keyboard decoding |
| `rtl8139-rs` | 0.1 | RTL8139 NIC driver |
| `uefi` | 0.28 | UEFI boot services |

## 📄 License

MIT License

## 👤 Author

**mason00398** — Personal project for operating system development and learning.

---

<div align="center">

**Built with ❤️ using Rust**

</div>
