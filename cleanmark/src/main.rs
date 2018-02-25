#![feature(nll)]

#[macro_use]
extern crate failure;
extern crate pulldown_cmark;
#[macro_use]
extern crate maplit;
extern crate pulldown_cmark_to_cmark;


use failure::Error;
use std::borrow::Cow;
use std::collections::HashMap;
// use std::fmt::Write;
use pulldown_cmark_to_cmark::fmt::cmark;

use pulldown_cmark::{html, Parser};

use pulldown_cmark::{
    Event,
    Tag,
    Event::{Start, End, Text, Html, InlineHtml, SoftBreak, HardBreak, FootnoteReference},
    Alignment,
};

// use pulldown_cmark::Parser;
// use pulldown_cmark_to_cmark::fmt::cmark;
use std::env;
use std::io::stdout;

use std::io::{Read, Write};
use std::fs::File;
use std::ffi::OsString;
// use std::io::prelude::*;

fn main() {
    run().expect("error");
}

fn run() -> Result<(), Error> {
    let parser = Parser::new(INPUT);

    let doc = markdown_to_doc(parser)?;


    let mut buf = String::new();
    cmark(
        Parser::new_ext(INPUT, pulldown_cmark::Options::all()),
        &mut buf,
        None,
    ).unwrap();
    stdout().write_all(buf.as_bytes()).unwrap();

    // println!("res:\n\n{:?}", doc);
    Ok(())
}


struct DocToMarkdown {
    doc_stepper: DocStepper,
}

impl DocToMarkdown {
    fn new(doc: &DocSpan) -> Self {
        DocToMarkdown {
            doc_stepper: DocStepper::new(),
        }
    }
}

impl Iterator for DocToMarkdown {
    type Item = Event;

    fn next(&mut self) {
        doc_stepper.next();
        match doc_stepper.head() {
            Some(DocGroup(ref attrs, _)) => {
                match attrs["tag"] {
                    "p" => Event::Start(Tag::Paragraph),
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => Event::Start(Tag::Header(level)),
                    "pre" => Event::Start(Tag::CodeBlock(info)),
                    "bullet" => Event::Start(Tag::Item),
                    _ => unimplemented!(),
                }
            }
            DocChars(ref text) => {
                Event::Text(text)
            }
        }
    }
}








type DocSpan = Vec<DocElement>;

#[derive(Clone, Debug)]
pub enum DocElement {
    DocGroup(HashMap<String, String>, DocSpan),
    DocChars(String),
}

pub use self::DocElement::*;

pub trait DocPlaceable {
    fn place_all(&mut self, all: &[DocElement]);
    fn place(&mut self, value: &DocElement);
}

impl DocPlaceable for DocSpan {
    fn place_all(&mut self, all: &[DocElement]) {
        for i in all {
            self.place(i);
        }
    }

    fn place(&mut self, elem: &DocElement) {
        match *elem {
            DocChars(ref text) => {
                assert!(text.chars().count() > 0);
                if let Some(&mut DocChars(ref mut value)) = self.last_mut() {
                    value.push_str(text);
                } else {
                    self.push(DocChars(text.to_owned()));
                }
            }
            DocGroup(..) => {
                self.push(elem.clone());
            }
        }
    }
}


#[derive(Clone, Debug)]
pub struct DocWriter {
    pub past: Vec<DocElement>,
    stack: Vec<Vec<DocElement>>,
}

impl DocWriter {
    pub fn new() -> DocWriter {
        DocWriter {
            past: vec![],
            stack: vec![],
        }
    }

    pub fn begin(&mut self) {
        let past = self.past.clone();
        self.past = vec![];
        self.stack.push(past);
    }

    pub fn close(&mut self, attrs: HashMap<String, String>) {
        let past = self.past.clone();
        self.past = self.stack.pop().unwrap();
        self.past.push(DocGroup(attrs, past));
    }

    pub fn place(&mut self, elem: &DocElement) {
        self.past.place(elem);
    }

    pub fn place_all(&mut self, span: &DocSpan) {
        self.past.place_all(span);
    }

    pub fn result(self) -> Result<DocSpan, Error> {
        if !self.stack.is_empty() {
            println!("{:?}", self);
            bail!("cannot get result when stack is still full");
        }
        Ok(self.past)
    }
}







struct Ctx<'b, I> {
    iter: I,
    body: &'b mut DocWriter,
}

impl<'a, 'b, I: Iterator<Item=Event<'a>>> Ctx<'b, I> {
    pub fn run(&mut self) {
        let mut numbers = HashMap::new();
        while let Some(event) = self.iter.next() {
            match event {
                Start(tag) => self.start_tag(tag, &mut numbers),
                End(tag) => self.end_tag(tag),
                Text(text) => {
                    self.body.place(&DocChars(text.to_string()));
                }
                HardBreak => {
                    self.body.place(&DocChars("\n".to_string()));
                }
                SoftBreak |
                Html(..) |
                InlineHtml(..) |
                FootnoteReference(..) => {}
            }
        }
    }

    fn start_tag(&mut self, tag: Tag<'a>, numbers: &mut HashMap<Cow<'a, str>, usize>) {
        match tag {
            Tag::Paragraph =>  {
                self.body.begin();
            }
            Tag::Header(level) => {
                self.body.begin();
            }
            Tag::CodeBlock(info) => {
                self.body.begin();
            }
            Tag::Item => {
                self.body.begin();
            }

            Tag::Rule |
            Tag::Table(..) |
            Tag::TableHead |
            Tag::TableRow |
            Tag::TableCell |
            Tag::BlockQuote |
            Tag::Emphasis |
            Tag::Strong |
            Tag::Code |
            Tag::List(_) |
            Tag::Link(..) |
            Tag::Image(..) |
            Tag::FootnoteDefinition(_) => { }
        }
    }

    fn end_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Paragraph => {
                self.body.close(hashmap! { "tag".into() => "p".into() });
            },
            Tag::Header(level) => {
                let tag = format!("h{}", level);
                self.body.close(hashmap! { "tag".into() => tag });
            }
            Tag::CodeBlock(_) => {
                self.body.close(hashmap! { "tag".into() => "pre".into() });
                // self.buf.push_str("</pre>\n"),
            }
            Tag::Item => {
                self.body.close(hashmap! { "tag".into() => "bullet".into() });
            }

            Tag::Rule => (),
            Tag::Image(_, _) => (), // shouldn't happen, handled in start

            Tag::FootnoteDefinition(_) |
            Tag::Code |
            Tag::TableCell |
            Tag::Link(_, _) |
            Tag::Table(_) |
            Tag::TableHead |
            Tag::TableRow |
            Tag::Emphasis |
            Tag::Strong |
            Tag::List(_) |
            Tag::BlockQuote => {}
        }
    }
}

pub fn markdown_to_doc<'a, I: Iterator<Item=Event<'a>>>(iter: I) -> Result<DocSpan, Error> {
    let mut doc_writer = DocWriter::new();
    let mut ctx = Ctx {
        iter: iter,
        body: &mut doc_writer,
    };
    ctx.run();
    doc_writer.result()
}





const INPUT: &'static str = r##"# The Rust Programming Language

This is the main source code repository for [Rust]. It contains the compiler,
standard library, and documentation.

[Rust]: https://www.rust-lang.org

## Quick Start
[quick-start]: #quick-start

Read ["Installation"] from [The Book].

["Installation"]: https://doc.rust-lang.org/book/second-edition/ch01-01-installation.html
[The Book]: https://doc.rust-lang.org/book/index.html

## Building from Source
[building-from-source]: #building-from-source

### Building on *nix
1. Make sure you have installed the dependencies:

   * `g++` 4.7 or later or `clang++` 3.x or later
   * `python` 2.7 (but not 3.x)
   * GNU `make` 3.81 or later
   * `cmake` 3.4.3 or later
   * `curl`
   * `git`

2. Clone the [source] with `git`:

   ```sh
   $ git clone https://github.com/rust-lang/rust.git
   $ cd rust
   ```

[source]: https://github.com/rust-lang/rust

3. Build and install:

    ```sh
    $ ./x.py build && sudo ./x.py install
    ```

    > ***Note:*** Install locations can be adjusted by copying the config file
    > from `./config.toml.example` to `./config.toml`, and
    > adjusting the `prefix` option under `[install]`. Various other options, such
    > as enabling debug information, are also supported, and are documented in
    > the config file.

    When complete, `sudo ./x.py install` will place several programs into
    `/usr/local/bin`: `rustc`, the Rust compiler, and `rustdoc`, the
    API-documentation tool. This install does not include [Cargo],
    Rust's package manager, which you may also want to build.

[Cargo]: https://github.com/rust-lang/cargo

### Building on Windows
[building-on-windows]: #building-on-windows

There are two prominent ABIs in use on Windows: the native (MSVC) ABI used by
Visual Studio, and the GNU ABI used by the GCC toolchain. Which version of Rust
you need depends largely on what C/C++ libraries you want to interoperate with:
for interop with software produced by Visual Studio use the MSVC build of Rust;
for interop with GNU software built using the MinGW/MSYS2 toolchain use the GNU
build.

#### MinGW
[windows-mingw]: #windows-mingw

[MSYS2][msys2] can be used to easily build Rust on Windows:

[msys2]: https://msys2.github.io/

1. Grab the latest [MSYS2 installer][msys2] and go through the installer.

2. Run `mingw32_shell.bat` or `mingw64_shell.bat` from wherever you installed
   MSYS2 (i.e. `C:\msys64`), depending on whether you want 32-bit or 64-bit
   Rust. (As of the latest version of MSYS2 you have to run `msys2_shell.cmd
   -mingw32` or `msys2_shell.cmd -mingw64` from the command line instead)

3. From this terminal, install the required tools:

   ```sh
   # Update package mirrors (may be needed if you have a fresh install of MSYS2)
   $ pacman -Sy pacman-mirrors

   # Install build tools needed for Rust. If you're building a 32-bit compiler,
   # then replace "x86_64" below with "i686". If you've already got git, python,
   # or CMake installed and in PATH you can remove them from this list. Note
   # that it is important that you do **not** use the 'python2' and 'cmake'
   # packages from the 'msys2' subsystem. The build has historically been known
   # to fail with these packages.
   $ pacman -S git \
               make \
               diffutils \
               tar \
               mingw-w64-x86_64-python2 \
               mingw-w64-x86_64-cmake \
               mingw-w64-x86_64-gcc
   ```

4. Navigate to Rust's source code (or clone it), then build it:

   ```sh
   $ ./x.py build && ./x.py install
   ```

#### MSVC
[windows-msvc]: #windows-msvc

MSVC builds of Rust additionally require an installation of Visual Studio 2013
(or later) so `rustc` can use its linker. Make sure to check the “C++ tools”
option.

With these dependencies installed, you can build the compiler in a `cmd.exe`
shell with:

```sh
> python x.py build
```

Currently building Rust only works with some known versions of Visual Studio. If
you have a more recent version installed the build system doesn't understand
then you may need to force rustbuild to use an older version. This can be done
by manually calling the appropriate vcvars file before running the bootstrap.

```
CALL "C:\Program Files (x86)\Microsoft Visual Studio 14.0\VC\bin\amd64\vcvars64.bat"
python x.py build
```

If you are seeing build failure when compiling `rustc_binaryen`, make sure the path
length of the rust folder is not longer than 22 characters.

#### Specifying an ABI
[specifying-an-abi]: #specifying-an-abi

Each specific ABI can also be used from either environment (for example, using
the GNU ABI in powershell) by using an explicit build triple. The available
Windows build triples are:
- GNU ABI (using GCC)
    - `i686-pc-windows-gnu`
    - `x86_64-pc-windows-gnu`
- The MSVC ABI
    - `i686-pc-windows-msvc`
    - `x86_64-pc-windows-msvc`

The build triple can be specified by either specifying `--build=<triple>` when
invoking `x.py` commands, or by copying the `config.toml` file (as described
in Building From Source), and modifying the `build` option under the `[build]`
section.

### Configure and Make
[configure-and-make]: #configure-and-make

While it's not the recommended build system, this project also provides a
configure script and makefile (the latter of which just invokes `x.py`).

```sh
$ ./configure
$ make && sudo make install
```

When using the configure script, the generated `config.mk` file may override the
`config.toml` file. To go back to the `config.toml` file, delete the generated
`config.mk` file.

## Building Documentation
[building-documentation]: #building-documentation

If you’d like to build the documentation, it’s almost the same:

```sh
$ ./x.py doc
```

The generated documentation will appear under `doc` in the `build` directory for
the ABI used. I.e., if the ABI was `x86_64-pc-windows-msvc`, the directory will be
`build\x86_64-pc-windows-msvc\doc`.

## Notes
[notes]: #notes

Since the Rust compiler is written in Rust, it must be built by a
precompiled "snapshot" version of itself (made in an earlier state of
development). As such, source builds require a connection to the Internet, to
fetch snapshots, and an OS that can execute the available snapshot binaries.

Snapshot binaries are currently built and tested on several platforms:

| Platform / Architecture        | x86 | x86_64 |
|--------------------------------|-----|--------|
| Windows (7, 8, Server 2008 R2) | ✓   | ✓      |
| Linux (2.6.18 or later)        | ✓   | ✓      |
| OSX (10.7 Lion or later)       | ✓   | ✓      |

You may find that other platforms work, but these are our officially
supported build environments that are most likely to work.

Rust currently needs between 600MiB and 1.5GiB of RAM to build, depending on platform.
If it hits swap, it will take a very long time to build.

There is more advice about hacking on Rust in [CONTRIBUTING.md].

[CONTRIBUTING.md]: https://github.com/rust-lang/rust/blob/master/CONTRIBUTING.md

## Getting Help
[getting-help]: #getting-help

The Rust community congregates in a few places:

* [Stack Overflow] - Direct questions about using the language.
* [users.rust-lang.org] - General discussion and broader questions.
* [/r/rust] - News and general discussion.

[Stack Overflow]: https://stackoverflow.com/questions/tagged/rust
[/r/rust]: https://reddit.com/r/rust
[users.rust-lang.org]: https://users.rust-lang.org/

## Contributing
[contributing]: #contributing

To contribute to Rust, please see [CONTRIBUTING](CONTRIBUTING.md).

Rust has an [IRC] culture and most real-time collaboration happens in a
variety of channels on Mozilla's IRC network, irc.mozilla.org. The
most popular channel is [#rust], a venue for general discussion about
Rust. And a good place to ask for help would be [#rust-beginners].

[IRC]: https://en.wikipedia.org/wiki/Internet_Relay_Chat
[#rust]: irc://irc.mozilla.org/rust
[#rust-beginners]: irc://irc.mozilla.org/rust-beginners

## License
[license]: #license

Rust is primarily distributed under the terms of both the MIT license
and the Apache License (Version 2.0), with portions covered by various
BSD-like licenses.

See [LICENSE-APACHE](LICENSE-APACHE), [LICENSE-MIT](LICENSE-MIT), and
[COPYRIGHT](COPYRIGHT) for details.
"##;
