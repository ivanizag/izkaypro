name=$(basename "$1" .s)
z80asm -lout.prn $1
cp blank.img program.img
cpmcp -f kpii program.img a.bin 0:A.COM
echo Ready, execute B:A in cargo run -- $ program.img