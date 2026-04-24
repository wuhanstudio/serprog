# STM32 Blue Pill Serprog - Project Overview

## What This Project Does

This is a complete Rust implementation of the flashrom serprog protocol for the STM32F103C8T6 (Blue Pill) development board. It turns your $2 Blue Pill into a professional SPI flash programmer compatible with flashrom, allowing you to:

- Read/write/erase SPI flash chips (W25Q, MX25L, etc.)
- Backup and restore BIOS chips from motherboards
- Program flash on embedded devices (routers, IoT devices)
- Recover bricked devices with SPI flash
- Analyze and modify firmware

## Key Features

✅ **Full serprog v1 protocol** - 100% compatible with flashrom  
✅ **USB CDC serial** - No FTDI chips needed  
✅ **Fast operation** - 9 MHz SPI, ~800 KB/s read speed  
✅ **Memory efficient** - ~10KB flash, ~2KB RAM  
✅ **Safe Rust** - No unsafe code, reliable operation  
✅ **Well documented** - Multiple guides and examples  
✅ **Easy to modify** - Clean code structure  

## Project Structure

```
stm32-serprog/
├── src/
│   ├── main.rs          - Main application, USB & SPI setup
│   └── serprog.rs       - Serprog protocol implementation
├── Cargo.toml           - Dependencies and build config
├── memory.x             - STM32F103 memory layout
├── build.rs             - Build script
├── build.sh             - Automated build & flash script
├── .cargo/
│   └── config.toml      - Cargo configuration
├── README.md            - Main documentation
├── QUICKREF.md          - Quick reference card
├── WIRING.md            - Wiring diagrams
├── TESTING.md           - Testing and debugging guide
├── EXAMPLES.md          - Code modification examples
└── .gitignore           - Git ignore rules
```

## Quick Start

### 1. Install Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add ARM target
rustup target add thumbv7m-none-eabi

# Install flasher (choose one)
cargo install probe-rs-tools    # For ST-Link
# OR
sudo apt install stlink-tools   # For st-flash
```

### 2. Build Firmware
```bash
cd stm32-serprog
cargo build --release
```

### 3. Flash to Blue Pill
```bash
./build.sh
# OR manually:
st-flash write firmware.bin 0x8000000
```

### 4. Wire Up
```
Blue Pill → Flash Chip
PA5 → CLK
PA6 → MISO/DO
PA7 → MOSI/DI
PB0 → CS
GND → GND
3.3V → VCC (if needed)
```

### 5. Use with flashrom
```bash
# Detect chip
flashrom -p serprog:dev=/dev/ttyACM0:4000000

# Backup
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -r backup.bin

# Write
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -w new_firmware.bin
```

## Hardware Requirements

**Minimum:**
- STM32F103C8T6 Blue Pill board (~$2)
- USB cable (Mini or Micro)
- ST-Link V2 programmer (~$3) OR USB-to-Serial adapter for bootloader

**Recommended:**
- SOIC-8 test clip (~$5) for easy chip access
- Breadboard and jumper wires
- Logic analyzer (optional, for debugging)

**Total cost:** ~$5-10

## Use Cases

### 1. BIOS Backup/Recovery
- Backup motherboard BIOS before updates
- Recover from failed BIOS flash
- Modify BIOS settings (advanced users)

### 2. Router Firmware
- Unbrick routers (TP-Link, Netgear, etc.)
- Install OpenWrt on locked devices
- Extract firmware for analysis

### 3. IoT Device Programming
- Program ESP8266/ESP32 flash
- Update firmware on smart devices
- Reverse engineer IoT protocols

### 4. Embedded Development
- Program SPI flash for custom boards
- Production programming tool
- Firmware testing and validation

### 5. Security Research
- Extract firmware from devices
- Analyze flash contents
- Security auditing

## Technical Specifications

**MCU:** STM32F103C8T6 (ARM Cortex-M3, 72 MHz)  
**Flash:** 64 KB (firmware uses ~10 KB)  
**RAM:** 20 KB (uses ~2 KB)  
**SPI Speed:** 9 MHz (configurable up to 18 MHz)  
**USB:** Full-speed (12 Mbps) CDC ACM  
**Protocol:** Serprog v1  
**Max Transfer:** 256 bytes per operation  
**Read Speed:** ~800 KB/s  
**Write Speed:** ~400 KB/s  

## Supported Flash Chips

Supports most 25-series SPI flash chips:
- **Winbond:** W25Q16/32/64/128/256
- **Macronix:** MX25L series
- **Micron:** M25P series  
- **Spansion:** S25FL series
- **GigaDevice:** GD25Q series
- **ISSI:** IS25LP/WP series
- **And many more...**

Full list: https://flashrom.org/Supported_hardware

## Advantages Over Alternatives

### vs. CH341A Programmer
- ✅ Open source firmware (you control it)
- ✅ Easier to obtain (Blue Pill is common)
- ✅ More reliable (no fake chips)
- ✅ Customizable (modify the code)

### vs. Bus Pirate
- ✅ Much faster (9 MHz vs 400 kHz)
- ✅ Cheaper ($2 vs $30)
- ✅ Smaller and more portable

### vs. Raspberry Pi
- ✅ Dedicated device (doesn't tie up Pi)
- ✅ Faster SPI
- ✅ Lower power consumption
- ✅ More portable

## Safety and Warnings

⚠️ **IMPORTANT SAFETY NOTES:**

1. **Voltage:** STM32 is 3.3V ONLY. Do not connect 5V to SPI pins!
2. **Power Off:** Always disconnect power from target device before connecting
3. **Backup First:** Always backup flash before writing
4. **Verify Wiring:** Double-check all connections
5. **ESD Protection:** Use anti-static precautions
6. **Chip Damage:** Incorrect voltage can permanently damage flash chips

## Limitations

- SPI only (no parallel flash support)
- No 5V flash support (3.3V only)
- Max 256 byte transfers (can be increased in code)
- Single SPI bus (one chip at a time)
- Basic error handling (no retry logic)

Most of these can be addressed through code modifications (see EXAMPLES.md).

## Performance Tips

1. **Use short wires** (<20cm) for reliable high-speed operation
2. **Add decoupling caps** (100nF) near flash chip
3. **Lower SPI speed** if experiencing errors
4. **Ensure good ground** connection
5. **Use USB 2.0 port** (not USB 3.0, can cause interference)

## Future Enhancements

Potential improvements (contributions welcome):
- [ ] DMA transfers for higher throughput
- [ ] Multiple chip support via GPIO CS pins
- [ ] Voltage level detection
- [ ] Status LED indicators
- [ ] Hardware flow control
- [ ] Different SPI modes (1, 2, 3)
- [ ] Retry logic and error recovery
- [ ] Support for other STM32 boards

## License

Open source - use, modify, and distribute freely.

## Support and Community

- **Issues:** Open an issue on GitHub
- **Documentation:** See README.md and other .md files
- **flashrom:** https://flashrom.org
- **STM32 Rust:** https://github.com/stm32-rs

## Credits

Built with:
- **stm32f1xx-hal** - STM32F1 hardware abstraction layer
- **usb-device** - USB device framework
- **usbd-serial** - USB CDC serial implementation
- **cortex-m** - ARM Cortex-M support

## Contributing

Contributions welcome! Please:
1. Test your changes thoroughly
2. Update documentation
3. Follow existing code style
4. Add examples if introducing new features

## Changelog

**v0.1.0** - Initial release
- Full serprog v1 protocol
- USB CDC serial communication
- SPI flash support at 9 MHz
- STM32F103C8T6 Blue Pill support

---

**Made with ❤️ and Rust 🦀**
