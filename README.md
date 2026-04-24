# STM32 Blue Pill Serprog - Rust Implementation

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

## Features

- ✅ Full serprog protocol v1 support
- ✅ USB CDC serial communication
- ✅ SPI flash programming at 9 MHz
- ✅ Compatible with flashrom
- ✅ Low resource usage (~10KB flash, ~2KB RAM)
- ✅ Support for common SPI flash chips (25 series, etc.)

## Hardware Requirements

- STM32F103C8T6 Blue Pill board
- SPI flash chip (or device with SPI flash)
- USB cable (Mini or Micro USB depending on your Blue Pill)

## Pin Configuration

### SPI Pins (SPI1)
```
STM32 Pin  | Function | SPI Flash Pin
-----------|----------|---------------
PA5        | SCK      | CLK/SCK
PA6        | MISO     | DO/MISO
PA7        | MOSI     | DI/MOSI
PB0        | CS       | CS/SS
GND        | Ground   | GND
3.3V       | Power    | VCC (if powering the chip)
```

**IMPORTANT:** 
- Most SPI flash chips operate at 3.3V. Do NOT connect 5V!
- If your target device powers the flash chip, do NOT connect 3.3V
- Always connect GND between Blue Pill and target

### USB Pins (Built-in)
```
PA11 - USB D-
PA12 - USB D+
```

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

## Troubleshooting

### "No EEPROM/flash device found"
- Check wiring connections
- Verify chip is powered (3.3V)
- Ensure common ground
- Try slower SPI speed (modify `9.MHz()` in main.rs)

### "Device not found" / Permission denied
```bash
# Linux: Add user to dialout group
sudo usermod -a -G dialout $USER
# Log out and back in

# Or use sudo
sudo flashrom -p serprog:dev=/dev/ttyACM0:4000000
```

### Blue Pill not showing up as USB device
- Check USB cable (must support data, not just power)
- Try different USB port
- Verify firmware flashed correctly
- Check BOOT0 is set to 0 (normal operation)

### Communication errors
- Try lower baud rate: `dev=/dev/ttyACM0:115200`
- Add delays between operations
- Check for loose connections

## Performance

- SPI Clock: 9 MHz (adjustable in code)
- Read speed: ~800 KB/s
- Write speed: ~400 KB/s (depends on chip)
- Erase speed: Depends on chip (typically 1-3s per 64KB sector)

## Customization

### Change SPI Speed
Edit `main.rs`:
```rust
let mut spi = Spi::spi1(
    dp.SPI1,
    (sck, miso, mosi),
    SPI_MODE,
    9.MHz(),  // Change this: 18.MHz(), 4500.kHz(), etc.
    clocks,
);
```

### Change CS Pin
Edit `main.rs`:
```rust
let mut cs = gpiob.pb0.into_push_pull_output(&mut gpiob.crl);
// Change to: gpiob.pb1, gpioa.pa4, etc.
```

### Change Buffer Sizes
Edit `serprog.rs`:
```rust
const BUFFER_SIZE: usize = 512; // Increase for better performance
```

## Technical Details

### Serprog Protocol
This implementation supports serprog protocol version 1 with the following commands:
- NOP, Q_IFACE, Q_CMDMAP, Q_PGMNAME
- Q_SERBUF, Q_BUSTYPE, Q_OPBUF
- Q_WRNMAXLEN, Q_RDNMAXLEN
- O_INIT, O_DELAY, SYNCNOP
- S_SPI_OP, S_BUSTYPE, O_SPIOP, S_PIN

### SPI Configuration
- Mode: 0 (CPOL=0, CPHA=0)
- Bit order: MSB first
- Default speed: 9 MHz
- Max transfer: 256 bytes

### Memory Usage
- Flash: ~8-10 KB
- RAM: ~2 KB (including USB buffers)

## License

This project is open source. Feel free to modify and distribute.

## Contributing

Contributions welcome! Areas for improvement:
- [ ] Support for different SPI modes
- [ ] Hardware flow control
- [ ] Status LED indicators
- [ ] Support for other STM32 boards
- [ ] DMA transfers for higher speeds

## References

- [flashrom serprog protocol](https://www.flashrom.org/Serprog)
- [STM32F1 HAL](https://github.com/stm32-rs/stm32f1xx-hal)
- [Blue Pill pinout](https://stm32-base.org/boards/STM32F103C8T6-Blue-Pill)

## Safety Warning

⚠️ **CAUTION:** 
- Always verify voltage levels (3.3V for most flash chips)
- Never hot-plug SPI connections
- Back up original flash contents before writing
- Some chips have write-protection that must be disabled first
- Incorrect wiring can damage your flash chip or Blue Pill
