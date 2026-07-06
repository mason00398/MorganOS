# MorganOS v0.4

A minimal experimental operating system kernel written in Rust.

## Features
- Process management (fork, terminate, preemptive scheduling)
- Memory management (heap allocation, virtual memory framework)
- Storage drivers (AHCI + FAT32)
- Network stack (RTL8139 + TCP/IP + ARP)
- Terminal command line (ls, cat, mkdir, ping, net, etc.)
- PCI bus scanning
- VGA text console

## Status
- **Version**: v0.4
- **Type**: Experimental kernel
- **Language**: Rust (no_std)
- **Architecture**: x86_64 (UEFI)

## Roadmap
- [ ] System call mechanism
- [ ] User/kernel space separation
- [ ] Virtual memory (page tables, page faults)
- [ ] ext4 filesystem support
- [ ] USB driver
- [ ] Sound card driver
- [ ] TCP handshake & congestion control
- [ ] GUI engine

## License
MIT
