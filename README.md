# Serprog - Rust Implementation

A Rust implementation of the flashrom serprog protocol for the STM32F103C8T6 (Blue Pill) board. This allows you to use the Blue Pill as an SPI flash programmer with flashrom.

## Basic Setup - Reading W25Q64 Flash Chip

```
┌─────────────────────┐
│   STM32 Blue Pill   │
│                     │
│  PA5 (SCK)  ────────┼──────── CLK    ┌──────────────┐
│  PA6 (MISO) ────────┼──────── DO     │              │
│  PA7 (MOSI) ────────┼──────── DI     │   W25Q64     │
│  PB0 (CS)   ────────┼──────── CS     │  SPI Flash   │
│  3.3V       ────────┼──────── VCC    │              │
│  GND        ────────┼──────── GND    └──────────────┘
│                     │
│  PA11 (USB D-)      │
│  PA12 (USB D+)      │
└──────┬──────────────┘
       │
       │ USB Cable
       ▼
   Computer
```

## Hardware Requirements

- STM32F103C8T6 Blue Pill board
- SPI flash chip (or device with SPI flash)
- USB cable (Mini or Micro USB depending on your Blue Pill)


## Building

### Prerequisites

1. Install Rust and cargo:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Add thumbv7m target:
```bash
rustup target add thumbv7m-none-eabi
```

3. Install probe-rs (for flashing):
```bash
cargo install probe-rs-tools --locked
```

Or use st-flash:
```bash
sudo apt-get install stlink-tools  # Ubuntu/Debian
brew install stlink                # macOS
```

### Build the firmware

```bash
cargo build --release
```

The binary will be at `target/thumbv7m-none-eabi/release/stm32-serprog`

## Flashing to Blue Pill

### Method 1: Using probe-rs (ST-Link)
```bash
cargo run --release
```

### Method 2: Using st-flash
```bash
# Convert to binary
arm-none-eabi-objcopy -O binary \
  target/thumbv7m-none-eabi/release/stm32-serprog \
  firmware.bin

# Flash
st-flash write firmware.bin 0x8000000
```

### Method 3: Using DFU mode
1. Set BOOT0 jumper to 1
2. Reset the board
3. Flash using dfu-util:
```bash
dfu-util -a 0 -s 0x08000000 -D firmware.bin
```
4. Set BOOT0 back to 0
5. Reset the board

## Usage with flashrom

### 1. Connect the Blue Pill to your computer via USB

### 2. Find the serial device
```bash
# Linux
ls /dev/ttyACM*
# Usually /dev/ttyACM0

# macOS
ls /dev/cu.usbmodem*

# Windows
# Check Device Manager for COM port (e.g., COM3)
```

### 3. Read flash chip ID
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000
```

Expected output:
```
Found chip "Winbond W25Q64.V" (8192 kB, SPI) on serprog.
```

### 4. Read flash contents
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -r flash_backup.bin
```

### 5. Write to flash
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -w firmware.bin
```

### 6. Erase flash
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -E
```

### 7. Verify flash
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -v firmware.bin
```

## Common Flash Chips Supported

- Winbond W25Q series (W25Q16, W25Q32, W25Q64, W25Q128)
- Macronix MX25L series
- Micron/Numonyx M25P series
- Spansion S25FL series
- ISSI IS25LP/WP series
- GigaDevice GD25Q series
- And many more...

## References

- [flashrom serprog protocol](https://www.flashrom.org/Serprog)
