use std::fs::File;
use std::io::{Read /*, Write*/};

const SECTOR_COUNT: usize = 10; // For the DD disk
const SECTOR_SIZE: usize = 512;

static DISK_IMAGE_DEFAULT: &'static [u8] = include_bytes!("../disks/KPII-149.BIN");
static DISK_IMAGE_DEFAULT_2: &'static [u8] = include_bytes!("../disks/K-PFILER.BIN");

pub struct FloppyController {
    pub motor_on: bool,
    pub disk: u8,
    track: u8,
    sector: u8,
    pub single_density: bool,
    data: u8,
    status: u8,

    content_a: Vec<u8>,
    content_b: Vec<u8>,

    read_index: usize,
    read_last: usize,

    data_buffer: Vec<u8>,


    pub raise_nmi: bool,
    pub trace: bool
}

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum FDCStatus {
    _NotReady = 0x01,
    _WriteProtected = 0x02,
    _WriteFault = 0x04,
    _RecorddNotFound = 0x08,
    _CRCError = 0x10,
    LostDataOrTrack0 = 0x20,
    _DataRequest = 0x40,
    Busy = 0x80,
}

impl FloppyController {
    pub fn new(trace: bool) -> FloppyController {
        FloppyController {
            motor_on: false,
            disk: 0,
            track: 0,
            sector: 0,
            single_density: false,
            data: 0,
            status: 0,

            content_a: DISK_IMAGE_DEFAULT.to_vec(),
            content_b: DISK_IMAGE_DEFAULT_2.to_vec(),

            read_index: 0,
            read_last: 0,

            data_buffer: Vec::new(),

            raise_nmi: false,
            trace: trace,
        }
    }

    pub fn load_disk(&mut self, filename: &str, disk_b: bool) {
        // Load the file contents in content_a
        let mut file = File::open(filename).unwrap();
        let mut content = Vec::new();
        file.read_to_end(&mut content).unwrap();
        if disk_b {
            self.content_b = content;
        } else {
            self.content_a = content;
        }
    }

    pub fn set_motor(&mut self, motor_on: bool) {
        self.motor_on = motor_on;
    }

    pub fn set_single_density(&mut self, single_density: bool) {
        self.single_density = single_density;
    }

    pub fn set_disk(&mut self, disk: u8) {
        self.disk = disk;
    }

    fn content(&mut self) -> &mut Vec<u8> {
        if self.disk == 0 {
            &mut self.content_a
        } else {
            &mut self.content_b
        }
    }

    fn inc_sector(&mut self) {
        self.sector += 1;
        if self.sector == SECTOR_COUNT as u8 {
            self.sector = 0;
        }
    }

    pub fn put_command(&mut self, command: u8) {
        if (command & 0xf0) == 0x00 {
            // RESTORE command, type I
            // 0000_hVrr
            if self.trace {
                println!("FDC: Restore");
            }
            self.read_index = 0;
            self.read_last = 0;
            self.track = 0x00;
            self.status = FDCStatus::LostDataOrTrack0 as u8;
            self.raise_nmi = true;

        } else if (command & 0xf0) == 0x10 {
            // SEEK command, type I
            // 0001_hVrr
            let track = self.data;
            if self.trace {
                println!("FDC: Seek track {}", track);
            }
            self.track = track;
            self.status = 0;
            self.raise_nmi = true;
        
        } else if (command & 0xe0) == 0x80 {
            // READ SECTOR command, type II
            // 100mFEFx
            if command & 0x10 != 0 {
                panic!("Multiple sector reads not supported")
            }
            if self.trace {
                println!("FDC: Read sector (T:{}, S:{})", self.track, self.sector);
            }

            self.read_index = (self.track as usize * SECTOR_COUNT + self.sector as usize) * SECTOR_SIZE;
            self.read_last = self.read_index + SECTOR_SIZE;
            let read_index = self.read_index;
            self.data = self.content()[read_index];
            self.read_index += 1;
            self.status = FDCStatus::Busy as u8;
            self.raise_nmi = true;

        } else if (command & 0xe0) == 0xa0 {
            // WRITE SECTOR command, type II
            // 101mFEFa
            if command & 0x10 != 0 {
                panic!("Multiple sector reads not supported")
            }
            if command & 0x01 != 0 {
                panic!("Delete data mark not supported")
            }
            if self.trace {
                println!("FDC: Write sector (T:{}, S:{})", self.track, self.sector);
            }

            self.read_index = (self.track as usize * SECTOR_COUNT + self.sector as usize) * SECTOR_SIZE;
            self.read_last = self.read_index + SECTOR_SIZE;
            self.status = FDCStatus::Busy as u8;
            self.raise_nmi = true;

        } else if (command & 0xf0) == 0xc0 {
            // READ ADDRESS command, type III
            // 1100_0E00
            if self.trace {
                println!("FDC: Read address");
            }
            self.inc_sector();
            self.status = 0;
            self.data_buffer.push(self.track);
            self.data_buffer.push(self.disk);
            self.data_buffer.push(self.sector);
            self.data_buffer.push(2); // For sector size 512
            self.data_buffer.push(0); // CRC 1
            self.data_buffer.push(0); // CRC 2
            self.raise_nmi = true;
        } else if (command & 0xf0) == 0xd0 {
            // FORCE INTERRUPT command, type IV
            // 1101_IIII
            let interrupts = command & 0x0f;
            if self.trace {
                println!("FDC: Force interrupt {}", interrupts);
            }

            if interrupts == 0 {
                // The current command is terminated and busy is reset.
                self.read_index = 0;
                self.read_last = 0;
                self.data_buffer.clear();
            } else {
                panic!("FDC: Interrupt forced with non zero I");
            }
        } else {
            if self.trace {
                println!("FDC: ${:02x} command not implemented", command);
            }
            panic!();
        }
    }

    pub fn get_status(&self) -> u8 {
        self.status
    }

    pub fn put_track(&mut self, value: u8) {
        self.track = value;
        if self.trace {
            println!("FDC: Set track {}", value);
        }
    }

    pub fn get_track(&self) -> u8 {
        self.track
    }

    pub fn put_sector(&mut self, value: u8) {
        self.sector = value;
        if self.trace {
            println!("FDC: Set sector {}", value);
        }
    }

    pub fn get_sector(&self) -> u8 {
        self.sector
    }

    pub fn put_data(&mut self, value: u8) {
        self.data = value;

        if self.read_index < self.read_last {
            // Store byte
            let read_index = self.read_index;
            self.content()[read_index] = self.data;
            self.read_index += 1;
            self.raise_nmi = true;
            if self.read_index == self.read_last {
                // We are done writing
                if self.trace {
                    println!("FDC: Set data completed ${:02x} {}-{}-{}", self.data, self.read_index, self.read_last, self.sector);
                }
                self.status = 0;
                self.read_index = 0;
                self.read_last = 0;
                self.sector += 1;
            }
        }

        //if self.trace {
        //    println!("FDC: Set data ${:02x}", value);
        //}
    }

    pub fn get_data(&mut self) -> u8 {
        let data = self.data;
        if self.data_buffer.len() > 0 {
            self.data = self.data_buffer[0];
            self.data_buffer.remove(0);
            self.raise_nmi = true;
        } else if self.read_index < self.read_last {
            // Prepare next byte
            let read_index = self.read_index;
            self.data = self.content()[read_index];
            self.read_index += 1;
            self.raise_nmi = true;
        } else if self.read_index != 0 {
            // We are done reading
            if self.trace {
                println!("FDC: Get data completed ${:02x} {}-{}-{}", data, self.read_index, self.read_last, self.sector);
            }
            self.status = 0;
            self.read_index = 0;
            self.read_last = 0;
            self.data = 0;
            self.sector += 1;
            self.raise_nmi = true;
        }
        //if self.trace {
        //    println!("FDC: Get data ${:02x} {}-{}-{}", data, self.read_index, self.read_last, self.sector);
        //}
        data
    }
}
