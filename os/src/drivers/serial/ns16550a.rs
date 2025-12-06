
use bitflags::bitflags;

use crate::drivers::{mmio::Register, serial::Uart};

pub struct Ns16550a {
    mmio: &'static mut Ns16550aReg,
}

#[repr(C, packed)]
pub struct Ns16550aReg {
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

impl Ns16550a {
    pub fn create(base: usize) -> Ns16550a {
        Ns16550a {
            mmio: unsafe { &mut *(base as *mut Ns16550aReg) },
        }
    }
}

impl Uart for Ns16550a {
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
