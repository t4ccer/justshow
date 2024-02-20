use justshow_x11::{error::Error, extensions::randr, replies, requests, XDisplay};
use std::process::ExitCode;

macro_rules! request_blocking {
    ($display:expr, $request:expr) => {{
        let pending_reply = $display.send_request(&($request))?;
        $display.flush()?;
        let reply = $display.await_pending_reply(pending_reply)?;
        reply
    }};
}

// Adapted from https://gitlab.freedesktop.org/xorg/app/xlsfonts/-/blob/master/xlsfonts.c
// Copyright 1989, 1998  The Open Group
// Equivalent to running `xlsfonts -l`
fn lsfonts(display: &mut XDisplay) -> Result<(), Error> {
    let mut reply = request_blocking!(
        display,
        requests::ListFontsWithInfo {
            max_names: u16::MAX,
            pattern: b"*".to_vec(),
        }
    );
    reply.replies.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name));

    println!("DIR  MIN  MAX EXIST DFLT PROP ASC DESC NAME");
    for piece in reply.replies {
        if let Ok(name) = std::str::from_utf8(&piece.name) {
            print!(
                "{}",
                match piece.draw_direction {
                    replies::DrawDirection::LeftToRight => "--> ",
                    replies::DrawDirection::RightToLeft => "<-- ",
                }
            );

            if piece.min_byte1 == 0 && piece.max_byte1 == 0 {
                print!(" {:>3} ", piece.min_char_or_byte2);
                print!(" {:>3} ", piece.max_char_or_byte2);
            } else {
                print!("*{:>3} ", piece.min_char_or_byte2);
                print!("*{:>3} ", piece.max_char_or_byte2);
            }

            print!("{:>5} ", if piece.all_chars_exist { "all" } else { "some" });
            print!("{:>4} ", piece.default_char);
            print!("{:>4} ", piece.properties.len());
            print!("{:>3} ", piece.font_ascent);
            print!("{:>4} ", piece.font_descent);
            println!("{}", name);
        } else {
            eprintln!(
                "xinfo: warning: Could not parse font name as utf8: '{:?}'",
                piece.name
            );
        }
    }

    Ok(())
}

fn lsextensions(display: &mut XDisplay) -> Result<(), Error> {
    let extensions_list = request_blocking!(display, requests::ListExtensions);
    for raw_name in extensions_list.names.strings {
        match std::str::from_utf8(&raw_name) {
            Ok(name) => {
                let extension_info = request_blocking!(
                    display,
                    requests::QueryExtension {
                        name: raw_name.clone()
                    }
                );
                println!(
                    "{} => major opcode: {}, first event: {}, first error: {}",
                    name,
                    extension_info.major_opcode,
                    extension_info.first_event,
                    extension_info.first_error
                )
            }
            Err(err) => eprintln!(
                "xinfo: warning: Could not parse extension name as utf8: '{:?}': {}",
                raw_name, err
            ),
        }
    }

    Ok(())
}

fn lsmonitors(display: &mut XDisplay) -> Result<(), Error> {
    let r = randr::requests::GetMonitors {
        window: display.screens()[0].root,
        get_active: false,
    };
    let _ = display.send_request(&r)?;
    let pending = display.send_request(&r)?;
    display.flush()?;

    let reply = display.await_pending_reply(pending)?;

    for monitor in &reply.monitors {
        // dbg!(monitor);

        let p = display.send_request(&requests::GetAtomName { atom: monitor.name })?;
        display.flush()?;
        let r = display.await_pending_reply(p)?;
        eprintln!(
            "{} {}x{}+{}+{} (...) {}mmx{}mm",
            r.name.to_string(),
            monitor.width_in_pixels,
            monitor.height_in_pixels,
            monitor.x,
            monitor.y,
            monitor.width_in_millimeters,
            monitor.height_in_millimeters,
        );
    }

    for e in display.errors() {
        dbg!(e);
    }
    Ok(())
}

enum Mode {
    LsFonts,
    LsExtensions,
    LsMonitors,
}

fn go(mode: Mode) -> Result<(), Error> {
    let mut display = XDisplay::open()?;

    match mode {
        Mode::LsFonts => lsfonts(&mut display),
        Mode::LsExtensions => lsextensions(&mut display),
        Mode::LsMonitors => lsmonitors(&mut display),
    }
}

fn main() -> ExitCode {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let args = args.iter().map(|s| s.as_ref()).collect::<Vec<_>>();
    let mode = match &args.as_slice() {
        ["ls", "fonts"] => Mode::LsFonts,
        ["ls", "extensions"] => Mode::LsExtensions,
        ["ls", "monitors"] => Mode::LsMonitors,
        _ => {
            eprintln!("xinfo: error: Invalid arguments: '{:?}'", args);
            return ExitCode::FAILURE;
        }
    };

    match go(mode) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("xinfo: error: {}", err);
            ExitCode::FAILURE
        }
    }
}
