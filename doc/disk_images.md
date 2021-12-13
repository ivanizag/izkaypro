# Disk images

The emulator uses raw images of the SSDD disks (204800 bytes), but most of the images online are in Teledisk (.TD0) or ImageDisk (.IMD) formats and must be convertes. [ImageDisk](http://dunfield.classiccmp.org/img/index.htm) is a DOS program that can do that. It can run with dosbox.

## Prerequisites
Install dosbox (`sudo apt install dosbox` in Ubuntu).

Extract the ImageDisk 1.18 files from http://dunfield.classiccmp.org/img/index.htm

To convert TD0 files to IMD:

- On the host run `dosbox .`
- Inside dosbox run `TD02IMD FILE.TD0` (use ALT-F12 in dosbox for full speed)

To convert IMD files to raw:

- On the host run `dosbox .` (if needed)
- Inside dosbox run `IMDU FILE.IMD FILE.IMG /B` (use ALT-F12 in dosbox for full speed)

You can use the .IMG file with izkaypro 


1- CPM 2.2
2- Wordstar 3.3
3- Perfect Calc
4- Perfect Speller -> Writer 1.02
5- Profit Plan (Working Disk #2)
6- Select Work Processor (Working disk #3)
7- Teach/Install (Working Disk #4)
8- Perfect Writer, Docs and utilities
9- Perfect Writer, Lessons