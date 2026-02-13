use crate::dev::{
    Device,
    driver::{Driver, DriverProbeError, MmioError},
    handle::Handle,
    mmio::{IoRangeValidationType, reg::Register},
    serial::Uart,
};
use bitflags::bitflags;

pub struct Uart16550 {
    mmio: &'static mut Uart16550Registers,
}

#[repr(C, packed)]
pub struct Uart16550Registers {
    /// RBR(R) & THR(W) :0x00
    buffer: Register<u8>,
    /// IER(RW)         :0x01
    intr_en: Register<u8>,
    /// IIR(R) & FCR(W) :0x02
    iid_fifo: Register<u8>,
    /// LCR(RW)         :0x03
    line_ctl: Register<u8>,
    /// MCR(W)          :0x04
    modem_crl: Register<u8>,
    /// LSR(R)          :0x05
    line_stat: Register<u8>,
    /// MSR(R)          :0x06
    modem_stat: Register<u8>,
}

impl Uart16550 {
    pub fn create(base: usize) -> Uart16550 {
        Uart16550 {
            mmio: unsafe { &mut *(base as *mut Uart16550Registers) },
        }
    }
}

impl Uart for Uart16550 {
    fn flush(&mut self) {}

    fn read(&mut self) -> Option<u8> {
        None
    }

    fn write(&mut self, word: u8) {
        self.mmio.buffer.write(word);
    }
}

bitflags! {
    pub struct LineStatus: u8{
        /// Data Ready (DR) indicator.
        const DR            = 0b00000001;
        /// Overrun Error (OE) indicator
        const OE            = 0b00000010;
        /// Parity Error (PE) indicator
        const PE            = 0b00000100;
        /// Framing Error (FE) indicator
        const FE            = 0b00001000;
        /// Break Interrupt (BI) indicator
        const BI            = 0b00010000;
        /// Transmit FIFO is empty
        const THR_EMPTY     = 0b00100000;
        /// Transmitter Empty indicator
        const EMPTY_TRANS   = 0b01000000;
        /// Whether at least one parity error, framing error or break indications have been received and are insidethe FIFO.
        const ERR           = 0b10000000;
    }
}

#[derive(Debug)]
pub struct Uart16550Driver;
impl Driver for Uart16550Driver {
    fn get_name(&self) -> &'static str {
        "Uart 16550"
    }

    fn get_comp_strs(&self) -> &'static [&'static str] {
        &["ns16550a"]
    }

    fn probe(&self, dev: Handle<Device>) -> Result<(), DriverProbeError> {
        let io_addr = &dev.info.io_addr;
        if io_addr.is_empty() {
            return Err(DriverProbeError::Mmio(MmioError::AddressNotSpecified));
        }
        let io_addr = &io_addr[0];
        if !io_addr.validate::<Uart16550Registers>(IoRangeValidationType::Compatible) {
            return Err(DriverProbeError::Mmio(MmioError::NotEnoughSpace));
        }
        Ok(())
    }

    fn on_registered(&self) {}
}
