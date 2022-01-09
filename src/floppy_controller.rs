use super::media::*;

static DISK_CPM22: &'static [u8] = include_bytes!("../disks/cpm22-bios149.img");
static DISK_BLANK: &'static [u8] = include_bytes!("../disks/blank.img");

pub enum Drive {
    A = 0,
    B = 1,
}

pub struct FloppyController {
    pub motor_on: bool,
    pub drive: u8,
    side_2: bool,
    track: u8,
    sector: u8,
    pub single_density: bool,
    data: u8,
    status: u8,

    media: [Media ;2],

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
    SeekErrorOrRecordNotFound = 0x08,
    _CRCError = 0x10,
    LostDataOrTrack0 = 0x20,
    _DataRequest = 0x40,
    Busy = 0x80,
}

impl FloppyController {
    pub fn new(trace: bool) -> FloppyController {
        FloppyController {
            motor_on: false,
            drive: 0,
            side_2: false,
            track: 0,
            sector: 0,
            single_density: false,
            data: 0,
            status: 0,
            media: [
                Media {
                    file: None,
                    name: "CPM/2.2 embedded".to_owned(),
                    content: DISK_CPM22.to_vec(),
                    format: MediaFormat::SSDD,
                    write_min: usize::MAX,
                    write_max: 0,
                },
                Media {
                    file: None,
                    name: "Blank disk embedded".to_owned(),
                    content: DISK_BLANK.to_vec(),
                    format: MediaFormat::SSDD,
                    write_min: usize::MAX,
                    write_max: 0,
                },
            ],

            read_index: 0,
            read_last: 0,

            data_buffer: Vec::new(),

            raise_nmi: false,
            trace: trace,
        }
    }

    pub fn media_a(&self) -> &Media {
        &self.media[Drive::A as usize]
    }

    pub fn media_b(&self) -> &Media {
        &self.media[Drive::B as usize]
    }

    pub fn media_a_mut(&mut self) -> &mut Media {
        &mut self.media[Drive::A as usize]
    }

    pub fn media_b_mut(&mut self) -> &mut Media {
        &mut self.media[Drive::B as usize]
    }

    pub fn media_selected(&mut self) -> &mut Media {
        &mut self.media[self.drive as usize]
    }

    pub fn set_motor(&mut self, motor_on: bool) {
        self.media_selected().flush_disk();
        self.motor_on = motor_on;
    }

    pub fn set_single_density(&mut self, single_density: bool) {
        self.single_density = single_density;
    }

    pub fn set_side(&mut self, side_2: bool) {
        self.side_2 = side_2;
    }

    pub fn set_drive(&mut self, drive: u8) {
        self.media_selected().flush_disk();
        self.drive = drive;
    }

    pub fn put_command(&mut self, command: u8) {
        self.media_selected().flush_disk();

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
            if self.media_selected().is_valid_track(track) {
                self.track = track;
                self.status = 0;
                self.raise_nmi = true;
            } else {
                self.status = FDCStatus::SeekErrorOrRecordNotFound as u8;
                self.raise_nmi = true;
            }
        } else if (command & 0xe0) == 0x80 {
            // READ SECTOR command, type II
            // 100mFEFx
            if command & 0x10 != 0 {
                panic!("Multiple sector reads not supported")
            }
            if self.trace {
                println!("FDC: Read sector (S:{}, T:{}, S:{})", self.side_2, self.track, self.sector);
            }

            let side_2 = self.side_2;
            let track = self.track;
            let sector = self.sector;
            let (index, last) =  self.media_selected().sector_index(side_2, track, sector);
            self.read_index = index;
            self.read_last = last;

            self.data = self.media_selected().read_byte(index);
            self.read_index += 1;
            self.status = FDCStatus::Busy as u8;
            self.raise_nmi = true;

        } else if (command & 0xe0) == 0xa0 {
            // WRITE SECTOR command, type II
            // 101mFEFa
            if command & 0x10 != 0 {
                panic!("Multiple sector writes not supported")
            }
            if command & 0x01 != 0 {
                panic!("Delete data mark not supported")
            }
            if self.trace {
                println!("FDC: Write sector (T:{}, S:{})", self.track, self.sector);
            }

            let side_2 = self.side_2;
            let track = self.track;
            let sector = self.sector;
            let (a, b) =  self.media_selected().sector_index(side_2, track, sector);
            self.read_index = a;
            self.read_last = b;

            self.status = FDCStatus::Busy as u8;
            self.raise_nmi = true;

        } else if (command & 0xf0) == 0xc0 {
            // READ ADDRESS command, type III
            // 1100_0E00
            let side_2 = self.side_2;
            let track = self.track;
            let sector = self.sector;
            if self.media_selected().is_valid_sector(side_2, track, sector) {
                if self.trace {
                    println!("FDC: Read address ({},{},{})", side_2, track, sector);
                }
                self.sector = self.media_selected().inc_sector(sector);
                self.status = 0;
                self.data_buffer.push(self.track);
                self.data_buffer.push(if side_2 {1} else {0}); // Side
                self.data_buffer.push(self.sector);
                self.data_buffer.push(2); // For sector size 512
                self.data_buffer.push(0); // CRC 1
                self.data_buffer.push(0); // CRC 2
            } else {
                if self.trace {
                    println!("FDC: Read address ({},{},{}) = Error", side_2, track, sector);
                }
                self.status = FDCStatus::SeekErrorOrRecordNotFound as u8;
                self.data_buffer.push(0);
                self.data_buffer.push(0);
                self.data_buffer.push(0);
                self.data_buffer.push(0);
                self.data_buffer.push(0);
                self.data_buffer.push(0);
                self.data_buffer.push(0);
            }
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
            let index = self.read_index;
            let data = self.data;
            self.media_selected().write_byte(index, data);
            self.read_index += 1;
            self.raise_nmi = true;
            if self.read_index == self.read_last {
                // We are done writing
                self.media_selected().flush_disk();
                if self.trace {
                    println!("FDC: Set data completed ${:02x} {}-{}-{}", self.data, self.read_index, self.read_last, self.sector);
                }
                self.status = 0;
                self.read_index = 0;
                self.read_last = 0;
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
            let index = self.read_index;
            self.data = self.media_selected().read_byte(index);
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
