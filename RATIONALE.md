
There are already a couple of IUP wrappers. Why write Clear Coat? I could have helped with an existing project, but if there are enough things I'd do differently, I'd rather work with something I wrote entirely.

iup-rust
--------

https://github.com/dcampbell24/iup-rust

- iup-rust uses `set_attrib("EXPAND", "YES")` whereas Clear Coat provides `set_expand(true)`. It would take a lot of work to add type-safe wrappers.
- Only supports one callback per event.
- Callbacks take parameters separately instead of in a struct. Clear Coat uses a struct for ease of use.
- Layouts use an `elements![]` macro instead of Clear Coat's `hbox!()`, `vbox!()`, etc. macros that are shorter.
- Unnecessary `with_iup` function to initialize and deinitialize IUP. It is the easy way for the library writer at the expense of more boilerplate in every app.
- The biggest reason is the lack of memory safety. There's no reference counting for controls. Looks like you have to manually call `destroy()` on them or they will be leaked. More importantly, it looks like wrapper structs are `Copy` and `Clone`. If you create a copy, destroy the original, then use the copy, you have a use-after-free bug, no `unsafe` required. Clear Coat goes to some trouble to ensure memory safety. There's no `destroy()` in Clear Coat because it frees controls when they are unused.

kiss-ui
-------

https://github.com/KISS-UI/kiss-ui

- No callbacks with any parameters are implemented.
- Only supports one callback per event.
- Unnecessary `show_gui` function. See above about `with_iup`.
- Layouts use a `children![]` macro, similarly to the above `elements![]` macro.
- Same lack of memory safety and no reference counting. Controls implement `Copy` and `Clone` and some have a `destroy()` method, which can cause a use-after-free with no unsafe code. Other controls like `Button` don't have any way to free them, so if you keep creating controls, you will leak memory indefinitely.
- The project scope was mentioned as not including the more complicated IUP controls like trees, whereas I think they should be included in any UI library.
- The author has said he wants to write a new GUI library written in Rust rather than wrapping IUP, so he's probably not interested in working on KISS more anyway.

I'm also trying to use macros less than either project. After I made the above changes, there wouldn't be a lot of the original code left.
