use gtk::prelude::*;
use gtk::pango;

use std::ops::Range;
use strum::EnumIter;
use strum::IntoEnumIterator;

struct FormatSpan {
	range: Range<usize>,
	tag: Tag,
}

#[derive(EnumIter)]
pub enum Tag {
	Italics,
	Bold
}

impl Tag {
	fn to_string(&self) -> String {
		String::from(match self {
			Self::Italics => "i",
			Self::Bold => "b"
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
				}
			}
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
			.italics();
	}

	pub fn display(&mut self, buffer: &gtk::TextBuffer) {
		self.render();
		for format_span in self.format_spans.iter() {
			buffer.apply_tag_by_name(
				&format_span.tag.to_string(),
				&buffer.iter_at_offset(format_span.range.start as i32),
				&buffer.iter_at_offset(format_span.range.end as i32)
			);
		}
	}
	
	fn bold(&mut self) -> &mut Self {
		self
	}

	fn italics(&mut self) -> &mut Self {
		let mut in_italics = false;
		let mut start = 0;
		let mut end;
		for (i, chr) in self.unparsed_chars().iter() {
			if *chr != '*' {
				continue;
			}
			if in_italics {
				end = *i;
				self.format_spans.push(FormatSpan {
					range: start..end,
					tag: Tag::Italics
				});
			} else {
				start = *i;
			}
			self.parsed_chars.push(*i);
			in_italics = !in_italics;
		}
		self
	}
}
