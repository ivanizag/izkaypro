use std::fmt;

use iz80::Machine;
use super::FloppyController;
use super::keyboard_unix::Keyboard;

/* Memory map:

    0x0000-0xffff: 64Kb of RAM
    If bank1 is selected:
        0x0000-0x2fff: 12Kb of ROM
        0x3000-0x3fff: 4Kb of VRAM

*/

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum SystemBit {
    DriveA = 0x01,
    DriveB = 0x02,
    Unused = 0x04,
    CentronicsReady = 0x08,
    CentronicsStrobe = 0x10,
    DoubleDensity = 0x20,
    Motors = 0x40,
    Bank = 0x80,
}

impl fmt::Display for SystemBit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if *self as u8 & SystemBit::DriveA as u8 != 0           {write!(f, "DriveA ")?;}
        if *self as u8 & SystemBit::DriveB as u8 != 0           {write!(f, "DriveB ")?;}
        if *self as u8 & SystemBit::Unused as u8 != 0           {write!(f, "Unused ")?;}
        if *self as u8 & SystemBit::CentronicsReady  as u8 != 0 {write!(f, "CentronicsReady ")?;}
        if *self as u8 & SystemBit::CentronicsStrobe as u8 != 0 {write!(f, "CentronicsStrobe ")?;}
        if *self as u8 & SystemBit::DoubleDensity as u8 != 0    {write!(f, "DoubleDensity ")?;}
        if *self as u8 & SystemBit::Motors as u8 != 0           {write!(f, "Motors ")?;}
        if *self as u8 & SystemBit::Bank as u8 != 0             {write!(f, "ROM ")?;}
        Ok(())
    }
}

const IO_PORT_NAMES: [&'static str; 32] = [
    /* 0x00 */"Baud rate A, serial",
    /* 0x01 */"-",
    /* 0x02 */"-",
    /* 0x03 */"-",
    /* 0x04 */"SIO A data register.",
    /* 0x05 */"SIO B data register, keyboard.",
    /* 0x06 */"SIO A control register.",
    /* 0x07 */"SIO B control register, keyboard.",
    /* 0x08 */"PIO 1 channel A data register.",
    /* 0x09 */"PIO 1 channel A control register.",
    /* 0x0a */"PIO 1 channel B data register.",
    /* 0x0b */"PIO 1 channel B control register.",
    /* 0x0c */"Baud rate B, keyboard.",
    /* 0x0d */"-",
    /* 0x0e */"-",
    /* 0x0f */"-",
    /* 0x10 */"Floppy controller, Command register.",
    /* 0x11 */"Floppy controller, Track register.",
    /* 0x12 */"Floppy controller, Sector register.",
    /* 0x13 */"Floppy controller, Data register.",
    /* 0x14 */"-",
    /* 0x15 */"-",
    /* 0x16 */"-",
    /* 0x17 */"-",
    /* 0x18 */"-",
    /* 0x19 */"-",
    /* 0x1a */"-",
    /* 0x1b */"-",
    /* 0x1c */"SIO 2 channel A data register: ",
    /* 0x1d */"PIO 2 channel A control register.",
    /* 0x1e */"PIO 2 channel B data register.",
    /* 0x1f */"PIO 2 channel B control register.",
    ];


static ROM: &'static [u8] = include_bytes!("../roms/81-149c.rom");
//static ROM: &'static [u8] = include_bytes!("../roms/81-232.rom");

pub struct KayproMachine {
    ram: [u8; 65536],
    pub vram: [u8; 4096],
    pub vram_dirty: bool,
    system_bits: u8,

    trace_io: bool,

    pub keyboard: Keyboard,
    pub floppy_controller: FloppyController,
}

impl KayproMachine {
    pub fn new(floppy_controller: FloppyController, trace_io: bool) -> KayproMachine {
        KayproMachine {
            ram: [0; 65536],
            vram: [0; 4096],
            vram_dirty: false,
            system_bits: SystemBit::Bank as u8,
            trace_io: trace_io,
            keyboard: Keyboard::new(),
            floppy_controller: floppy_controller,
        }
    }

    fn is_rom_rank(&self) -> bool {
        self.system_bits & SystemBit::Bank as u8 != 0
    }
}

impl Machine for KayproMachine {
    fn peek(&self, address: u16) -> u8 {
        if address < 0x3000 && self.is_rom_rank() {
            ROM[(address as usize) % ROM.len()]
        } else if address < 0x4000 && self.is_rom_rank() {
            self.vram[address as usize - 0x3000]
        } else {
            self.ram[address as usize]
        }
    }

    fn poke(&mut self, address: u16, value: u8) {
        if address < 0x3000 && self.is_rom_rank() {
            // Ignore writes to ROM
        } else if address < 0x4000 && self.is_rom_rank() {
            self.vram[address as usize - 0x3000] = value;
            self.vram_dirty = true;
        } else {
            self.ram[address as usize] = value;
        }
    }



    fn port_out(&mut self, address: u16, value: u8) {

        let port = address as u8 & 0b_1001_1111; // Pins used
        if port > 0x80 {
            // Pin 7 is tied to enable of the 3-8 decoder
            if self.trace_io {
                println!("OUT(0x{:02x} 'Ignored', 0x{:02x})", port, value);
            }
            return
        }

        if self.trace_io {
            println!("OUT(0x{:02x} '{}', 0x{:02x}): ", port, IO_PORT_NAMES[port as usize], value);
        }
        match port {
            // Floppy controller
            0x10 => self.floppy_controller.put_command(value),
            0x11 => self.floppy_controller.put_track(value),
            0x12 => self.floppy_controller.put_sector(value),
            0x13 => self.floppy_controller.put_data(value),
            // System bits
            0x1c => {
                self.system_bits = value;
                if self.trace_io {
                    println!("{}", self.system_bits);
                }
            },
            _ => {}
        } 
    }

    fn port_in(&mut self, address: u16) -> u8 {
        let port = address as u8 & 0b_1001_1111; // Pins used
        if port > 0x80 {
            // Pin 7 is tied to enable of the 3-8 decoder
            if self.trace_io {
                println!("IN(0x{:02x} 'Ignored')", port);
            }
            return 0x00
        }

        let value = match port {

            0x05 => self.keyboard.get_key(),
            0x07 => if self.keyboard.is_key_pressed() {0x01} else {0x00},

            // Floppy controller
            0x10 => self.floppy_controller.get_status(),
            0x11 => self.floppy_controller.get_track(),
            0x12 => self.floppy_controller.get_sector(),
            0x13 => self.floppy_controller.get_data(),
            0x1c => self.system_bits,
            _ => 0xca,
        }; 
        if self.trace_io && port != 0x13 {
            println!("IN(0x{:02x} '{}') = 0x{:02x}", port, IO_PORT_NAMES[port as usize], value);
        }
        value
    }
}
