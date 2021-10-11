use clipboard_master::{Master, ClipboardHandler, CallbackResult};
use clipboard_win::{Clipboard, Getter, formats, set_clipboard};
use clipboard_win::raw::{is_format_avail};
use clipboard_win::raw::clipboardimage::{ClipboardImage};

use clap::{App, Arg, ArgGroup};

use std::io::{Read, Write, Error};

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use signal_hook::consts::TERM_SIGNALS;
use signal_hook::flag;

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

fn paste(sourceform: Option<&str>) -> Result<(), Error> {
    match sourceform {
        Some(format) => pasteimg(format),
        None => {
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
    }
}

fn pasteimg(sourceform: &str) -> Result<(), Error> {
    let _clip = Clipboard::new_attempts(10).expect("clipboard timed out");
    // note that CF_BITMAP, CF_DIB are auto-converted when requesting CF_DIBV5
    let preferred_formats: &[&str];
    let sf = [sourceform];
    if sourceform != "*" {
        preferred_formats = &sf;
    } else {
        preferred_formats = &[
            "Svg", "Ico", "Png", "Bmp", "Jpeg", "Gif"
        ];
    }

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
    let cli = App::new("coppas-platform-win")
        .author("Michael Rommel <rommel@layer-7.net>")
        .about("Copies into and pastes from the clipboard")
        .version("v0.2")
        .arg(Arg::new("copy")
                .takes_value(false)
                .long("copy")
                .short('c')
                .display_order(1)
                .about("sets copy mode"))
        .arg(Arg::new("paste")
                .takes_value(false)
                .long("paste")
                .short('p')
                .display_order(2)
                .about("sets paste mode"))
        .arg(Arg::new("listen")
                .takes_value(false)
                .long("listen")
                .short('l')
                .display_order(3)
                .about("sets listen mode"))
        .group(ArgGroup::new("mode")
                .args(&["copy", "paste", "listen"])
                .required(true))
        .arg(Arg::new("infile")
                .takes_value(true)
                .long("in")
                .short('i')
                .conflicts_with_all(&["outfile", "destform"])
                .about("name of the input file (optional)"))
        .arg(Arg::new("sourceform")
                .takes_value(true)
                .long("sourceform")
                .possible_values(&["*", "Svg", "Ico", "Png", "Bmp", "Jpeg", "Gif"])
                .about("specifies the source image format (optional)")
                .long_about("In copy mode, this designates the source format of the input file 
which will always be placed as PNG onto the clipboard. In paste mode,
this designates the format that will be looked for on the clipboard.
If * is given, the internal list of preferred formats (stated hereafter)
is iterated over until a matching format is found. The preferred formats
are:"))
        .arg(Arg::new("outfile")
                .takes_value(true)
                .long("out")
                .short('o')
                .about("name of the output file (optional)"))
        .arg(Arg::new("destform")
                .takes_value(true)
                .long("destform")
                .short('d')
                .about("specifies the destination image format (optional)")
                .long_about("In paste mode, this designates the format that 
will be written to the file or stdout."))
        .get_matches();

    if cli.occurrences_of("listen") > 0 {
        // listen to clipboard changes, write message to stdout on change
        listen().expect("Could not listen to clipboard changes.");
    }
    if cli.occurrences_of("copy") > 0 {
        // copy into clipboard, for now without images...
        copy().expect("Could not copy into clipboard.");
    }
    if cli.occurrences_of("paste") > 0 {
        // pastes from clipboard, for now with selection of clipboard 
        // format to select
        paste(cli.value_of("sourceform")).expect("Could not paste from clipboard.");
    }
}
