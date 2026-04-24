# Testing and Debugging Guide

## Logic Analyzer Testing

If you have a logic analyzer, you can verify the SPI communication:

### Connections
```
Logic Analyzer | STM32 Pin
---------------|----------
CH0            | PA5 (SCK)
CH1            | PA7 (MOSI)
CH2            | PA6 (MISO)
CH3            | PB0 (CS)
GND            | GND
```

### Expected Signals

When flashrom reads chip ID (command 0x9F):
```
1. CS goes LOW
2. MOSI sends: 0x9F
3. MISO receives: Manufacturer ID, Device ID (2 bytes)
4. CS goes HIGH
```

For Winbond W25Q64:
```
MOSI: 0x9F
MISO: 0xEF 0x40 0x17
```

## Serial Debug Output

You can add debug output by uncommenting debug features in Cargo.toml:

```toml
[dependencies]
# defmt = "0.3"
# defmt-rtt = "0.4"

# [features]
# default = ["debug"]
# debug = ["defmt", "defmt-rtt"]
```

## Manual Testing with Python

Test the serprog protocol manually:

```python
#!/usr/bin/env python3
import serial
import time

# Open serial port
ser = serial.Serial('/dev/ttyACM0', 115200, timeout=1)
time.sleep(0.5)

# Query interface version (0x01)
ser.write(b'\x01')
response = ser.read(3)
print(f"Interface version: {response.hex()}")
# Expected: 06 01 00 (ACK, version 1.0)

# Query programmer name (0x03)
ser.write(b'\x03')
response = ser.read(20)
print(f"Programmer name: {response}")
# Expected: ACK + "stm32-serprog\0"

# Query supported commands (0x02)
ser.write(b'\x02')
response = ser.read(33)
print(f"Command map: {response.hex()}")

# Initialize SPI (0x0B)
ser.write(b'\x0B')
response = ser.read(1)
print(f"Init response: {response.hex()}")
# Expected: 06 (ACK)

# SPI operation enable (0x12)
ser.write(b'\x12')
response = ser.read(1)
print(f"SPIOP response: {response.hex()}")
# Expected: 06 (ACK)

# Read flash ID using S_SPI_OP (0x10)
# Format: cmd(1) + write_len(3) + read_len(3) + data
cmd = b'\x10'           # S_SPI_OP command
write_len = b'\x01\x00\x00'  # Write 1 byte
read_len = b'\x03\x00\x00'   # Read 3 bytes
data = b'\x9F'          # RDID command

ser.write(cmd + write_len + read_len + data)
response = ser.read(4)  # ACK + 3 bytes of ID
print(f"Flash ID: {response.hex()}")
# Expected: 06 EF 40 17 (for W25Q64)

ser.close()
```

Save as `test_serprog.py` and run:
```bash
python3 test_serprog.py
```

## Common Issues and Solutions

### Issue: SPI data corruption
**Solution:** 
- Reduce SPI speed in main.rs
- Add decoupling capacitors (100nF) near flash chip
- Use shorter wires (<15cm)
- Use twisted pairs for SCK/MOSI and SCK/MISO

### Issue: Intermittent errors
**Solution:**
- Add pull-up resistor (10kΩ) on CS line
- Check power supply stability (measure 3.3V rail)
- Add bulk capacitor (10µF) on 3.3V rail

### Issue: Can't read flash at high speeds
**Solution:**
```rust
// In main.rs, change SPI speed
let mut spi = Spi::spi1(
    dp.SPI1,
    (sck, miso, mosi),
    SPI_MODE,
    4500.kHz(),  // Try 4.5 MHz instead of 9 MHz
    clocks,
);
```

## Oscilloscope Measurements

Expected signal characteristics:
- SCK frequency: 9 MHz (111 ns period)
- Rise time: < 10 ns
- Fall time: < 10 ns
- CS setup time: > 5 ns
- CS hold time: > 5 ns

## Performance Benchmarking

Test read speed:
```bash
time flashrom -p serprog:dev=/dev/ttyACM0:4000000 -r test.bin
```

Expected results for 8MB flash:
```
Real: ~10-15 seconds
Speed: ~500-800 KB/s
```

Test write speed:
```bash
time flashrom -p serprog:dev=/dev/ttyACM0:4000000 -w firmware.bin
```

## Advanced Debugging

### Enable detailed flashrom output
```bash
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -V -r dump.bin
```

### Use strace to see serial communication
```bash
strace -e read,write flashrom -p serprog:dev=/dev/ttyACM0:4000000 2>&1 | grep ACM
```

### Monitor with minicom
```bash
minicom -D /dev/ttyACM0 -b 115200
```

## Electrical Specifications

### STM32F103 SPI Pins
- Logic level: 3.3V
- Max current per pin: 25 mA
- Input voltage: -0.3V to 3.6V
- Max SPI speed: 18 MHz (APB2/2)

### Recommended External Components
```
Flash Chip:
  - 100nF ceramic cap between VCC and GND (close to chip)
  - 10kΩ pull-up on CS (optional, improves reliability)
  
Blue Pill:
  - 100nF ceramic cap on 3.3V rail
  - 10µF electrolytic cap on 3.3V rail
```

## Firmware Verification

After flashing, verify the firmware is running:

```bash
# Linux
dmesg | tail -20
# Should show: usb 1-1: New USB device strings: Mfr=1, Product=2, SerialNumber=3
#              cdc_acm 1-1:1.0: ttyACM0: USB ACM device

# List USB devices
lsusb
# Should show: Bus 001 Device 005: ID 16c0:27dd Van Ooijen Technische Informatica

# Check serial device
ls -l /dev/ttyACM0
# Should exist and be accessible
```
