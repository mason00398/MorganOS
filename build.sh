#!/bin/bash
# RunST X 构建脚本
# 用法: chmod +x build.sh && ./build.sh

set -e

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
KERNEL_EFI="$PROJECT_DIR/kernel/target/x86_64-unknown-uefi/release/runst_kernel.efi"
BOOT_EFI="$PROJECT_DIR/bootloader/target/x86_64-unknown-uefi/release/runst_bootloader.efi"
ISO_FILE="$PROJECT_DIR/runst_x.iso"
QEMU_CMD="qemu-system-x86_64 -drive format=raw,file=$PROJECT_DIR/runst_x.img -bios OVMF.fd -serial stdio -m 256"

echo "========================================"
echo "  RunST X Build Script v0.3"
echo "========================================"
echo ""

# 检查 Rust
if ! command -v cargo &> /dev/null; then
    echo "ERROR: Rust/Cargo not found"
    echo "Install: https://rustup.rs/"
    exit 1
fi

# 添加 UEFI 目标
echo "[1/4] Checking Rust targets..."
rustup target add x86_64-unknown-uefi 2>/dev/null || true
rustup component add rust-src 2>/dev/null || true
echo "  OK: Rust ready"

# 构建 bootloader
echo "[2/4] Building bootloader..."
cd "$PROJECT_DIR/bootloader"
cargo build --release 2>&1 | tail -3
echo "  OK: bootloader.efi"

# 构建 kernel
echo "[3/4] Building kernel..."
cd "$PROJECT_DIR/kernel"
cargo build --release 2>&1 | tail -3
echo "  OK: kernel.efi"

# 打包
echo "[4/4] Packaging..."
cd "$PROJECT_DIR"
OUTPUT_DIR="efi-output"
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR/EFI/BOOT"

# 复制 bootloader 为 bootx64.efi (UEFI 标准启动名)
cp "$BOOT_EFI" "$OUTPUT_DIR/EFI/BOOT/bootx64.efi"

# 复制 kernel 为 kernel.nep
cp "$KERNEL_EFI" "$OUTPUT_DIR/kernel.nep"

# 创建 ISO
if command -v xorriso &> /dev/null; then
    xorriso -as mkisofs -o "$ISO_FILE" \
        -b EFI/BOOT/bootx64.efi \
        -no-emul-boot \
        -boot-load-size 4 \
        -boot-info-table \
        -iso-level 4 \
        -J -R \
        "$OUTPUT_DIR"
elif command -v genisoimage &> /dev/null; then
    genisoimage -o "$ISO_FILE" \
        -R -J -b EFI/BOOT/bootx64.efi \
        -no-emul-boot -boot-load-size 4 \
        -iso-level 4 \
        "$OUTPUT_DIR"
else
    echo "WARNING: No ISO tool found (xorriso/genisoimage)"
    echo "  ISO not created, but EFI files are in $OUTPUT_DIR/"
    ISO_FILE=""
fi

# 清理
rm -rf "$OUTPUT_DIR"

echo ""
echo "========================================"
echo "  BUILD COMPLETE"
echo "========================================"
if [ -n "$ISO_FILE" ] && [ -f "$ISO_FILE" ]; then
    SIZE=$(du -h "$ISO_FILE" | cut -f1)
    echo "  ISO: $ISO_FILE ($SIZE)"
    echo ""
    echo "  Run with QEMU:"
    echo "    $QEMU_CMD"
else
    echo "  EFI files ready in current directory"
    echo "  Copy EFI/BOOT/bootx64.efi and kernel.nep to USB ESP"
fi
