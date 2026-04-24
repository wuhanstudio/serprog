# Quick Reference Card - STM32 Serprog

## Pin Connections
```
STM32   →  SPI Flash
─────────────────────
PA5     →  CLK/SCK
PA6     →  DO/MISO  
PA7     →  DI/MOSI
PB0     →  CS
3.3V    →  VCC (if powering chip)
GND     →  GND (always connect)
```

## Common Commands

### Read Chip Info
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000
```

### Backup Flash
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -r backup.bin
```

### Write Flash
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -w firmware.bin
```

### Erase Flash
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -E
```

### Verify Flash
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -v firmware.bin
```

## Build Commands

### Build Firmware
```bash
cargo build --release
```

### Flash with probe-rs
```bash
cargo run --release
```

### Flash with st-flash
```bash
arm-none-eabi-objcopy -O binary \
  target/thumbv7m-none-eabi/release/stm32-serprog firmware.bin
st-flash write firmware.bin 0x8000000
```

### Use Build Script
```bash
./build.sh
```

## Troubleshooting Quick Fixes

### Device not found
```bash
# Linux: Add user to dialout group
sudo usermod -a -G dialout $USER
# Or use sudo
sudo flashrom -p serprog:dev=/dev/ttyACM0:4000000
```

### No chip detected
- Check all wiring
- Verify 3.3V on flash chip
- Try lower speed: `dev=/dev/ttyACM0:115200`

### Communication errors
- Shorter wires (<20cm)
- Add 100nF cap on flash VCC/GND
- Lower SPI speed in code

### USB not showing up
- Different USB cable
- BOOT0 jumper set to 0
- Re-flash firmware

## Supported Flash Chips
- Winbond W25Q series
- Macronix MX25L series  
- Micron M25P series
- Spansion S25FL series
- GigaDevice GD25Q series
- Most 25-series SPI flash

## Safety Checklist
- [ ] Voltage is 3.3V (not 5V)
- [ ] GND connected
- [ ] Flash backup made
- [ ] Wires < 20cm
- [ ] Decoupling cap on flash
- [ ] Target device powered off (if external)

## Performance Specs
- SPI Speed: 9 MHz
- Read: ~800 KB/s
- Write: ~400 KB/s
- Max transfer: 256 bytes

## Serial Ports
- Linux: `/dev/ttyACM0`
- macOS: `/dev/cu.usbmodem*`
- Windows: `COM3` (check Device Manager)

## Common SPI Flash IDs
```
W25Q16:  EF 40 15 (2MB)
W25Q32:  EF 40 16 (4MB)
W25Q64:  EF 40 17 (8MB)
W25Q128: EF 40 18 (16MB)
MX25L:   C2 20 XX
GD25Q:   C8 40 XX
```
