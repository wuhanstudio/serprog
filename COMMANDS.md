# Command Reference

Complete command reference for building, flashing, and using STM32 serprog.

## Building Commands

### Build firmware (debug)
```bash
cargo build
```

### Build firmware (release/optimized)
```bash
cargo build --release
```

### Build and run with probe-rs
```bash
cargo run --release
```

### Check code without building
```bash
cargo check
```

### Format code
```bash
cargo fmt
```

### Lint with clippy
```bash
cargo clippy
```

### Clean build artifacts
```bash
cargo clean
```

### Build documentation
```bash
cargo doc --open
```

### Show binary size
```bash
arm-none-eabi-size target/thumbv7m-none-eabi/release/stm32-serprog
```

## Flashing Commands

### Convert ELF to binary
```bash
arm-none-eabi-objcopy -O binary \
  target/thumbv7m-none-eabi/release/stm32-serprog \
  firmware.bin
```

### Flash with st-flash
```bash
st-flash write firmware.bin 0x8000000
```

### Flash with st-flash (erase first)
```bash
st-flash --reset --format ihex erase
st-flash --reset write firmware.bin 0x8000000
```

### Flash with probe-rs
```bash
probe-rs run --chip STM32F103C8 target/thumbv7m-none-eabi/release/stm32-serprog
```

### Flash with DFU
```bash
# Set BOOT0=1, press reset, then:
dfu-util -a 0 -s 0x08000000 -D firmware.bin
# Set BOOT0=0, press reset
```

### List DFU devices
```bash
dfu-util -l
```

### Verify ST-Link connection
```bash
st-info --probe
```

### Read flash from STM32
```bash
st-flash read dump.bin 0x8000000 0x10000
```

## USB Device Commands

### List USB devices (Linux)
```bash
lsusb
# Should show: ID 16c0:27dd
```

### Check kernel messages (Linux)
```bash
dmesg | tail -20
# Should show CDC ACM device detected
```

### List serial devices (Linux)
```bash
ls -l /dev/ttyACM*
```

### List serial devices (macOS)
```bash
ls -l /dev/cu.usbmodem*
```

### Check device permissions (Linux)
```bash
ls -l /dev/ttyACM0
groups  # Check if user is in dialout group
```

### Add user to dialout group (Linux)
```bash
sudo usermod -a -G dialout $USER
# Log out and back in
```

### Test serial connection
```bash
screen /dev/ttyACM0 115200
# or
minicom -D /dev/ttyACM0 -b 115200
```

## flashrom Commands

### Detect programmer only
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000
```

### Verbose detection
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -V
```

### Read chip ID
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -V 2>&1 | grep "RDID"
```

### Read entire chip
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -r backup.bin
```

### Read specific region (1MB from start)
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -r backup.bin \
  --image region:0x0-0x100000
```

### Write to chip
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -w firmware.bin
```

### Write and verify
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -w firmware.bin -v
```

### Verify chip contents
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -v backup.bin
```

### Erase entire chip
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -E
```

### Erase and write
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -E -w firmware.bin
```

### Force specific chip
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -c "W25Q64.V" -r backup.bin
```

### List supported chips
```bash
flashrom -L | grep SPI
```

### Use slower baud rate (more reliable)
```bash
flashrom -p serprog:dev=/dev/ttyACM0:115200 -r backup.bin
```

### Read with progress indicator (Linux)
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -r backup.bin | pv > backup.bin
```

## Serial Port Configuration

### Different serial ports
```bash
# Linux - first device
flashrom -p serprog:dev=/dev/ttyACM0:4000000

# Linux - second device
flashrom -p serprog:dev=/dev/ttyACM1:4000000

# macOS
flashrom -p serprog:dev=/dev/cu.usbmodem1234:4000000

# Windows
flashrom -p serprog:dev=COM3:4000000
```

### Different baud rates
```bash
# Very fast (default)
flashrom -p serprog:dev=/dev/ttyACM0:4000000

# Fast
flashrom -p serprog:dev=/dev/ttyACM0:2000000

# Medium
flashrom -p serprog:dev=/dev/ttyACM0:1000000

# Standard
flashrom -p serprog:dev=/dev/ttyACM0:115200

# Slow (most reliable)
flashrom -p serprog:dev=/dev/ttyACM0:57600
```

## Diagnostic Commands

### Check flashrom version
```bash
flashrom --version
```

### List all programmers
```bash
flashrom -L | grep programmer
```

### Test connection (no operation)
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000 --progress
```

### Detailed chip information
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -V 2>&1 | grep -A 20 "Found chip"
```

### Monitor serial traffic (Linux)
```bash
strace -e read,write flashrom -p serprog:dev=/dev/ttyACM0:4000000 2>&1 | grep ACM
```

### Check USB device details (Linux)
```bash
udevadm info -a -n /dev/ttyACM0
```

## File Operations

### Compare two flash dumps
```bash
cmp backup1.bin backup2.bin
# or
diff <(xxd backup1.bin) <(xxd backup2.bin)
```

### View hex dump
```bash
xxd backup.bin | less
```

### Search for string in dump
```bash
strings backup.bin | grep "something"
```

### Get file info
```bash
file backup.bin
ls -lh backup.bin
md5sum backup.bin
```

### Split large dump
```bash
split -b 1M backup.bin chunk_
```

### Combine dumps
```bash
cat chunk_* > backup.bin
```

## Makefile Commands (if using Makefile)

### Build
```bash
make build
```

### Flash with ST-Link
```bash
make flash-stlink
```

### Flash with probe-rs
```bash
make flash-probe
```

### Test with flashrom
```bash
make test
```

### Read chip
```bash
make read
```

### Write to chip
```bash
make write FILE=firmware.bin
```

### Verify chip
```bash
make verify FILE=firmware.bin
```

### Erase chip
```bash
make erase
```

### Clean
```bash
make clean
```

### Show help
```bash
make help
```

### Use custom serial port
```bash
SERIAL_PORT=/dev/ttyACM1 make test
```

## Build Script Commands

### Interactive build and flash
```bash
./build.sh
```

### Make executable (if needed)
```bash
chmod +x build.sh
```

## Advanced flashrom Options

### Skip blank check (faster writes)
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -w firmware.bin -n
```

### Force write even if chip ID doesn't match
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -w firmware.bin -f
```

### Write specific layout
```bash
# Create layout file (layout.txt):
# 0x000000:0x00ffff bootloader
# 0x010000:0x7fffff application

flashrom -p serprog:dev=/dev/ttyACM0:4000000 \
  -l layout.txt -i application -w app.bin
```

### Generate flashrom log
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -r backup.bin \
  -o logfile.txt
```

## Python Testing Commands

### Run test script
```bash
python3 test_serprog.py
```

### Quick serial test
```bash
python3 -c "import serial; s=serial.Serial('/dev/ttyACM0', 115200); \
  s.write(b'\\x01'); print(s.read(3).hex())"
```

## Common Workflows

### Complete chip backup
```bash
# Detect chip
flashrom -p serprog:dev=/dev/ttyACM0:4000000

# Read and verify
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -r backup.bin
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -v backup.bin

# Create checksum
md5sum backup.bin > backup.bin.md5
```

### Reprogram chip
```bash
# Backup first!
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -r backup.bin

# Erase and write
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -E -w new_firmware.bin

# Verify
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -v new_firmware.bin
```

### Partial update
```bash
# Read current
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -r current.bin

# Modify current.bin as needed with hex editor

# Write back
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -w current.bin
```

### Recovery from bad flash
```bash
# Force read even if chip detection fails
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -c "W25Q64.V" -f -r dump.bin

# Force write backup
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -c "W25Q64.V" -f -w backup.bin
```

## Troubleshooting Commands

### Permission fix (Linux)
```bash
sudo chmod a+rw /dev/ttyACM0
# or
sudo flashrom -p serprog:dev=/dev/ttyACM0:4000000
```

### Reset USB device (Linux)
```bash
sudo usbreset /dev/bus/usb/001/002  # adjust bus/device numbers
# or
sudo sh -c "echo 0 > /sys/bus/usb/devices/1-1/authorized"
sudo sh -c "echo 1 > /sys/bus/usb/devices/1-1/authorized"
```

### Check for kernel driver conflicts (Linux)
```bash
lsmod | grep cdc
dmesg | grep -i usb | tail -20
```

### Reload cdc_acm module (Linux)
```bash
sudo modprobe -r cdc_acm
sudo modprobe cdc_acm
```

## Environment Variables

### Set default serial port
```bash
export SERPROG_DEV=/dev/ttyACM0
flashrom -p serprog:dev=$SERPROG_DEV:4000000
```

### Set Rust flags for smaller binary
```bash
export RUSTFLAGS="-C opt-level=z"
cargo build --release
```

## Continuous Integration / Automation

### Build and verify size
```bash
cargo build --release
SIZE=$(arm-none-eabi-size target/thumbv7m-none-eabi/release/stm32-serprog | \
  awk 'NR==2 {print $1+$2}')
if [ $SIZE -gt 65536 ]; then
  echo "Binary too large: $SIZE bytes"
  exit 1
fi
```

### Automated testing
```bash
#!/bin/bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000 || exit 1
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -r test.bin || exit 1
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -v test.bin || exit 1
echo "All tests passed"
```

## Quick Reference Card

```bash
# Build
cargo build --release

# Flash (choose one)
cargo run --release                          # probe-rs
st-flash write firmware.bin 0x8000000        # st-flash  
./build.sh                                    # interactive

# Detect chip
flashrom -p serprog:dev=/dev/ttyACM0:4000000

# Backup
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -r backup.bin

# Write
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -w firmware.bin
```
