
# Clear Coat

Clear Coat is a Rust wrapper for the [IUP](http://webserver2.tecgraf.puc-rio.br/iup/) GUI library.

    [dependencies]
    clear-coat = { git = "https://github.com/jminer/clear-coat" }

For the most part, the wrapper should be thin and use the same naming so that it is easy to use IUP's documentation. There are a couple of attributes like FGCOLOR that may not ever be supported because they are generally not good to use (easy to mess up accessibility, etc.). Otherwise, the goal is to support every feature in IUP. There are a couple of features that IUP does not have, but the wrapper will:

- Support for notifying the event thread from another thread (IUP really should have this ability)
- Support for registering multiple callbacks to one event (can be worked around, but is still useful)
- The default minimum size of buttons on Windows is correct.

Of course, being written in Rust, Clear Coat provides a memory-safe interface. It should be impossible to use-after-free or get a segfault/access violation without using unsafe code. If you find a way, I'd love to hear about it so that I can fix it.

## License

This library is licensed under the MIT license, the same as IUP is.
