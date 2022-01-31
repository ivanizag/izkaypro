# Disk images

## Image conversion
The emulator uses raw images of the SSDD disks (204800 bytes), but most of the images online are in Teledisk (.TD0) or ImageDisk (.IMD) formats and must be convertes. [ImageDisk](http://dunfield.classiccmp.org/img/index.htm) is a DOS program that can do that. It can run with dosbox.

### Prerequisites
Install dosbox (`sudo apt install dosbox` in Ubuntu).

Extract the ImageDisk 1.18 files from http://dunfield.classiccmp.org/img/index.htm

### To convert TD0 files to IMD:

- On the host run `dosbox .`
- Inside dosbox run `TD02IMD FILE.TD0` (use ALT-F12 in dosbox for full speed)

### To convert IMD files to raw:

- On the host run `dosbox .` (if needed)
- Inside dosbox run `IMDU FILE.IMD FILE.IMG /B` (use ALT-F12 in dosbox for full speed)

You can use the .IMG file with izkaypro 

## Moving files in and out of the disk images

### Prerequisites
Install cpmtools (`sudo apt install cpmtools` in Ubuntu).
Use `-f kpii` for SSDD disks (204800 bytes).
Use `-f kpiv` for DSDD disks (409600 bytes).

### Extract files from the image
`cpmcp -f kpii kayprodisk.img 0:*.* destination`

### Add file to a disk
`cpmcp -f kpii test.img source/sbasic.com 0:`
