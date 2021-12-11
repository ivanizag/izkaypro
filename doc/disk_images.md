# Disk images

The emulator uses raw images of the SSDD disks (204800 bytes), but most of the images online are in Teledisk (.TD0) or ImageDisk (.IMD) formats and must be convertes. [ImageDisk](http://dunfield.classiccmp.org/img/index.htm) is a DOS program that can do that. It can run with dosbox.

## Prerequisites
Install dosbox (`sudo apt install dosbox` in Ubuntu).

Extract the ImageDisk 1.18 files from http://dunfield.classiccmp.org/img/index.htm

To convert TD0 files to IMD:

- On the host run `dosbox .`
- Inside dosbox run `TD02IMD FILE.TD0`

To convert IMD files to raw:

- On the host run `dosbox .` (if needed)
- Inside dosbox run `IMDU FILE.IMD FILE.IMG /B`

You can use the .IMG file with izkaypro 


