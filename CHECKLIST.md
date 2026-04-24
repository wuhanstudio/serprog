# Getting Started Checklist

Use this checklist to get up and running with STM32 serprog.

## Prerequisites Setup

- [ ] **Install Rust toolchain**
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  source $HOME/.cargo/env
  ```

- [ ] **Add ARM Cortex-M3 target**
  ```bash
  rustup target add thumbv7m-none-eabi
  ```

- [ ] **Install flashing tool** (choose one):
  - [ ] probe-rs: `cargo install probe-rs-tools --locked`
  - [ ] st-flash: `sudo apt install stlink-tools` (Linux)
  - [ ] dfu-util: `sudo apt install dfu-util` (for bootloader)

- [ ] **Install flashrom**
  - Linux: `sudo apt install flashrom`
  - macOS: `brew install flashrom`
  - Windows: Download from https://flashrom.org

- [ ] **Install arm-none-eabi-gcc** (optional, for objcopy)
  ```bash
  sudo apt install gcc-arm-none-eabi  # Linux
  brew install --cask gcc-arm-embedded  # macOS
  ```

## Hardware Preparation

- [ ] **Acquire Blue Pill board**
  - STM32F103C8T6 (not the fake CS32 chips!)
  - Check with `st-flash --version` or `st-info --probe`

- [ ] **Acquire programmer** (choose one):
  - [ ] ST-Link V2 (~$3)
  - [ ] USB-to-Serial (for bootloader method)
  - [ ] Another Blue Pill as Black Magic Probe

- [ ] **Get jumper wires** (at least 6)
  - Female-to-Female recommended
  - Keep them short (<20cm)

- [ ] **Optional but recommended:**
  - [ ] SOIC-8 test clip
  - [ ] Breadboard
  - [ ] Logic analyzer (for debugging)
  - [ ] 100nF ceramic capacitors

## Build and Flash Firmware

- [ ] **Navigate to project directory**
  ```bash
  cd stm32-serprog
  ```

- [ ] **Build the firmware**
  ```bash
  cargo build --release
  ```
  - [ ] Build completed without errors
  - [ ] Binary at `target/thumbv7m-none-eabi/release/stm32-serprog`

- [ ] **Flash to Blue Pill** (choose method):

  ### Method 1: ST-Link with probe-rs
  - [ ] Connect ST-Link to Blue Pill (SWDIO, SWCLK, GND, 3.3V)
  - [ ] Connect ST-Link to computer
  - [ ] Run: `cargo run --release`
  - [ ] Verify successful flash

  ### Method 2: ST-Link with st-flash
  - [ ] Connect ST-Link to Blue Pill
  - [ ] Convert binary:
    ```bash
    arm-none-eabi-objcopy -O binary \
      target/thumbv7m-none-eabi/release/stm32-serprog \
      firmware.bin
    ```
  - [ ] Flash: `st-flash write firmware.bin 0x8000000`
  - [ ] Verify "Flash written and verified! jolly good!"

  ### Method 3: DFU Bootloader
  - [ ] Set BOOT0 jumper to 1
  - [ ] Press RESET button
  - [ ] Convert to binary (if not done)
  - [ ] Flash: `dfu-util -a 0 -s 0x08000000 -D firmware.bin`
  - [ ] Set BOOT0 back to 0
  - [ ] Press RESET

- [ ] **Verify USB connection**
  - [ ] Connect Blue Pill to computer via USB
  - [ ] Check device appears:
    - Linux: `ls /dev/ttyACM*` (should show /dev/ttyACM0)
    - macOS: `ls /dev/cu.usbmodem*`
    - Windows: Check Device Manager for COM port
  - [ ] Check USB descriptor: `lsusb` (Linux/macOS)
    - Should show: `ID 16c0:27dd Van Ooijen Technische Informatica`

## First Test - Verify Installation

- [ ] **Test with flashrom (no chip connected)**
  ```bash
  flashrom -p serprog:dev=/dev/ttyACM0:4000000
  ```
  - [ ] flashrom detects programmer
  - [ ] Shows "serprog" in output
  - [ ] May show "No EEPROM/flash device found" (OK if no chip connected)

- [ ] **Fix permissions if needed (Linux)**
  ```bash
  sudo usermod -a -G dialout $USER
  # Log out and back in, or use sudo
  ```

## Wire Up Test Flash Chip

- [ ] **Identify flash chip pins** (see WIRING.md)
  - [ ] Pin 1: CS
  - [ ] Pin 2: DO (MISO)
  - [ ] Pin 5: DI (MOSI)
  - [ ] Pin 6: CLK (SCK)
  - [ ] Pin 4: GND
  - [ ] Pin 8: VCC

- [ ] **Connect Blue Pill to Flash Chip**
  ```
  Blue Pill → Flash Chip
  PA5       → Pin 6 (CLK)
  PA6       → Pin 2 (DO/MISO)
  PA7       → Pin 5 (DI/MOSI)
  PB0       → Pin 1 (CS)
  GND       → Pin 4 (GND)
  3.3V      → Pin 8 (VCC) - only if chip not powered elsewhere
  ```

- [ ] **Double-check connections**
  - [ ] All 6 wires connected correctly
  - [ ] No shorts between adjacent pins
  - [ ] Voltage is 3.3V (measure with multimeter if possible)
  - [ ] Wires are short and secure

## First Real Test - Read Chip

- [ ] **Detect chip**
  ```bash
  flashrom -p serprog:dev=/dev/ttyACM0:4000000
  ```
  - [ ] Shows chip manufacturer and model
  - [ ] Example: "Found chip 'Winbond W25Q64.V' (8192 kB, SPI)"

- [ ] **Read chip ID only (fast test)**
  ```bash
  flashrom -p serprog:dev=/dev/ttyACM0:4000000 -V 2>&1 | grep "RDID"
  ```
  - [ ] Shows chip ID bytes (e.g., EF 40 17 for W25Q64)

- [ ] **Create backup**
  ```bash
  flashrom -p serprog:dev=/dev/ttyACM0:4000000 -r backup.bin
  ```
  - [ ] Read completes without errors
  - [ ] File created: `ls -lh backup.bin`
  - [ ] File size matches chip size

- [ ] **Verify backup**
  ```bash
  flashrom -p serprog:dev=/dev/ttyACM0:4000000 -v backup.bin
  ```
  - [ ] Verification passes

## Troubleshooting (if tests fail)

- [ ] **No chip detected:**
  - [ ] Verify all wiring connections
  - [ ] Check voltage on flash chip VCC (should be 3.3V)
  - [ ] Try lower speed: `dev=/dev/ttyACM0:115200`
  - [ ] Check flash chip datasheet for correct pinout

- [ ] **Device not found:**
  - [ ] Try different USB cable (must support data)
  - [ ] Try different USB port
  - [ ] Check `dmesg | tail` for USB errors (Linux)
  - [ ] Reflash firmware

- [ ] **Communication errors:**
  - [ ] Shorten wires (<15cm)
  - [ ] Add 100nF capacitor on flash VCC/GND
  - [ ] Lower SPI speed in code (change 9.MHz to 4500.kHz)
  - [ ] Check for loose connections

- [ ] **Permission denied (Linux):**
  - [ ] Run with sudo (temporary)
  - [ ] Add user to dialout group (permanent)
  - [ ] Log out and back in after adding to group

## You're Ready!

Once all the above is working, you can:

- [ ] **Read from flash chips** - Extract firmware for analysis
- [ ] **Write to flash chips** - Program new firmware
- [ ] **Erase flash chips** - Prepare for reprogramming
- [ ] **Backup BIOS chips** - Protect against failed updates
- [ ] **Unbrick devices** - Recover routers, IoT devices
- [ ] **Custom modifications** - See EXAMPLES.md for code changes

## Quick Reference

**Most common commands:**
```bash
# Detect chip
flashrom -p serprog:dev=/dev/ttyACM0:4000000

# Backup
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -r backup.bin

# Write
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -w firmware.bin

# Erase
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -E

# Verify
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -v firmware.bin
```

**Serial ports:**
- Linux: `/dev/ttyACM0`
- macOS: `/dev/cu.usbmodem*`
- Windows: `COM3` (or check Device Manager)

**Baud rates:**
- 4000000 (fast, recommended)
- 115200 (slow, more reliable)

## Next Steps

- [ ] Read through WIRING.md for more connection examples
- [ ] Check TESTING.md for debugging techniques
- [ ] Explore EXAMPLES.md for customizations
- [ ] Keep QUICKREF.md handy for quick lookups

## Safety Reminders

- [ ] Always disconnect target power before connecting
- [ ] Always backup before writing
- [ ] Double-check voltage levels (3.3V only!)
- [ ] Never hot-plug SPI connections
- [ ] Use anti-static precautions

---

**Congratulations! You now have a working SPI flash programmer for under $5!** 🎉
