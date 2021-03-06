use gtk::pango;
use gtk::prelude::*;
use gtk::gdk::RGBA;

use std::ops::Range;
use strum::EnumIter;
use strum::IntoEnumIterator;

use unicode_segmentation::UnicodeSegmentation;

struct FormatSpan {
    range: Range<usize>,
    tag: Tag,
}

#[derive(Clone, Copy, EnumIter)]
pub enum Tag {
    Emphasis,
    Strong,
    Strikethrough,
    Code,
    Syntax,
}

impl Tag {
    fn to_string(&self) -> String {
        String::from(match self {
            Self::Emphasis => "em",
            Self::Strong => "strong",
            Self::Strikethrough => "s",
            Self::Code => "code",
            Self::Syntax => "syntax",
        })
    }

    pub fn init_tags(buffer: &gtk::TextBuffer) {
        let tag_table = buffer.tag_table().unwrap();
        for tag in Self::iter() {
            let text_tag = Box::new(gtk::TextTag::new(Some(&tag.to_string())));
            match tag {
                Self::Emphasis => {
                    text_tag.set_property("style", pango::Style::Italic);
                },
                Self::Strong => {
                    text_tag.set_property::<i32>("weight", 700);
                },
                Self::Strikethrough => {
                    text_tag.set_property("strikethrough", true);
                },
                Self::Code => {
                    text_tag.set_property("font", "Ubuntu Mono");
                },
                Self::Syntax => {
                    text_tag.set_property("foreground-rgba", RGBA::new(0.5, 0.5, 0.5, 1.0));
                },
            };
            tag_table.add(&*text_tag);
        }
    }
}

#[derive(Default)]
pub struct Renderer {
    format_spans: Vec<FormatSpan>,
    parsed_chars: Vec<usize>,
    escape_chars: Vec<usize>,
    update_range: Option<Range<usize>>,
    pub text: String,
}

impl Renderer {
    pub fn from(buffer: &gtk::TextBuffer) -> Self {
        let (start, end) = buffer.bounds();
        Self {
            text: String::from(buffer.text(&start, &end, true).unwrap().as_str()),
            ..Default::default()
        }
    }

    fn unparsed_chars(&self) -> Vec<(usize, String)> {
        let chars = self.text
            .graphemes(true)
            .enumerate()
            .collect::<Vec<(usize, &str)>>();
        let chars = match &self.update_range {
            Some(update_range) => chars[update_range.clone().start..update_range.clone().end].into(),
            None => chars
        };
        chars
            .iter()
            .filter(|&(i, _)| !self.parsed_chars.contains(&i))
            .map(|(i, chr)| (*i, String::from(*chr)))
            .collect::<Vec<(usize, String)>>()
    }

    fn render(&mut self) {
        self
            .escape_chars()
            .strong()
            .emphasis()
            .strikethrough()
            .code()
            .syntax();
    }

    // force_all should be true for example when loading a document the first time,
    // when not only the single paragraph needs to be reformatted
    pub fn display(&mut self, buffer: &gtk::TextBuffer, force_all: bool) {
        let start;
        let end;
        if force_all {
            (start, end) = buffer.bounds();
        } else {
            let unparsed_chars = self.unparsed_chars();
            let mut update_range = 0..unparsed_chars.len();
            for (i, _chr) in unparsed_chars[0..buffer.cursor_position() as usize].iter().rev() {
                if self.text[self.text.char_indices().map(|(i, _)| i).nth(*i).unwrap()..].starts_with("\n\n") {
                    update_range.start = *i + 1;
                    break;
                }
            }
            for (i, _chr) in unparsed_chars[buffer.cursor_position() as usize..].iter() {
                if self.text[self.text.char_indices().map(|(i, _)| i).nth(*i).unwrap()..].starts_with("\n\n") {
                    update_range.end = *i;
                    break;
                }
            }
            println!("{:?}", update_range);
            start = buffer.iter_at_offset(update_range.start as i32);
            end = buffer.iter_at_offset(update_range.end as i32);
            self.update_range = Some(update_range);
        }
        buffer.remove_all_tags(&start, &end);
        self.render();
        for format_span in self.format_spans.iter() {
            buffer.apply_tag_by_name(
                &format_span.tag.to_string(),
                &buffer.iter_at_offset(format_span.range.start as i32),
                &buffer.iter_at_offset(format_span.range.end as i32),
            );
        }
    }

    fn escape_chars(&mut self) -> &mut Self {
        let mut escaped = false;
        for (i, chr) in self.text.chars().enumerate() {
            if chr == '\n' {
                escaped = false;
            } else if escaped {
                self.escape_chars.push(i - 1);
                if chr == '\\' {
                    self.parsed_chars.push(i - 1);
                }
                escaped = false;
            } else if chr == '\\' {
                escaped = true;
            }
        }
        self
    }

    fn inline(&mut self, syntax: &str, tag: Tag) -> &mut Self {
        let mut in_syntax = false;
        let mut start = 0;
        let mut end;
        let len = syntax.len();
        let unparsed_chars = self.unparsed_chars();
        let mut unparsed_chars_iter = unparsed_chars.iter();
        while unparsed_chars_iter.size_hint().0 > 0 {
            let (i, _chr) = unparsed_chars_iter.next().unwrap();
            let i = *i;
            // https://stackoverflow.com/a/51983601
            let next_text = &self.text[self.text.char_indices().map(|(i, _)| i).nth(i).unwrap()..];
            // Double new line resets parsing,
            // also skips next character (another new line)
            if next_text.starts_with("\n\n") {
                in_syntax = false;
                unparsed_chars_iter.next();
                continue;
            }
            if next_text.starts_with(syntax) {
                for _ in 0..len {
                    unparsed_chars_iter.next();
                }
                if in_syntax {
                    // If at syntax string not directly before another syntax string
                    // (i.e. not **** but **a**)
                    if start + len == i {
                        continue;
                    }
                    end = i + len;
                    self.format_spans.push(FormatSpan {
                        range: start..end,
                        tag,
                    });
                    for i in 0..len {
                        self.parsed_chars.push(start + i);
                        self.parsed_chars.push(end - i - 1);
                    }
                } else {
                    start = i;
                }
                in_syntax = !in_syntax;
            } else if next_text.starts_with(&format!("\\{syntax}")) {
                // Skip next character if escaped
                self.parsed_chars.push(i);
                unparsed_chars_iter.next();
            }
        }
        self
    }

    fn strong(&mut self) -> &mut Self {
        self
            .inline("**", Tag::Strong)
            .inline("__", Tag::Strong)
    }

    fn strikethrough(&mut self) -> &mut Self {
        self.inline("~~", Tag::Strikethrough)
    }

    fn emphasis(&mut self) -> &mut Self {
        self
            .inline("*", Tag::Emphasis)
            .inline("_", Tag::Emphasis)
    }

    fn code(&mut self) -> &mut Self {
        self.inline("`", Tag::Code)
    }

    fn syntax(&mut self) -> &mut Self {
        for i in self.parsed_chars.iter() {
            self.format_spans.push(FormatSpan {
                range: *i..*i + 1,
                tag: Tag::Syntax,
            });
        }
        self
    }
}
