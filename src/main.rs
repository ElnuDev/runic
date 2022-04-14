use gtk::prelude::*;
use gtk::glib;
use gtk::pango;

fn build_ui(application: &gtk::Application) {
    let glade_src = include_str!("runic.glade");
    let builder = gtk::Builder::new();
    builder.add_from_string(glade_src).unwrap();

    let window: gtk::ApplicationWindow = builder.object("window").unwrap();
    window.set_application(Some(application));

    let button: gtk::Button = builder.object("button").unwrap();
    let text_view: gtk::TextView = builder.object("text_view").unwrap();
    let buffer = text_view.buffer().unwrap();

    let italics_text_tag = gtk::TextTag::new(Some("bold"));
    italics_text_tag.set_property("style", pango::Style::Italic);
    buffer.tag_table().unwrap().add(&italics_text_tag);

    buffer.connect_changed(glib::clone!(@weak buffer => move |_| {
        let text = {
            let (start, end) = buffer.bounds();
            buffer.remove_all_tags(&start, &end);
            String::from(buffer.text(&start, &end, false).unwrap().as_str())
        };
        let mut in_italics = false;
        let mut start = buffer.iter_at_offset(0);
        let mut end;
        for (i, char) in text.chars().enumerate() {
            let i = i as i32;
            if char == '*' {
                if in_italics {
                    end = buffer.iter_at_offset(i);
                    buffer.apply_tag(&italics_text_tag, &start, &end);
                } else {
                    start = buffer.iter_at_offset(i);
                }
                in_italics = !in_italics;
            }
        }
    }));

    // TODO: What does glib::clone! and @weak do, exactly?
    button.connect_clicked(move |_| {
        buffer.insert_at_cursor("ඞ");
    });

    window.show_all();
}

fn main() {
    let application = gtk::Application::new(
        Some("com.github.ElnuDev.runic"),
        Default::default()
    );

    application.connect_activate(build_ui);

    application.run();
}