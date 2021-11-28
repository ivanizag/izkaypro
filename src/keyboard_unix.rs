use std::io::{Read, stdin};

use termios::*;

const STDIN_FD: i32 = 0;

pub struct Keyboard {
    initial_termios: Option<Termios>,
    key: u8,
    key_available: bool,
}

impl Keyboard {
    pub fn new() -> Keyboard {
        // Prepare terminal
        let initial_termios = Termios::from_fd(STDIN_FD).ok();

        let c = Keyboard {
            initial_termios: initial_termios,
            key: 0,
            key_available: false,
        };

        c.setup_host_terminal(false);
        c
    }

    fn setup_host_terminal(&self, blocking: bool) {
        if let Some(initial) = self.initial_termios {
            let mut new_term = initial.clone();
            new_term.c_iflag &= !(IXON | ICRNL);
            new_term.c_lflag &= !(/*ISIG |*/ ECHO | ICANON | IEXTEN);
            new_term.c_cc[VMIN] = if blocking {1} else {0};
            new_term.c_cc[VTIME] = 0;
            tcsetattr(STDIN_FD, TCSANOW, &new_term).unwrap();
        }
    }

    pub fn is_key_pressed(&mut self) -> bool {
        if !self.key_available {
            let mut buf = [0];
            let size = stdin().read(&mut buf).unwrap_or(0);
            if size != 0 {
                self.key = *buf.last().unwrap();
                self.key = match self.key {
                    0x7f => 0x08, // Backspace to ^H
                    _ => self.key,
                };
                self.key_available = true;
            }
        }
        self.key_available
    }

    pub fn get_key(&mut self) -> u8 {
        self.is_key_pressed();
        self.key_available = false;
        self.key
    }

    pub fn peek_key(&mut self) -> u8 {
        self.key
    }


}

impl Drop for Keyboard {
    fn drop(&mut self) {
        if let Some(initial) = self.initial_termios {
            tcsetattr(STDIN_FD, TCSANOW, &initial).unwrap();
        }
    }
}
