NUM DB 1

MVI A, 10
HERE:
MVI A, 10N
MVI A, 10O
MVI A, 10H
JMP $ + 1
JMP 4+3
JMP 'a'
JMP NUM
JMP NUM + 1
JMP HERE
JMP 5*5