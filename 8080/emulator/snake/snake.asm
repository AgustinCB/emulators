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
PUSH A
PUSH B
PUSH D
PUSH H
MOV A, B
CPI 0x04 ; Did initialization finished?
JNZ RETURN_TO_WORK ; No! Come back to work!
CALL DRAW_NEW_STEP
CALL UPDATE_TAIL
CALL READ_INPUT
CALL UPDATE_TIMER
RETURN_TO_WORK:
POP H
POP D
POP B
POP A
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
; Default: Up
LXI H, STATUS
MVI D, 08H
MOV M, D

; Starting position of the snake
; Snake array.
; First byte is the size of the structure.
; Then each couple of bytes is the location of a node, starting from the head.
; Each node are the vertices of the snake.
ADD B, B
LXI H, SNAKE
MVI M, 2
INX H
LXI D, MID_SCREEN
CALL SAVE_SNAKE_POINT
INX D
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
LXI H, SNAKE
MOV C, M ; Get the size of the snake
DCR C
FIND_PREV_LAST_LOOP:
INX H
DCR C
JNZ FIND_PREV_LAST_LOOP
CALL LOAD_NODE
MOV B, D
MOV C, E
CALL LOAD_NODE
FINISH:
; Here in DE we have the coordinates of the last vertex of the snake
; And in BC we have the coordinates of the previous to last vertex of the snake
; We have to:
;   Clear the bit in the frame buffer for the last vertex.
;   Update the position of the last vertex
;   Remove the last vertex if it's the same of the previous one

; Update the bit
PUSH B
CALL GET_BIT
MOV A, C
XRA FFH
MOV C, A
MOV A, M
ANA C
MOV M, A
RET

DRAW_NODE:
CALL GET_BIT
MOV A, M
ORA C
MOV M, A
RET

GET_BIT:
; This method will get in HL the address of the byte in which that contains the bit in the frame buffer
; That affects the position at DE
; It will also set C to contain a mask that refers to that pixel.
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
; TODO: Now C contains the rest, but we need a mask built from that rest (i.e. map C to 2^C).

SAVE_NODE:
; Puts D in the x coordinate of the node pointed by HL
; And E in the y coordinate of the node pointed by HL+1
; Also advance H to the next node.
MOV M, D
INX H
MOV M, E
INX H
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
; Also advance H to the next node
MOV D, M
INX H
MOV E, M
INX H
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