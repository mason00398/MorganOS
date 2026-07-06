# RunST X 操作系统

基于 Rust 的 UEFI 操作系统，支持 FAT32、TCP/IP、多进程。

## 版本 0.3

### 新增功能

| 模块 | 状态 | 说明 |
|------|------|------|
| FAT32 文件系统 | ✅ 框架 | BPB解析、目录读取、文件读取 |
| TCP/IP 协议栈 | ✅ 基础 | ICMP Ping、UDP、ARP缓存、校验和 |
| 进程管理器 | ✅ 实现 | 协作式多任务、spawn/kill/ps |
| 内存管理器 | ✅ 实现 | 4MB堆、动态分配、使用率统计 |
| AHCI 驱动 | ✅ 增强 | 真实端口检测、NCQ命令框架 |
| 键盘驱动 | ✅ 增强 | 方向键、F键、Tab、Backspace |
| VGA 驱动 | ✅ 增强 | 真正清屏、光标控制、颜色枚举 |
| 网卡驱动 | ✅ 增强 | 收发完整、MAC读取 |

### 命令列表 (22个)

```
ver        显示版本信息
help       显示帮助
reboot     重启系统
shutdown   关机
echo       输出文本
clear/cls  清屏
meminfo    内存使用情况
uptime/time 当前时间
test       硬件诊断
pcilist    列出PCI设备
net        网络测试
ls/dir     列出目录
cat        查看文件
mkdir      创建目录
ps         列出进程
kill       杀死进程
ifconfig/ip 网络配置
ping       Ping主机
date       显示日期
cal        日历
whoami     当前用户
env        环境变量
```

### 快速开始

```bash
# 安装 Rust (如果还没有)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 添加 UEFI 目标
rustup target add x86_64-unknown-uefi
rustup component add rust-src

# 构建
cd runst_x
cargo build --release

# 运行 (QEMU)
qemu-system-x86_64 -drive format=raw,file=runst_x.img -bios OVMF.fd -serial stdio -m 256
```

### 目录结构

```
runst_x/
├── .cargo/
│   └── config.toml        # UEFI 编译配置
├── Cargo.toml             # Workspace
├── bootloader/
│   ├── Cargo.toml
│   └── src/main.rs        # 引导程序 (SHA256校验)
├── kernel/
│   ├── Cargo.toml
│   ├── build.rs           # 构建时注入日期/GitHash
│   └── src/
│       ├── main.rs        # 内核入口
│       ├── console.rs     # 命令Shell
│       ├── pci.rs         # PCI枚举
│       ├── drivers/
│       │   ├── mod.rs     # 驱动模块
│       │   ├── vga.rs     # VGA文本显示
│       │   ├── keyboard.rs # 键盘驱动
│       │   ├── rtc.rs     # 实时时钟
│       │   ├── rtl8139.rs # 网卡驱动
│       │   ├── ahci.rs    # SATA/AHCI驱动
│       │   ├── fat32.rs   # FAT32文件系统
│       │   ├── memory.rs  # 内存管理器
│       │   ├── process.rs # 进程管理器
│       │   └── tcpip.rs  # TCP/IP协议栈
│       └── commands/      # 22个命令
│           ├── mod.rs
│           ├── ver.rs
│           ├── help.rs
│           ├── ...
├── build.sh               # 一键构建脚本
└── README.md
```

### 技术栈

- **语言**: Rust (no_std, no_main)
- **目标**: x86_64-unknown-uefi
- **UEFI**: 0.28 + uefi-services 0.25
- **内存**: linked_list_allocator
- **同步**: spin (Mutex)

### License

MIT
