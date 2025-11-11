//! # Soupa
//!
//! Provides a single macro, [soupa], which implements a hypothetical block
//! transformation where `super { ... }` expressions are eagerly evaluated _prior_
//! to the current scope.
//!
//! This allows closures and async blocks to borrow from their parent scope without
//! affecting the lifetime of said block, provided the borrow is within a `super { ... }`
//! expression _and_ the result of said expression does not borrow from the parent scope.
//!
//! This can be thought of as the dual of `defer` in languages like [Zig](https://zig.guide/language-basics/defer/).
//!
//! ## Example
//!
//! As an example, consider an `Arc` you want to use inside a closure.
//!
//! ```ignore,rust
//! let foo = Arc::new(/* Some expensive resource */);
//!
//! let func = move || {
//!     super_expensive_computation(foo.clone())
//! };
//!
//! some_more_operations(foo); // ERROR: `foo` was moved!
//! ```
//!
//! While you're only ever using a clone of `foo` within the closure `func`, you
//! lose access to the original because it was moved.
//!
//! The typical way to avoid this is to simply clone items _before_ the closure
//! is created.
//!
//! ```ignore,rust
//! let foo = Arc::new(/* Some expensive resource */);
//!
//! let func = {
//!     let foo = foo.clone();
//!     move || {
//!         super_expensive_computation(foo)
//!     }
//! };
//!
//! some_more_operations(foo); // Ok!
//! ```
//!
//! This crate automates this process by allowing expressions within `super { ... }`
//! blocks to be automatically lifted to the parent scope and assigned to a variable.
//!
//! ```ignore,rust
//! let foo = Arc::new(/* Some expensive resource */);
//!
//! let func = soupa! {
//!     move || {
//!         super_expensive_computation(super { foo.clone() })
//!     }
//! };
//!
//! some_more_operations(foo); // Ok!
//! ```
//!
//! ## But Why?
//!
//! It's strange to support out-of-order execution like this!
//! Suddenly a piece of code halfway through a function body is lifted all the way
//! to the top of the scope and evaluated _before_ everything else.
//!
//! However, it is my belief that this better reflects how an author _thinks_ about
//! writing these kinds of statements.
//! You write some business logic, and then partway through need access to a value in scope.
//! That's fine, you're writing a `move` closure so it'll automatically be included.
//!
//! Except _oh no!_: I need that value later too, or I drop it before I drop the closure.
//! Now I need to scroll to the top of my business logic and add some initialization
//! code to allow using a value which would otherwise be automatically available.
//!
//! With [`soupa`], the error can be addressed within the business logic as you
//! encounter it by simply wrapping the troublesome value in `super { ... }`.
//!
//! I would _also_ argue that `super { ... }` behaves like a granular version of `const { ... }`.
//! Currently, you can write a `const { ... }` block to guarantee that an expression
//! is evaluated at the outermost scope possible: compile time.
//! From that perspective, I'd say `super { ... }` fits neatly between `const { ... }` and `{ ... }`.
//!
//! * Now: `{ ... }`
//! * Earlier: `super { ... }`
//! * Earliest: `const { ... }`

#![no_std]

/// Provides access to `super` blocks, a hypothetical language feature which
/// reorders inline `super { ... }` blocks into init statements at the top of the
/// inner scope.
///
/// # Examples
///
/// ```rust
/// # use std::sync::Arc;
/// # use soupa::soupa;
/// let foo = Arc::new(123usize);
///
/// let func = soupa!(move || {
///     // Any super { ... } expressions are eagerly evaluated and stored in
///     // temporary variables.
///     println!("Foo: {:?}", super { foo.clone() })
/// });
///
/// // The clone of foo was eagerly evaluated, so this foo can be dropped
/// // while func is live.
/// let _ = foo;
///
/// func();
/// ```
#[macro_export]
macro_rules! soupa {
    (
        @temps { $($temp:ident)* },
        @stack: {},
        @init: { $($init:tt)* },
        @body: { $($body:tt)* },
    ) => {
        // Stack is empty
        // Output the initialization and body statements
        {
            $($init)*
            $($body)*
        }
    };

    (
        @temps { $next_ident:ident $($temp:ident)* },
        @stack: {
            {
                @paren: $top_paren:tt,
                @body: { $($top_body:tt)* },
                @rest: {
                    super { $($next:tt)* }
                    $($top_rest:tt)*
                },
            }
            $($stack:tt)*
        },
        @init: { $($init:tt)* },
        @body: { $($body:tt)* },
    ) => {
        // Process a super block into an init statement
        // Place an identifier of the declaration into the top of the stack
        $crate::soupa! {
            @temps { $($temp)* },
            @stack: {
                {
                    @paren: $top_paren,
                    @body: {
                        $($top_body)*
                        $next_ident
                    },
                    @rest: { $($top_rest)* },
                }
                $($stack)*
            },
            @init: {
                $($init)*
                let $next_ident = { $($next)* };
            },
            @body: { $($body)* },
        }
    };
    (
        @temps { $($temp:ident)* },
        @stack: {
            {
                @paren: $top_paren:tt,
                @body: { $($top_body:tt)* },
                @rest: {
                    { $($next:tt)* }
                    $($top_rest:tt)*
                },
            }
            $($stack:tt)*
        },
        @init: { $($init:tt)* },
        @body: { $($body:tt)* },
    ) => {
        // Peel off a {} tree and place it onto the top of the stack
        $crate::soupa! {
            @temps { $($temp)* },
            @stack: {
                {
                    @paren: {},
                    @body: {},
                    @rest: { $($next)* },
                }
                {
                    @paren: $top_paren,
                    @body: { $($top_body)* },
                    @rest: { $($top_rest)* },
                }
                $($stack)*
            },
            @init: { $($init)* },
            @body: { $($body)* },
        }
    };
    (
        @temps { $($temp:ident)* },
        @stack: {
            {
                @paren: $top_paren:tt,
                @body: { $($top_body:tt)* },
                @rest: {
                    ( $($next:tt)* )
                    $($top_rest:tt)*
                },
            }
            $($stack:tt)*
        },
        @init: { $($init:tt)* },
        @body: { $($body:tt)* },
    ) => {
        // Peel off a () tree and place it onto the top of the stack
        $crate::soupa! {
            @temps { $($temp)* },
            @stack: {
                {
                    @paren: (),
                    @body: {},
                    @rest: { $($next)* },
                }
                {
                    @paren: $top_paren,
                    @body: { $($top_body)* },
                    @rest: { $($top_rest)* },
                }
                $($stack)*
            },
            @init: { $($init)* },
            @body: { $($body)* },
        }
    };
    (
        @temps { $($temp:ident)* },
        @stack: {
            {
                @paren: $top_paren:tt,
                @body: { $($top_body:tt)* },
                @rest: {
                    [ $($next:tt)* ]
                    $($top_rest:tt)*
                },
            }
            $($stack:tt)*
        },
        @init: { $($init:tt)* },
        @body: { $($body:tt)* },
    ) => {
        // Peel off a [] tree and place it onto the top of the stack
        $crate::soupa! {
            @temps { $($temp)* },
            @stack: {
                {
                    @paren: [],
                    @body: {},
                    @rest: { $($next)* },
                }
                {
                    @paren: $top_paren,
                    @body: { $($top_body)* },
                    @rest: { $($top_rest)* },
                }
                $($stack)*
            },
            @init: { $($init)* },
            @body: { $($body)* },
        }
    };
    (
        @temps { $($temp:ident)* },
        @stack: {
            {
                @paren: $top_paren:tt,
                @body: { $($top_body:tt)* },
                @rest: {
                    $next:tt
                    $($top_rest:tt)*
                },
            }
            $($stack:tt)*
        },
        @init: { $($init:tt)* },
        @body: { $($body:tt)* },
    ) => {
        // Peel off a misc token and place in the top scope output
        $crate::soupa! {
            @temps { $($temp)* },
            @stack: {
                {
                    @paren: $top_paren,
                    @body: {
                        $($top_body)*
                        $next
                    },
                    @rest: { $($top_rest)* },
                }
                $($stack)*
            },
            @init: { $($init)* },
            @body: { $($body)* },
        }
    };

    (
        @temps { $($temp:ident)* },
        @stack: {
            {
                @paren: {},
                @body: { $($next:tt)* },
                @rest: { },
            }
            {
                @paren: $top_paren:tt,
                @body: { $($top_body:tt)* },
                @rest: { $($top_rest:tt)* },
            }
            $($stack:tt)*
        },
        @init: { $($init:tt)* },
        @body: { $($body:tt)* },
    ) => {
        // Top item on the stack is done
        // Combine it with the next item down
        $crate::soupa! {
            @temps { $($temp)* },
            @stack: {
                {
                    @paren: $top_paren,
                    @body: {
                        $($top_body)*
                        { $($next)* }
                    },
                    @rest: { $($top_rest)* },
                }
                $($stack)*
            },
            @init: { $($init)* },
            @body: { $($body)* },
        }
    };
    (
        @temps { $($temp:ident)* },
        @stack: {
            {
                @paren: (),
                @body: { $($next:tt)* },
                @rest: { },
            }
            {
                @paren: $top_paren:tt,
                @body: { $($top_body:tt)* },
                @rest: { $($top_rest:tt)* },
            }
            $($stack:tt)*
        },
        @init: { $($init:tt)* },
        @body: { $($body:tt)* },
    ) => {
        // Top item on the stack is done
        // Combine it with the next item down
        $crate::soupa! {
            @temps { $($temp)* },
            @stack: {
                {
                    @paren: $top_paren,
                    @body: {
                        $($top_body)*
                        ( $($next)* )
                    },
                    @rest: { $($top_rest)* },
                }
                $($stack)*
            },
            @init: { $($init)* },
            @body: { $($body)* },
        }
    };
    (
        @temps { $($temp:ident)* },
        @stack: {
            {
                @paren: [],
                @body: { $($next:tt)* },
                @rest: { },
            }
            {
                @paren: $top_paren:tt,
                @body: { $($top_body:tt)* },
                @rest: { $($top_rest:tt)* },
            }
            $($stack:tt)*
        },
        @init: { $($init:tt)* },
        @body: { $($body:tt)* },
    ) => {
        // Top item on the stack is done
        // Combine it with the next item down
        $crate::soupa! {
            @temps { $($temp)* },
            @stack: {
                {
                    @paren: $top_paren,
                    @body: {
                        $($top_body)*
                        [ $($next)* ]
                    },
                    @rest: { $($top_rest)* },
                }
                $($stack)*
            },
            @init: { $($init)* },
            @body: { $($body)* },
        }
    };

    (
        @temps { $($temp:ident)* },
        @stack: {
            {
                @paren: {},
                @body: { $($next:tt)* },
                @rest: { },
            }
        },
        @init: { $($init:tt)* },
        @body: { $($body:tt)* },
    ) => {
        // Stack fully processed
        // Only a {} wrapped body is left, so output it
        $crate::soupa! {
            @temps { $($temp)* },
            @stack: {},
            @init: { $($init)* },
            @body: {
                $($body)*
                { $($next)* }
            },
        }
    };
    (
        @temps { $($temp:ident)* },
        @stack: {
            {
                @paren: (),
                @body: { $($next:tt)* },
                @rest: { },
            }
        },
        @init: { $($init:tt)* },
        @body: { $($body:tt)* },
    ) => {
        // Stack fully processed
        // Only a () wrapped body is left, so output it
        $crate::soupa! {
            @temps { $($temp)* },
            @stack: {},
            @init: { $($init)* },
            @body: {
                $($body)*
                ( $($next)* )
            },
        }
    };
    (
        @temps { $($temp:ident)* },
        @stack: {
            {
                @paren: [],
                @body: { $($next:tt)* },
                @rest: { },
            }
        },
        @init: { $($init:tt)* },
        @body: { $($body:tt)* },
    ) => {
        // Stack fully processed
        // Only a [] wrapped body is left, so output it
        $crate::soupa! {
            @temps { $($temp)* },
            @stack: {},
            @init: { $($init)* },
            @body: {
                $($body)*
                [ $($next)* ]
            },
        }
    };
    (
        @temps { $($temp:ident)* },
        @stack: {
            {
                @paren: None,
                @body: { $($next:tt)* },
                @rest: { },
            }
        },
        @init: { $($init:tt)* },
        @body: { $($body:tt)* },
    ) => {
        // Stack fully processed
        // Only an unwrapped body is left, so output it
        $crate::soupa! {
            @temps { $($temp)* },
            @stack: {},
            @init: { $($init)* },
            @body: {
                $($body)*
                $($next)*
            },
        }
    };

    (
        $($rest:tt)*
    ) => {
        // No other rule matches
        // Implies this is user supplied, so initialize with some temp variable names
        $crate::soupa! {
            @temps {
                __soupa_temp__a __soupa_temp__b __soupa_temp__c __soupa_temp__d __soupa_temp__e __soupa_temp__f __soupa_temp__g __soupa_temp__h __soupa_temp__i __soupa_temp__j __soupa_temp__k __soupa_temp__l __soupa_temp__m __soupa_temp__n __soupa_temp__o __soupa_temp__p __soupa_temp__q __soupa_temp__r __soupa_temp__s __soupa_temp__t __soupa_temp__u __soupa_temp__v __soupa_temp__w __soupa_temp__x __soupa_temp__y __soupa_temp__z
                __soupa_temp__aa __soupa_temp__ab __soupa_temp__ac __soupa_temp__ad __soupa_temp__ae __soupa_temp__af __soupa_temp__ag __soupa_temp__ah __soupa_temp__ai __soupa_temp__aj __soupa_temp__ak __soupa_temp__al __soupa_temp__am __soupa_temp__an __soupa_temp__ao __soupa_temp__ap __soupa_temp__aq __soupa_temp__ar __soupa_temp__as __soupa_temp__at __soupa_temp__au __soupa_temp__av __soupa_temp__aw __soupa_temp__ax __soupa_temp__ay __soupa_temp__az
                __soupa_temp__ba __soupa_temp__bb __soupa_temp__bc __soupa_temp__bd __soupa_temp__be __soupa_temp__bf __soupa_temp__bg __soupa_temp__bh __soupa_temp__bi __soupa_temp__bj __soupa_temp__bk __soupa_temp__bl __soupa_temp__bm __soupa_temp__bn __soupa_temp__bo __soupa_temp__bp __soupa_temp__bq __soupa_temp__br __soupa_temp__bs __soupa_temp__bt __soupa_temp__bu __soupa_temp__bv __soupa_temp__bw __soupa_temp__bx __soupa_temp__by __soupa_temp__bz
                __soupa_temp__ca __soupa_temp__cb __soupa_temp__cc __soupa_temp__cd __soupa_temp__ce __soupa_temp__cf __soupa_temp__cg __soupa_temp__ch __soupa_temp__ci __soupa_temp__cj __soupa_temp__ck __soupa_temp__cl __soupa_temp__cm __soupa_temp__cn __soupa_temp__co __soupa_temp__cp __soupa_temp__cq __soupa_temp__cr __soupa_temp__cs __soupa_temp__ct __soupa_temp__cu __soupa_temp__cv __soupa_temp__cw __soupa_temp__cx __soupa_temp__cy __soupa_temp__cz
                __soupa_temp__da __soupa_temp__db __soupa_temp__dc __soupa_temp__dd __soupa_temp__de __soupa_temp__df __soupa_temp__dg __soupa_temp__dh __soupa_temp__di __soupa_temp__dj __soupa_temp__dk __soupa_temp__dl __soupa_temp__dm __soupa_temp__dn __soupa_temp__do __soupa_temp__dp __soupa_temp__dq __soupa_temp__dr __soupa_temp__ds __soupa_temp__dt __soupa_temp__du __soupa_temp__dv __soupa_temp__dw __soupa_temp__dx __soupa_temp__dy __soupa_temp__dz
                __soupa_temp__ea __soupa_temp__eb __soupa_temp__ec __soupa_temp__ed __soupa_temp__ee __soupa_temp__ef __soupa_temp__eg __soupa_temp__eh __soupa_temp__ei __soupa_temp__ej __soupa_temp__ek __soupa_temp__el __soupa_temp__em __soupa_temp__en __soupa_temp__eo __soupa_temp__ep __soupa_temp__eq __soupa_temp__er __soupa_temp__es __soupa_temp__et __soupa_temp__eu __soupa_temp__ev __soupa_temp__ew __soupa_temp__ex __soupa_temp__ey __soupa_temp__ez
                __soupa_temp__fa __soupa_temp__fb __soupa_temp__fc __soupa_temp__fd __soupa_temp__fe __soupa_temp__ff __soupa_temp__fg __soupa_temp__fh __soupa_temp__fi __soupa_temp__fj __soupa_temp__fk __soupa_temp__fl __soupa_temp__fm __soupa_temp__fn __soupa_temp__fo __soupa_temp__fp __soupa_temp__fq __soupa_temp__fr __soupa_temp__fs __soupa_temp__ft __soupa_temp__fu __soupa_temp__fv __soupa_temp__fw __soupa_temp__fx __soupa_temp__fy __soupa_temp__fz
                __soupa_temp__ga __soupa_temp__gb __soupa_temp__gc __soupa_temp__gd __soupa_temp__ge __soupa_temp__gf __soupa_temp__gg __soupa_temp__gh __soupa_temp__gi __soupa_temp__gj __soupa_temp__gk __soupa_temp__gl __soupa_temp__gm __soupa_temp__gn __soupa_temp__go __soupa_temp__gp __soupa_temp__gq __soupa_temp__gr __soupa_temp__gs __soupa_temp__gt __soupa_temp__gu __soupa_temp__gv __soupa_temp__gw __soupa_temp__gx __soupa_temp__gy __soupa_temp__gz
                __soupa_temp__ha __soupa_temp__hb __soupa_temp__hc __soupa_temp__hd __soupa_temp__he __soupa_temp__hf __soupa_temp__hg __soupa_temp__hh __soupa_temp__hi __soupa_temp__hj __soupa_temp__hk __soupa_temp__hl __soupa_temp__hm __soupa_temp__hn __soupa_temp__ho __soupa_temp__hp __soupa_temp__hq __soupa_temp__hr __soupa_temp__hs __soupa_temp__ht __soupa_temp__hu __soupa_temp__hv __soupa_temp__hw __soupa_temp__hx __soupa_temp__hy __soupa_temp__hz
                __soupa_temp__ia __soupa_temp__ib __soupa_temp__ic __soupa_temp__id __soupa_temp__ie __soupa_temp__if __soupa_temp__ig __soupa_temp__ih __soupa_temp__ii __soupa_temp__ij __soupa_temp__ik __soupa_temp__il __soupa_temp__im __soupa_temp__in __soupa_temp__io __soupa_temp__ip __soupa_temp__iq __soupa_temp__ir __soupa_temp__is __soupa_temp__it __soupa_temp__iu __soupa_temp__iv __soupa_temp__iw __soupa_temp__ix __soupa_temp__iy __soupa_temp__iz
            },
            @stack: {
                {
                    @paren: None,
                    @body: {},
                    @rest: { $($rest)* },
                }
            },
            @init: {},
            @body: {},
        }
    };
}
