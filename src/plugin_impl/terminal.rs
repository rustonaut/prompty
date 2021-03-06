use std::{
    io::{self, Write},
    ops::Range,
    cmp::min,
    iter::Peekable
};

use crate::{
    iface::{TerminalPlugin, FormatLike},
    config
};

use smallvec::{smallvec, SmallVec};
use terminfo::{expand, Database, capability as cap};

// pub const CORNER_SW: char = '╗';
const CORNER_SE: char = '╔';
const CORNER_NSE: char = '╠';
const LINE: char = '═';
const TEXT_START: char = '⟦';
const TEXT_END: char = '⟧';
const CORNER_NS: char = '║';
// pub const ERROR_START: char = '!';
// pub const ERROR_END: char = '!';
// pub const CORNER_NW: char = '╝';
const CORNER_NE: char = '╚';
const ERR_START: &str = "!!";


type Color = u8;

mod color {
    #![allow(unused)]
    use super::Color;

    pub const TEXT_WHITE: Color = 251;
    pub const CYAN: Color = 6;
    pub const YELLOW: Color = 3;
    pub const RED: Color = 1;
    pub const BRIGHT_RED: Color = 9;
    pub const BRIGHT_GREEN: Color = 10;
    pub const LIGHT_GRAY: Color = 243;
    pub const LESS_LIGHT_GRAY: Color = 240;
    pub const JUNGLE_GREEN: Color = 112;
    pub const ORANGE: Color = 208;
    pub const SIGNALING_RED: Color = 196;
}

fn fmt_to_color(fmt: FormatLike) -> Color {
    use self::FormatLike::*;

    match fmt {
        Text => color::TEXT_WHITE,
        PrimaryText => color::JUNGLE_GREEN,
        Lines => color::LIGHT_GRAY,
        SoftWarning => color::ORANGE,
        HardWarning => color::SIGNALING_RED,
        Error => color::RED,
        ExplicitOk => color::BRIGHT_GREEN,
        Hidden => color::LESS_LIGHT_GRAY
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
        //TODO split into multiple functions
        // - one for outputting text segments
        // - one for outputting error segments

        let layout = self.calculate_layout();

        let stdout = io::stdout();
        let mut term = self.writer(stdout.lock());

        self.render_text_segments(&mut term, layout);
        self.render_error_segments(&mut term);

        term.fmt(FormatLike::Lines);
        write!(term, "{}{}", CORNER_NE, prompt_ending).unwrap();
        term.reset_fmt();
        term.flush().unwrap();
    }
}


impl Terminal {

    fn render_text_segments<W>(&self, term: &mut TermWriter<W>, layout: Vec<LineLayout>)
        where W: Write
    {
        let mut first = true;
        for LineLayout { segments, join_padding, rem_padding } in layout {
            term.fmt(FormatLike::Lines);
            if first {
                first = false;
                write!(term, "{}", CORNER_SE).unwrap();
            } else {
                write!(term, "{}", CORNER_NSE).unwrap();
            }

            for segment_group in &self.text_segments[segments] {
                for segment in segment_group {
                    term.fmt(FormatLike::Lines);
                    write!(term, "{}", TEXT_START).unwrap();
                    term.fmt(segment.fmt);
                    write!(term, "{}", &segment.text).unwrap();
                    term.fmt(FormatLike::Lines);
                    write!(term, "{}", TEXT_END).unwrap();
                }
                for _ in 0..join_padding {
                    write!(term, "{}", LINE).unwrap();
                }
            }

            for _ in 0..rem_padding {
                write!(term, "{}", LINE).unwrap();
            }
            write!(term, "\n").unwrap();
        }
    }

    fn render_error_segments<W>(&self, term: &mut TermWriter<W>)
        where W: Write
    {
        for (scope, text) in self.error_segments.iter() {
            term.fmt(FormatLike::Lines);
            write!(term, "{}", CORNER_NSE).unwrap();
            term.fmt(FormatLike::Error);
            let mut text = text.trim();
            write!(term, "{} {}: ", ERR_START, scope).unwrap();
            let bulk_len = 1 + ERR_START.len() + 1 + scope.len() + 2;
            let mut rem_len = self.column_count.checked_sub(bulk_len).unwrap_or(0);
            loop {
                if text.len() <= rem_len {
                    term.fmt(FormatLike::Error);
                    write!(term, "{}", text).unwrap();
                    break;
                } else {
                    //find split point and split text
                    let split_idx = find_viable_split_idx(text, rem_len);
                    let (line_text, new_text) = text.split_at(split_idx);
                    text = new_text.trim_start();
                    rem_len = self.column_count - 3;

                    term.fmt(FormatLike::Error);
                    write!(term, "{text}", text=line_text.trim_end()).unwrap();
                    term.fmt(FormatLike::Lines);
                    write!(term, "\n{sep}", sep=CORNER_NS).unwrap();
                    for _ in 0..ERR_START.len()+1 {
                        write!(term, " ").unwrap();
                    }
                }
            }
            write!(term, "\n").unwrap();
        }
    }
}

fn find_viable_split_idx(text: &str, max_len: usize) -> usize {
    let mut last_split_idx = 0;
    let mut last_char_idx = 0;
    for (idx, ch) in text.char_indices() {
        if idx + ch.len_utf8() > max_len {
            break;
        }
        last_char_idx = idx;
        if !(ch.is_alphanumeric() || ch == '.' || ch=='!' || ch==':' || ch=='?') {
            last_split_idx = idx;
        }
    }

    if last_split_idx == 0 {
        last_char_idx
    } else {
        last_split_idx
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

    fn calculate_layout(&self) -> Vec<LineLayout> {
        // -1 as it starts with a `╠` or similar
        let init_rem_space = self.column_count - 1;

        let mut lines = Vec::new();
        let mut text_segments = self.text_segments.iter().peekable();

        let mut idx_offset = 0;
        while let Some(line) = calc_next_line_layout(&mut text_segments, init_rem_space, idx_offset) {
            idx_offset = line.segments.end;
            lines.push(line)
        }

        lines
    }
}

fn calc_next_line_layout<'a>(
    iter: &mut Peekable<impl Iterator<Item=impl IntoIterator<Item=&'a TextSegment>+Copy>>,
    init_rem_space: usize,
    idx_offset: usize
) -> Option<LineLayout> {
        let first_seg =
            match iter.next() {
                Some(seg) => seg,
                None => {return None;}
            };

        let first_item = idx_offset;
        let mut after_last_item = idx_offset + 1;
        let first_len =  calc_min_segment_group_len(first_seg);
        if first_len >= init_rem_space {
            let segments = first_item..after_last_item;
            return Some(LineLayout {
                segments,
                join_padding: 0,
                rem_padding: 0
            });
        }

        let mut rem_space = init_rem_space - first_len;

        while let Some(segment_group_iter) = iter.peek().map(|i| *i) {
            let min_len = calc_min_segment_group_len(segment_group_iter);

            if rem_space > min_len {
                rem_space -= min_len;
                after_last_item += 1;
                iter.next();
            } else {
                let segments = first_item..after_last_item;
                let (join_padding, rem_padding) = calc_padding(first_item, after_last_item, rem_space);
                return Some(LineLayout { segments, join_padding, rem_padding })
            }
        }

        let segments = first_item..after_last_item;
        let (join_padding, rem_padding) = calc_padding(first_item, after_last_item, rem_space);
        Some(LineLayout { segments, join_padding, rem_padding })
}

fn calc_padding(
    first_item: usize,
    after_last_item: usize,
    rem_space: usize
) -> (usize, usize) {
    let nr_items = after_last_item - first_item;
    let join_padding = rem_space / nr_items;
    let join_padding = min(join_padding, config::MAX_JOIN_PADDING);
    let rem_padding = rem_space - (join_padding * nr_items);
    (join_padding, rem_padding)
}

fn calc_min_segment_group_len<'a>(group: impl IntoIterator<Item=&'a TextSegment>) -> usize {
    // +2 as in TEXT_START(char) + TEXT_END(char)
    group.into_iter().map(|seg| seg.pre_calculated_length + 2).sum()
}

struct LineLayout {
    segments: Range<usize>,
    join_padding: usize,
    rem_padding: usize
}

struct TermWriter<'a, W: Write+'a> {
    terminal: &'a Terminal,
    out: W
}

impl<'a, W: 'a> TermWriter<'a, W>
    where W: Write
{
    fn fmt(&mut self, fmt: FormatLike) {
        write!(&mut self.out, "\x01").unwrap();
        let color = fmt_to_color(fmt);
        if let Some(cap) = self.terminal.terminfo.get::<cap::SetAForeground>() {
            expand!(&mut self.out, cap.as_ref(); color).unwrap();
        }
        write!(&mut self.out, "\x02").unwrap();
    }

    fn reset_fmt(&mut self) {
        write!(&mut self.out, "\x01").unwrap();
        let terminfo = &self.terminal.terminfo;
        if let Some(cap) = terminfo.get::<cap::ExitAttributeMode>() {
            expand!(&mut self.out, cap.as_ref();).unwrap();
        } else if let Some(cap) = terminfo.get::<cap::SetAttributes>() {
            expand!(&mut self.out, cap.as_ref(); 0).unwrap();
        } else if let Some(cap) = terminfo.get::<cap::OrigPair>() {
            expand!(&mut self.out, cap.as_ref();).unwrap()
        }
        write!(&mut self.out, "\x02").unwrap();
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