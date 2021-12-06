use clap::{Arg, App};
use iz80::*;

mod kaypro_machine;
mod floppy_controller;
mod keyboard_unix;
mod screen;

use self::kaypro_machine::KayproMachine;
use self::floppy_controller::FloppyController;
use self::screen::Screen;
use self::keyboard_unix::Command;

// Welcome message
const WELCOME: &'static str =
"Kaypro https://github.com/ivanizag/izkaypro
Emulation of the Kaypro II computer";


fn main() {
    // Parse arguments
    let matches = App::new(WELCOME)
        .arg(Arg::with_name("DISKA")
            .help("Disk A: image file. Empty or $ to load CP/M")
            .required(false)
            .index(1))
        .arg(Arg::with_name("DISKB")
            .help("Disk B: image file. Default is a blank disk")
            .required(false)
            .index(2))
        .arg(Arg::with_name("cpu_trace")
            .short("c")
            .long("cpu-trace")
            .help("Traces CPU instructions execuions"))
        .arg(Arg::with_name("io_trace")
            .short("i")
            .long("io-trace")
            .help("Traces ports IN and OUT"))
        .arg(Arg::with_name("fdc_trace")
            .short("f")
            .long("fdc-trace")
            .help("Traces access to the floppy disk controller"))
        .arg(Arg::with_name("system_bits")
            .short("s")
            .long("system-bits")
            .help("Traces changes to the system bits values"))
        .arg(Arg::with_name("rom_trace")
            .short("ro")
            .long("rom-trace")
            .help("Traces calls to the ROM entrypoints"))
        .get_matches();

    let disk_a = matches.value_of("DISKA");
    let disk_b = matches.value_of("DISKB");
    let trace_cpu = matches.is_present("cpu_trace");
    let trace_io = matches.is_present("io_trace");
    let trace_fdc = matches.is_present("fdc_trace");
    let trace_system_bits = matches.is_present("system_bits");
    let trace_rom = matches.is_present("rom_trace");

    let any_trace = trace_io
        || trace_cpu
        || trace_fdc
        || trace_rom
        || trace_system_bits;

    // Init device
    let floppy_controller = FloppyController::new(trace_fdc);
    let mut screen = Screen::new(!any_trace);
    let mut machine = KayproMachine::new(floppy_controller,
        trace_io, trace_system_bits);
    let mut cpu = Cpu::new_z80();
    cpu.set_trace(trace_cpu);

    // Load disk images
    if let Some(disk_a) = disk_a {
        if  disk_a != "$" {
            machine.floppy_controller.load_disk(disk_a, false).unwrap();
        }
    }
    if let Some(disk_b) = disk_b {
        println!("B: {}", disk_b);
        machine.floppy_controller.load_disk(disk_b, true).unwrap();
    }

    // Start the cpu
    println!("{}", WELCOME);
    screen.init();

    let mut counter: u64 = 1;
    let mut next_signal: u64 = 0;
    let mut done = false;
    while !done {
        cpu.execute_instruction(&mut machine);
        counter += 1;

        // IO refresh
        if counter % 2048 == 0 {
            machine.keyboard.consume_input();
            screen.update(&mut machine, false);
        }

        if machine.keyboard.commands.len() != 0 {
            let commands = machine.keyboard.commands.clone();
            for command in commands {
                match command {
                    Command::Quit => {
                        machine.floppy_controller.flush_disk();
                        done = true;
                    },
                    Command::Help => {
                        screen.show_help = !screen.show_help;
                        screen.update(&mut machine, true);
                    },
                    Command::ShowStatus => {
                        screen.show_status = !screen.show_status;
                        screen.update(&mut machine, true);
                    },
                    Command::SelectDiskA => {
                        let path = screen.prompt(& mut machine, "File to load in Drive A");
                        machine.floppy_controller.load_disk(path.as_str(), false).unwrap();
                    }
                    Command::SelectDiskB => {
                        let path = screen.prompt(& mut machine, "File to load in Drive B");
                        machine.floppy_controller.load_disk(path.as_str(), true).unwrap();
                    }
                }
            }
            machine.keyboard.commands.clear();
        }

        // NMI processing
        if machine.floppy_controller.raise_nmi {
            machine.floppy_controller.raise_nmi = false;
            next_signal = counter + 10_000_000;
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
            screen.update(&mut machine, true);
            println!("HALT instruction that will never be interrupted");
            break;
        }

        // Tracing
        if trace_rom && machine.is_rom_rank(){
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
                0xfa00 => println!("FUNC: OS start"),
                _ => {}
            }
        }
    }
}



