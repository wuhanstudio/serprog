use embedded_hal::spi::SpiBus;
use embedded_hal::digital::OutputPin;

use rtt_target::rprintln;

/// =========================
/// Protocol constants
/// =========================
const S_ACK: u8 = 0x06;
const S_NAK: u8 = 0x15;

/// =========================
/// Response type
/// =========================
#[derive(Debug)]
pub enum SerprogResponse {
    Ack,
    Nak,

    InterfaceVersion,
    CommandMap([u8; 32]),
    ProgrammerName(&'static str),
    SerialBufferSize(u16),
    BusTypes(u8),
    WriteNMaxLen(u32),
    ReadNMaxLen(u32),

    SyncNOP,
    SpiFreq([u8; 4]),

    ReadData(usize),
}

/// =========================
/// Response encoding
/// =========================
impl SerprogResponse {
    pub fn to_bytes<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        match self {
            SerprogResponse::Ack => {
                buf[0] = S_ACK;
                &buf[..1]
            }

            SerprogResponse::Nak => {
                buf[0] = S_NAK;
                &buf[..1]
            }

            SerprogResponse::InterfaceVersion => {
                buf[0] = S_ACK;
                buf[1] = 0x01;
                buf[2] = 0x00;
                &buf[..3]
            }

            SerprogResponse::CommandMap(map) => {
                buf[0] = S_ACK;
                buf[1..33].copy_from_slice(map);
                &buf[..33]
            }

        SerprogResponse::ProgrammerName(name) => {
            buf[0] = S_ACK;

            let b = name.as_bytes();
            let fixed_len = 16;

            // Copy at most 16 bytes
            let copy_len = b.len().min(fixed_len);
            buf[1..1 + copy_len].copy_from_slice(&b[..copy_len]);

            // Pad remaining bytes with 0
            for i in copy_len..fixed_len {
                buf[1 + i] = 0;
            }

            // Total response: 1 (ACK) + 16 bytes
            &buf[..1 + fixed_len]
        }

            SerprogResponse::SerialBufferSize(v) => {
                buf[0] = S_ACK;
                buf[1] = (*v & 0xFF) as u8;
                buf[2] = (*v >> 8) as u8;
                &buf[..3]
            }

            SerprogResponse::BusTypes(v) => {
                buf[0] = S_ACK;
                buf[1] = *v;
                &buf[..2]
            }

            SerprogResponse::WriteNMaxLen(v) => {
                buf[0] = S_ACK;
                buf[1] = (v & 0xFF) as u8;
                buf[2] = ((v >> 8) & 0xFF) as u8;
                buf[3] = ((v >> 16) & 0xFF) as u8;
                &buf[..4]
            }

            SerprogResponse::ReadNMaxLen(v) => {
                buf[0] = S_ACK;
                buf[1] = (v & 0xFF) as u8;
                buf[2] = ((v >> 8) & 0xFF) as u8;
                buf[3] = ((v >> 16) & 0xFF) as u8;
                &buf[..4]
            }

            SerprogResponse::SyncNOP => {
                buf[0] = S_NAK;
                buf[1] = S_ACK;
                &buf[..2]
            }

            SerprogResponse::SpiFreq(v) => {
                buf[0] = S_ACK;
                buf[1..5].copy_from_slice(v);
                &buf[..5]
            }

            SerprogResponse::ReadData(len) => {
                buf[0] = S_ACK;
                &buf[..1 + len]
            }
        }
    }
}

/// =========================
/// State machine
/// =========================
enum ParseState {
    Idle,

    SpiHeader { idx: usize },
    SpiData { remaining: usize },

    Delay { idx: u8, value: u32 },

    WaitBustype,
    WaitSpiFreq { idx: u8 },
    WaitPin,

    // =========================
    // PRESERVED 0x16–0x18
    // =========================
    // WaitSpiCs,
    // WaitSpiMode,
    // WaitSpiCsMode,
}

/// =========================
/// Main state
/// =========================
pub struct SerprogState {
    state: ParseState,

    buffer: [u8; 512],
    buffer_pos: usize,

    spi_initialized: bool,
    delay_value: u32,
}

impl SerprogState {
    pub fn new() -> Self {
        Self {
            state: ParseState::Idle,
            buffer: [0; 512],
            buffer_pos: 0,
            spi_initialized: false,
            delay_value: 0,
        }
    }

    /// =========================
    /// Command map (C-compatible)
    /// =========================
    fn command_map() -> [u8; 32] {
        let mut map = [0u8; 32];
        map[0] = 0x3F;
        map[1] = 0xC9; // FIXED (matches C)
        map[2] = 0x3F;
        map
    }

    /// =========================
    /// Entry point
    /// =========================
    pub fn process_byte<SPI, CS>(
        &mut self,
        byte: u8,
        spi: &mut SPI,
        cs: &mut CS,
    ) -> Option<SerprogResponse>
    where
        SPI: SpiBus<u8>,
        CS: OutputPin,
    {
        // rprintln!("Received byte: 0x{:02X}", byte);
        match &mut self.state {
            ParseState::Idle => self.handle_command(byte),

            ParseState::SpiHeader { idx } => {
                // rprintln!("SPI Header byte {}: 0x{:02X}", idx, byte);

                self.buffer[*idx] = byte;
                *idx += 1;

                if *idx == 6 {
                    let write_len = (self.buffer[0] as usize)
                        | ((self.buffer[1] as usize) << 8)
                        | ((self.buffer[2] as usize) << 16);

                    if write_len > 0 {
                        // rprintln!("SPI Write length: {}", write_len);
                        self.state = ParseState::SpiData {
                            remaining: write_len,
                        };
                    } else {
                        return self.execute_spi(spi, cs);
                    }
                }
                None
            }

            ParseState::SpiData { remaining } => {
                // rprintln!("SPI Data byte ({} remaining): 0x{:02X}", remaining, byte);
                self.buffer[6 + self.buffer_pos] = byte;
                self.buffer_pos += 1;
                *remaining -= 1;

                if *remaining == 0 {
                    // rprintln!("SPI Data complete");
                    self.state = ParseState::Idle;
                    return self.execute_spi(spi, cs);
                }
                None
            }

            ParseState::Delay { idx, value } => {
                *value |= (byte as u32) << (8 * (*idx as u32));
                *idx += 1;

                if *idx == 4 {
                    self.delay_value = *value;
                    self.state = ParseState::Idle;
                    Some(SerprogResponse::Ack)
                } else {
                    None
                }
            }

            ParseState::WaitSpiFreq { idx } => {
                self.buffer[*idx as usize] = byte;
                *idx += 1;

                if *idx == 4 {
                    let resp = SerprogResponse::SpiFreq([
                        self.buffer[0],
                        self.buffer[1],
                        self.buffer[2],
                        self.buffer[3],
                    ]);
                    self.state = ParseState::Idle;
                    Some(resp)
                } else {
                    None
                }
            }

            ParseState::WaitBustype
            | ParseState::WaitPin
            => {
                // All are 1-byte consume + ACK behavior (C-compatible)
                self.state = ParseState::Idle;
                Some(SerprogResponse::Ack)
            }
        }
    }

    /// =========================
    /// Command handler
    /// =========================
    fn handle_command(&mut self, cmd: u8) -> Option<SerprogResponse> {
        match cmd {
            0x00 => Some(SerprogResponse::Ack),

            0x01 => Some(SerprogResponse::InterfaceVersion),

            0x02 => Some(SerprogResponse::CommandMap(Self::command_map())),

            0x03 => Some(SerprogResponse::ProgrammerName("stm32-serprog")),

            0x04 => Some(SerprogResponse::SerialBufferSize(512)),

            0x05 => Some(SerprogResponse::BusTypes(0x08)),

            0x08 => Some(SerprogResponse::WriteNMaxLen(256)),

            0x0A => Some(SerprogResponse::ReadNMaxLen(256)),

            0x0B => {
                self.spi_initialized = true;
                Some(SerprogResponse::Ack)
            }

            0x0E => {
                self.state = ParseState::Delay { idx: 0, value: 0 };
                None
            }

            0x0F => {
                cortex_m::asm::delay(self.delay_value);
                Some(SerprogResponse::Ack)
            }

            0x10 => Some(SerprogResponse::SyncNOP),
            0x11 => Some(SerprogResponse::ReadNMaxLen(256)),

            0x12 => {
                self.state = ParseState::WaitBustype;
                None
            }

            0x13 => {
                self.buffer_pos = 0;
                self.state = ParseState::SpiHeader { idx: 0 };
                None
            }

            0x14 => {
                self.state = ParseState::WaitSpiFreq { idx: 0 };
                None
            }

            // =========================
            // PRESERVED C COMMANDS
            // =========================
            0x15 => {
                self.state = ParseState::WaitPin;
                None
            }

            // 0x16 => {
            //     self.state = ParseState::WaitSpiCs;
            //     None
            // }

            // 0x17 => {
            //     self.state = ParseState::WaitSpiMode;
            //     None
            // }

            // 0x18 => {
            //     self.state = ParseState::WaitSpiCsMode;
            //     None
            // }

            _ => Some(SerprogResponse::Nak),
        }
    }

    /// =========================
    /// SPI execution
    /// =========================
    fn execute_spi<SPI, CS>(
        &mut self,
        spi: &mut SPI,
        cs: &mut CS,
    ) -> Option<SerprogResponse>
    where
        SPI: SpiBus<u8>,
        CS: OutputPin,
    {
        let write_len = (self.buffer[0] as usize)
            | ((self.buffer[1] as usize) << 8)
            | ((self.buffer[2] as usize) << 16);

        let read_len = (self.buffer[3] as usize)
            | ((self.buffer[4] as usize) << 8)
            | ((self.buffer[5] as usize) << 16);

        if write_len > 256 || read_len > 256 {
            return Some(SerprogResponse::Nak);
        }

        let _ = cs.set_low();

        for i in 0..write_len {
            let mut b = [self.buffer[6 + i]];
            if spi.transfer_in_place(&mut b).is_err() {
                let _ = cs.set_high();
                return Some(SerprogResponse::Nak);
            }
        }

        // rprintln!("SPI write complete: {} bytes", write_len);

        for i in 0..read_len {
            let mut b = [0xFF];
            let _ = spi.transfer_in_place(&mut b);
            self.buffer[i+1] = b[0];
        }

        let _ = cs.set_high();

        Some(SerprogResponse::ReadData(read_len))
    }
}
