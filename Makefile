# Makefile for STM32 serprog
# Provides convenient targets for building and flashing

# Configuration
TARGET = thumbv7m-none-eabi
BINARY = stm32-serprog
BUILD_DIR = target/$(TARGET)/release
FIRMWARE_BIN = firmware.bin
FLASH_ADDR = 0x08000000

# Serial port - adjust as needed
SERIAL_PORT ?= /dev/ttyACM0
SERIAL_SPEED ?= 4000000

# Default target
.PHONY: all
all: build

# Build the firmware
.PHONY: build
build:
	@echo "Building firmware..."
	cargo build --release

# Build and flash with probe-rs (requires ST-Link)
.PHONY: flash-probe
flash-probe: build
	@echo "Flashing with probe-rs..."
	cargo run --release

# Build and flash with st-flash (requires ST-Link)
.PHONY: flash-stlink
flash-stlink: build binary
	@echo "Flashing with st-flash..."
	st-flash write $(FIRMWARE_BIN) $(FLASH_ADDR)

# Build and flash with DFU
.PHONY: flash-dfu
flash-dfu: build binary
	@echo "Flashing with dfu-util..."
	@echo "Make sure BOOT0 is set to 1 and board is reset!"
	@read -p "Press Enter to continue..." dummy
	dfu-util -a 0 -s $(FLASH_ADDR) -D $(FIRMWARE_BIN)
	@echo "Set BOOT0 back to 0 and press reset!"

# Convert ELF to binary
.PHONY: binary
binary: build
	@echo "Converting to binary..."
	arm-none-eabi-objcopy -O binary \
		$(BUILD_DIR)/$(BINARY) \
		$(FIRMWARE_BIN)

# Test with flashrom (detect chip)
.PHONY: test
test:
	@echo "Testing with flashrom..."
	flashrom -p serprog:dev=$(SERIAL_PORT):$(SERIAL_SPEED)

# Read flash chip
.PHONY: read
read:
	@echo "Reading flash to backup.bin..."
	flashrom -p serprog:dev=$(SERIAL_PORT):$(SERIAL_SPEED) -r backup.bin
	@ls -lh backup.bin

# Write flash chip
.PHONY: write
write:
	@if [ ! -f "$(FILE)" ]; then \
		echo "Error: FILE not specified. Use: make write FILE=firmware.bin"; \
		exit 1; \
	fi
	@echo "Writing $(FILE) to flash..."
	flashrom -p serprog:dev=$(SERIAL_PORT):$(SERIAL_SPEED) -w $(FILE)

# Verify flash chip
.PHONY: verify
verify:
	@if [ ! -f "$(FILE)" ]; then \
		echo "Error: FILE not specified. Use: make verify FILE=firmware.bin"; \
		exit 1; \
	fi
	@echo "Verifying $(FILE)..."
	flashrom -p serprog:dev=$(SERIAL_PORT):$(SERIAL_SPEED) -v $(FILE)

# Erase flash chip
.PHONY: erase
erase:
	@echo "WARNING: This will erase the entire flash chip!"
	@read -p "Are you sure? (y/N) " -n 1 -r; \
	echo; \
	if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		flashrom -p serprog:dev=$(SERIAL_PORT):$(SERIAL_SPEED) -E; \
	else \
		echo "Cancelled."; \
	fi

# Clean build artifacts
.PHONY: clean
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -f $(FIRMWARE_BIN) backup.bin

# Check code formatting
.PHONY: fmt
fmt:
	cargo fmt

# Check code with clippy
.PHONY: clippy
clippy:
	cargo clippy -- -D warnings

# Build documentation
.PHONY: doc
doc:
	cargo doc --open

# Show size of binary
.PHONY: size
size: build
	@echo "Binary size:"
	@arm-none-eabi-size $(BUILD_DIR)/$(BINARY)

# Show help
.PHONY: help
help:
	@echo "STM32 serprog Makefile"
	@echo ""
	@echo "Build targets:"
	@echo "  make build          - Build the firmware"
	@echo "  make binary         - Convert ELF to binary"
	@echo "  make size           - Show binary size"
	@echo ""
	@echo "Flash targets:"
	@echo "  make flash-probe    - Flash using probe-rs (ST-Link)"
	@echo "  make flash-stlink   - Flash using st-flash (ST-Link)"
	@echo "  make flash-dfu      - Flash using DFU bootloader"
	@echo ""
	@echo "Test targets:"
	@echo "  make test           - Detect flash chip"
	@echo "  make read           - Read chip to backup.bin"
	@echo "  make write FILE=x   - Write file to chip"
	@echo "  make verify FILE=x  - Verify file matches chip"
	@echo "  make erase          - Erase entire chip"
	@echo ""
	@echo "Development:"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make fmt            - Format code"
	@echo "  make clippy         - Run clippy linter"
	@echo "  make doc            - Build and open documentation"
	@echo ""
	@echo "Options:"
	@echo "  SERIAL_PORT         - Serial port (default: /dev/ttyACM0)"
	@echo "  SERIAL_SPEED        - Baud rate (default: 4000000)"
	@echo ""
	@echo "Examples:"
	@echo "  make flash-stlink"
	@echo "  make test"
	@echo "  make read"
	@echo "  make write FILE=firmware.bin"
	@echo "  SERIAL_PORT=/dev/ttyACM1 make test"
