NUM DB 1

MVI A, 10
HERE:
MVI A, 10B
MVI A, 10O
MVI A, 10H
JMP $ + 1
JMP 4+3
JMP 'a'
JMP NUM
JMP NUM + 1
JMP HERE
JMP 5-1
JMP 5*5
JMP 5/5
JMP 5 MOD 3
JMP 5 SHL 1
JMP 5 SHR 1
JMP 7 AND 1
JMP 7 OR 1
JMP 7 XOR 1
JMP NOT 0
JMP 2*2+4/2
JMP 2*(2+2)