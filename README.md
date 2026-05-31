# Lox.rs

A tree-walk [Lox](https://craftinginterpreters.com/the-lox-language.html) interpreter written in Rust.
This project serves as a learning exercise for studying interpreter architecture and language design. It was implemented based on the book "[Crafting Interpreters](https://craftinginterpreters.com/)".

This implementation is not recommended for use in production environments.

## Correctness

The interpreter has been tested using the test suite from the "Crafting Interpreters" repository (version "jlox") and passes 235 out of 239 tests.

### Failed tests

- test/method/too_many_parameters.lox
  - Implementation detail: my interpreter reports an additional syntax error following synchronization in the last method of the class. However, the error expected by the test is reported correctly.
- test/super/super_at_top_level.lox
  - The interpreter expects two errors, but in my implementation, the resolver does not group errors; consequently, the interpreter stops immediately after the first one. 
- test/unexpected_character.lox
  - My interpreter formats "unexpected symbol" errors slightly differently.
- test/variable/collide_with_parameter.lox
  - In my implementation, function arguments reside in a separate scope from the function body; therefore, shadowing them is not an error.

## Limitations

The following limitations restrict the application of the project:
- No exception handling (try/except)
- No user input (input)
- No garbage collection (cyclic references persist and lead to memory leaks)
- No built-in collections (dictionaries, lists, etc.)
  - Although the language does allow for the implementation of linked lists and certain other data structures directly in Lox.
- No imports

## Syntax

### Comments

```lox
// Single-line comment
```

### Output

```lox
print "Hello";   // Output a value
```

### Literals and Types

Strings, numbers, and Boolean values are stored by value and copy every time.
Objects, classes, and functions are stored by pointer with a reference counter.

```lox
var nothing = nil;       // Absence of value
var flag = true;         // Boolean
var text = "Lox";        // String in double quotes
var num = 42;            // Only 64-bit floating point numbers

```

### Variables and Scope

```lox
var a = "global";
{
    var a = "block";     // A block creates a local scope
    print a;             // "block"
}
print a;                 // "global"
```

### Control Flow

```lox
if (cond) {
    print "yes";
} else {
    print "no";
}

while (a < 10) {
    a = a + 1;
}

// for (var i = 0; i < 10; i = i + 1) — syntactic sugar over while
```

### Functions (Closures)

```lox
fun makeCounter() {
    var i = 0;
    fun count() {
        i = i + 1;
        print i;
    }
    return count;        // Function is a first-class value
}
var counter = makeCounter();
counter(); // 1
```

### Classes and Objects

```lox
class Person {
    init(name) {
        this.name = name;
    }
    say() {
        print this.name;
    }
}
var alice = Person("Alice");  // init called automatically
alice.say();
```

### Inheritance

```lox
class Dog < Animal {          // Inheritance using <
    speak() {
        super.speak();        // Call parent method
        print "Woof!";
    }
}
```

### Built-in Function

```lox
var start = clock();          // Time in seconds
```
