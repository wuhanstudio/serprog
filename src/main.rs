#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use stm32f1xx_hal::{
    pac,
    prelude::*,
    spi::{Mode, Phase, Polarity, Spi},
    usb::{Peripheral, UsbBus},
};
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

mod serprog;
use serprog::{SerprogCommand, SerprogResponse, SerprogState};

// SPI Mode 0: CPOL=0, CPHA=0
const SPI_MODE: Mode = Mode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnFirstTransition,
};

#[entry]
fn main() -> ! {
    // Get device peripherals
    let dp = pac::Peripherals::take().unwrap();
    
    // Configure clocks
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    
    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(72.MHz())
        .pclk1(36.MHz())
        .freeze(&mut flash.acr);
    
    assert!(clocks.usbclk_valid());
    
    // Configure GPIO
    let mut gpioa = dp.GPIOA.split();
    let mut gpiob = dp.GPIOB.split();
    
    // SPI1 pins: SCK=PA5, MISO=PA6, MOSI=PA7
    let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
    let miso = gpioa.pa6;
    let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);
    
    // CS pin: PB0 (manually controlled)
    let mut cs = gpiob.pb0.into_push_pull_output(&mut gpiob.crl);
    cs.set_high(); // CS inactive (high)
    
    // Configure SPI at 9 MHz (PCLK2/8 = 72MHz/8)
    let mut spi = Spi::spi1(
        dp.SPI1,
        (sck, miso, mosi),
        SPI_MODE,
        9.MHz(),
        clocks,
    );
    
    // Configure USB
    let mut gpioa_usb = gpioa.crl;
    let usb_dm = gpioa.pa11;
    let usb_dp = gpioa.pa12.into_floating_input(&mut gpioa.crh);
    
    let usb = Peripheral {
        usb: dp.USB,
        pin_dm: usb_dm,
        pin_dp: usb_dp,
    };
    
    let usb_bus = UsbBus::new(usb);
    
    let mut serial = SerialPort::new(&usb_bus);
    
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("STM32")
        .product("serprog")
        .serial_number("SERPROG01")
        .device_class(USB_CLASS_CDC)
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
