# eac-compiler

A simple C compiler for learning, based on "Engineering a Compiler (the 2nd Edition)".

[C grammar](https://github.com/antlr/grammars-v4)

cargo test -- --test-threads=1

Compounds statments inside functions will not produce new scopes, so we can construct complex data flow from simple program structures.

parser is not thread safe

https://software.intel.com/content/www/us/en/develop/articles/introduction-to-x64-assembly.html
https://docs.microsoft.com/en-us/cpp/build/x64-software-conventions?view=msvc-160

cfg truncate early return

# If No Alt (IfNoAlt)
cond
    false jump TagE

body
    TagE

# If Alt (IfBody, IfAlt)
cond
    false jump TagS

body
    jump TagE

    TagS
alt
    TagE

# While (WhileBody)
    TagS
cond
    false jump tagE

body
    jump TagS
    TagE

## Calling Convention

- Return value (if any) is in `rax`.
- Arguments are stored in `rcx`, `rdx`, `r8`, `r9`, and stack, from left to right. "shadow space" is reserved.
- `rsp` is the stack pointer (from 2^64-1 to 0). `rax` is the return register. `rcx`, `rdx`, `r8:r11` are volatile registers (caller-saved). `rbx`, `rsi`, `rdi`, `r12:r15` are callee-saved. Specially, we will use `rbp` to hold the base stack pointer (callee-saved).
- It is the caller's responsibility to clean the stack.

- caller/callee saved regs
- clean the stack

``` EBNF
<non-digit> ::= "A" | "B" | "C" | "D" | "E" | "F" | "G"
                | "H" | "I" | "J" | "K" | "L" | "M" | "N"
                | "O" | "P" | "Q" | "R" | "S" | "T"
                | "U" | "V" | "W" | "X" | "Y" | "Z"
                | "a" | "b" | "c" | "d" | "e" | "f" | "g"
                | "h" | "i" | "j" | "k" | "l" | "m" | "n"
                | "o" | "p" | "q" | "r" | "s" | "t"
                | "u" | "v" | "w" | "x" | "y" | "z"
                | "_";
<digit> ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9";
<identifier> ::= <non-digit> (<digit> | <non-digit>)*;
<number> ::= ["+" | "-"] <digit>+;


<primary-expression> ::= <identifier> | <number> | "(" <expression> ")";
<postfix-expression> ::= <primary-expression> | <postfix-expression> "(" <argument-list> ")";
<argument-list> ::= <expression> | <argument-list> "," <expression>;
<prefix-expression> ::= <postfix-expression> | "!" <postfix-expression> | "-" <postfix-expression>;
<multiplicative-expression> ::= <prefix-expression>
                            | <multiplicative-expression> "*" <prefix-expression>
                            | <multiplicative-expression> "/" <prefix-expression>;
<additive-expression> ::= <multiplicative-expression>
                            | <additive-expression> "+" <multiplicative-expression>
                            | <additive-expression> "-" <multiplicative-expression>;
<relational-expression> ::= <additive-expression>
                            | <relational-expression> "<" <additive-expression>
                            | <relational-expression> ">" <additive-expression>
                            | <relational-expression> "<=" <additive-expression>
                            | <relational-expression> ">=" <additive-expression>;
<equality-expression> ::= <relational-expression>
                            | <equality-expression> "==" <relational-expression>
                            | <equality-expression> "!=" <relational-expression>;
<logical-AND-expression> ::= <equality-expression>
                            | <logical-AND-expression> "&&" <equality-expression>;
<logical-OR-expression> ::= <logical-AND-expression>
                            | <logical-OR-expression> "||" <logical-AND-expression>;
<assignment-expression> ::= <identifier> "=" <logical-OR-expression>;
<expression> ::= <assignment-expression>;


<statement> ::= <declaration-statement>
                | <compound-statement>
                | <expression-statement>
                | <selection-statement>
                | <iteration-statement>
                | <jump-statement>;
<declaration-statement> ::= "int" <identifier>;
<compound-statement> ::= "{" <statement>* "}";
<expression-statement> ::= <expression> ";";
<selection-statement> ::= "if" "(" <expression> ")" <statement> ["else" <statement>];
<iteration-statement> ::= "while" "(" <expression> ")" <statement>;
<jump-statement> ::= "return" [<expression>] ";";


<function> ::= "void" | "int" <identifier> "(" <parameter-list> ")" <compound-statement>;
<parameter-list> ::= ["int" <identifier>] | <parameter-list> "," ["int" <identifier>];
```
