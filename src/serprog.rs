use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::OutputPin;

// Serprog protocol commands
#[derive(Debug, Clone, Copy)]
pub enum SerprogCommand {
    Nop = 0x00,
    QIface = 0x01,
    QCmdmap = 0x02,
    QPgmname = 0x03,
    QSerbuf = 0x04,
    QBustype = 0x05,
    QChipsize = 0x06,
    QOpbuf = 0x07,
    QWrnmaxlen = 0x08,
    RByte = 0x09,
    RNbytes = 0x0A,
    OInit = 0x0B,
    ODelay = 0x0C,
    OExec = 0x0D,
    SyncNop = 0x0E,
    QRdnmaxlen = 0x0F,
    SSpiOp = 0x10,
    SBustype = 0x11,
    OSpiop = 0x12,
    SPin = 0x13,
}

#[derive(Debug)]
pub enum SerprogResponse {
    Ack,
    Nak,
    InterfaceVersion,
    CommandMap([u8; 32]),
    ProgrammerName(&'static str),
    SerialBufferSize(u16),
    BusTypes(u8),
    OperationBufferSize(u16),
    WriteNMaxLen(u16),
    ReadNMaxLen(u16),
    ReadData(usize), // Index into data buffer
}

impl SerprogResponse {
    pub fn to_bytes<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        match self {
            SerprogResponse::Ack => {
                buf[0] = 0x06; // ACK
                &buf[..1]
            }
            SerprogResponse::Nak => {
                buf[0] = 0x15; // NAK
                &buf[..1]
            }
            SerprogResponse::InterfaceVersion => {
                buf[0] = 0x06; // ACK
                buf[1] = 0x01; // Version 1
                buf[2] = 0x00;
                &buf[..3]
            }
            SerprogResponse::CommandMap(map) => {
                buf[0] = 0x06; // ACK
                buf[1..33].copy_from_slice(map);
                &buf[..33]
            }
            SerprogResponse::ProgrammerName(name) => {
                buf[0] = 0x06; // ACK
                let name_bytes = name.as_bytes();
                let len = name_bytes.len().min(15);
                buf[1..1 + len].copy_from_slice(&name_bytes[..len]);
                buf[1 + len] = 0; // Null terminator
                &buf[..2 + len]
            }
            SerprogResponse::SerialBufferSize(size) => {
                buf[0] = 0x06; // ACK
                buf[1] = (*size & 0xFF) as u8;
                buf[2] = (*size >> 8) as u8;
                &buf[..3]
            }
            SerprogResponse::BusTypes(bus) => {
                buf[0] = 0x06; // ACK
                buf[1] = *bus;
                &buf[..2]
            }
            SerprogResponse::OperationBufferSize(size) => {
                buf[0] = 0x06; // ACK
                buf[1] = (*size & 0xFF) as u8;
                buf[2] = (*size >> 8) as u8;
                &buf[..3]
            }
            SerprogResponse::WriteNMaxLen(len) => {
                buf[0] = 0x06; // ACK
                buf[1] = (*len & 0xFF) as u8;
                buf[2] = (*len >> 8) as u8;
                buf[3] = 0x00;
                &buf[..4]
            }
            SerprogResponse::ReadNMaxLen(len) => {
                buf[0] = 0x06; // ACK
                buf[1] = (*len & 0xFF) as u8;
                buf[2] = (*len >> 8) as u8;
                buf[3] = 0x00;
                &buf[..4]
            }
            SerprogResponse::ReadData(len) => {
                buf[0] = 0x06; // ACK
                &buf[..1 + len]
            }
        }
    }
}

enum ParseState {
    WaitingForCommand,
    SSpiOp { bytes_remaining: usize, write_len: usize, read_len: usize },
    RNbytes { bytes_remaining: usize },
    SBustype,
    ODelay { bytes_remaining: u8 },
}

pub struct SerprogState {
    state: ParseState,
    buffer: [u8; 512],
    buffer_pos: usize,
    spi_initialized: bool,
}

impl SerprogState {
    pub fn new() -> Self {
        Self {
            state: ParseState::WaitingForCommand,
            buffer: [0u8; 512],
            buffer_pos: 0,
            spi_initialized: false,
        }
    }
    
    fn get_command_map() -> [u8; 32] {
        let mut map = [0u8; 32];
        // Set bits for supported commands
        map[0] = 0xFF; // 0x00-0x07 (NOP through QOpbuf)
        map[1] = 0xFF; // 0x08-0x0F (QWrnmaxlen through QRdnmaxlen)
        map[2] = 0x1F; // 0x10-0x13 (SSpiOp, SBustype, OSpiop, SPin)
        map
    }
    
    pub fn process_byte<SPI, CS, E>(
        &mut self,
        byte: u8,
        spi: &mut SPI,
        cs: &mut CS,
    ) -> Option<SerprogResponse>
    where
        SPI: Transfer<u8, Error = E>,
        CS: OutputPin,
    {
        match &mut self.state {
            ParseState::WaitingForCommand => {
                self.handle_command(byte, spi, cs)
            }
            ParseState::SSpiOp { bytes_remaining, write_len, read_len } => {
                self.buffer[self.buffer_pos] = byte;
                self.buffer_pos += 1;
                *bytes_remaining -= 1;
                
                if *bytes_remaining == 0 {
                    let result = self.execute_spi_op(*write_len, *read_len, spi, cs);
                    self.state = ParseState::WaitingForCommand;
                    self.buffer_pos = 0;
                    Some(result)
                } else {
                    None
                }
            }
            ParseState::RNbytes { bytes_remaining } => {
                self.buffer[self.buffer_pos] = byte;
                self.buffer_pos += 1;
                *bytes_remaining -= 1;
                
                if *bytes_remaining == 0 {
                    // For now, just return NAK as we don't support parallel reading
                    self.state = ParseState::WaitingForCommand;
                    self.buffer_pos = 0;
                    Some(SerprogResponse::Nak)
                } else {
                    None
                }
            }
            ParseState::SBustype => {
                self.state = ParseState::WaitingForCommand;
                // Just acknowledge - we only support SPI
                Some(SerprogResponse::Ack)
            }
            ParseState::ODelay { bytes_remaining } => {
                *bytes_remaining -= 1;
                
                if *bytes_remaining == 0 {
                    self.state = ParseState::WaitingForCommand;
                    // Simple delay implementation
                    cortex_m::asm::delay(72000); // ~1ms at 72MHz
                    Some(SerprogResponse::Ack)
                } else {
                    None
                }
            }
        }
    }
    
    fn handle_command<SPI, CS, E>(
        &mut self,
        cmd: u8,
        _spi: &mut SPI,
        _cs: &mut CS,
    ) -> Option<SerprogResponse>
    where
        SPI: Transfer<u8, Error = E>,
        CS: OutputPin,
    {
        match cmd {
            0x00 => Some(SerprogResponse::Ack), // NOP
            
            0x01 => Some(SerprogResponse::InterfaceVersion), // Q_IFACE
            
            0x02 => Some(SerprogResponse::CommandMap(Self::get_command_map())), // Q_CMDMAP
            
            0x03 => Some(SerprogResponse::ProgrammerName("stm32-serprog")), // Q_PGMNAME
            
            0x04 => Some(SerprogResponse::SerialBufferSize(512)), // Q_SERBUF
            
            0x05 => Some(SerprogResponse::BusTypes(0x08)), // Q_BUSTYPE (SPI only)
            
            0x07 => Some(SerprogResponse::OperationBufferSize(512)), // Q_OPBUF
            
            0x08 => Some(SerprogResponse::WriteNMaxLen(256)), // Q_WRNMAXLEN
            
            0x0F => Some(SerprogResponse::ReadNMaxLen(256)), // Q_RDNMAXLEN
            
            0x0B => {
                // O_INIT
                self.spi_initialized = true;
                Some(SerprogResponse::Ack)
            }
            
            0x0C => {
                // O_DELAY - expect 4 more bytes
                self.state = ParseState::ODelay { bytes_remaining: 4 };
                None
            }
            
            0x0E => Some(SerprogResponse::Ack), // SYNCNOP
            
            0x10 => {
                // S_SPI_OP - expect 6 more bytes (write_len:3, read_len:3)
                self.buffer_pos = 0;
                self.state = ParseState::SSpiOp {
                    bytes_remaining: 6,
                    write_len: 0,
                    read_len: 0,
                };
                None
            }
            
            0x11 => {
                // S_BUSTYPE - expect 1 more byte
                self.state = ParseState::SBustype;
                None
            }
            
            0x12 => {
                // O_SPIOP
                if self.spi_initialized {
                    Some(SerprogResponse::Ack)
                } else {
                    Some(SerprogResponse::Nak)
                }
            }
            
            0x13 => Some(SerprogResponse::Ack), // S_PIN_STATE
            
            _ => Some(SerprogResponse::Nak), // Unknown command
        }
    }
    
    fn execute_spi_op<SPI, CS, E>(
        &mut self,
        mut write_len: usize,
        read_len: usize,
        spi: &mut SPI,
        cs: &mut CS,
    ) -> SerprogResponse
    where
        SPI: Transfer<u8, Error = E>,
        CS: OutputPin,
    {
        // Parse lengths from first 6 bytes
        write_len = (self.buffer[0] as usize) 
            | ((self.buffer[1] as usize) << 8)
            | ((self.buffer[2] as usize) << 16);
        
        let read_len = (self.buffer[3] as usize)
            | ((self.buffer[4] as usize) << 8)
            | ((self.buffer[5] as usize) << 16);
        
        if write_len > 256 || read_len > 256 {
            return SerprogResponse::Nak;
        }
        
        // CS low
        let _ = cs.set_low();
        
        // Write phase
        for i in 0..write_len {
            let byte = self.buffer[6 + i];
            let mut data = [byte];
            let _ = spi.transfer(&mut data);
        }
        
        // Read phase
        for i in 0..read_len {
            let mut data = [0xFF];
            match spi.transfer(&mut data) {
                Ok(result) => self.buffer[1 + i] = result[0],
                Err(_) => {
                    let _ = cs.set_high();
                    return SerprogResponse::Nak;
                }
            }
        }
        
        // CS high
        let _ = cs.set_high();
        
        SerprogResponse::ReadData(read_len)
    }
}
