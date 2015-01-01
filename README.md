Rust-IOCP
==============

Rust-IOCP is a Windows Input/Ouput completion port (IOCP) library written in Rust.

The crate only compiles in Windows - in other operating systems the crate is simply empty.

## Installation

From the git repository:

```INI
[dependencies.iocp]

git = "https://github.com/cyderize/rust-iocp.git"
```

From crates.io:

```INI
[dependencies]

iocp = "0.0.2"
```

And add ```extern crate iocp;``` to your project.

## Usage

See the example ``` examples/main.rs`` which can be run with

```
cargo run --example main
```

## License

### The MIT License (MIT)

Copyright (c) 2014 Cyderize

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
