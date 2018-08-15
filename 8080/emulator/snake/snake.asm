SNAKE               EQU     4001H
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

ORG 100H
INIT:
; Current step will be stored in B
; 0x01 -> Saving status
; 0x02 -> Init snake
; 0x03 -> Clearing screen
; 0x04 -> All done! Game running!
MVI B, 1

INIT_STATUS:
MVI SP, FFFFH
; Direction:
; 0x01 -> Right
; 0x02 -> Up
; 0x04 -> Left
; 0x08 -> Down
; Default: Right
LXI H, STATUS
MVI D, 01H
MOV M, D

INIT_SNAKE:
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

INIT_SCREEN:
ADD B, B
CALL CLEAR_SCREEN
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
CALL DRAW_NEW_STEP
CALL UPDATE_TAIL
JMP GAME_LOOP

DRAW_NEW_STEP:
LXI H, SNAKE
INX L
CALL LOAD_NODE
LXI H, STATUS
MOV A, M
CPI 0x01
CZ MOVE_RIGHT
CPI 0x02
CZ MOVE_UP
CPI 0x04
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