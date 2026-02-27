use std::cell::RefCell;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, gio};

const DEFAULT_LOG_LIMIT: usize = 1000;
const DEFAULT_LOG_HEIGHT: i32 = 220;
const MIN_LOG_HEIGHT: i32 = 72;
const MAX_LOG_HEIGHT: i32 = 900;
const LOG_HEADER_HEIGHT: i32 = 56;

#[derive(Clone)]
pub struct LogDrawer {
    root: gtk::Box,
    scroller: gtk::ScrolledWindow,
    buffer: gtk::TextBuffer,
    text_view: gtk::TextView,
    lines: Rc<RefCell<Vec<String>>>,
    min_height: Rc<RefCell<i32>>,
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
        let clear_lock_btn = gtk::Button::with_label("Clear Lock");
        let resize_btn = gtk::Button::with_label("Resize");
        let shorter_btn = gtk::Button::with_label("Shorter");
        let taller_btn = gtk::Button::with_label("Taller");

        header.append(&title);
        header.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        header.append(&minimize_btn);
        header.append(&close_btn);
        header.append(&copy_btn);
        header.append(&save_btn);
        header.append(&clear_btn);
        header.append(&clear_lock_btn);
        header.append(&resize_btn);
        header.append(&shorter_btn);
        header.append(&taller_btn);

        let text_view = gtk::TextView::new();
        text_view.set_editable(false);
        text_view.set_cursor_visible(false);
        text_view.set_monospace(true);

        let buffer = text_view.buffer();

        let scroller = gtk::ScrolledWindow::new();
        scroller.set_vexpand(true);
        scroller.set_child(Some(&text_view));
        scroller.set_min_content_height(DEFAULT_LOG_HEIGHT);

        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root.append(&header);
        root.append(&scroller);
        root.set_height_request(DEFAULT_LOG_HEIGHT + LOG_HEADER_HEIGHT);
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

        let lines_lock = lines.clone();
        let buffer_lock = buffer.clone();
        let text_view_lock = text_view.clone();
        clear_lock_btn.connect_clicked(move |_| {
            Self::append_line_internal(
                &lines_lock,
                &buffer_lock,
                &text_view_lock,
                "Checking for active package managers before lock cleanup...",
                DEFAULT_LOG_LIMIT,
            );

            let running = match Self::running_package_managers() {
                Ok(running) => running,
                Err(err) => {
                    Self::append_line_internal(
                        &lines_lock,
                        &buffer_lock,
                        &text_view_lock,
                        &format!("Safety check failed: {err}"),
                        DEFAULT_LOG_LIMIT,
                    );
                    return;
                }
            };

            if !running.is_empty() {
                Self::append_line_internal(
                    &lines_lock,
                    &buffer_lock,
                    &text_view_lock,
                    &format!(
                        "Refusing to clear pacman lock because these processes are active: {}",
                        running.join(", ")
                    ),
                    DEFAULT_LOG_LIMIT,
                );
                return;
            }

            Self::append_line_internal(
                &lines_lock,
                &buffer_lock,
                &text_view_lock,
                "No active package manager found. Requesting authentication...",
                DEFAULT_LOG_LIMIT,
            );

            match Self::clear_stale_pacman_lock() {
                Ok(message) => {
                    Self::append_line_internal(
                        &lines_lock,
                        &buffer_lock,
                        &text_view_lock,
                        &message,
                        DEFAULT_LOG_LIMIT,
                    );
                }
                Err(err) => {
                    Self::append_line_internal(
                        &lines_lock,
                        &buffer_lock,
                        &text_view_lock,
                        &format!("Failed to clear pacman lock: {err}"),
                        DEFAULT_LOG_LIMIT,
                    );
                }
            }
        });

        let min_height = Rc::new(RefCell::new(DEFAULT_LOG_HEIGHT));
        let expanded_height = Rc::new(RefCell::new(DEFAULT_LOG_HEIGHT));
        let minimized = Rc::new(RefCell::new(false));

        let min_height_shorter = min_height.clone();
        let expanded_height_shorter = expanded_height.clone();
        let scroller_shorter = scroller.clone();
        let root_shorter = root.clone();
        shorter_btn.connect_clicked(move |_| {
            let mut height = min_height_shorter.borrow_mut();
            *height = (*height - 40).max(MIN_LOG_HEIGHT);
            scroller_shorter.set_min_content_height(*height);
            root_shorter.set_height_request(*height + LOG_HEADER_HEIGHT);
            *expanded_height_shorter.borrow_mut() = *height;
        });

        let min_height_taller = min_height.clone();
        let expanded_height_taller = expanded_height.clone();
        let scroller_taller = scroller.clone();
        let root_taller = root.clone();
        taller_btn.connect_clicked(move |_| {
            let mut height = min_height_taller.borrow_mut();
            *height = (*height + 40).min(MAX_LOG_HEIGHT);
            scroller_taller.set_min_content_height(*height);
            root_taller.set_height_request(*height + LOG_HEADER_HEIGHT);
            *expanded_height_taller.borrow_mut() = *height;
        });

        let drag_start_height = Rc::new(RefCell::new(DEFAULT_LOG_HEIGHT));
        let min_height_drag = min_height.clone();
        let expanded_height_drag = expanded_height.clone();
        let minimized_drag = minimized.clone();
        let scroller_drag = scroller.clone();
        let root_drag = root.clone();
        let minimize_btn_drag = minimize_btn.clone();
        let drag_start_height_begin = drag_start_height.clone();
        let drag = gtk::GestureDrag::new();
        drag.connect_drag_begin(move |_, _, _| {
            *drag_start_height_begin.borrow_mut() = *min_height_drag.borrow();
        });
        let min_height_drag_update = min_height.clone();
        let expanded_height_drag_update = expanded_height_drag.clone();
        let minimized_drag_update = minimized_drag.clone();
        let scroller_drag_update = scroller_drag.clone();
        let root_drag_update = root_drag.clone();
        let minimize_btn_drag_update = minimize_btn_drag.clone();
        drag.connect_drag_update(move |_, _, dy| {
            let start_height = *drag_start_height.borrow();
            let next = (start_height - dy as i32).clamp(MIN_LOG_HEIGHT, MAX_LOG_HEIGHT);
            *min_height_drag_update.borrow_mut() = next;
            scroller_drag_update.set_min_content_height(next);
            root_drag_update.set_height_request(next + LOG_HEADER_HEIGHT);
            if next > MIN_LOG_HEIGHT {
                *expanded_height_drag_update.borrow_mut() = next;
                *minimized_drag_update.borrow_mut() = false;
                minimize_btn_drag_update.set_icon_name("pan-down-symbolic");
            }
        });
        resize_btn.add_controller(drag);

        let min_height_toggle = min_height.clone();
        let expanded_height_toggle = expanded_height.clone();
        let minimized_toggle = minimized.clone();
        let scroller_toggle = scroller.clone();
        let root_toggle = root.clone();
        let minimize_btn_toggle = minimize_btn.clone();
        minimize_btn.connect_clicked(move |_| {
            let mut is_minimized = minimized_toggle.borrow_mut();
            if *is_minimized {
                let restore = *expanded_height_toggle.borrow();
                *min_height_toggle.borrow_mut() = restore;
                scroller_toggle.set_min_content_height(restore);
                root_toggle.set_height_request(restore + LOG_HEADER_HEIGHT);
                minimize_btn_toggle.set_icon_name("pan-down-symbolic");
                *is_minimized = false;
            } else {
                let current = *min_height_toggle.borrow();
                *expanded_height_toggle.borrow_mut() = current;
                *min_height_toggle.borrow_mut() = MIN_LOG_HEIGHT;
                scroller_toggle.set_min_content_height(MIN_LOG_HEIGHT);
                root_toggle.set_height_request(MIN_LOG_HEIGHT + LOG_HEADER_HEIGHT);
                minimize_btn_toggle.set_icon_name("pan-up-symbolic");
                *is_minimized = true;
            }
        });

        let root_hide = root.clone();
        close_btn.connect_clicked(move |_| {
            root_hide.set_visible(false);
        });

        Self {
            root,
            scroller,
            buffer,
            text_view,
            lines,
            min_height,
        }
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.root
    }

    pub fn append_line(&self, line: &str, limit: usize) {
        Self::append_line_internal(&self.lines, &self.buffer, &self.text_view, line, limit);
    }

    pub fn clear(&self) {
        self.lines.borrow_mut().clear();
        self.buffer.set_text("");
    }

    pub fn set_visible(&self, visible: bool) {
        self.root.set_visible(visible);
        if visible {
            let height = *self.min_height.borrow();
            self.scroller.set_min_content_height(height);
            self.root.set_height_request(height + LOG_HEADER_HEIGHT);
            self.scroll_to_bottom();
        }
    }

    pub fn is_visible(&self) -> bool {
        self.root.is_visible()
    }

    fn scroll_to_bottom(&self) {
        Self::scroll_to_bottom_internal(&self.buffer, &self.text_view);
    }

    fn append_line_internal(
        lines: &Rc<RefCell<Vec<String>>>,
        buffer: &gtk::TextBuffer,
        text_view: &gtk::TextView,
        line: &str,
        limit: usize,
    ) {
        let mut lines = lines.borrow_mut();
        lines.push(line.to_string());
        while lines.len() > limit {
            lines.remove(0);
        }
        buffer.set_text(&lines.join("\n"));
        Self::scroll_to_bottom_internal(buffer, text_view);
    }

    fn scroll_to_bottom_internal(buffer: &gtk::TextBuffer, text_view: &gtk::TextView) {
        let mut end = buffer.end_iter();
        buffer.place_cursor(&end);
        text_view.scroll_to_iter(&mut end, 0.0, false, 0.0, 1.0);
    }

    fn running_package_managers() -> Result<Vec<String>, String> {
        let mut running = Vec::new();
        let candidates = ["pacman", "yay", "paru", "pamac", "pkcon", "packagekitd"];

        for proc_name in candidates {
            match Command::new("pgrep").arg("-x").arg(proc_name).status() {
                Ok(status) if status.success() => running.push(proc_name.to_string()),
                Ok(_) => {}
                Err(err) => return Err(format!("failed to run pgrep: {err}")),
            }
        }

        Ok(running)
    }

    fn clear_stale_pacman_lock() -> Result<String, String> {
        let helper = Self::helper_path();
        let output = Command::new("pkexec")
            .arg(&helper)
            .arg("clear-pacman-lock")
            .output()
            .map_err(|err| format!("failed to run pkexec: {err}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

        if output.status.success() {
            if stdout.is_empty() {
                Ok("Pacman lock cleanup completed.".to_string())
            } else {
                Ok(stdout)
            }
        } else if !stderr.is_empty() {
            Err(stderr)
        } else if !stdout.is_empty() {
            Err(stdout)
        } else {
            Err(format!("command failed with status {}", output.status))
        }
    }

    fn helper_path() -> String {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let mut candidate = PathBuf::from(dir);
                candidate.push("aurora-helper");
                if candidate.exists() {
                    return candidate.to_string_lossy().to_string();
                }
            }
        }
        "/usr/bin/aurora-helper".to_string()
    }
}
