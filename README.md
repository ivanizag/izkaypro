# Kaypro II emulator on the terminal

## What is this?

This is a Kaypro II emulator that runs on a Linux terminal. It can boot and use disk images for the Kaypro and display.

Uses the [iz80](https://github.com/ivanizag/iz80) library. Made with Rust.

## What is/was a Kaypro II computer?

The Kaypro II computer was a luggable computer from 1982 capable of running CP/M 2.2. It was considered "a rugged, functional and practical computer system marketed at a reasonable price." (From [Wipedia](https://en.wikipedia.org/wiki/Kaypro))

It's a typical CP/M computer of the early 80s, built on a metal case with standard components, a 9" green monochrome CRT, a detachable keyboard and two disk drives. Main features:

- Zilog Z80 at 2.5 MHz
- 64 KB of main RAM
- 2 KB of ROM
- 2 KB of video RAM
- 80*24 text mode (no graphics capabilities)
- Two single side double density drives with 195kb capacity
- A serial port (not emulated by izkaypro)
- A parallel port (not emulated by izkaypro)

## Usage examples

izkaypro does not require installation, you just need the executable. It has the ROM embedded as well as the boot CP/M disk and a blank disk. You can provide additional disk images as separate files.

### Usage with no arguments
Run the executable on a terminal and type the CP/M commands (you can try DIR and changing drives with B:). Press F4 to exit back to the host shell prompt.
```
casa@servidor:~/$ ./izkaypro
Kaypro https://github.com/ivanizag/izkaypro
Emulation of the Kaypro II computer

//==================================================================================\\
||                                                                                  ||
|| KAYPRO II 64k CP/M vers 2.2                                                      ||
||                                                                                  ||
|| A>dir                                                                            ||
|| A: MOVCPM   COM : PIP      COM : SUBMIT   COM : XSUB     COM                     ||
|| A: ED       COM : ASM      COM : DDT      COM : STAT     COM                     ||
|| A: SYSGEN   COM : DUMP     ASM : COPY     COM : BAUD     COM                     ||
|| A: TERM     COM : SBASIC   COM : D        COM : OVERLAYB COM                     ||
|| A: BASICLIB REL : USERLIB  REL : FAC      BAS : XAMN     BAS                     ||
|| A: DPLAY    BAS : CONFIG   COM : LOAD     COM : DUMP     COM                     ||
|| A: SETDISK  COM : INITDISK COM :          PRN :          HEX                     ||
|| A>b:                                                                             ||
|| B>dir                                                                            ||
|| NO FILE                                                                          ||
|| B>a:stat                                                                         ||
|| A: R/W, Space: 4k                                                                ||
|| B: R/W, Space: 191k                                                              ||
||                                                                                  ||
||                                                                                  ||
|| B>_                                                                              ||
||                                                                                  ||
||                                                                                  ||
||                                                                                  ||
||                                                                                  ||
\\================================================= F1 for help ==== F4 to exit ====//```
```
### Usage with external images
You can provide up two disk images as binary files to use as A: and B: drives. If only an image is provided, it will be the A: disk, B: will be a blank disk.

The images have to be raw binary images of single sided disks. The size must be 204800 bytes. See [disk images](doc/disk_images.md).

```
casa@servidor:~/$ ./izkaypro disks/cpmish.img disks/WordStar33.img 
B: disks/WordStar33.img
Kaypro https://github.com/ivanizag/izkaypro
Emulation of the Kaypro II computer

//==================================================================================\\
||                                                                                  ||
|| CP/Mish 2.2r0 for Kaypro II                                                      ||
||                                                                                  ||
|| A>dir                                                                            ||
|| COPY    .COM  |  DUMP    .COM  |  ASM     .COM  |  STAT    .COM                  ||
|| BBCBASIC.COM  |  SUBMIT  .COM  |  QE      .COM                                   ||
|| A>dir b:                                                                         ||
|| WS      .COM  |  WSOVLY1 .OVR  |  WSMSGS  .OVR  |  WS      .INS                  ||
|| WINSTALL.COM  |  PRINT   .TST                                                    ||
|| A>stat                                                                           ||
|| A: R/W, space: 135/195kB                                                         ||
|| B: R/W, space: 27/195kB                                                          ||
||                                                                                  ||
|| A>bbcbasic                                                                       ||
|| BBC BASIC (Z80) Version 3.00+1                                                   ||
|| (C) Copyright R.T.Russell 1987                                                   ||
|| >PRINT "Hi!"                                                                     ||
|| Hi!                                                                              ||
|| >*BYE                                                                            ||
||                                                                                  ||
|| A>_                                                                              ||
||                                                                                  ||
||                                                                                  ||
||                                                                                  ||
\\================================================= F1 for help ==== F4 to exit ====//```
```
### Online help
Press F1 to get additional help:

```
//==================================================================================\\
||                                                                                  ||
|| KAYPRO II 64k CP/M vers 2.2                                                      ||
||                                                                                  ||
|| A>_                                                                              ||
||        +----------------------------------------------------------------+        ||
||        |  izkaypro: Kaypro II emulator for console terminals            |        ||
||        |----------------------------------------------------------------|        ||
||        |  F1: Show/hide help           | Host keys to Kaypro keys:      |        ||
||        |  F2: Show/hide disk status    |  Delete to DEL                 |        ||
||        |  F4: Quit the emulator        |  Insert to LINEFEED            |        ||
||        |  F5: Select file for drive A: |                                |        ||
||        |  F6: Select file for drive B: |                                |        ||
||        |  F8: Toggle CPU trace         |                                |        ||
||        +----------------------------------------------------------------+        ||
||        |  Loaded images:                                                |        ||
||        |  A: CPM/2.2 embedded (transient)                               |        ||
||        |  B: Blank disk embedded (transient)                            |        ||
||        +----------------------------------------------------------------+        ||
||                                                                                  ||
||                                                                                  ||
||                                                                                  ||
||                                                                                  ||
||                                                                                  ||
||                                                                                  ||
\\================================================= F1 for help ==== F4 to exit ====//
```

## Build from source

To build from source, install the latest Rust compiler, clone the repo and run `cargo rust --release`. To build and run directly execute `cargo run`.

## Command line usage
```
USAGE:
    izkaypro [FLAGS] [ARGS]

FLAGS:
    -b, --bdos-trace     Traces calls to the CP/M BDOS entrypoints
    -c, --cpu-trace      Traces CPU instructions execuions
    -f, --fdc-trace      Traces access to the floppy disk controller
    -h, --help           Prints help information
    -i, --io-trace       Traces ports IN and OUT
    -r, --rom-trace      Traces calls to the ROM entrypoints
    -s, --system-bits    Traces changes to the system bits values
    -V, --version        Prints version information

ARGS:
    <DISKA>    Disk A: image file. Empty or $ to load CP/M
    <DISKB>    Disk B: image file. Default is a blank disk
```

## Resources

- [ROM disassembled and commented](https://github.com/ivanizag/kaypro-disassembled)
- [Kaypro manuals in bitsavers](http://bitsavers.informatik.uni-stuttgart.de/pdf/kaypro/)
- [Disks from retriarchive](http://www.retroarchive.org/maslin/disks/kaypro/)
- [ImageDisk and system images](http://dunfield.classiccmp.org/img/index.htm)
