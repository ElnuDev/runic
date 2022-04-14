use gtk::glib;
use gtk::prelude::*;

mod markdown;
use markdown::{Renderer, Tag};

fn build_ui(application: &gtk::Application) {
    let glade_src = include_str!("runic.glade");
    let builder = gtk::Builder::new();
    builder.add_from_string(glade_src).unwrap();

    let window: gtk::ApplicationWindow = builder.object("window").unwrap();
    window.set_application(Some(application));

    let button: gtk::Button = builder.object("button").unwrap();
    let text_view: gtk::TextView = builder.object("text_view").unwrap();
    let buffer = text_view.buffer().unwrap();

    Tag::init_tags(&buffer);

    buffer.connect_changed(glib::clone!(@weak buffer => move |_| {
        let (start, end) = buffer.bounds();
        Renderer::from(buffer.text(&start, &end, true).unwrap().as_str()).display(&buffer);
    }));

    button.connect_clicked(move |_| {
        buffer.insert_at_cursor("ඞ");
    });

    window.show_all();
}

fn main() {
    let application = gtk::Application::new(Some("com.github.ElnuDev.runic"), Default::default());

    application.connect_activate(build_ui);

    application.run();
}
