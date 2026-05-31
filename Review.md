1. Code organization of the modules - 
I'd change structure from flat to hierarchical, so different parts
of logic would be placed in different modules (e.g. tokenizer, parser, model, interpreter, etc.)
2. Top-level module docs + some types docs may simlify code readability
(but keep them brief)
3. clippy?
4. Can use parser-combinator (like nom, or chumsky) instead of custom lexer/parser.