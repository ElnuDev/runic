use gtk::pango;
use gtk::prelude::*;
use gtk::gdk::RGBA;

use std::ops::Range;
use strum::EnumIter;
use strum::IntoEnumIterator;

struct FormatSpan {
    range: Range<usize>,
    tag: Tag,
}

#[derive(Clone, Copy, EnumIter)]
pub enum Tag {
    Italics,
    Bold,
    Strikethrough,
    Code,
    Syntax,
}

impl Tag {
    fn to_string(&self) -> String {
        String::from(match self {
            Self::Italics => "i",
            Self::Bold => "b",
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
                Self::Italics => {
                    text_tag.set_property("style", pango::Style::Italic);
                },
                Self::Bold => {
                    text_tag.set_property::<i32>("weight", 700);
                },
                Self::Strikethrough => {
                    text_tag.set_property("strikethrough", true);
                },
                Self::Code => {
                    text_tag.set_property("font", "Ubuntu Mono");
                }
                Self::Syntax => {
                    text_tag.set_property("foreground-rgba", RGBA::new(0.5, 0.5, 0.5, 1.0));
                }
            };
            tag_table.add(&*text_tag);
        }
    }
}

#[derive(Default)]
pub struct Renderer {
    format_spans: Vec<FormatSpan>,
    parsed_chars: Vec<usize>,
    pub text: String,
}

impl Renderer {
    pub fn from(text: &str) -> Self {
        Self {
            text: String::from(text),
            ..Default::default()
        }
    }

    fn unparsed_chars(&self) -> Vec<(usize, char)> {
        self.text
            .chars()
            .enumerate()
            .filter(|&(i, _)| !self.parsed_chars.contains(&i))
            .collect()
    }

    fn render(&mut self) {
        self
            .bold()
            .italics()
            .strikethrough()
            .code()
            .syntax();
    }

    pub fn display(&mut self, buffer: &gtk::TextBuffer) {
        self.render();
        let (start, end) = buffer.bounds();
        buffer.remove_all_tags(&start, &end);
        for format_span in self.format_spans.iter() {
            buffer.apply_tag_by_name(
                &format_span.tag.to_string(),
                &buffer.iter_at_offset(format_span.range.start as i32),
                &buffer.iter_at_offset(format_span.range.end as i32),
            );
        }
    }

    fn two_chr(&mut self, syntax_chr: char, tag: Tag) -> &mut Self {
        let mut in_syntax = false;
        let mut potential_syntax = false;
        let mut start = 0;
        let mut end;
        for (i, chr) in self.unparsed_chars().iter() {
            if *chr != syntax_chr {
                potential_syntax = false;
                continue;
            }
            if potential_syntax {
                if in_syntax {
                    end = *i + 1;
                    self.format_spans.push(FormatSpan {
                        range: start..end,
                        tag,
                    });
                    self.parsed_chars.push(start);
                    self.parsed_chars.push(start + 1);
                    self.parsed_chars.push(end - 2);
                    self.parsed_chars.push(end - 1);
                } else {
                    start = *i - 1;
                }
                potential_syntax = false;
                in_syntax = !in_syntax;
            } else {
                potential_syntax = true;
            }
        }
        self
    }

    fn bold(&mut self) -> &mut Self {
        self
            .two_chr('*', Tag::Bold)
            .two_chr('_', Tag::Bold)
    }

    fn strikethrough(&mut self) -> &mut Self {
        self.two_chr('~', Tag::Strikethrough)
    }

    fn one_chr(&mut self, syntax_chr: char, tag: Tag) -> &mut Self {
        let mut in_syntax = false;
        let mut start = 0;
        let mut end;
        for (i, chr) in self.unparsed_chars().iter() {
            if *chr != syntax_chr {
                continue;
            }
            if in_syntax {
                end = *i + 1;
                self.format_spans.push(FormatSpan {
                    range: start..end,
                    tag,
                });
                self.parsed_chars.push(start);
                self.parsed_chars.push(*i);
            } else {
                start = *i;
            }
            in_syntax = !in_syntax;
        }
        self
    }

    fn italics(&mut self) -> &mut Self {
        self
            .one_chr('*', Tag::Italics)
            .one_chr('_', Tag::Italics)
    }

    fn code(&mut self) -> &mut Self {
        self.one_chr('`', Tag::Code)
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
