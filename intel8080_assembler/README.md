# Intel 8080 assembler

Very simple (and therefore stupid) assembler for the Intel 8080 CPU.

## Grammar

```$xslt
program             → ( dataDefinition | labelDefinition | instruction ) * ;
instruction         → instructionCode
                    | instructionCode argument
                    | instructionCode argument "," argument ;
dataDefinition      → label "EQU" number ;
labelDefinition     → label ":" ;
argument            → number
                    | dataStore ;
number              → numberLiteral
                    | ( label | numberLiteral ) ( "+" | "-" ) ( label | numberLiteral ) ;
instructionCode     → [A-Z]{2,3} ;
label               → [A-Z_]+ ;
numberLiteral       → decimalNumber | hexadecimalNumber ;
decimalNumber       → [0-9]+ ;
hexadecimalNumber   → [0-9] ( [0-9A-Fa-f] )* "H" ;
dataStore           → "A" | "B" | "C" | "D" | "E" | "H" | "L" | "M" | "P" | "SP" ;
```