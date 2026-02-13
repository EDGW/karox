use bitflags::bitflags;

use crate::dev::{mmio::reg::Register, serial::Uart};

pub struct Uart16550 {
    mmio: &'static mut Uart16550Reg,
}

#[repr(C, packed)]
pub struct Uart16550Reg {
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
            mmio: unsafe { &mut *(base as *mut Uart16550Reg) },
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
