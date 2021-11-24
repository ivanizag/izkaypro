use iz80::*;

mod kaypro_machine;
mod floppy_controller;

use self::kaypro_machine::KayproMachine;
use self::floppy_controller::FloppyController;

// Welcome message
const WELCOME: &'static str =
"Kaypro https://github.com/ivanizag/izkaypro
Emulation of the Kaypro II computer
Press ctrl-c to return to host";


fn main() {
    let trace_io = false;
    let trace_cpu = false;
    let trace_fdc = true;

    // Init device
    let floppy_controller = FloppyController::new(trace_fdc);
    let mut machine = KayproMachine::new(floppy_controller, trace_io);
    let mut cpu = Cpu::new_z80();
    cpu.set_trace(trace_cpu);

    // Start the cpu
    println!("{}", WELCOME);
    let mut counter: u64 = 1;
    let mut next_signal: u64 = 0;
    loop {
        cpu.execute_instruction(&mut machine);
        counter += 1;

        if machine.floppy_controller.raise_nmi {
            //cpu.set_trace(true);
            machine.floppy_controller.raise_nmi = false;
            next_signal = counter + 1000;
        }

        if counter == next_signal {
            cpu.signal_nmi();
            next_signal = 0;
        }

        if counter < next_signal && cpu.is_halted() {
            cpu.signal_nmi();
            next_signal = 0;
        }

        if cpu.is_halted() {
            //cpu.set_trace(true);
            if machine.floppy_controller.raise_nmi {
                machine.floppy_controller.raise_nmi = false;
                cpu.signal_nmi();
            } else {
                machine.print_screen();
                println!("HALT instruction");
                cpu.signal_nmi();
                break;
            }
        }

        if cpu.registers().pc() == 0x0132 {
            machine.print_screen();
            break;
        }

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



