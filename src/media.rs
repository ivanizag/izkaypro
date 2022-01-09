use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom, Result, Error, ErrorKind};

#[derive(PartialEq)]
pub enum MediaFormat {
    Unformatted,
    SSSD,     // Single-sided, single-density
    SSDD,     // Single-sided, double-density
    DSDD,     // Double-sided, double-density
}

const SECTOR_SIZE: usize = 512;

fn detect_media_format(len: usize) -> MediaFormat {
    if len == 102400 {
        MediaFormat::SSSD
    } else if len >= 204800 && len <= 205824 {
        // Some valid disk images are a bit bigger, I don't know why
        MediaFormat::SSDD
    } else if len == 409600 {
        MediaFormat::DSDD
    } else {
        MediaFormat::Unformatted
    }
}

pub struct Media {
    pub file: Option<File>,
    pub name: String,
    pub content: Vec<u8>,
    pub format: MediaFormat,

    pub write_min: usize,
    pub write_max: usize,
}

impl Media {
    pub fn double_sided(&self) -> bool {
        self.format == MediaFormat::DSDD
    }

    pub fn tracks(&self) -> u8 {
        match self.format {
            MediaFormat::SSSD => 40,
            MediaFormat::SSDD => 40,
            MediaFormat::DSDD => 40,
            MediaFormat::Unformatted => 0,
        }
    }

    pub fn sectors(&self) -> u8 {
        match self.format {
            MediaFormat::SSSD => 10,
            MediaFormat::SSDD => 10,
            MediaFormat::DSDD => 20,
            MediaFormat::Unformatted => 0,
        }
    }

    pub fn sector_start(&self, track: u8, sector: u8) -> usize {
        let track = track as usize;
        let sector = sector as usize;

        if self.format == MediaFormat::DSDD {
            if sector < 10 {
                // On DSDD, the first 10 sectors are on the first side
                (track * 10 + sector) * SECTOR_SIZE
            } else {
                // The rest of the DSDD sectors are on the second side
                (track * 10 + sector - 10) * SECTOR_SIZE
            }
        } else {
            (track * self.sectors() as usize + sector) * SECTOR_SIZE
        }
    }

    pub fn load_disk(&mut self, filename: &str) -> Result<()>{
        self.flush_disk();

        // Try opening writable, then read only
        let (mut file, readonly) = match OpenOptions::new()
            .read(true)
            .write(true)
            .open(filename)
            {
                Ok(file) => (file, false),
                _ => {
                    // Try opening read-only
                    match OpenOptions::new()
                        .read(true)
                        .open(filename)
                        {
                            Ok(file) => (file, true),
                            Err(err) => {
                                return Err(err);
                            }
                        }
                }
            };

        // Load content
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;

        // Store the file descriptor for writable files
        let file = if readonly {
            None
        } else {
            Some(file)
        };

        let format = detect_media_format(self.content.len());
        if format == MediaFormat::Unformatted {
            return Err(Error::new(ErrorKind::Other, "Unrecognized disk image format"));
        }

        self.file = file;
        self.name = filename.to_owned();
        self.content = content;
        self.format = detect_media_format(self.content.len());

        Ok(())
    }

    pub fn flush_disk(&mut self) {
        if self.write_max < self.write_min {
            // nothing to write
            return;
        }

        if let Some(ref mut file) = self.file {
            file.seek(SeekFrom::Start(self.write_min as u64)).unwrap();
            file.write_all(&self.content[self.write_min..=self.write_max]).unwrap();
        }

        self.write_max = 0;
        self.write_min = usize::MAX;
    }

    pub fn is_valid_track(&self, track: u8) -> bool {
        track < self.tracks()
    }

    pub fn is_valid_sector(&self, side_2: bool, track: u8, sector: u8) -> bool {
        track < self.tracks() && sector < self.sectors() && (!side_2 || self.double_sided())
    }

    pub fn inc_sector(&self, sector: u8) -> u8 {
        let new_sector = sector + 1;
        if new_sector >= self.sectors() {
            0
        } else {
            new_sector
        }
    }

    pub fn sector_index(&self, side_2: bool, track: u8, sector: u8) -> (usize, usize) {
        let mut index = self.sector_start(track, sector);
        if side_2 && self.double_sided() {
            index += self.sectors() as usize / 2 * self.tracks() as usize * SECTOR_SIZE;
        }
        let last = index + SECTOR_SIZE;
        (index, last)
    }

    pub fn read_byte(&self, index: usize) -> u8 {
        self.content[index]
    }

    pub fn write_byte(&mut self, index: usize, value: u8) {
        self.content[index] = value;
        if index < self.write_min {
            self.write_min = index;
        }
        if index > self.write_max {
            self.write_max = index;
        }
    }

    pub fn info(&self) -> String {
        self.name.clone() + " (" +
            match self.file {
                Some(_) => "persistent",
                _ => "transient"
            } + " " +
            match self.format {
                MediaFormat::Unformatted => " (unformatted)",
                MediaFormat::SSSD => " (SSSD)",
                MediaFormat::SSDD => " (SSDD)",
                MediaFormat::DSDD => " (DSDD)",
            } + ")"
    }
}