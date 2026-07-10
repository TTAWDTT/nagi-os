BUILD_DIR := build
TARGET := x86_64-unknown-none
KERNEL_ELF := target/$(TARGET)/release/nagi-kernel
KERNEL_BIN := $(BUILD_DIR)/kernel.bin
STAGE1_BIN := $(BUILD_DIR)/stage1.bin
STAGE2_BIN := $(BUILD_DIR)/stage2.bin
IMAGE := $(BUILD_DIR)/nagi-os.img
KERNEL_SECTORS_FILE := $(BUILD_DIR)/kernel_sectors.txt

.PHONY: all clean run kernel boot image

all: image

kernel:
	cargo build --release --target $(TARGET)
	mkdir -p $(BUILD_DIR)
	objcopy -O binary $(KERNEL_ELF) $(KERNEL_BIN)
	@sectors=$$(( (`stat -c%s $(KERNEL_BIN)` + 511) / 512 )); \
		echo $$sectors > $(KERNEL_SECTORS_FILE); \
		truncate -s $$(( sectors * 512 )) $(KERNEL_BIN)

boot: kernel
	nasm -f bin boot/stage1.asm -o $(STAGE1_BIN)
	nasm -f bin -D__KERNEL_SECTORS__=`cat $(KERNEL_SECTORS_FILE)` boot/stage2.asm -o $(STAGE2_BIN)

image: boot
	dd if=/dev/zero of=$(IMAGE) bs=512 count=2880 status=none
	dd if=$(STAGE1_BIN) of=$(IMAGE) conv=notrunc status=none
	dd if=$(STAGE2_BIN) of=$(IMAGE) bs=512 seek=1 conv=notrunc status=none
	dd if=$(KERNEL_BIN) of=$(IMAGE) bs=512 seek=9 conv=notrunc status=none
	@echo "Built $(IMAGE)"

run: image
	qemu-system-x86_64 -drive file=$(IMAGE),format=raw,if=floppy -boot a

clean:
	cargo clean
	rm -rf $(BUILD_DIR)

