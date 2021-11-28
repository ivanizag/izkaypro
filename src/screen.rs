use super::KayproMachine;

pub struct Screen {
}

const CONTROL_CHARS: [char; 32] = [
    '`', 'α', 'β', 'γ', 'δ', 'ϵ', 'ϕ', 'ν',
    'θ', 'ι', 'σ', 'κ', 'λ', 'μ', 'υ', 'ω',
    'π', 'η', 'ρ', 'Σ', 'τ', 'χ', 'ψ', '≠',
    'Ξ', 'Ω', 'ζ', '{', '|', '}', '~', '█'];

impl Screen {
    pub fn new() -> Screen {
        Screen {
        }
    }

    pub fn update(&self, machine: &mut KayproMachine) {
        if !machine.vram_dirty {
            return;
        }

        // Move cursor up with ansi escape sequence
        print!("\x1b[{}A", 26);

        println!("//==================================================================================\\\\");
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
        println!("\\\\==================================================================================//");
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
