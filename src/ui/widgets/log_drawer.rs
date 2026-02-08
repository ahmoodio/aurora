use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, gio};

#[derive(Clone)]
pub struct LogDrawer {
    root: gtk::Box,
    body_revealer: gtk::Revealer,
    buffer: gtk::TextBuffer,
    lines: Rc<RefCell<Vec<String>>>,
}

impl LogDrawer {
    pub fn new() -> Self {
        let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        header.set_margin_top(8);
        header.set_margin_bottom(8);
        header.set_margin_start(8);
        header.set_margin_end(8);

        let title = gtk::Label::new(Some("Logs"));
        title.add_css_class("title-4");
        title.set_xalign(0.0);

        let minimize_btn = gtk::Button::from_icon_name("pan-down-symbolic");
        let close_btn = gtk::Button::from_icon_name("window-close-symbolic");
        let copy_btn = gtk::Button::with_label("Copy");
        let save_btn = gtk::Button::with_label("Save");
        let clear_btn = gtk::Button::with_label("Clear");

        header.append(&title);
        header.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        header.append(&minimize_btn);
        header.append(&close_btn);
        header.append(&copy_btn);
        header.append(&save_btn);
        header.append(&clear_btn);

        let text_view = gtk::TextView::new();
        text_view.set_editable(false);
        text_view.set_cursor_visible(false);
        text_view.set_monospace(true);

        let buffer = text_view.buffer();

        let scroller = gtk::ScrolledWindow::new();
        scroller.set_vexpand(true);
        scroller.set_child(Some(&text_view));
        scroller.set_min_content_height(160);

        let body = gtk::Box::new(gtk::Orientation::Vertical, 0);
        body.append(&scroller);

        let body_revealer = gtk::Revealer::new();
        body_revealer.set_transition_type(gtk::RevealerTransitionType::SlideUp);
        body_revealer.set_reveal_child(true);
        body_revealer.set_child(Some(&body));

        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root.append(&header);
        root.append(&body_revealer);
        root.set_visible(false);

        let lines = Rc::new(RefCell::new(Vec::new()));
        let lines_copy = lines.clone();
        copy_btn.connect_clicked(move |_| {
            let text = lines_copy.borrow().join("\n");
            if let Some(display) = gdk::Display::default() {
                let clipboard = display.clipboard();
                clipboard.set_text(&text);
            }
        });

        let lines_save = lines.clone();
        save_btn.connect_clicked(move |_| {
            let dialog = gtk::FileDialog::new();
            dialog.set_title("Save Logs");
            let text = lines_save.borrow().join("\n");
            dialog.save(None::<&gtk::Window>, gio::Cancellable::NONE, move |res| {
                if let Ok(file) = res {
                    if let Some(path) = file.path() {
                        let _ = std::fs::write(path, text);
                    }
                }
            });
        });

        let lines_clear = lines.clone();
        let buffer_clear = buffer.clone();
        clear_btn.connect_clicked(move |_| {
            lines_clear.borrow_mut().clear();
            buffer_clear.set_text("");
        });

        let body_toggle = body_revealer.clone();
        minimize_btn.connect_clicked(move |_| {
            let next = !body_toggle.reveals_child();
            body_toggle.set_reveal_child(next);
        });

        let root_hide = root.clone();
        close_btn.connect_clicked(move |_| {
            root_hide.set_visible(false);
        });

        Self {
            root,
            body_revealer,
            buffer,
            lines,
        }
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.root
    }

    pub fn append_line(&self, line: &str, limit: usize) {
        let mut lines = self.lines.borrow_mut();
        lines.push(line.to_string());
        while lines.len() > limit {
            lines.remove(0);
        }
        self.buffer.set_text(&lines.join("\n"));
    }

    pub fn clear(&self) {
        self.lines.borrow_mut().clear();
        self.buffer.set_text("");
    }

    pub fn set_visible(&self, visible: bool) {
        self.root.set_visible(visible);
        if visible {
            self.body_revealer.set_reveal_child(true);
        }
    }

    pub fn is_visible(&self) -> bool {
        self.root.is_visible()
    }
}
