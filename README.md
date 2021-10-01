## Clipboard copy and paste tool for Windows

### Motivation

The tool was derived from Sindre Sorhus' `win-clipboard` implementation,
mainly because I wanted to use the tool not only to copy and paste text
strings, but also later for images. Additionally, I wanted to
automatically get the contents of the clipboard, whenever the clipboard
changes. So I used `node-clipboard-event` to detect the change and then
`win-clipboard` to get the contents, but when s.o. copies an image to the
clipboard, `win-clipboard` also tried to copy that as a string and complained
on stderr and threw.

This will be eventually part of a larger synchronised clipboard implementation
in node.js, called `coppas`, hence the name.

### Implementation

I combined the functionality to listen to events and to copy/paste strings
into one executable. On instance can then be forked for listening to
clipboard events and then trigger some actions and other short-lived
instances can be used to copy/paste unicode strings. Non-text contents
will be silently ignored in this implementation.

The listening application can be terminated cleanly by sending it the
SIGINT or SIGTERM signals.


### Usage

```
coppas-platform-win.exe – Access the Windows clipboard (copy/paste)

Usage:
  echo Hello | coppas-platform-win.exe --copy
  coppas-platform-win.exe --paste

    --copy   - stores stdin into clipboard
    --paste  - pastes clipboard content to stdout
    --listen - writes CBCHANGED to stdout whenever the clipboard changes

MIT © Michael Rommel
based on the original version © Sindre Sorhus
```

### Building

#### On Debian

- install rust via rustup.rs: `curl https://sh.rustup.rs -sSf | sh`
- update to nightly: `rustup update; rustup default nightly`
- add toolchain: `rustup target add x86_64-pc-windows-gnu`
- install compiler via apt: `sudo apt install mingw-w64`
- run `cargo build --release`

#### On Windows

- install rust via rustup.rs: `curl https://sh.rustup.rs -sSf | sh`
- update to nightly: `rustup update; rustup default nightly`
- add toolchain: `rustup target add x86_64-pc-windows-gnu` or
  `rustup target add x86_64-pc-windows-msvc`
- install matching compiler: either the MingW64 suite via
  `https://www.mingw-w64.org/downloads/#mingw-builds` (I chose the
  Sourceforge one) or the Microsoft Visual Studio Suite via:
  `https://visualstudio.microsoft.com/visual-cpp-build-tools/` and add
  them to your PATH
- run `cargo build --target x86_64-pc-windows-(msvc|gnu) --release`

(c) Michael Rommel, MIT License

