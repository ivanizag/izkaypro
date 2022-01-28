BDOS:          EQU 0005h
; BDOS calls:
CREAD:         EQU 01h
CWRITE:        EQU 02h
CWRITESTR:     EQU 09h

org	0100h
	ld de, message
	ld c, CWRITESTR
	call BDOS
loop:
	ld c, CREAD
	call BDOS
	ld e, c
	ld c, CWRITE
	call BDOS
	jp loop
message:
	db "\r\nEcho chars:\r\n$"
