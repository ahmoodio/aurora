use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;

use crate::core::cache::{clear_screenshots_cache, save_settings};
use crate::core::models::{AurHelperKind, ThemeMode};
use crate::ui::{apply_theme, AppContext};

#[derive(Clone)]
pub struct SettingsPage {
    pub root: adw::PreferencesPage,
    theme_row: adw::ComboRow,
    helper_row: adw::ComboRow,
    noconfirm_row: adw::SwitchRow,
    clear_cache: gtk::Button,
    about_btn: gtk::Button,
}

impl SettingsPage {
    pub fn new() -> Self {
        let root = adw::PreferencesPage::new();
        root.set_hexpand(true);
        root.set_vexpand(true);

        let appearance_group = adw::PreferencesGroup::new();
        appearance_group.set_title("Appearance");
        let theme_labels = ThemeMode::all()
            .iter()
            .map(|theme| theme.label())
            .collect::<Vec<_>>();
        let theme_list = gtk::StringList::new(&theme_labels);
        let theme_row = adw::ComboRow::new();
        theme_row.set_title("Theme");
        theme_row.set_model(Some(&theme_list));
        appearance_group.add(&theme_row);

        let group = adw::PreferencesGroup::new();
        group.set_title("General");

        let list = gtk::StringList::new(&["yay", "paru"]);
        let helper_row = adw::ComboRow::new();
        helper_row.set_title("AUR Helper");
        helper_row.set_model(Some(&list));

        let noconfirm_row = adw::SwitchRow::new();
        noconfirm_row.set_title("Allow --noconfirm");
        noconfirm_row.set_subtitle("Never use without explicit user opt-in");

        let cache_group = adw::PreferencesGroup::new();
        cache_group.set_title("Cache");
        let clear_cache = gtk::Button::with_label("Clear screenshots cache");
        let cache_row = adw::ActionRow::new();
        cache_row.set_title("Screenshots");
        cache_row.add_suffix(&clear_cache);
        cache_row.set_activatable(false);

        let about_group = adw::PreferencesGroup::new();
        about_group.set_title("About");
        let about_btn = gtk::Button::with_label("About Aurora");
        let about_row = adw::ActionRow::new();
        about_row.set_title("About");
        about_row.add_suffix(&about_btn);
        about_row.set_activatable(false);
        about_group.add(&about_row);

        group.add(&helper_row);
        group.add(&noconfirm_row);
        cache_group.add(&cache_row);

        root.add(&appearance_group);
        root.add(&group);
        root.add(&cache_group);
        root.add(&about_group);

        Self {
            root,
            theme_row,
            helper_row,
            noconfirm_row,
            clear_cache,
            about_btn,
        }
    }

    pub fn bind(&self, ctx: AppContext) {
        let settings = ctx.settings.lock().unwrap().clone();
        self.theme_row.set_selected(settings.theme.to_index());
        match settings.aur_helper {
            AurHelperKind::Yay => self.helper_row.set_selected(0),
            AurHelperKind::Paru => self.helper_row.set_selected(1),
        }
        self.noconfirm_row.set_active(settings.allow_noconfirm);

        let ctx_clone = ctx.clone();
        self.theme_row
            .connect_selected_notify(move |row: &adw::ComboRow| {
                let selected = row.selected();
                let mut settings = ctx_clone.settings.lock().unwrap();
                settings.theme = ThemeMode::from_index(selected);
                apply_theme(settings.theme);
                let _ = save_settings(&settings);
            });

        let ctx_clone = ctx.clone();
        self.helper_row
            .connect_selected_notify(move |row: &adw::ComboRow| {
            let selected = row.selected();
            let mut settings = ctx_clone.settings.lock().unwrap();
            settings.aur_helper = if selected == 0 {
                AurHelperKind::Yay
            } else {
                AurHelperKind::Paru
            };
            let _ = save_settings(&settings);
        });

        let ctx_clone = ctx.clone();
        self.noconfirm_row.connect_active_notify(move |row| {
            let mut settings = ctx_clone.settings.lock().unwrap();
            settings.allow_noconfirm = row.is_active();
            let _ = save_settings(&settings);
        });

        self.clear_cache.connect_clicked(move |_| {
            let _ = clear_screenshots_cache();
        });

        self.about_btn.connect_clicked(move |_| {
            let about = adw::AboutWindow::new();
            about.set_application_name("Aurora");
            about.set_application_icon("io.github.ahmoodio.aurora");
            about.set_developer_name("ahmoodio");
            about.set_version(env!("CARGO_PKG_VERSION"));
            about.set_website("https://github.com/ahmoodio/yay-gui-manager");
            about.set_issue_url("https://github.com/ahmoodio/yay-gui-manager/issues");
            about.present();
        });
    }
}
