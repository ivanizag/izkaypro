use super::KayproMachine;

pub struct Screen {
    in_place: bool,
    last_system_bits: u8,
    pub show_status: bool,
}

const CONTROL_CHARS: [char; 32] = [
    '`', 'α', 'β', 'γ', 'δ', 'ϵ', 'ϕ', 'ν',
    'θ', 'ι', 'σ', 'κ', 'λ', 'μ', 'υ', 'ω',
    'π', 'η', 'ρ', 'Σ', 'τ', 'χ', 'ψ', '≠',
    'Ξ', 'Ω', 'ζ', '{', '|', '}', '~', '█'];

const SHOWN_SYSTEM_BITS: u8 = 0b0110_0011;

impl Screen {
    pub fn new(in_place: bool) -> Screen {
        Screen {
            in_place: in_place,
            last_system_bits: 0,
            show_status: false,
        }
    }

    pub fn init(&self) {
        if self.in_place {
            for _ in 0..27 {
                println!();
            }
        }
    }

    pub fn update(&mut self, machine: &mut KayproMachine, force: bool) {
        let relevant_system_bits = machine.system_bits & SHOWN_SYSTEM_BITS;
        if !force && !machine.vram_dirty && self.last_system_bits == relevant_system_bits {
            return;
        }
        self.last_system_bits = relevant_system_bits;

        // Move cursor up with ansi escape sequence
        if self.in_place {
            print!("\x1b[{}A", 26);
        }

        let mut disk_status = "========".to_owned();
        if self.show_status && machine.floppy_controller.motor_on {
            if machine.floppy_controller.disk == 0 {
                disk_status = " A".to_owned();
            } else {
                disk_status = " B".to_owned();
            }
            if machine.floppy_controller.single_density {
                disk_status += " SSSD ";
            } else {
                disk_status += " SSDD ";
            }
        }

        if self.show_status {
            println!("//====Last key: 0x{:02x}================================================================\\\\", machine.keyboard.peek_key());
        } else {
            println!("//==================================================================================\\\\");
        }
        for row in 0..24 {
            print!("|| ");
            for col in 0..80 {
                let code = machine.vram[(row * 128 + col) as usize];
                let ch = translate_char(code);
                if code & 0x80 == 0 {
                    print!("{}", ch);
                } else {
                    // Blinking
                    print!("\x1b[5m{}\x1b[25m", ch);
                }
            }
            println!(" ||");
        }
        println!("\\\\======{}=================================== F1 for help ==== F4 to exit ====//", disk_status);
        //println!("\\\\==================================================================================//");
        machine.vram_dirty = false;
    }
}

fn translate_char(code: u8) -> char {
    let index = code & 0x7f;
    if index < 0x20 {
        CONTROL_CHARS[index as usize]
    } else if index == 0x7f {
        '▒'
    } else {
        index as char
    }
}
