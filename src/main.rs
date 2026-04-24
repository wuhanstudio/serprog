#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m::asm::delay;
use cortex_m_rt::entry;
use stm32f1xx_hal::{
    pac,
    prelude::*,
    spi::{Mode, Phase, Polarity},
    usb::{Peripheral, UsbBus},
};
use stm32f1xx_hal::rcc;

use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

mod serprog;
use serprog::SerprogState;

use rtt_target::{rtt_init_print, rprintln};

// SPI Mode 0: CPOL=0, CPHA=0
// const SPI_MODE: Mode = Mode {
//     polarity: Polarity::IdleLow,
//     phase: Phase::CaptureOnFirstTransition,
// };

pub const MODE: Mode = Mode {
    phase: Phase::CaptureOnSecondTransition,
    polarity: Polarity::IdleHigh,
};

#[entry]
fn main() -> ! {
    rtt_init_print!();

    // Get device peripherals
    let Some(dp) = pac::Peripherals::take() else {
        rprintln!("Failed to take peripheral ownership");
        loop {}
    };

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.freeze(
        rcc::Config::hse(8.MHz()).sysclk(48.MHz()).pclk1(24.MHz()),
        &mut flash.acr,
    );
    
    // Configure GPIO
    let mut gpioa = dp.GPIOA.split(&mut rcc);
    let mut gpiob = dp.GPIOB.split(&mut rcc);
    
    // SPI1 pins: SCK=PA5, MISO=PA6, MOSI=PA7
    let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
    let miso = gpioa.pa6;
    let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);

    // CS pin: PB0 (manually controlled)
    let mut cs = gpiob.pb0.into_push_pull_output(&mut gpiob.crl);
    cs.set_high(); // CS inactive (high)

    // let sck = gpioa.pa5;
    // let miso = gpioa.pa6;
    // let mosi = gpioa.pa7;
    // let cs = gpioa.pa4.into_push_pull_output(&mut gpioa.crl);

    let mut spi = dp
        .SPI1
        //.remap(&mut afio.mapr) // if you want to use PB3, PB4, PB5
        .spi((Some(sck), Some(miso), Some(mosi)), MODE, 9.MHz(), &mut rcc);

    // BluePill board has a pull-up resistor on the D+ line.
    // Pull the D+ pin down to send a RESET condition to the USB bus.
    // This forced reset is needed only for development, without it host
    // will not reset your device when you upload new firmware.
    let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
    usb_dp.set_low();
    delay(rcc.clocks.sysclk().raw() / 100);

    let usb = Peripheral {
        usb: dp.USB,
        pin_dm: gpioa.pa11,
        pin_dp: usb_dp.into_floating_input(&mut gpioa.crh),
    };
    let usb_bus = UsbBus::new(usb);
    let mut serial = SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .device_class(USB_CLASS_CDC)
        .strings(&[StringDescriptors::default()
            .manufacturer("Fake Company")
            .product("Serial port")
            .serial_number("TEST")])
        .unwrap()
        .build();
    
    let mut serprog_state = SerprogState::new();
    let mut rx_buf = [0u8; 64];
    let mut tx_buf = [0u8; 512];
    
    loop {
        if !usb_dev.poll(&mut [&mut serial]) {
            continue;
        }
        
        // Read incoming data
        match serial.read(&mut rx_buf) {
            Ok(count) if count > 0 => {
                // Process each byte as a potential command
                for &byte in &rx_buf[..count] {
                    if let Some(response) = serprog_state.process_byte(byte, &mut spi, &mut cs) {
                        let response_bytes = response.to_bytes(&mut tx_buf);
                        let mut written = 0;
                        
                        while written < response_bytes.len() {
                            match serial.write(&response_bytes[written..]) {
                                Ok(len) => written += len,
                                Err(_) => break,
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
