use super::KayproMachine;

pub struct Screen {
}

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
                let mut ch = machine.vram[(row * 128 + col) as usize];
                if ch < 20 {
                    ch = '@' as u8;
                }
                if ch & 0x80 == 0 {
                    print!("{}", ch as char);
                } else {
                    // Blinking
                    print!("\x1b[5m{}\x1b[25m", (ch & 0x7f) as char);
                }
            }
            println!(" ||");
        }
        println!("\\\\==================================================================================//");
        machine.vram_dirty = false;
    }

}