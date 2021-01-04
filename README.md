# eac-compiler

A simple C compiler for learning, based on "Engineering a Compiler (the 2nd Edition)".

[C grammar](https://github.com/antlr/grammars-v4)

cargo test -- --test-threads=1

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


<primary-expression> ::= <identifier> | <number>;
<postfix-expression> ::= <primary-expression> | <postfix-expression> "(" <argument-list> ")";
<argument-list> ::= <expression> | <argument-list> "," <expression>;
<prefix-expression> ::= <postfix-expression> | "!" <postfix-expression>;
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
<expression> ::= <logical-OR-expression>;


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
