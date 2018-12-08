use std::io::{self, Write};

use crate::iface::{TerminalPlugin, FormatLike};

use term::{self, color::{self, Color}};
use smallvec::{smallvec, SmallVec};

pub const CORNER_SW: char = '╗';
pub const CORNER_SE: char = '╔';
pub const CORNER_NSE: char = '╠';
pub const LINE: char = '═';
pub const TEXT_START: char = '⟦';
pub const TEXT_END: char = '⟧';
pub const ERROR_START: char = '!';
pub const ERROR_END: char = '!';
pub const CORNER_NW: char = '╝';
pub const CORNER_NE: char = '╚';

const LINE_COLOR: Color = color::YELLOW;
const ERROR_COLOR: Color = color::BRIGHT_RED;
const TEXT_COLOR: Color = color::CYAN;

fn fmt_to_color(fmt: FormatLike) -> Color {
    use self::FormatLike::*;

    match fmt {
        Text => color::CYAN,
        Lines => color::YELLOW,
        SoftWarning => color::RED,
        HardWarning => color::BRIGHT_RED,
        ExplicitOk => color::BRIGHT_GREEN
    }
}

#[derive(Debug)]
pub struct Terminal {
    column_count: usize,
    text_segments: SmallVec<[SmallVec<[TextSegment; 2]>; 2]>,
    error_segments: Vec<(&'static str, String)>
}

#[derive(Debug)]
struct TextSegment {
    text: String,
    fmt: FormatLike,
    pre_calculated_length: usize
}

impl TextSegment {

    pub fn new(text: impl Into<String>, fmt: FormatLike) -> Self {
        let text = text.into();
        let len = text.chars().count();
        TextSegment {
            text,
            fmt,
            pre_calculated_length: len
        }
    }
}

impl TerminalPlugin for Terminal {
    fn new(column_count: usize) -> Self {
        Terminal {
            column_count,
            text_segments: Default::default(),
            error_segments: Default::default()
        }
    }

    fn add_text_segment(&mut self, text: &str, fmt_args: FormatLike) {
        self.text_segments.push(smallvec![TextSegment::new(text, fmt_args)]);
    }

    fn add_error_segment(&mut self, scope: &'static str, msg: &str) {
        self.error_segments.push((scope, msg.into()));
    }

    fn extend_previous_segment(&mut self, text: &str, fmt_args: FormatLike) {
        {
            if let Some(last) = self.text_segments.last_mut() {
                last.push(TextSegment::new(text, fmt_args));
                return;
            }
        }
        self.add_text_segment(text, fmt_args);
    }

    fn flush_to_stdout(&self, prompt_ending: &str) {
        //FIXME this doesn't work fox `xterm-termite` for some reason
        let term_info = term::terminfo::TermInfo::from_name("xterm").unwrap();
        let stdout = io::stdout();
        let stdout = stdout.lock();
        let mut term = term::terminfo::TerminfoTerminal::new_with_terminfo(stdout, term_info);
        let term = &mut term as &mut term::Terminal<Output=io::StdoutLock>;

        for segment_group in self.text_segments.iter() {
            for segment in segment_group {
                term.fg(LINE_COLOR).ok();
                write!(term, ".{}", TEXT_START).unwrap();
                term.fg(fmt_to_color(segment.fmt)).ok();
                write!(term, "{}", &segment.text).unwrap();
                term.fg(LINE_COLOR).ok();
                write!(term, "{}", TEXT_END).unwrap();
            }
            write!(term, "\n").unwrap();
        }

        for (scope, text) in self.error_segments.iter() {
            term.fg(LINE_COLOR).ok();
            write!(term, "╠").unwrap();
            term.fg(ERROR_COLOR).ok();
            writeln!(term, "!! {}: {}", scope, text.trim()).unwrap();
        }

        term.fg(LINE_COLOR).ok();
        write!(term, "{}{}", CORNER_NE, prompt_ending).unwrap();
        term.flush().unwrap();
        term.reset().unwrap();
    }
}