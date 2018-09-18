# Intel 8080 assembler

Very simple assembler for the Intel 8080 CPU.

## Grammar

```$xslt
program         → ( dataDefinition | labelDefinition | instruction ) * ;
instruction     → instructionCode
                | instructionCode argument
                | instructionCode argument "," argument ;
dataDefinition  → label "EQU" number ;
labelDefinition → label ":" ;
argument        → number
                | dataStore ;
number          → numberLiteral
                | ( label | numberLiteral ) ( "+" | "-" ) ( label | numberLiteral ) ;
instructionCode → [A-Z]{2,3} ;
label           → [A-Z_]+ ;
numberLiteral   → [0-9]+ ( "H" )? ;
dataStore       → "A" | "B" | "C" | "D" | "E" | "H" | "L" | "M" | "P" | "SP" ;
```