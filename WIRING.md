# Wiring Diagrams and Examples

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

## Reading SPI Flash from Router/Board

```
┌─────────────────────┐
│   STM32 Blue Pill   │
│                     │
│  PA5 (SCK)  ────────┼────┐
│  PA6 (MISO) ────────┼────┤
│  PA7 (MOSI) ────────┼────┤    ┌─────────────────┐
│  PB0 (CS)   ────────┼────┤    │  Target Board   │
│  GND        ────────┼────┼────┤ (e.g., Router)  │
└─────────────────────┘    │    │                 │
                           │    │  [SPI Flash]    │
                           └────┤  Chip on board  │
                                └─────────────────┘

NOTE: DO NOT connect 3.3V if target board is powered!
      Only connect: SCK, MISO, MOSI, CS, GND
```

## With Logic Analyzer for Debugging

```
┌─────────────────────┐
│   STM32 Blue Pill   │           ┌──────────────┐
│                     │           │              │
│  PA5 (SCK)  ────────┼───┬───────┤ CLK  W25Q64  │
│  PA6 (MISO) ────────┼───┼───┬───┤ DO   Flash   │
│  PA7 (MOSI) ────────┼───┼───┼───┤ DI           │
│  PB0 (CS)   ────────┼───┼───┼───┤ CS           │
│  GND        ────────┼───┼───┼───┤ GND          │
└─────────────────────┘   │   │   └──────────────┘
                          │   │
        ┌─────────────────┴───┴─────────┐
        │    Logic Analyzer              │
        │  CH0: SCK                      │
        │  CH1: MOSI                     │
        │  CH2: MISO                     │
        │  CH3: CS                       │
        │  GND: Common Ground            │
        └────────────────────────────────┘
```

## Pin Compatibility Table

### Common SPI Flash Pinouts

#### 8-pin SOIC/SOP Package
```
     ┌───────┐
 CS  │1    8│ VCC
 DO  │2    7│ HOLD#
 WP# │3    6│ CLK
 GND │4    5│ DI
     └───────┘
```

#### 8-pin WSON Package
```
     ┌───────┐
 CS  │1    8│ VCC
 DO  │2    7│ HOLD#
 WP# │3    6│ CLK
 GND │4    5│ DI
     └───────┘
```

### Blue Pill to Flash Mapping
```
Blue Pill | Flash Pin | Function
----------|-----------|----------
PA5       | Pin 6     | CLK/SCK
PA6       | Pin 2     | DO/MISO
PA7       | Pin 5     | DI/MOSI
PB0       | Pin 1     | CS
GND       | Pin 4     | GND
3.3V      | Pin 8     | VCC (if needed)
```

## Example: Reading BIOS from Desktop Motherboard

```
WARNING: Only do this with the computer POWERED OFF and unplugged!

┌─────────────────────┐
│   STM32 Blue Pill   │
│                     │
│  PA5 (SCK)  ────────┼────────────┐
│  PA6 (MISO) ────────┼───────┐    │
│  PA7 (MOSI) ────────┼──┐    │    │
│  PB0 (CS)   ────────┼──┼────┼────┼────┐
│  GND        ────────┼──┼────┼────┼────┼────┐
└─────────────────────┘  │    │    │    │    │
                         │    │    │    │    │
                    ┌────┴────┴────┴────┴────┴─────┐
                    │ SOIC-8 Clip or Test Hooks    │
                    │          on                  │
                    │   BIOS Flash Chip (25 series)│
                    │    on Motherboard            │
                    └──────────────────────────────┘

Recommended: Use SOIC-8 test clip for easy connection
```

## Example: Programming ESP8266 Flash

```
┌─────────────────────┐          ┌──────────────┐
│   STM32 Blue Pill   │          │   ESP8266    │
│                     │          │              │
│  PA5 (SCK)  ────────┼──────────┤ GPIO14 (SCK) │
│  PA6 (MISO) ────────┼──────────┤ GPIO12 (MISO)│
│  PA7 (MOSI) ────────┼──────────┤ GPIO13 (MOSI)│
│  PB0 (CS)   ────────┼──────────┤ GPIO15 (CS)  │
│  GND        ────────┼──────────┤ GND          │
└─────────────────────┘          └──────────────┘

NOTE: ESP8266 must be in flash download mode:
      - GPIO0: LOW (connected to GND)
      - GPIO2: HIGH (pull-up or 3.3V)
      - GPIO15: LOW (pull-down or our CS)
      - CH_PD: HIGH (pull-up or 3.3V)
```

## Pin Alternative Configurations

### Alternative CS Pins
```rust
// In main.rs, you can use different GPIO pins for CS:

// Option 1: PB1 (default in code is PB0)
let mut cs = gpiob.pb1.into_push_pull_output(&mut gpiob.crl);

// Option 2: PA4
let mut cs = gpioa.pa4.into_push_pull_output(&mut gpioa.crl);

// Option 3: PA3
let mut cs = gpioa.pa3.into_push_pull_output(&mut gpioa.crl);
```

### Using SPI2 Instead of SPI1
```rust
// SPI2 pins: SCK=PB13, MISO=PB14, MOSI=PB15
let sck = gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh);
let miso = gpiob.pb14;
let mosi = gpiob.pb15.into_alternate_push_pull(&mut gpiob.crh);

let mut spi = Spi::spi2(
    dp.SPI2,
    (sck, miso, mosi),
    SPI_MODE,
    9.MHz(),
    clocks,
);
```

## Level Shifting for 5V Systems

If you need to program a 5V flash chip (rare, but exists):

```
┌─────────────────────┐          ┌──────────────┐
│   STM32 Blue Pill   │          │ Level Shifter│
│       (3.3V)        │          │  (TXS0108E)  │
│                     │          │              │
│  PA5 (SCK)  ────────┼──────────┤ A1 ────── B1 ├─────── CLK
│  PA6 (MISO) ────────┼──────────┤ A2 ────── B2 ├─────── DO
│  PA7 (MOSI) ────────┼──────────┤ A3 ────── B3 ├─────── DI
│  PB0 (CS)   ────────┼──────────┤ A4 ────── B4 ├─────── CS
│                     │          │              │
│  3.3V       ────────┼──────────┤ VCCA  VCCB   ├─────── 5V
│  GND        ────────┼──────────┤ GND          ├─────── GND
└─────────────────────┘          └──────────────┘
```

## Breadboard Layout Example

```
     STM32 Blue Pill
    ┌──────────────────────┐
3.3V│○                    ○│GND
 GND│○                    ○│GND
  5V│○                    ○│3.3V
  B9│○                    ○│B8
  B8│○                    ○│B7
  B7│○                    ○│B6
  B6│○                    ○│B5
  B5│○                    ○│B4
  B4│○                    ○│B3
  B3│○                    ○│A15
 A15│○                    ○│A12  ← USB D+
 A12│○                    ○│A11  ← USB D-
 A11│○                    ○│A10
 A10│○                    ○│A9
  A9│○                    ○│A8
  A8│○                    ○│B15
 B15│○                    ○│B14
 B14│○                    ○│B13
 B13│○                    ○│B12
 B12│○                    ○│B11
 B11│○                    ○│B10
 B10│○                    ○│B1
  B1│○                    ○│B0   ← CS
  B0│○                    ○│A7   ← MOSI
  A7│○                    ○│A6   ← MISO
  A6│○                    ○│A5   ← SCK
  A5│○                    ○│A4
  A4│○                    ○│A3
  A3│○                    ○│A2
  A2│○                    ○│A1
  A1│○                    ○│A0
  A0│○                    ○│C15
 C15│○                    ○│C14
 C14│○                    ○│C13
 C13│○                    ○│VBAT
    └──────────────────────┘
```

## Important Notes

1. **Voltage Levels**: STM32 is 3.3V. Most SPI flash chips are 3.3V. Never apply 5V to SPI pins.

2. **Current Limits**: STM32 pins can source/sink max 25mA. If powering flash from Blue Pill, ensure total current < 25mA.

3. **Grounding**: Always connect GND between Blue Pill and target. Poor grounding causes communication errors.

4. **Wire Length**: Keep SPI wires short (<20cm) for reliable 9 MHz operation. Longer wires may require lower SPI speed.

5. **Decoupling**: Add 100nF ceramic capacitor close to flash chip VCC/GND for stability.

6. **Write Protection**: Some flash chips have WP# (Write Protect) and HOLD# pins. Connect both to VCC (3.3V) to disable protection.
