# Intel 8080 assembler

Very simple (and therefore stupid) assembler for the Intel 8080 CPU.

## Grammar

```$xslt
program             → ( dataDefinition | labelDefinition | orgStatement | instruction ) * ;
instruction         → INTEL8080INSTRUCTION
                    | INTEL8080INSTRUCTION argument
                    | INTEL8080INSTRUCTION argument "," argument ;
orgStatement        → "ORG" numberExpression ;
dataDefinition      → label ( "DB" | "DW" ) numberExpression ;
labelDefinition     → label ":" ;
argument            → numberExpression
                    | dataStore ;
numberExpression    → ( number | sumExpression | restExpression ); 
sumExpression       → number "+" number ;
restExpression      → number "-" number ;
number              → numberLiteral
                    | label ;
label               → [A-Za-z_]+ ;
numberLiteral       → decimalNumber | hexadecimalNumber ;
decimalNumber       → [0-9]+ ;
hexadecimalNumber   → [0-9] ( [0-9A-Fa-f] )* "H" ;
dataStore           → "A" | "B" | "C" | "D" | "E" | "H" | "L" | "M" | "P" | "SP" ;
```