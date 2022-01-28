use std::fs::{File};
use std::io::{Write};

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
    Side2 = 0x04,
    CentronicsReady = 0x08,
    CentronicsStrobe = 0x10,
    SingleDensity = 0x20,
    MotorsOff = 0x40,
    Bank = 0x80,
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
    /* 0x1c */"PIO 2 channel A data register: ",
    /* 0x1d */"PIO 2 channel A control register.",
    /* 0x1e */"PIO 2 channel B data register.",
    /* 0x1f */"PIO 2 channel B control register.",
    ];


//static ROM: &'static [u8] = include_bytes!("../roms/81-149c.rom");
static ROM: &'static [u8] = include_bytes!("../roms/81-232.rom");
//static ROM: &'static [u8] = include_bytes!("../roms/kplus83.rom");
//static ROM: &'static [u8] = include_bytes!("../roms/omni2.u47");
//static ROM: &'static [u8] = include_bytes!("../roms/kaypro_ii_roadrunner_1_5.bin");
//static ROM: &'static [u8] = include_bytes!("../roms/trom34_3.rom");

pub struct KayproMachine {
    ram: [u8; 65536],
    pub vram: [u8; 4096],
    pub vram_dirty: bool,
    pub system_bits: u8,

    trace_io: bool,
    trace_system_bits: bool,

    pub keyboard: Keyboard,
    pub floppy_controller: FloppyController,
}

impl KayproMachine {
    pub fn new(floppy_controller: FloppyController,
            trace_io: bool, trace_system_bits: bool) -> KayproMachine {
        KayproMachine {
            ram: [0; 65536],
            vram: [0; 4096],
            vram_dirty: false,
            system_bits: SystemBit::Bank as u8 | SystemBit::MotorsOff as u8,
            trace_io: trace_io,
            trace_system_bits: trace_system_bits,
            keyboard: Keyboard::new(),
            floppy_controller: floppy_controller,
        }
    }

    pub fn is_rom_rank(&self) -> bool {
        self.system_bits & SystemBit::Bank as u8 != 0
    }

    fn update_system_bits(&mut self, bits: u8) {
        self.system_bits = bits;
        if bits & SystemBit::DriveA as u8 != 0 {
            self.floppy_controller.set_drive(0);
        } else if bits & SystemBit::DriveB as u8 != 0 {
            self.floppy_controller.set_drive(1);
        }

        let motor_off = bits & SystemBit::MotorsOff as u8 != 0;
        self.floppy_controller.set_motor(!motor_off);

        let single_density = bits & SystemBit::SingleDensity as u8 != 0;
        self.floppy_controller.set_single_density(single_density);

        let side_2 = bits & SystemBit::Side2 as u8 != 0;
        self.floppy_controller.set_side(side_2);

        if self.trace_system_bits {
            print_system_bits(self.system_bits);
        }
    }

    pub fn save_bios(&self) {
        let start = self.ram[1] as usize +
            ((self.ram[2] as usize) << 8) - 3;
        let end = 0xfc00;

        let mut file = File::create(format!("bios_{:x}.bin", start),).unwrap();
        file.write_all(&self.ram[start..end]).unwrap();
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
            // Writes to ROM go to the RAM
            self.ram[address as usize] = value;
        } else if address < 0x4000 && self.is_rom_rank() {
            self.vram[address as usize - 0x3000] = value;
            self.vram_dirty = true;
        } else {
            self.ram[address as usize] = value;
        }
    }

    fn port_out(&mut self, address: u16, value: u8) {

        let port = address as u8 & 0b_1001_1111; // Pins used
        if port >= 0x80 {
            // Pin 7 is tied to enable of the 3-8 decoder
            if self.trace_io {
                println!("OUT(0x{:02x} 'Ignored', 0x{:02x})", port, value);
            }
            return
        }

        if self.trace_io && port != 0x1c {
            println!("OUT(0x{:02x} '{}', 0x{:02x}): ", port, IO_PORT_NAMES[port as usize], value);
        }
        match port {
            // Floppy controller
            0x10 => self.floppy_controller.put_command(value),
            0x11 => self.floppy_controller.put_track(value),
            0x12 => self.floppy_controller.put_sector(value),
            0x13 => self.floppy_controller.put_data(value),
            // System bits
            0x1c => self.update_system_bits(value),
            _ => {}
        } 
    }

    fn port_in(&mut self, address: u16) -> u8 {
        let port = address as u8 & 0b_1001_1111; // Pins used
        if port > 0x80 { // Pin 7 is tied to enable of the 3-8 decoder
            if self.trace_io {
                println!("IN(0x{:02x} 'Ignored')", port);
            }
            return 0x00
        }

        let value = match port {

            0x05 => self.keyboard.get_key(),
            0x07 => (if self.keyboard.is_key_pressed() {1} else {0}) + 0x04,

            // Floppy controller
            0x10 => self.floppy_controller.get_status(),
            0x11 => self.floppy_controller.get_track(),
            0x12 => self.floppy_controller.get_sector(),
            0x13 => self.floppy_controller.get_data(),
            0x1c => self.system_bits,
            _ => 0xca,
        }; 

        if self.trace_io && port != 0x13 && port != 0x07 && port != 0x1c {
            println!("IN(0x{:02x} '{}') = 0x{:02x}", port, IO_PORT_NAMES[port as usize], value);
        }
        value
    }
}

fn print_system_bits(system_bits: u8) {
    print!("System bits: ");
    if system_bits & SystemBit::DriveA as u8 != 0           {print!("DriveA ");}
    if system_bits & SystemBit::DriveB as u8 != 0           {print!("DriveB ");}
    if system_bits & SystemBit::Side2 as u8 != 0           {print!("Side2 ");}
    if system_bits & SystemBit::CentronicsReady  as u8 != 0 {print!("CentronicsReady ");}
    if system_bits & SystemBit::CentronicsStrobe as u8 != 0 {print!("CentronicsStrobe ");}
    if system_bits & SystemBit::SingleDensity as u8 != 0    {print!("SingleDensity ");}
    if system_bits & SystemBit::MotorsOff as u8 != 0        {print!("MotorsOff ");}
    if system_bits & SystemBit::Bank as u8 != 0             {print!("ROM ");}
    println!();
}
