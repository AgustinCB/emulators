SNAKE               EQU     4001H
; Status storage
; 2000 -> Direction
; 2001 -> Next direction
; 2002 -> Timer
STATUS              EQU     2000H
MID_WIDTH           EQU     112
MID_HEIGHT          EQU     128
MID_SCREEN          EQU     7080H
START_FRAME_BUFFER  EQU     2400H
END_FRAME_BUFFER    EQU     4000H

ORG 03H
JMP INIT

ORG 08H
HANDLE_MID_SCREEN:
JMP HANDLE_FULL_SCREEN

ORG 10H
HANDLE_FULL_SCREEN:
MOV A, B
CPI 0x04 ; Did initialization finished?
JNZ RETURN_TO_WORK ; No! Come back to work!
CALL DRAW_NEW_STEP
CALL UPDATE_TAIL
CALL READ_INPUT
CALL UPDATE_TIMER
RETURN_TO_WORK:
EI
RET

ORG 100H
INIT:
; Current step will be stored in B
; 0x01 -> Saving status
; 0x02 -> Init snake
; 0x03 -> Clearing screen
; 0x04 -> All done! Game running!
MVI B, 1

; Initialize the status
MVI SP, FFFFH
; Direction:
; 0x40 -> Right
; 0x08 -> Up
; 0x20 -> Left
; 0x80 -> Down
; Default: Right
LXI H, STATUS
MVI D, 40H
MOV M, D

; Starting position of the snake
; Snake array.
; First byte is the size of the structure.
; Then each couple of bytes is the location of a node, starting from the tail.
; Each node are the vertices of the snake.
ADD B, B
LXI H, SNAKE
MVI M, 1
INX H
LXI D, MID_SCREEN
CALL SAVE_SNAKE_POINT

; Initialize the screen
ADD B, B
CALL CLEAR_SCREEN
CALL DRAW_NODE
ADD B, B
CALL GAME_LOOP

SAVE_SNAKE_POINT:
MOV M, D
INX H
MOV M, E
INX H
RET

CLEAR_SCREEN:
LXI H, START_FRAME_BUFFER
CLEAR_SCREEN_LOOP:
MVI M, 00
INX H
MOV A, H
CPI 40
JNZ CLEAR_SCREEN_LOOP
RET

GAME_LOOP:
CALL WAIT_HALF_SECOND
CALL UPDATE_STATUS
JMP GAME_LOOP

DRAW_NEW_STEP:
LXI H, SNAKE
INX L
CALL LOAD_NODE
LXI H, STATUS
MOV A, M
CPI 0x40
CZ MOVE_RIGHT
CPI 0x08
CZ MOVE_UP
CPI 0x20
CZ MOVE_LEFT
CALL MOVE_DOWN
CALL SAVE_NODE
CALL DRAW_NODE
RET

UPDATE_TAIL:
RET

DRAW_NODE:
MOV C, E
; Multiply the x coordinate by 0x20
MVI A, 5
OUT 4
MVI A, 0
OUT 2
MOV A, D
OUT 2
IN 3
MOV E, A
MOV A, D
OUT 2
MVI A, 0
OUT 2
IN 3
; Sum the result to 0x2400 to get the base address
MOV D, A
LXI H, START_FRAME_BUFFER
DAD D
; Divide the y coordinate by 8 (integer division) (confirmed)
MOV A, C
OUT 2
MVI A, 0
OUT 2
IN 3
; Sum to HL to get the location of byte that affects the pixel.
MOV E, A
MVI D, 0
DAD D
MOV A, C
; Get the rest to know which bit to modify
ANI 7
MOV C, A
MOV A, M
ORA C
MOV M, A
RET

SAVE_NODE:
; Puts D in the x coordinate of the node pointed by HL
; And E in the y coordinate of the node pointed by HL+1
MOV M, D
INX H
MOV M, E
DCX H
RET

MOVE_RIGHT:
INC D
RET

MOVE_UP:
INC E
RET

MOVE_LEFT:
DCR D
RET

MOVE_DOWN:
DCR E
RET

LOAD_NODE:
; Puts the x coordinate of the node pointed by HL int D
; And the y coordinate in E
MOV D, M
INX H
MOV E, M
DCX H
RET

READ_INPUT:
IN 1
ANA 0xe8
ANA 0x40
JNZ SAVE
ANA 0x08
JNZ SAVE
ANA 0x20
JNZ SAVE
ANA 0x80
JNZ SAVE
RET
SAVE:
LXI H, STATUS
INX H
MOV M, A
RET

WAIT_HALF_SECOND:
LXI H, STATUS
INX H
INX H
MVI M, 60 ; The variable will get updated every 1/120 second. So there'd be 120 updates per second
WAIT_HALF_SECOND_LOOP:
MOV A, M
CPI 0
JNZ WAIT_HALF_SECOND_LOOP
RET

UPDATE_STATUS:
LXI H, STATUS
INX H
MOV C, M
DCX H
MOV M, C
RET

UPDATE_TIMER:
LXI H, STATUS
INX H
INX H
MOV C, M
DCR C
MOV M, C
RET