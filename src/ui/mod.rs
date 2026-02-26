use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use glib::clone;
use glib::ControlFlow;
use gtk::prelude::*;
use gtk::gdk;
use libadwaita as adw;
use adw::prelude::*;

use crate::core::appstream::AppStreamClient;
use crate::core::cache::{ensure_cache_dirs, load_settings};
use crate::core::models::{ActionKind, PackageSource, Settings, ThemeMode, TransactionAction, TransactionQueue};
use crate::core::providers::aur::Aur;
use crate::core::providers::flatpak::Flatpak;
use crate::core::providers::pacman::Pacman;
use crate::core::providers::{AurProvider, FlatpakProvider, PacmanProvider};
use crate::core::runner::{CommandRunner, LogEvent};
use crate::core::transactions::{plan_transactions, TransactionPlan};

pub mod details;
pub mod home;
pub mod installed;
pub mod search;
pub mod settings;
pub mod updates;
pub mod widgets;

#[derive(Clone)]
pub struct AppContext {
    pub pacman: Arc<dyn PacmanProvider>,
    pub aur: Arc<dyn AurProvider>,
    pub flatpak: Arc<dyn FlatpakProvider>,
    pub appstream: Arc<AppStreamClient>,
    pub settings: Arc<Mutex<Settings>>,
    pub queue: Arc<Mutex<TransactionQueue>>,
    pub runner: Arc<CommandRunner>,
}

#[derive(Clone)]
pub struct UiHandles {
    pub nav_view: adw::NavigationView,
    pub log_drawer: widgets::log_drawer::LogDrawer,
    pub queue: QueueController,
    pub toasts: adw::ToastOverlay,
}

#[derive(Clone)]
pub struct QueueController {
    ctx: AppContext,
    button: gtk::Button,
    log_drawer: widgets::log_drawer::LogDrawer,
    parent: adw::ApplicationWindow,
    toasts: adw::ToastOverlay,
}

impl QueueController {
    pub fn new(
        ctx: AppContext,
        button: gtk::Button,
        log_drawer: widgets::log_drawer::LogDrawer,
        parent: adw::ApplicationWindow,
        toasts: adw::ToastOverlay,
    ) -> Self {
        Self {
            ctx,
            button,
            log_drawer,
            parent,
            toasts,
        }
    }

    fn update_label(&self) {
        let len = self.ctx.queue.lock().unwrap().len();
        self.button.set_label(&format!("Queue ({len})"));
    }

    fn toast(&self, message: &str) {
        let toast = adw::Toast::new(message);
        self.toasts.add_toast(toast);
    }

    pub fn add_install(&self, name: String, source: PackageSource, origin: Option<String>) {
        let mut queue = self.ctx.queue.lock().unwrap();
        queue.push(TransactionAction {
            name,
            source,
            kind: ActionKind::Install,
            origin,
        });
        drop(queue);
        self.update_label();
        self.toast("Added to queue");
    }

    pub fn add_remove(&self, name: String, source: PackageSource) {
        let mut queue = self.ctx.queue.lock().unwrap();
        queue.push(TransactionAction {
            name,
            source,
            kind: ActionKind::Remove,
            origin: None,
        });
        drop(queue);
        self.update_label();
        self.toast("Added to queue");
    }

    pub fn add_upgrade_repo(&self) {
        let mut queue = self.ctx.queue.lock().unwrap();
        queue.push(TransactionAction {
            name: String::from("system"),
            source: PackageSource::Repo,
            kind: ActionKind::Upgrade,
            origin: None,
        });
        drop(queue);
        self.update_label();
        self.toast("System upgrade queued");
    }

    pub fn add_upgrade_packages(&self, actions: Vec<TransactionAction>) {
        if actions.is_empty() {
            self.toast("No packages selected");
            return;
        }
        let mut queue = self.ctx.queue.lock().unwrap();
        for action in actions {
            queue.push(action);
        }
        drop(queue);
        self.update_label();
        self.toast("Selected updates queued");
    }

    pub fn add_upgrade_all(&self) {
        let mut queue = self.ctx.queue.lock().unwrap();
        queue.push(TransactionAction {
            name: String::from("system"),
            source: PackageSource::Repo,
            kind: ActionKind::Upgrade,
            origin: None,
        });
        queue.push(TransactionAction {
            name: String::from("aur"),
            source: PackageSource::Aur,
            kind: ActionKind::Upgrade,
            origin: None,
        });
        queue.push(TransactionAction {
            name: String::from("flatpak"),
            source: PackageSource::Flatpak,
            kind: ActionKind::Upgrade,
            origin: None,
        });
        drop(queue);
        self.update_label();
        self.toast("All updates queued");
    }

    pub fn show_review_dialog(&self) {
        let queue = self.ctx.queue.lock().unwrap().clone();
        if queue.is_empty() {
            let dialog = adw::MessageDialog::new(
                Some(&self.parent),
                Some("Queue is empty"),
                Some("Add install/remove actions first."),
            );
            dialog.add_response("ok", "OK");
            dialog.connect_response(None, |d: &adw::MessageDialog, _| d.close());
            dialog.present();
            return;
        }

        let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
        for action in &queue.actions {
            let row = gtk::Label::new(Some(&format!(
                "{:?} {} ({:?})",
                action.kind, action.name, action.source
            )));
            row.set_xalign(0.0);
            content.append(&row);
        }

        let scroller = gtk::ScrolledWindow::new();
        scroller.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        scroller.set_min_content_height(180);
        scroller.set_max_content_height(420);
        scroller.set_child(Some(&content));

        let dialog = adw::MessageDialog::new(
            Some(&self.parent),
            Some("Review Transactions"),
            Some("Confirm before executing."),
        );
        dialog.set_extra_child(Some(&scroller));
        dialog.add_response("cancel", "Cancel");
        dialog.add_response("execute", "Execute");
        dialog.set_response_appearance("execute", adw::ResponseAppearance::Suggested);

        let ctx = self.ctx.clone();
        let log_drawer = self.log_drawer.clone();
        let parent = self.parent.clone();
        let button = self.button.clone();
        let toasts = self.toasts.clone();

        dialog.connect_response(None, move |d: &adw::MessageDialog, resp| {
            if resp == "execute" {
                let plan = plan_transactions(&queue, &ctx.settings.lock().unwrap());
                run_plan(plan, &ctx, &log_drawer, &parent, &toasts);
                ctx.queue.lock().unwrap().clear();
                button.set_label("Queue (0)");
            }
            d.close();
        });
        dialog.present();
    }
}

pub fn build_ui(app: &adw::Application) {
    let _ = ensure_cache_dirs();

    let settings = load_settings();
    apply_theme(settings.theme);
    let settings_arc = Arc::new(Mutex::new(settings));
    let ctx = AppContext {
        pacman: Arc::new(Pacman::default()),
        aur: Arc::new(Aur::new(settings_arc.clone())),
        flatpak: Arc::new(Flatpak::default()),
        appstream: Arc::new(AppStreamClient::default()),
        settings: settings_arc,
        queue: Arc::new(Mutex::new(TransactionQueue::default())),
        runner: Arc::new(CommandRunner::default()),
    };

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Aurora")
        .default_width(1200)
        .default_height(800)
        .build();

    if let Some(display) = gdk::Display::default() {
        let icon_theme = gtk::IconTheme::for_display(&display);
        icon_theme.add_search_path("assets/icons");
    }
    window.set_icon_name(Some("io.github.ahmoodio.aurora"));

    let header = adw::HeaderBar::new();

    let queue_button = gtk::Button::with_label("Queue (0)");
    header.pack_end(&queue_button);

    let sidebar = gtk::ListBox::new();
    sidebar.add_css_class("navigation-sidebar");

    let home_row = gtk::Label::new(Some("Home"));
    let search_row = gtk::Label::new(Some("Search"));
    let installed_row = gtk::Label::new(Some("Installed"));
    let updates_row = gtk::Label::new(Some("Updates"));
    let settings_row = gtk::Label::new(Some("Settings"));

    sidebar.append(&home_row);
    sidebar.append(&search_row);
    sidebar.append(&installed_row);
    sidebar.append(&updates_row);
    sidebar.append(&settings_row);

    let stack = gtk::Stack::new();
    stack.set_hexpand(true);
    stack.set_vexpand(true);

    let home_page = home::HomePage::new();
    let search_page = search::SearchPage::new();
    let installed_page = installed::InstalledPage::new();
    let updates_page = updates::UpdatesPage::new();
    let settings_page = settings::SettingsPage::new();

    stack.add_named(&home_page.root, Some("home"));
    stack.add_named(&search_page.root, Some("search"));
    stack.add_named(&installed_page.root, Some("installed"));
    stack.add_named(&updates_page.root, Some("updates"));
    stack.add_named(&settings_page.root, Some("settings"));
    stack.set_visible_child_name("home");

    let nav_view = adw::NavigationView::new();
    let main_page = adw::NavigationPage::builder()
        .title("Aurora")
        .child(&stack)
        .build();
    nav_view.push(&main_page);

    let split = adw::NavigationSplitView::new();
    split.set_hexpand(true);
    split.set_vexpand(true);
    let sidebar_page = adw::NavigationPage::builder()
        .title("Navigation")
        .child(&sidebar)
        .build();
    let content_page = adw::NavigationPage::builder()
        .title("Content")
        .child(&nav_view)
        .build();
    split.set_sidebar(Some(&sidebar_page));
    split.set_content(Some(&content_page));

    let log_drawer = widgets::log_drawer::LogDrawer::new();
    let toast_overlay = adw::ToastOverlay::new();

    let queue_controller = QueueController::new(
        ctx.clone(),
        queue_button.clone(),
        log_drawer.clone(),
        window.clone(),
        toast_overlay.clone(),
    );

    let handles = UiHandles {
        nav_view: nav_view.clone(),
        log_drawer: log_drawer.clone(),
        queue: queue_controller.clone(),
        toasts: toast_overlay.clone(),
    };

    let toolbar_view = adw::ToolbarView::new();
    toolbar_view.add_top_bar(&header);
    toolbar_view.set_content(Some(&split));
    toolbar_view.set_vexpand(true);

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root.set_hexpand(true);
    root.set_vexpand(true);
    root.append(&toolbar_view);
    root.append(log_drawer.widget());

    toast_overlay.set_child(Some(&root));
    toast_overlay.set_hexpand(true);
    toast_overlay.set_vexpand(true);
    window.set_content(Some(&toast_overlay));

    setup_css();

    let stack_for_home_search = stack.clone();
    home_page.open_search_btn.connect_clicked(move |_| {
        stack_for_home_search.set_visible_child_name("search");
    });

    let stack_for_home_updates = stack.clone();
    let updates_for_home = updates_page.clone();
    let ctx_for_home_updates = ctx.clone();
    let toasts_for_home_updates = handles.toasts.clone();
    home_page.open_updates_btn.connect_clicked(move |_| {
        stack_for_home_updates.set_visible_child_name("updates");
        updates_for_home.refresh(
            ctx_for_home_updates.clone(),
            Some(toasts_for_home_updates.clone()),
        );
    });

    let stack_for_home_installed = stack.clone();
    let installed_for_home = installed_page.clone();
    let ctx_for_home_installed = ctx.clone();
    let handles_for_home_installed = handles.clone();
    home_page.open_installed_btn.connect_clicked(move |_| {
        stack_for_home_installed.set_visible_child_name("installed");
        installed_for_home.refresh(
            ctx_for_home_installed.clone(),
            handles_for_home_installed.clone(),
        );
    });


    queue_button.connect_clicked(clone!(@strong queue_controller => move |_| {
        queue_controller.show_review_dialog();
    }));

    let ctx_for_sidebar = ctx.clone();
    let stack_for_sidebar = stack.clone();
    let handles_for_sidebar = handles.clone();
    let nav_for_sidebar = nav_view.clone();
    let main_page_for_sidebar = main_page.clone();
    sidebar.connect_row_selected(move |_, row| {
        if let Some(row) = row {
            let _ = nav_for_sidebar.pop_to_page(&main_page_for_sidebar);
            let index = row.index();
            match index {
                0 => stack_for_sidebar.set_visible_child_name("home"),
                1 => stack_for_sidebar.set_visible_child_name("search"),
                2 => {
                    stack_for_sidebar.set_visible_child_name("installed");
                    installed_page.refresh(ctx_for_sidebar.clone(), handles_for_sidebar.clone());
                }
                3 => stack_for_sidebar.set_visible_child_name("updates"),
                4 => stack_for_sidebar.set_visible_child_name("settings"),
                _ => {}
            }
        }
    });

    updates_page.connect_apply_all(clone!(@strong queue_controller => move || {
        queue_controller.add_upgrade_all();
    }));

    updates_page.connect_apply_selected(clone!(@strong queue_controller => move |actions| {
        queue_controller.add_upgrade_packages(actions);
    }));

    updates_page.bind(ctx.clone());
    settings_page.bind(ctx.clone());
    search_page.bind_search(ctx.clone(), handles.clone(), stack.clone());
    home_page.bind(ctx.clone());

    let updates_page_refresh = updates_page.clone();
    let ctx_updates = ctx.clone();
    let toasts_updates = handles.toasts.clone();
    updates_page_refresh.refresh(ctx_updates.clone(), Some(toasts_updates.clone()));
    glib::timeout_add_local(Duration::from_secs(1800), move || {
        updates_page_refresh.refresh(ctx_updates.clone(), Some(toasts_updates.clone()));
        ControlFlow::Continue
    });

    window.present();
}

fn run_search(query: String, ctx: AppContext, search_page: search::SearchPage, handles: UiHandles) {
    let (tx, rx) = std::sync::mpsc::channel();
    let ctx_thread = ctx.clone();
    std::thread::spawn(move || {
        let mut pacman_results = ctx_thread.pacman.search(&query).unwrap_or_default();
        let mut aur = ctx_thread.aur.search(&query).unwrap_or_default();
        let mut flatpak = ctx_thread.flatpak.search(&query).unwrap_or_default();

        let pacman_installed: HashSet<String> = ctx_thread
            .pacman
            .list_installed()
            .unwrap_or_default()
            .into_iter()
            .map(|pkg| pkg.name)
            .collect();
        let flatpak_installed: HashSet<String> = ctx_thread
            .flatpak
            .list_installed()
            .unwrap_or_default()
            .into_iter()
            .map(|pkg| pkg.name)
            .collect();

        for pkg in &mut pacman_results {
            pkg.installed = pacman_installed.contains(&pkg.name);
        }
        for pkg in &mut aur {
            pkg.installed = pacman_installed.contains(&pkg.name);
        }
        for pkg in &mut flatpak {
            pkg.installed = flatpak_installed.contains(&pkg.name);
        }

        let mut dedup: HashMap<(PackageSource, String), crate::core::models::PackageSummary> =
            HashMap::new();
        for pkg in pacman_results
            .into_iter()
            .chain(aur.into_iter())
            .chain(flatpak.into_iter())
        {
            let key = (pkg.source, pkg.name.clone());
            dedup.insert(key, pkg);
        }

        let mut results: Vec<_> = dedup.into_values().collect();
        results.sort_by(|a, b| a.name.cmp(&b.name));
        let _ = tx.send(results);
    });

    glib::idle_add_local(move || match rx.try_recv() {
        Ok(results) => {
            search_page.set_results(results, &ctx, &handles);
            ControlFlow::Break
        }
        Err(std::sync::mpsc::TryRecvError::Empty) => ControlFlow::Continue,
        Err(std::sync::mpsc::TryRecvError::Disconnected) => ControlFlow::Break,
    });
}

fn run_plan(
    plan: TransactionPlan,
    ctx: &AppContext,
    log_drawer: &widgets::log_drawer::LogDrawer,
    parent: &adw::ApplicationWindow,
    toasts: &adw::ToastOverlay,
) {
    if plan.commands.is_empty() {
        return;
    }

    log_drawer.clear();
    log_drawer.set_visible(true);

    let commands = Rc::new(RefCell::new(plan.commands));
    let ctx_clone = ctx.clone();
    let log_drawer = log_drawer.clone();
    let parent = parent.clone();
    let toasts = toasts.clone();
    let prompt_open = Rc::new(RefCell::new(false));

    let next: Rc<RefCell<Option<Box<dyn Fn()>>>> = Rc::new(RefCell::new(None));
    let next_clone = next.clone();

    *next.borrow_mut() = Some(Box::new(move || {
        let mut cmds = commands.borrow_mut();
        if cmds.is_empty() {
            let dialog = adw::MessageDialog::new(
                Some(&parent),
                Some("Transactions complete"),
                Some("All actions finished."),
            );
            dialog.add_response("ok", "OK");
            dialog.connect_response(None, |d: &adw::MessageDialog, _| d.close());
            dialog.present();
            toasts.add_toast(adw::Toast::new("Transactions complete"));
            return;
        }
        let cmd = cmds.remove(0);
        let (tx, rx) = mpsc::channel();
        let (input_tx, input_rx) = mpsc::channel();
        let runner = ctx_clone.runner.clone();
        let log_limit = runner.log_limit;
        let _ = runner.run_streaming(cmd, tx, Some(input_rx));
        let next_inner = next_clone.clone();
        let log_drawer = log_drawer.clone();
        let toasts = toasts.clone();
        let parent = parent.clone();
        let prompt_open = prompt_open.clone();
        glib::idle_add_local(move || match rx.try_recv() {
            Ok(event) => {
                match event {
                    LogEvent::Line(line) => {
                        if should_prompt(&line) && !*prompt_open.borrow() {
                            *prompt_open.borrow_mut() = true;
                            show_prompt_dialog(
                                &parent,
                                &line,
                                input_tx.clone(),
                                prompt_open.clone(),
                            );
                        }
                        log_drawer.append_line(&line, log_limit)
                    }
                    LogEvent::Finished(code) => {
                        if code != 0 {
                            toasts.add_toast(adw::Toast::new(&format!(
                                "Command failed ({code})"
                            )));
                        } else if let Some(next) = &*next_inner.borrow() {
                            next();
                        }
                        return ControlFlow::Break;
                    }
                }
                ControlFlow::Continue
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => ControlFlow::Continue,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => ControlFlow::Break,
        });
    }) as Box<dyn Fn()>);

    {
        let next_ref = next.borrow();
        if let Some(next_fn) = next_ref.as_ref() {
            next_fn();
        }
    }
}

fn should_prompt(line: &str) -> bool {
    let l = line.to_lowercase();
    l.contains("packages to cleanbuild")
        || l.contains("proceed with installation")
        || l.contains("proceed with transaction")
        || l.contains("enter a number")
        || l.contains("[y/n]")
        || l.contains("[y]")
        || l.ends_with("?")
}

fn show_prompt_dialog(
    parent: &adw::ApplicationWindow,
    prompt: &str,
    input_tx: mpsc::Sender<String>,
    prompt_open: Rc<RefCell<bool>>,
) {
    let dialog = adw::MessageDialog::new(
        Some(parent),
        Some("Input Required"),
        Some(prompt),
    );
    let entry = gtk::Entry::new();
    entry.set_placeholder_text(Some("Enter response (e.g., y, n, 1)"));
    dialog.set_extra_child(Some(&entry));
    dialog.add_response("y", "Yes");
    dialog.add_response("n", "No");
    dialog.add_response("a", "All");
    dialog.add_response("i", "Installed");
    dialog.add_response("no", "NotInstalled");
    dialog.add_response("ab", "Abort");
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("send", "Send");
    dialog.set_response_appearance("send", adw::ResponseAppearance::Suggested);
    dialog.set_response_appearance("y", adw::ResponseAppearance::Suggested);
    let input_tx_quick = input_tx.clone();
    let prompt_open_quick = prompt_open.clone();
    dialog.connect_response(None, move |d, resp| {
        let val = match resp {
            "y" => Some("y"),
            "n" => Some("n"),
            "a" => Some("a"),
            "i" => Some("i"),
            "no" => Some("no"),
            "ab" => Some("ab"),
            _ => None,
        };
        if let Some(v) = val {
            let _ = input_tx_quick.send(v.to_string());
            *prompt_open_quick.borrow_mut() = false;
            d.close();
        }
    });
    let prompt_open_send = prompt_open.clone();
    dialog.connect_response(Some("send"), move |d, _| {
        let text = entry.text().to_string();
        let _ = input_tx.send(text);
        *prompt_open_send.borrow_mut() = false;
        d.close();
    });
    let prompt_open_cancel = prompt_open.clone();
    dialog.connect_response(Some("cancel"), move |d, _| {
        *prompt_open_cancel.borrow_mut() = false;
        d.close();
    });
    dialog.present();
}

fn setup_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(
        "
        .card {\n\
            background: @window_bg_color;\n\
            border-radius: 12px;\n\
            border: 1px solid @borders;\n\
            padding: 12px;\n\
        }\n\
        .pill {\n\
            background: @accent_bg_color;\n\
            color: @accent_fg_color;\n\
            border-radius: 999px;\n\
            padding: 2px 8px;\n\
        }\n\
        .dim-label {\n\
            color: @dim_label_color;\n\
        }\n\
        ",
    );
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().unwrap(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn apply_theme(theme: ThemeMode) {
    let manager = adw::StyleManager::default();
    match theme {
        ThemeMode::System => manager.set_color_scheme(adw::ColorScheme::Default),
        ThemeMode::Light => manager.set_color_scheme(adw::ColorScheme::ForceLight),
        ThemeMode::Dark => manager.set_color_scheme(adw::ColorScheme::ForceDark),
    }
}
