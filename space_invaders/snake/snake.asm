SNAKE               DW      4001H
; Status storage
; 2000 -> Direction
; 2001 -> Next direction
; 2002 -> Timer
; 2003 -> INIT status
STATUS              DW      2000H
MID_WIDTH           DB      112
MID_HEIGHT          DB      128
MID_SCREEN          DW      7080H
START_FRAME_BUFFER  DW      2400H
END_FRAME_BUFFER    DW      4000H

ORG 03H
JMP INIT

ORG 08H
HANDLE_MID_SCREEN:
JMP HANDLE_FULL_SCREEN

ORG 10H
HANDLE_FULL_SCREEN:
PUSH PSW
PUSH B
PUSH D
PUSH H
LXI H, STATUS+3
MOV A, M
CPI 08H ; Did initialization finished?
JNZ RETURN_TO_WORK ; No! Come back to work!
CALL READ_INPUT
CALL UPDATE_TIMER
RETURN_TO_WORK:
POP H
POP D
POP B
POP PSW
EI
RET

ORG 100H
INIT:
; Current step will be stored in B
; 0x01 -> Saving status
; 0x08 -> All done! Game running!
LXI H, STATUS+3
MVI M, 1

; Initialize the status
LXI SP, 0FFFFH
; Direction:
; 0x40 -> Right
; 0x08 -> Up
; 0x20 -> Left
; 0x80 -> Down
; Default: Up
LXI H, STATUS
MVI D, 08H
MOV M, D
INX H
MOV M, D

; Initialize the screen
CALL CLEAR_SCREEN

; Starting position of the snake
; Snake array.
; First byte is the size of the structure.
; Then each couple of bytes is the location of a node, starting from the head.
; Each node are the vertices of the snake.
LXI H, SNAKE
MVI M, 2
INX H
LXI D, MID_SCREEN
INX D
CALL SAVE_SNAKE_POINT
CALL DRAW_NODE
DCX D
CALL SAVE_SNAKE_POINT
CALL DRAW_NODE

LXI H, STATUS+3
MVI M, 08H
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
; Wait half a second.
; Then update the direction of the snake according to user input.
; This basically means that the snake will move one position in the screen by half a second.
CALL WAIT_HALF_SECOND
CALL UPDATE_STATUS
CALL DRAW_NEW_STEP
CALL UPDATE_TAIL
JMP GAME_LOOP

DRAW_NEW_STEP:
LXI H, SNAKE
INR L
CALL LOAD_NODE
LXI H, STATUS
MOV A, M
CPI 40H
CZ MOVE_RIGHT
CPI 08H
CZ MOVE_UP
CPI 20H
CZ MOVE_LEFT
CPI 80H
CZ MOVE_DOWN
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
;   Save new state

; Update the bit
; GET_BIT uses BC and DE, so let's save them in the stack
PUSH B
PUSH D
CALL GET_BIT
MOV A, C
XRI 0FFH
MOV C, A
MOV A, M
ANA C
MOV M, A

; Update the position of the last vertex
POP D
POP B
MOV A, D
SUB B
JZ SAME_COLUMN
JM GO_RIGHT
; If it's in different columns and the last vertex is further right than the previous one, move it left.
DCR D
JMP REMOVE_LAST_VERTEX
; If it's in different columns and the last vertex is further left than the previous one, move it right.
GO_RIGHT:
INR D
JMP REMOVE_LAST_VERTEX
SAME_COLUMN:
MOV A, E
SUB C
JM GO_UP
; If it's in the same columns (i.e. different rows) and the last vertex is further up than the previous one, move it down.
DCR E
JMP REMOVE_LAST_VERTEX
; If it's in the same columns (i.e. different rows) and the last vertex is further down than the previous one, move it up.
GO_UP:
INR E
JMP REMOVE_LAST_VERTEX

REMOVE_LAST_VERTEX: ; If they're the same
MOV A, E
CMP C
JNZ UPDATE_VERTEX ; Not the same! Moving on
MOV A, D
CMP B
JNZ UPDATE_VERTEX ; Not the same! Moving on
; The same, removing that ugly vertex
LXI H, SNAKE
MOV A, M
DCR A
MOV M, A
RET

UPDATE_VERTEX:
DCX H
DCX H
CALL SAVE_NODE
RET

DRAW_NODE:
CALL GET_BIT
MOV A, M
ORA C
MOV M, A
RET

GET_BIT:
; This method will get in HL the address in the frame buffer of the byte in the position DE
; PARAMETERS: DE -> The coordinates of the pixel in the screen.
; RETURNS: HL -> The address in the frame buffer for DE
;          C  -> A mask for that pixel
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
OUT 4
MVI A, 0
OUT 2
MVI A, 1
OUT 2
IN 3
MOV C, A

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
INR D
RET

MOVE_UP:
INR E
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
MOV C, A
ANI 40H
JNZ SAVE
MOV A, C
ANI 08H
JNZ SAVE
MOV A, C
ANI 20H
JNZ SAVE
MOV A, C
ANI 80H
RZ
SAVE:
LXI H, STATUS
INX H
MOV M, A
RET

WAIT_HALF_SECOND:
LXI H, STATUS+2
MVI M, 60 ; The variable will get updated every 1/120 second. So there'd be 120 updates per second
WAIT_HALF_SECOND_LOOP:
MOV A, M
CPI 0
JNZ WAIT_HALF_SECOND_LOOP
RET

UPDATE_STATUS:
LXI H, STATUS+1
MOV C, M
DCX H
MOV M, C
RET

UPDATE_TIMER:
LXI H, STATUS+2
MOV C, M
DCR C
MOV M, C
RET