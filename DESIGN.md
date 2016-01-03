
## Resource Management

Clear Coat does reference counting of controls. Every wrapper struct holds a reference to a control. If the control has a parent, the parent holds a reference. Reference counting prevents dangling pointers that could happen using IUP directly and will automatically destroy controls when they are no longer used (there is no way in Clear Coat to manually destroy a control). Additionally, if a control is destroyed using `IupDestroy` directly, the wrapper will know, and if you try to access it again, it will panic instead of using a dangling pointer (as an extra safeguard). Since wrapper structs have shared ownership of controls, if you want to pass ownership to a C API, you have to use `UnwrapHandle::try_unwrap_handle` to keep the wrapper struct from destroying the control when it is dropped. In short, if you don't use any `unsafe` code and your code compiles, then you can't get a panic or memory unsafety due to a control being destroyed.

Since it is possible to drop wrapper structs without destroying the control they are wrapping, they can't really store anything. Callbacks and such need to be stored in statics.

`IupOpen` needs to be called before any other IUP functions. To accomplish this, the wrapper calls it at every entry point:

- Before creating any control like `IupDialog` or `IupButton`
- Before calling `IupMainLoop`

## Tests

Cargo will run tests in multiple threads, even if you run `cargo test -j 1`. Passing `-j 1` only makes it wait until one test finishes before starting the next, but it will still use a different thread for the next test. Such behavior is not acceptable for testing UI libraries. I found that if you use separate files in tests/, it will compile each file into a separate executable, and thus they will be run in separate processes (of course, `cargo test` runs them).

## IUP Behavior

IUP stores children as a singly-linked list starting with the first child. Iterating over them using `IupGetNextChild` and `IupGetBrother` is by far the fastest way (always O(n)). It is best to not use `IupGetChild`, as it is O(n), even though the docs don't mention it.
