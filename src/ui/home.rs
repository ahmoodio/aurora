use gtk::prelude::*;

use crate::core::cache::find_logo_path;
use crate::ui::widgets::card;

#[derive(Clone)]
pub struct HomePage {
    pub root: gtk::Box,
}

impl HomePage {
    pub fn new() -> Self {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 12);
        root.set_margin_top(12);
        root.set_margin_bottom(12);
        root.set_margin_start(12);
        root.set_margin_end(12);
        root.set_hexpand(true);
        root.set_vexpand(true);

        let header = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        let logo = logo_image(64);
        let title = gtk::Label::new(Some("Aurora"));
        title.add_css_class("title-1");
        title.set_xalign(0.0);
        header.append(&logo);
        header.append(&title);
        root.append(&header);

        let category_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        for label in ["Graphics", "Development", "Media", "Games", "Utilities"] {
            let chip = gtk::Label::new(Some(label));
            chip.add_css_class("pill");
            category_row.append(&chip);
        }
        root.append(&category_row);

        let title = gtk::Label::new(Some("Featured"));
        title.add_css_class("title-2");
        title.set_xalign(0.0);
        root.append(&title);

        let featured = gtk::FlowBox::new();
        featured.set_valign(gtk::Align::Start);
        featured.set_min_children_per_line(2);
        featured.set_max_children_per_line(4);
        featured.set_column_spacing(12);
        featured.set_row_spacing(12);
        featured.set_homogeneous(true);
        featured.set_selection_mode(gtk::SelectionMode::None);
        root.append(&featured);

        let popular = gtk::Label::new(Some("Popular"));
        popular.add_css_class("title-2");
        popular.set_xalign(0.0);
        root.append(&popular);

        let popular_grid = gtk::FlowBox::new();
        popular_grid.set_valign(gtk::Align::Start);
        popular_grid.set_min_children_per_line(2);
        popular_grid.set_max_children_per_line(4);
        popular_grid.set_column_spacing(12);
        popular_grid.set_row_spacing(12);
        popular_grid.set_homogeneous(true);
        popular_grid.set_selection_mode(gtk::SelectionMode::None);
        root.append(&popular_grid);

        // Lightweight placeholders so the page doesn't look empty.
        let placeholder = card::build_card(
            &crate::core::models::PackageSummary {
                name: "Discover Apps".to_string(),
                summary: "Search and install applications from repo and AUR."
                    .to_string(),
                version: String::from("-"),
                source: crate::core::models::PackageSource::Repo,
                installed: false,
                origin: None,
            },
            || {},
            || {},
        );
        featured.insert(&placeholder, -1);

        Self { root }
    }
}

fn logo_image(size: i32) -> gtk::Image {
    if let Some(path) = find_logo_path() {
        let image = gtk::Image::from_file(path);
        image.set_pixel_size(size);
        return image;
    }
    let image = gtk::Image::from_icon_name("io.github.ahmoodio.aurora");
    image.set_pixel_size(size);
    image
}
