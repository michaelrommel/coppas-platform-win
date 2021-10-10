use clipboard_master::{Master, ClipboardHandler, CallbackResult};
use clipboard_win::{Clipboard, Getter, formats, set_clipboard};
use clipboard_win::raw::{is_format_avail};
use clipboard_win::raw::clipboardimage::{ClipboardImage};

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
    println!("    --copy       - stores stdin into clipboard");
    println!("    --paste      - pastes clipboard content to stdout");
    println!("    --paste-img  - pastes images from clipboard to stdout");
    println!("                   (preferred formats: png, bmp)");
    println!("    --listen     - writes CBCHANGED to stdout whenever the clipboard changes");
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

fn copy() -> Result<(), Error> {
    let mut input = String::new();
    match std::io::stdin().read_to_string(&mut input) {
        Ok(_) => {
            match set_clipboard(formats::Unicode, input) {
                Ok(_) => Ok(()),
                Err(err) => panic!("set_clipboard failed: {:?}", err),
            }
        }
        Err(err) => panic!("read_line failed: {:?}", err)
    }
}

fn write_stdout(out: Vec<u8>) -> Result<(), Error> {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    match handle.write_all(&out) {
        Ok(_) => return Ok(()),
        Err(err) => panic!("stdout write_all failed: {:?}", err),
    }
}

fn paste() -> Result<(), Error> {
    let _clip = Clipboard::new_attempts(10).expect("clipboard timed out");
    // check whether a string is available
    if is_format_avail(formats::CF_OEMTEXT) ||
        is_format_avail(formats::CF_TEXT) ||
        is_format_avail(formats::CF_UNICODETEXT) {
        // get it
        let mut output = String::new();
        match formats::Unicode.read_clipboard(&mut output) {
            Ok(_) => {
                return write_stdout(output.into_bytes());
            },
            Err(err) => panic!("read_clipboard failed: {:?}", err)
        };
    } else {
        return Ok(());
    };
}

fn pasteimg() -> Result<(), Error> {
    let _clip = Clipboard::new_attempts(10).expect("clipboard timed out");
    // note that CF_BITMAP, CF_DIB are auto-converted when requesting CF_DIBV5
    let preferred_formats: &[&str] = &[
        "Svg", "Ico", "Png", "Bmp", "Jpeg", "Gif"
        // "Jpeg"
    ];

    let img: ClipboardImage = ClipboardImage::new(preferred_formats);

    match img {
        ClipboardImage::ImageString(ref id, ref name, ref _store) => {
            eprintln!("Created string based image: {:?}: {:?}", id, name);
        },
        ClipboardImage::ImageBinary(ref format, ref _store) => {
            eprintln!("Created binary image: {:?}", format);
        },
        ClipboardImage::NotFound => {
            eprintln!("No matching image type found on clipboard!");
            return Ok(());
        }
    }

    let mut output = Vec::new();
    match img.write_to_buffer(&mut output) {
        Ok(no_of_bytes) => {
            eprintln!("Wrote {:?} bytes to buffer", no_of_bytes);
            return write_stdout(output);
        },
        Err(err) => panic!("write_to_buffer failed {:?}", err),
    }
}

// maybe later extend this to directly save to a file:
// let di : DynamicImage = load_from_memory(&output).expect("no png found");
// eprintln!("Info: {:?}", di.color().has_alpha());
// di.save("C:\\Temp\\test_alpha.png").expect("Not saved");
// return Ok(());
//
// maybe extend this to produce string data urls from images like:
// img.src = "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///ywAAAAAAQABAAACAUwAOw==";
// data:[<MIME-Typ>][;charset=<Zeichensatz>][;base64],<Daten>

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("You should specify the operations mode. See `--help` for usage examples.");
        return;
    }

    let cmd = &args[1];

    match &cmd[..] {
        "--copy" => copy().expect("Error: Could not copy to clipboard"),
        // "--copy-img" => copyimg().expect("Error: Could not copy to clipboard"),
        "--paste" => paste().expect("Error: Could not paste from clipboard"),
        "--paste-img" => pasteimg().expect("Error: Could not paste from clipboard"),
        "--listen" => listen().expect("Error: could not listen to change events"),
        _ => help(),
    }
}
