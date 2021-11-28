use iz80::*;

mod kaypro_machine;
mod floppy_controller;
mod keyboard_unix;
mod screen;

use self::kaypro_machine::KayproMachine;
use self::floppy_controller::FloppyController;
use self::screen::Screen;

// Welcome message
const WELCOME: &'static str =
"Kaypro https://github.com/ivanizag/izkaypro
Emulation of the Kaypro II computer
Press ctrl-c to return to host";


fn main() {
    let trace_io = false;
    let trace_cpu = false;
    let trace_fdc = false;
    let trace_bios = false;
    let trace_system_bits = false;
    let any_trace = trace_io
        || trace_cpu
        || trace_fdc
        || trace_bios
        || trace_system_bits;

    // Init device
    let floppy_controller = FloppyController::new(trace_fdc);
    let screen = Screen::new(!any_trace);
    let mut machine = KayproMachine::new(floppy_controller,
        trace_io, trace_system_bits);
    let mut cpu = Cpu::new_z80();
    cpu.set_trace(trace_cpu);

    // Start the cpu
    println!("{}", WELCOME);
    screen.init();

    let mut counter: u64 = 1;
    let mut next_signal: u64 = 0;
    loop {
        cpu.execute_instruction(&mut machine);
        counter += 1;

        if counter % 2048 == 0 {
            screen.update(&mut machine);
        }

        if machine.floppy_controller.raise_nmi {
            machine.floppy_controller.raise_nmi = false;
            next_signal = counter + 1000;
        }

        if next_signal != 0 && counter >= next_signal {
            cpu.signal_nmi();
            next_signal = 0;
        }

        if counter < next_signal && cpu.is_halted() {
            cpu.signal_nmi();
            next_signal = 0;
        }

        if cpu.is_halted() {
            screen.update(&mut machine);
            println!("HALT instruction that will never be interrupted");
            //cpu.signal_nmi();
            break;
        }

        if trace_bios {
            let dma = machine.peek16(0xfc14);
            match cpu.registers().pc() {
                0x004b => println!("EP_COLD"),
                0x0186 => println!("EP_INITDSK"),
                0x0006 => println!("EP_INITVID"),
                0x0009 => println!("EP_INITDEV"),
                0x01d8 => println!("EP_HOME"),
                0x01b4 => println!("EP_SELDSK {}", cpu.registers().get8(Reg8::C)),
                0x01cc => println!("EP_SETTRK {}", cpu.registers().get8(Reg8::C)),
                0x01bb => println!("EP_SETSEC {}", cpu.registers().get8(Reg8::C)),
                0x01c7 => println!("EP_SETDMA"),
                0x01ec => println!("EP_READ {:04x}", dma),
                0x0207 => println!("EP_WRITE {:04x}", dma),
                0x03e4 => println!("EP_SECTRAN"),
                0x040f => println!("EP_DISKON"),
                0x041e => println!("EP_DISKOFF"),
                /*
                0x00c5 => println!("FUNC: First read sector in boot"),
                0x00c8 => println!("FUNC: Back from first read sector in boot"),
                0x00e7 => println!("FUNC: Back from read sector"),
                0x03e0 => println!("FUNC: Set sector C"),
                0x0425 => println!("FUNC: Wait"),
                0x0481 => println!("FUNC: Back from read internal"),
                0xfee8 => println!("FUNC: Reloc read internal"),
                */
                0xfa00 => println!("FUNC: OS start"),
                _ => {}
            }
        }
    }
}



