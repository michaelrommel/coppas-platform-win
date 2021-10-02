use clipboard_master::{Master, ClipboardHandler, CallbackResult};
use clipboard_win::{Clipboard, Getter, formats, set_clipboard};
use clipboard_win::raw::{is_format_avail};

use std::io::{Read, Write, Error};
use std::env;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use signal_hook::consts::TERM_SIGNALS;
use signal_hook::flag;

fn help() {
    println!("coppas-platform-win.exe – Access the Windows clipboard (copy/paste)");
    println!("");
    println!("Usage:");
    println!("  echo Hello | coppas-platform-win.exe --copy");
    println!("  coppas-platform-win.exe --paste");
    println!("");
    println!("    --copy   - stores stdin into clipboard");
    println!("    --paste  - pastes clipboard content to stdout");
    println!("    --listen - writes CBCHANGED to stdout whenever the clipboard changes");
    println!("");
    println!("MIT © Michael Rommel");
    println!("based on the original version © Sindre Sorhus");
}

struct Handler;

impl ClipboardHandler for Handler {
    fn on_clipboard_change(&mut self) -> CallbackResult {
        println!("CBCHANGED");
        CallbackResult::Next
    }

    fn on_clipboard_error(&mut self, err: std::io::Error) -> CallbackResult {
        eprintln!("Error: {:?}", err);
        CallbackResult::Next
    }
}

fn listen() -> Result<(), Error> {
    // Make sure double CTRL+C and similar kills
    let term_now = Arc::new(AtomicBool::new(false));
    for sig in TERM_SIGNALS {
        // When terminated by a second term signal, exit with exit code 1.
        // This will do nothing the first time (because term_now is false).
        flag::register_conditional_shutdown(*sig, 1, Arc::clone(&term_now))?;
        // But this will "arm" the above for the second time, by setting it to true.
        // The order of registering these is important, if you put this one first, it will
        // first arm and then terminate ‒ all in the first round.
        flag::register(*sig, Arc::clone(&term_now))?;
    }

    let _ = Master::new(Handler).run();

    return Ok(());
}

fn copy() -> std::io::Result<()> {
    let mut input = String::new();
    match std::io::stdin().read_to_string(&mut input) {
        Ok(_) => {
            match set_clipboard(formats::Unicode, input) {
                Ok(_) => Ok(()),
                Err(err) => panic!("set_clipboard {:?}", err),
            }
        }
        Err(err) => panic!("read_line: {:?}", err)
    }
}

fn paste() -> std::io::Result<()> {
    let _clip = Clipboard::new_attempts(10).expect("Has clipboard");
    // check whether a string is available
    if is_format_avail(formats::CF_OEMTEXT) ||
        is_format_avail(formats::CF_TEXT) ||
        is_format_avail(formats::CF_UNICODETEXT) {
        // get it
        let mut output = String::new();
        match formats::Unicode.read_clipboard(&mut output) {
            Ok(_) => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                match handle.write_all(&output.into_bytes()) {
                    Ok(_) => return Ok(()),
                    Err(err) => panic!("stdout write_all {:?}", err),
                }
            },
            Err(err) => panic!("read_clipboard: {:?}", err)
        };
    } else {
        return Ok(());
    };
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("You should specify `--copy`, `--paste` or `--listen` mode. See `--help` for usage examples.");
        return;
    }

    let cmd = &args[1];

    match &cmd[..] {
        "--copy" => copy().expect("Error: Could not copy to clipboard"),
        "--paste" => paste().expect("Error: Could not paste from clipboard"),
        "--listen" => listen().expect("Error: could not listen to change events"),
        _ => help(),
    }
}
