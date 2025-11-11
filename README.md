# Soupa

Provides a single macro, [soupa], which implements a hypothetical block
transformation where `super { ... }` expressions are eagerly evaluated _prior_
to the current scope.

This allows closures and async blocks to borrow from their parent scope without
affecting the lifetime of said block, provided the borrow is within a `super { ... }`
expression _and_ the result of said expression does not borrow from the parent scope.

This can be thought of as the dual of `defer` in languages like [Zig](https://zig.guide/language-basics/defer/).

## Example

As an example, consider an `Arc` you want to use inside a closure.

```ignore,rust
let foo = Arc::new(/* Some expensive resource */);

let func = move || {
    super_expensive_computation(foo.clone())
};

some_more_operations(foo); // ERROR: `foo` was moved!
```

While you're only ever using a clone of `foo` within the closure `func`, you
lose access to the original because it was moved.

The typical way to avoid this is to simply clone items _before_ the closure
is created.

```ignore,rust
let foo = Arc::new(/* Some expensive resource */);

let func = {
    let foo = foo.clone();
    move || {
        super_expensive_computation(foo)
    }
};

some_more_operations(foo); // Ok!
```

This crate automates this process by allowing expressions within `super { ... }`
blocks to be automatically lifted to the parent scope and assigned to a variable.

```ignore,rust
let foo = Arc::new(/* Some expensive resource */);

let func = soupa! {
    move || {
        super_expensive_computation(super { foo.clone() })
    }
};

some_more_operations(foo); // Ok!
```

## But Why?

It's strange to support out-of-order execution like this!
Suddenly a piece of code halfway through a function body is lifted all the way
to the top of the scope and evaluated _before_ everything else.

However, it is my belief that this better reflects how an author _thinks_ about
writing these kinds of statements.
You write some business logic, and then partway through need access to a value in scope.
That's fine, you're writing a `move` closure so it'll automatically be included.

Except _oh no!_: I need that value later too, or I drop it before I drop the closure.
Now I need to scroll to the top of my business logic and add some initialization
code to allow using a value which would otherwise be automatically available.

With [`soupa`], the error can be addressed within the business logic as you
encounter it by simply wrapping the troublesome value in `super { ... }`.

I would _also_ argue that `super { ... }` behaves like a granular version of `const { ... }`.
Currently, you can write a `const { ... }` block to guarantee that an expression
is evaluated at the outermost scope possible: compile time.
From that perspective, I'd say `super { ... }` fits neatly between `const { ... }` and `{ ... }`.

* Now: `{ ... }`
* Earlier: `super { ... }`
* Earliest: `const { ... }`
