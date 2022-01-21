use std::io::{Read, stdin};
use std::thread;
use std::time::Duration;

use termios::*;

const STDIN_FD: i32 = 0;

#[derive(Copy, Clone)]
pub enum Command {
    Help,
    Quit,
    SelectDiskA,
    SelectDiskB,
    ShowStatus,
    TraceCPU,
    SaveMemory,
}

pub struct Keyboard {
    initial_termios: Option<Termios>,
    key: u8,
    pub commands: Vec<Command>,
    key_available: bool,
}

impl Keyboard {
    pub fn new() -> Keyboard {
        // Prepare terminal
        let initial_termios = Termios::from_fd(STDIN_FD).ok();

        let c = Keyboard {
            initial_termios: initial_termios,
            key: 0,
            commands: Vec::<Command>::new(),
            key_available: false,
        };

        c.setup_host_terminal(false);
        c
    }

    fn setup_host_terminal(&self, blocking: bool) {
        if let Some(initial) = self.initial_termios {
            let mut new_term = initial.clone();
            new_term.c_iflag &= !(IXON | ICRNL);
            new_term.c_lflag &= !(ISIG | ECHO | ICANON | IEXTEN);
            new_term.c_cc[VMIN] = if blocking {1} else {0};
            new_term.c_cc[VTIME] = 0;
            tcsetattr(STDIN_FD, TCSANOW, &new_term).unwrap();
        }
    }

    pub fn is_key_pressed(&mut self) -> bool {
        self.consume_input();
        if !self.key_available {
            // Avoid 100% CPU usage waiting for input.
            thread::sleep(Duration::from_nanos(100));
        }
        self.key_available
    }

    pub fn get_key(&mut self) -> u8 {
        self.consume_input();
        self.key_available = false;
        self.key
    }

    pub fn peek_key(&mut self) -> u8 {
        self.key
    }

    pub fn read_line(&mut self) -> String {
        if let Some(initial) = self.initial_termios {
            tcsetattr(STDIN_FD, TCSANOW, &initial).unwrap();
        }
        let mut buffer = String::new();
        stdin().read_line(&mut buffer).unwrap();
        self.setup_host_terminal(false);
        buffer.trim().to_string()
    }

    pub fn consume_input(&mut self) {
        let mut buf = [0;100];
        let size = stdin().read(&mut buf).unwrap_or(0);
        if size > 0 {
            self.parse_input(size, &buf);
        }
    }

    fn parse_input(&mut self, size: usize, input: &[u8]) {
        if size == 0 {
            // No new keys
        } else if size > 2 && input[0] == 0x1b {
            // Escape sequences
            // See 5.4 in the ECMA-48 spec
            let mut seq = "".to_owned();
            // Second byte of the CSI
            seq.push(input[1] as char);
            let mut i = 2;
            // Parameter and Intermediate bytes
            while i < size && (
                    input[i] & 0xf0 == 0x20 ||
                    input[i] & 0xf0 == 0x30 ) {
                seq.push(input[i] as char);
                i += 1;
            }
            // Final byte
            if i < size {
                seq.push(input[i] as char);
                i += 1;
            }
            //println!("Escape sequence: {}", seq);

            // Execute "showkey -a" to find the key codes
            match seq.as_str() {
                "OP" => { // F1
                    self.commands.push(Command::Help);
                }
                "OQ" => { // F2
                    self.commands.push(Command::ShowStatus);
                }
                "OS" => { // F4
                    self.commands.push(Command::Quit);
                }
                "[15~" => { // F5
                    self.commands.push(Command::SelectDiskA);
                }
                "[17~" => { // F6
                    self.commands.push(Command::SelectDiskB);
                }
                "[18~" => { // F7
                    self.commands.push(Command::SaveMemory);
                }
                "[19~" => { // F8
                    self.commands.push(Command::TraceCPU);
                }
                "[3~" => {
                    // "Delete" key mapped to "DEL"
                    self.key = 0x7f;
                    self.key_available = true;
                }
                "[2~" => {
                    // "Insert" key mapped to "LINEFEED"
                    self.key = 0x0a;
                    self.key_available = true;
                }
                "[A" => {
                    // Up arrow mapped to ^K on the BIOS
                    self.key = 0xf1; //0x0b;
                    self.key_available = true;
                }
                "[B" => {
                    // Down arrow mapped to ^J on the BIOS
                    self.key = 0xf2; //0x0a;
                    self.key_available = true;
                }
                "[C" => {
                    // Right arrow mapped to ^L on the BIOS
                    self.key = 0xf4; //0x0c;
                    self.key_available = true;
                }
                "[D" => {
                    // Left arrow mapped to ^H on the BIOS
                    self.key = 0xf3; //0x08;
                    self.key_available = true;
                }
                _ => {}
            }
            // Parse the rest
            self.parse_input(size-i, &input[i..]);
        } else if size >= 2 && input[0] == 0xc3 && input[1] == 0xb1 {
            self.key = ':' as u8; // ñ is on the : position
            self.key_available = true;
        } else if size >= 2 && input[0] == 0xc3 && input[1] == 0x91 {
            self.key = ';' as u8; // Ñ is on the ; position
            self.key_available = true;
        } else {
            self.key = input[0];
            self.key = match self.key {
                0x7f => 0x08, // Backspace to ^H
                _ => self.key & 0x7f,
            };
            self.key_available = true;
            // Parse the rest
            self.parse_input(size-1, &input[1..]);
        }
    }
}


impl Drop for Keyboard {
    fn drop(&mut self) {
        if let Some(initial) = self.initial_termios {
            tcsetattr(STDIN_FD, TCSANOW, &initial).unwrap();
        }
    }
}
