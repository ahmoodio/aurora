use gtk::prelude::*;
use gtk::pango;

use crate::core::models::PackageSummary;

pub fn build_card<F, G>(pkg: &PackageSummary, on_action: F, on_details: G) -> gtk::Box
where
    F: Fn() + 'static,
    G: Fn() + 'static,
{
    let root = gtk::Box::new(gtk::Orientation::Vertical, 8);
    root.add_css_class("card");
    root.set_margin_top(12);
    root.set_margin_bottom(12);
    root.set_margin_start(12);
    root.set_margin_end(12);
    root.set_size_request(300, -1);
    root.set_hexpand(false);
    root.set_vexpand(false);
    root.set_halign(gtk::Align::Start);

    let icon = gtk::Image::from_icon_name("application-x-executable");
    icon.set_pixel_size(64);
    icon.set_halign(gtk::Align::Center);

    let name = gtk::Label::new(Some(&pkg.name));
    name.add_css_class("title-4");
    name.set_xalign(0.0);
    name.set_wrap(true);
    name.set_wrap_mode(pango::WrapMode::WordChar);
    name.set_lines(1);
    name.set_ellipsize(pango::EllipsizeMode::End);
    name.set_max_width_chars(28);

    let summary = gtk::Label::new(Some(&pkg.summary));
    summary.set_xalign(0.0);
    summary.set_wrap(true);
    summary.set_lines(2);
    summary.set_wrap_mode(pango::WrapMode::WordChar);
    summary.set_ellipsize(pango::EllipsizeMode::End);
    summary.set_max_width_chars(36);

    let badge = gtk::Label::new(Some(match pkg.source {
        crate::core::models::PackageSource::Repo => "Repo",
        crate::core::models::PackageSource::Aur => "AUR",
        crate::core::models::PackageSource::Flatpak => "Flatpak",
    }));
    badge.add_css_class("pill");
    badge.set_xalign(0.0);

    let button = gtk::Button::with_label("Install");
    button.add_css_class("suggested-action");
    button.connect_clicked(move |_| on_action());

    root.append(&icon);
    root.append(&name);
    root.append(&summary);
    root.append(&badge);
    root.append(&button);

    let gesture = gtk::GestureClick::new();
    gesture.connect_pressed(move |_, _, _, _| {
        on_details();
    });
    root.add_controller(gesture);

    root
}
