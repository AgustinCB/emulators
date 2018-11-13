# Intel 8080 assembler

Very simple (and therefore stupid) assembler for the Intel 8080 CPU.

## Grammar

```$xslt
program             → ( dataStatement | labelStatement | orgStatement | instructionExprStmt )* EOF ;
instructionExprStmt → INTEL8080INSTRUCTION
                    | INTEL8080INSTRUCTION argumentExpression
                    | INTEL8080INSTRUCTION argumentExpression "," argumentExpression ;
orgStatement        → "ORG" numberExpression ;
dataStatement       → label ( "DB" | "DW" ) numberExpression ;
labelStatement      → label ":" ;
argumentExpression  → numberExpression
                    | dataExpression ;
numberExpression    → ( numberVariable | sumExpression | restExpression ); 
sumExpression       → numberVariable "+" numberVariable ;
restExpression      → numberVariable "-" numberVariable ;
numberVariable      → numberLiteral
                    | label ;
label               → [A-Za-z_]+ ;
numberLiteral       → decimalNumber | hexadecimalNumber | octalNumber | binaryNumber ;
decimalNumber       → [0-9]+ ;
hexadecimalNumber   → [0-9] ( [0-9A-Fa-f] )* "H" ;
octalNumber         → [0-7]+ ("O" | "Q") ;
binaryNumber        → [0-1]+ "N" ;
dataExpression      → "A" | "B" | "C" | "D" | "E" | "H" | "L" | "M" | "P" | "SP" ;
```