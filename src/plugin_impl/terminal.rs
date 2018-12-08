use std::io::{self, Write};

use crate::iface::{TerminalPlugin, FormatLike};

use smallvec::{smallvec, SmallVec};
use terminfo::{expand, Database, capability as cap};

// pub const CORNER_SW: char = '╗';
// pub const CORNER_SE: char = '╔';
// pub const CORNER_NSE: char = '╠';
// pub const LINE: char = '═';
pub const TEXT_START: char = '⟦';
pub const TEXT_END: char = '⟧';
// pub const ERROR_START: char = '!';
// pub const ERROR_END: char = '!';
// pub const CORNER_NW: char = '╝';
pub const CORNER_NE: char = '╚';


type Color = u16;

fn fmt_to_color(fmt: FormatLike) -> Color {
    use self::FormatLike::*;

    const CYAN: Color = 6;
    const YELLOW: Color = 3;
    const RED: Color = 1;
    const BRIGHT_RED: Color = 9;
    const BRIGHT_GREEN: Color = 10;
    match fmt {
        Text => CYAN,
        Lines => YELLOW,
        SoftWarning => RED,
        HardWarning | Error => BRIGHT_RED,
        ExplicitOk => BRIGHT_GREEN,
    }
}



#[derive(Debug)]
pub struct Terminal {
    column_count: usize,
    text_segments: SmallVec<[SmallVec<[TextSegment; 2]>; 2]>,
    error_segments: Vec<(&'static str, String)>,
    terminfo: Database,
}



impl TerminalPlugin for Terminal {
    fn new(column_count: usize) -> Self {
        let terminfo = Database::from_env().unwrap();
        Terminal {
            column_count,
            text_segments: Default::default(),
            error_segments: Default::default(),
            terminfo
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
        //FIXME this doesn't work fox `xterm-termite` for some
        let stdout = io::stdout();
        let mut term = self.writer(stdout.lock());

        let mut first = true;
        term.fmt(FormatLike::Lines);
        for segment_group in self.text_segments.iter() {
            if first {
                first = false;
                write!(term, "╔").unwrap();
            } else {
                write!(term, "╠").unwrap();
            }
            for segment in segment_group {
                term.fmt(FormatLike::Lines);
                write!(term, "{}", TEXT_START).unwrap();
                term.fmt(segment.fmt);
                write!(term, "{}", &segment.text).unwrap();
                term.fmt(FormatLike::Lines);
                write!(term, "{}", TEXT_END).unwrap();
            }
            write!(term, "\n").unwrap();
        }

        for (scope, text) in self.error_segments.iter() {
            term.fmt(FormatLike::Lines);
            write!(term, "╠").unwrap();
            term.fmt(FormatLike::Error);
            writeln!(term, "!! {}: {}", scope, text.trim()).unwrap();
        }

        term.fmt(FormatLike::Lines);
        write!(term, "{}{}", CORNER_NE, prompt_ending).unwrap();
        term.reset_fmt();
        term.flush();
    }
}


impl Terminal {

    fn writer<W>(&self, out: W) -> TermWriter<W>
        where W: Write
    {
        TermWriter {
            terminal: self,
            out
        }
    }
}

struct TermWriter<'a, W: Write+'a> {
    terminal: &'a Terminal,
    out: W
}

impl<'a, W: 'a> TermWriter<'a, W>
    where W: Write
{
    fn fmt(&mut self, fmt: FormatLike) {
        let color = fmt_to_color(fmt);
        if let Some(cap) = self.terminal.terminfo.get::<cap::SetAForeground>() {
            let _ = expand!(&mut self.out, cap.as_ref(); color);
        }
    }

    fn reset_fmt(&mut self) {
        let terminfo = &self.terminal.terminfo;
        if let Some(cap) = terminfo.get::<cap::ExitAttributeMode>() {
            expand!(&mut self.out, cap.as_ref();).unwrap();
        } else if let Some(cap) = terminfo.get::<cap::SetAttributes>() {
            expand!(&mut self.out, cap.as_ref(); 0).unwrap();
        } else {
            let cap = terminfo.get::<cap::OrigPair>().unwrap();
            expand!(&mut self.out, cap.as_ref();).unwrap()
        }
    }
}

impl<'a, W: 'a> Write for TermWriter<'a, W>
    where W: Write
{
    fn flush(&mut self) -> Result<(), io::Error> {
        self.out.flush()
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        self.out.write(buf)
    }
}

#[derive(Debug)]
struct TextSegment {
    text: String,
    fmt: FormatLike,
    pre_calculated_length: usize,
}

impl TextSegment {

    pub fn new(text: impl Into<String>, fmt: FormatLike) -> Self {
        let text = text.into();
        let len = text.chars().count();
        TextSegment {
            text,
            fmt,
            pre_calculated_length: len,
        }
    }
}