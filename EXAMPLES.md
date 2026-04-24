# Code Modification Examples

## Example 1: Change SPI Speed

### Lower Speed for Long Wires (4.5 MHz)
```rust
// In src/main.rs, line ~45
let mut spi = Spi::spi1(
    dp.SPI1,
    (sck, miso, mosi),
    SPI_MODE,
    4500.kHz(),  // Changed from 9.MHz()
    clocks,
);
```

### Maximum Speed (18 MHz)
```rust
let mut spi = Spi::spi1(
    dp.SPI1,
    (sck, miso, mosi),
    SPI_MODE,
    18.MHz(),  // Maximum for STM32F103
    clocks,
);
```

## Example 2: Use Different CS Pin

### Use PA4 as CS
```rust
// In src/main.rs, line ~39
// Change from:
let mut cs = gpiob.pb0.into_push_pull_output(&mut gpiob.crl);

// To:
let mut cs = gpioa.pa4.into_push_pull_output(&mut gpioa.crl);
```

### Use PB1 as CS
```rust
let mut cs = gpiob.pb1.into_push_pull_output(&mut gpiob.crl);
```

## Example 3: Add LED Status Indicator

### Add LED on PC13 (built-in LED)
```rust
// In src/main.rs, after GPIO setup (~line 32)
let mut gpioc = dp.GPIOC.split();
let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

// In main loop, after successful USB initialization
if usb_dev.poll(&mut [&mut serial]) {
    led.set_low(); // LED on when USB active
} else {
    led.set_high(); // LED off when USB inactive
}
```

### Blink LED during SPI operations
```rust
// In src/serprog.rs, in execute_spi_op function, before CS low
led.set_low();  // Turn on LED

// After CS high
led.set_high(); // Turn off LED
```

## Example 4: Use SPI2 Instead of SPI1

```rust
// In src/main.rs, change SPI pins (~line 34)
// From:
let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
let miso = gpioa.pa6;
let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);

// To:
let sck = gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh);
let miso = gpiob.pb14;
let mosi = gpiob.pb15.into_alternate_push_pull(&mut gpiob.crh);

// And change SPI initialization (~line 45)
// From:
let mut spi = Spi::spi1(
    dp.SPI1,
    (sck, miso, mosi),
    SPI_MODE,
    9.MHz(),
    clocks,
);

// To:
let mut spi = Spi::spi2(
    dp.SPI2,
    (sck, miso, mosi),
    SPI_MODE,
    9.MHz(),
    clocks,
);
```

## Example 5: Increase Buffer Size for Performance

```rust
// In src/serprog.rs, change buffer size
// From:
buffer: [u8; 512],

// To:
buffer: [u8; 1024],

// And update max transfer length
// In get_command_map or response functions:
SerprogResponse::WriteNMaxLen(512)  // Changed from 256
SerprogResponse::ReadNMaxLen(512)   // Changed from 256

// Update bounds checks in execute_spi_op:
if write_len > 512 || read_len > 512 {  // Changed from 256
    return SerprogResponse::Nak;
}
```

## Example 6: Add Debug Output via SWO

Add to Cargo.toml:
```toml
[dependencies]
cortex-m-semihosting = "0.5"
```

Add to src/main.rs:
```rust
use cortex_m_semihosting::hprintln;

// In main loop, log commands
match serial.read(&mut rx_buf) {
    Ok(count) if count > 0 => {
        hprintln!("Received {} bytes: {:?}", count, &rx_buf[..count]).ok();
        // ... rest of code
    }
}
```

## Example 7: Multiple CS Pins (for multiple flash chips)

```rust
// In src/main.rs, setup multiple CS pins
let mut cs1 = gpiob.pb0.into_push_pull_output(&mut gpiob.crl);
let mut cs2 = gpiob.pb1.into_push_pull_output(&mut gpiob.crl);
cs1.set_high();
cs2.set_high();

// Modify serprog.rs to add chip selection
// Add to SerprogState:
pub struct SerprogState {
    // ... existing fields
    selected_chip: u8,  // 0 or 1
}

// Add new serprog command to select chip
const S_SELECT_CHIP: u8 = 0x20;  // Custom command

// In handle_command:
0x20 => {
    // Next byte will be chip number
    self.state = ParseState::SelectChip;
    None
}

// Add new ParseState variant:
SelectChip,

// In process_byte:
ParseState::SelectChip => {
    self.selected_chip = byte;
    self.state = ParseState::WaitingForCommand;
    Some(SerprogResponse::Ack)
}

// Modify execute_spi_op to use selected chip:
let cs = if self.selected_chip == 0 { &mut cs1 } else { &mut cs2 };
```

## Example 8: Change USB VID/PID

```rust
// In src/main.rs, USB device builder (~line 62)
// From:
let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))

// To your own VID/PID (requires USB-IF registration):
let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x1234, 0x5678))

// Or use different open-source VID:
let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x1209, 0x0001))
```

## Example 9: Add Timeout Protection

```rust
// In src/serprog.rs, add timeout counter
pub struct SerprogState {
    // ... existing fields
    timeout_counter: u32,
}

// In process_byte, increment counter
self.timeout_counter += 1;

// Reset on valid command
fn handle_command(...) {
    self.timeout_counter = 0;
    // ... rest of function
}

// In main.rs, check for timeout
if serprog_state.timeout_counter > 1000000 {
    serprog_state.reset(); // Reset state machine
}
```

## Example 10: Support Different SPI Modes

```rust
// Add to SerprogState:
spi_mode: Mode,

// Add command to set SPI mode (custom extension)
const S_SPI_MODE: u8 = 0x21;

// In handle_command:
0x21 => {
    self.state = ParseState::SetSpiMode;
    None
}

// Add ParseState:
SetSpiMode,

// Process mode byte:
ParseState::SetSpiMode => {
    self.spi_mode = match byte {
        0 => Mode { polarity: Polarity::IdleLow, phase: Phase::CaptureOnFirstTransition },
        1 => Mode { polarity: Polarity::IdleLow, phase: Phase::CaptureOnSecondTransition },
        2 => Mode { polarity: Polarity::IdleHigh, phase: Phase::CaptureOnFirstTransition },
        3 => Mode { polarity: Polarity::IdleHigh, phase: Phase::CaptureOnSecondTransition },
        _ => return Some(SerprogResponse::Nak),
    };
    self.state = ParseState::WaitingForCommand;
    Some(SerprogResponse::Ack)
}

// Note: Would require SPI reconfiguration in practice
```

## Example 11: Power Control for Target

```rust
// Add power control pin
let mut vcc_en = gpioa.pa8.into_push_pull_output(&mut gpioa.crl);
vcc_en.set_low(); // Power off initially

// Add custom command to control power
const S_POWER: u8 = 0x22;

// In handle_command:
0x22 => {
    self.state = ParseState::SetPower;
    None
}

// In process_byte:
ParseState::SetPower => {
    if byte == 1 {
        vcc_en.set_high(); // Power on
        // Add delay for power stabilization
        cortex_m::asm::delay(7200000); // 100ms
    } else {
        vcc_en.set_low(); // Power off
    }
    self.state = ParseState::WaitingForCommand;
    Some(SerprogResponse::Ack)
}
```

## Example 12: Add Hardware Verification

```rust
// Add at start of main(), after clock config
fn verify_hardware() -> bool {
    // Test SPI pins can be set
    let test_ok = true;
    
    // Could add loopback test: connect MOSI to MISO temporarily
    // Send byte, verify received same byte
    
    test_ok
}

if !verify_hardware() {
    // Blink LED rapidly to indicate error
    loop {
        led.toggle();
        cortex_m::asm::delay(3600000); // 50ms
    }
}
```

## Compilation and Testing

After making any modifications, rebuild:
```bash
cargo build --release
```

Test changes:
```bash
# Flash to board
./build.sh

# Test with flashrom
flashrom -p serprog:dev=/dev/ttyACM0:4000000 -V
```
