use colored::{ColoredString, Colorize};

pub struct LogOptions {
    pub border: Option<char>,
    pub border_bottom: Option<char>,
    pub quiet: bool,
}

/// Logs the given text to stdout (if quiet is False) and
/// to an optional log file. By default, we strip out newlines in order to
/// print our lines correctly, but you can override this functionality if you
/// want to print multi-line output.
pub fn log(text: ColoredString, options: LogOptions) {
    let mut border_string = ColoredString::from("");
    if let Some(border) = options.border {
        let mut buffer = String::new();
        for _ in 0..79 {
            buffer.push(border);
        }
        border_string = ColoredString::from(buffer.as_str());
        if text.fgcolor().is_some() {
            border_string = border_string.color(text.fgcolor().unwrap());
        }
        if text.bgcolor().is_some() {
            border_string = border_string.on_color(text.bgcolor().unwrap());
        }
        println!("{}{}", border_string, ColoredString::from("").clear());
    }

    println!("{}{}", text, ColoredString::from("").clear());

    if let Some(_border_bottom) = options.border_bottom {
        // This is a hold-over from legacy - if a use for this block is not
        // found shortly after release, then I suggest its removal.
    } else if !border_string.is_empty() {
        println!("{}{}", border_string, ColoredString::from("").clear());
    }
}
