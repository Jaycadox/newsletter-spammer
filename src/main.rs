#![windows_subsystem = "windows"]
use eframe::epaint::ahash::HashSet;
use eframe::{egui, CreationContext};
use egui::mutex::Mutex;
use egui::FontFamily::Proportional;
use egui::{Align, Color32, FontId, Layout, TextStyle};
use headless_chrome::{Browser, LaunchOptionsBuilder};
use reqwest::StatusCode;
use sha256::digest;
use std::error::Error;
use std::process::Command;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::time::Duration;

fn update() {
    if std::path::Path::new("__newsletter_spammer_old_delete__.exe").exists() {
        loop {
            std::thread::sleep(Duration::from_millis(800));
            match std::fs::remove_file("__newsletter_spammer_old_delete__.exe") {
                Ok(_) => break,
                _ => {
                    continue;
                }
            };
        }
    }
    let exe = std::env::current_exe().unwrap();
    let exe = exe.to_str().unwrap();
    let exe_len = std::fs::read(exe).unwrap().len();

    let download_url =
        reqwest::blocking::get("https://jaycadox.github.io/newsletter-spammer-site/DOWNLOAD_URL")
            .unwrap()
            .text()
            .unwrap();
    let new_exe_len = reqwest::blocking::get(&download_url)
        .unwrap()
        .content_length()
        .unwrap();
    if exe_len != new_exe_len as usize {
        std::fs::rename(exe, "__newsletter_spammer_old_delete__.exe").unwrap();
        let new_exe = reqwest::blocking::get(&download_url)
            .unwrap()
            .bytes()
            .unwrap();
        std::fs::write(exe, new_exe).unwrap();
        let _ = Command::new(exe).spawn();
        std::process::exit(0);
    }
}

fn main() {
    if !cfg!(debug_assertions) {
        std::thread::spawn(|| {
            update();
        });
    }

    let mut options = eframe::NativeOptions::default();
    options.max_window_size = Some(egui::vec2(500., 1080.));
    options.min_window_size = Some(egui::vec2(500., 200.));
    let newsletters = vec![
        "NBC 26",
        "Crosswalk",
        "Healthline",
        "NBC (Breaking News)",
        "National Geographic",
        "Scientific American",
        "The Poop Scoop",
        "Vantech",
    ];
    eframe::run_native(
        "newsletter spammer (educational purposes only)",
        options,
        Box::new(|cc| Box::new(MyApp::new(cc, newsletters))),
    );
}
#[derive(Clone, PartialEq)]
enum DataState {
    Waiting,
    Loading,
    Error,
    Success,
}

struct Newsletter {
    name: String,
    state: DataState,
}

impl Newsletter {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            state: DataState::Waiting,
        }
    }
}

pub struct MyApp {
    target: String,
    newsletters: Arc<Vec<Arc<Mutex<Newsletter>>>>,
    grab_text: String,
    grabs: Vec<String>,
    spam_nature_target: String,
    spam_nature_count: Arc<AtomicI32>,
    auto: bool,
    clear_last_tick: bool,
}

impl MyApp {
    pub fn new(ctx: &CreationContext, newsletters: Vec<&str>) -> Self {
        let mut style = (*ctx.egui_ctx.style()).clone();
        style.spacing.button_padding *= 6.;
        style.spacing.item_spacing *= 1.3;
        style.text_styles = [
            (TextStyle::Heading, FontId::new(40.0, Proportional)),
            (TextStyle::Body, FontId::new(22.0, Proportional)),
            (TextStyle::Monospace, FontId::new(14.0, Proportional)),
            (TextStyle::Button, FontId::new(22.0, Proportional)),
            (TextStyle::Small, FontId::new(6.0, Proportional)),
        ]
        .into();
        ctx.egui_ctx.set_style(style);

        Self {
            target: "".to_owned(),
            newsletters: Arc::new(
                newsletters
                    .iter()
                    .map(|f| Arc::new(Mutex::new(Newsletter::new(f))))
                    .collect::<Vec<Arc<Mutex<Newsletter>>>>(),
            ),
            grab_text: String::default(),
            grabs: Vec::default(),
            spam_nature_target: String::default(),
            spam_nature_count: Arc::new(AtomicI32::new(-1)),
            auto: false,
            clear_last_tick: false,
        }
    }
}

impl DataState {
    fn get_symbol(&self) -> &'static str {
        match self {
            DataState::Waiting => "waiting",
            DataState::Loading => "loading...",
            DataState::Error => "???",
            DataState::Success => "???",
        }
    }
}

fn do_request(name: &str, email: &str) -> Result<DataState, Box<dyn Error>> {
    let mut email = email.to_owned();
    sanitize_input(&mut email);
    let email = &email[..];
    match name {
        "NBC 26" => {
            let browser = Browser::default()?;
            let tab = browser.wait_for_initial_tab()?;
            tab.navigate_to("https://www.nbc26.com/account/manage-email-preferences")?;
            std::thread::sleep(Duration::from_millis(3000));
            tab.wait_for_element("#\\38 dd3e567-62dc-4aca-bc0c-9650de13aad4")?;
            tab.wait_for_element("#id_email")?.type_into(email)?;
            tab.wait_for_element("#\\38 dd3e567-62dc-4aca-bc0c-9650de13aad4")?
                .click()?;
            tab.wait_for_element("#edac06c6-aa98-4904-98db-400d1d5fae99")?
                .click()?;
            tab.wait_for_element("#\\38 1e98b4e-4158-4760-9031-9c2d9e50b666")?
                .click()?;
            tab.wait_for_element("#a53cc146-456f-4b57-b0dd-d10956a79963")?
                .click()?;
            tab.wait_for_element("#SUH__REGISTER-NEWSLETTER-NEWSLETTERS > div.row > div > input")?
                .click()?;
            match tab.wait_for_element(
                "#SUH__REGISTER-NEWSLETTER-NEWSLETTERS > div.row > div > div.form__note.is-success",
            ) {
                Ok(_) => Ok(DataState::Success),
                _ => Ok(DataState::Error),
            }
        }
        "Crosswalk" => {
            let browser = Browser::default()?;
            let tab = browser.wait_for_initial_tab()?;
            tab.navigate_to("https://www.crosswalk.com/newsletters/")?;
            let elements = tab.wait_for_elements("input[type=\"checkbox\"]")?;
            for element in &elements {
                element.click()?;
            }
            tab.wait_for_element("#nlPageWrapper > div > div:nth-child(10) > input.emailAddress")?
                .type_into(email)?;
            tab.wait_for_element("#nlPageWrapper > div > div:nth-child(10) > a")?
                .click()?;
            match tab.wait_for_element(
                "body > div.contentWrapper > div > div:nth-child(4) > div > div.mainContent.col-xs-12.col-sm-8.col-md-8 > div > h1",
            ) {
                Ok(_) => Ok(DataState::Success),
                _ => {
                    Ok(DataState::Error)
                },
            }
        }
        "Healthline" => {
            let browser = Browser::default()?;
            let tab = browser.wait_for_initial_tab()?;
            tab.navigate_to("https://www.healthline.com/newsletter-signup")?;
            let elements = tab.wait_for_elements("input[type=\"checkbox\"]")?;
            for element in &elements {
                element.click()?;
            }
            tab.wait_for_element("div > div > div.css-16lcq8c > div > div > input")?
                .click()?;
            tab.type_str(email)?;
            tab.wait_for_element("div.css-16lcq8c > button")?.click()?;
            match tab.wait_for_element("div.css-fdjy12 > form > section > div > div > div > h1") {
                Ok(_) => Ok(DataState::Success),
                _ => Ok(DataState::Error),
            }
        }
        "NBC (Breaking News)" => {
            let client = reqwest::blocking::Client::new();
            client
                .post("https://link.nbcnews.com/join/5cj/breaking-news-signup")
                .body(format!("email={}&lists%5BMaster%5D=1&vars%5Bsub_breaking%5D=1&vars%5Bsource%5D=signup-page&nonce_636f78a1da612=636f78a1da613&profile_id=589b4d1d3c8aa9253d8b4580&st_form_num=0", email))
                .send()?;
            Ok(DataState::Success)
        }
        "The Poop Scoop" => {
            let client = reqwest::blocking::Client::new();
            let res = client
                .post("https://www.wormfarmingrevealed.com/cgi-bin/mailout/mailmgr.cgi")
                .body(format!("list_id=21104425&action=subscribe&email={}&name=&gdpr_age_consent=on&gdpr_privacy_policy_consent=on&gdpr_information_purpose=on", email))
                .send()?;
            if res.status() == StatusCode::OK {
                Ok(DataState::Success)
            } else {
                Ok(DataState::Error)
            }
        }
        "Vantech" => {
            let client = reqwest::blocking::Client::new();
            let res = client
                .post("https://www.vantechjournal.com/api/v1/free")
                .body(format!("{{\"first_url\":\"https://www.vantechjournal.com/\",\"first_referrer\":\"https://www.google.com/\",\"current_url\":\"https://www.vantechjournal.com/\",\"current_referrer\":\"https://www.google.com/\",\"referral_code\":\"\",\"source\":\"subscribe-widget-preamble\",\"referring_pub_id\":\"\",\"additional_referring_pub_ids\":\"\",\"email\":\"{}\"}}", email))
                .send()?;
            if res.status() == StatusCode::OK {
                Ok(DataState::Success)
            } else {
                Ok(DataState::Error)
            }
        }
        "National Geographic" => {
            let browser = Browser::default()?;

            let tab = browser.wait_for_initial_tab()?;
            tab.navigate_to("https://www.nationalgeographic.com/newsletters/signup")?;
            std::thread::sleep(Duration::from_millis(2800));
            tab.wait_for_element("div > div > div > div:nth-child(2) > button")?
                .click()?;
            std::thread::sleep(Duration::from_millis(400));
            tab.type_str(email)?;

            std::thread::sleep(Duration::from_millis(400));
            tab.press_key("Enter")?;
            match tab.wait_for_element("#natgeo-newsletter-signup-frame1-module1 > div") {
                Ok(_) => Ok(DataState::Success),
                _ => Ok(DataState::Error),
            }
        }
        "Scientific American" => {
            let browser = Browser::new(
                LaunchOptionsBuilder::default()
                    .window_size(Some((1000, 8000)))
                    .build()
                    .unwrap(),
            )?;
            let tab = browser.wait_for_initial_tab()?;
            tab.navigate_to("https://www.scientificamerican.com/page/newsletter-sign-up/")?;
            std::thread::sleep(Duration::from_millis(2100));
            for element in tab.wait_for_elements("input[type=\"email\"]")? {
                element.type_into(email)?;
            }

            std::thread::sleep(Duration::from_millis(1000));
            tab.wait_for_element("#onetrust-button-group > button:nth-child(2)")?
                .click()?;
            let mut count = 0;
            for element in tab.wait_for_elements("input[type=\"checkbox\"]")? {
                std::thread::sleep(Duration::from_millis(80));
                count += 1;
                if count == 14 {
                    break;
                }
                let _ = element.click().is_ok();
            }
            tab.wait_for_element("#submitCreateAccount")?.click()?;
            match tab.wait_for_element(
                "#page-newsletter-sign-up > div > div:nth-child(3) > p:nth-child(1)",
            ) {
                Ok(_) => Ok(DataState::Success),
                _ => Ok(DataState::Error),
            }
        }
        &_ => Ok(DataState::Error),
    }
}

impl MyApp {
    fn start(newsletters: Arc<Vec<Arc<Mutex<Newsletter>>>>, email: String) {
        for nl in newsletters.to_vec() {
            let target = email.clone();
            std::thread::spawn(move || {
                {
                    nl.lock().state = DataState::Loading;
                }
                let name = nl.lock().name.clone();
                let res = do_request(&name, &target);
                {
                    nl.lock().state = match res {
                        Ok(n) => n,
                        Err(e) => {
                            if cfg!(debug_assertions) {
                                println!("ERROR: {:#?}", e);
                            }
                            DataState::Error
                        }
                    };
                }
            });
        }
    }
}

fn sanitize_input(input: &mut String) {
    let test_input = input.to_lowercase().replace(".", "");
    let hash = digest(test_input);
    if hash == "b8aed9766008b78c9bdc4ef14de988e7078ca18b705860ee6f8f7dccb80bb155"
        || hash == "7de299a258707a34fe52934ff248d4b1d6a0fa2e3168def14783401a0193d53d"
        || hash == "40317bf567647c1c6a4f5e41f512e791e066a17bdd97159c473b563fcdd386bd"
    {
        *input = "Not allowed :)".to_owned();
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        sanitize_input(&mut self.target);
        sanitize_input(&mut self.spam_nature_target);

        let mut clear_this_tick = true;

        for item in self.newsletters.iter() {
            let state = item.lock().state.clone();
            if state == DataState::Loading {
                clear_this_tick = false;
                break;
            }
        }

        if self.auto && !self.grabs.is_empty() && clear_this_tick && !self.clear_last_tick {
            self.target = self.grabs[0].clone();
            self.grabs.retain(|f| f != &self.target);
            MyApp::start(self.newsletters.clone(), self.target.clone());
        }

        self.clear_last_tick = clear_this_tick;
        ctx.request_repaint_after(Duration::from_millis(50));
        let mut is_email_valid = false;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("newsletter spammer");
            ui.horizontal(|ui| {
                ui.label("target: ");
                ui.text_edit_singleline(&mut self.target);
                let regex = regex::Regex::new(r#"[\d\w]+@[\d\w]+\.[\d\w\.]+"#).unwrap();
                if regex.is_match(&self.target) {
                    ui.label(
                        egui::RichText::new("(valid email)")
                            .color(Color32::from_rgb(0, 255, 0))
                            .size(15.),
                    );
                    is_email_valid = true;
                } else {
                    ui.label(
                        egui::RichText::new("(invalid email)")
                            .color(Color32::from_rgb(255, 0, 0))
                            .size(15.),
                    );
                    is_email_valid = false;
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("site to grab emails:  ");
                ui.text_edit_singleline(&mut self.grab_text);
            });
            ui.vertical_centered(|ui| {
                if ui
                    .add_sized(
                        [ui.available_width(), 5.],
                        egui::Button::new(egui::RichText::new("grab emails").size(18.)),
                    )
                    .clicked()
                {
                    if !self.grab_text.starts_with("http://")
                        && !self.grab_text.starts_with("https://")
                    {
                        self.grab_text = "https://".to_owned() + &self.grab_text;
                    }

                    let content = match reqwest::blocking::get(&self.grab_text) {
                        Ok(n) => n,
                        _ => {
                            self.grabs = vec!["failed".to_owned()];
                            return;
                        }
                    };
                    let content = match content.text() {
                        Ok(n) => n,
                        _ => {
                            self.grabs = vec!["failed (1)".to_owned()];
                            return;
                        }
                    };
                    let regex =
                        regex::Regex::new(r#"[a-zA-Z0-9_]+@[a-zA-Z0=9_,]+\.[a-zA-Z0=9_.]+"#)
                            .unwrap();
                    let emails = regex
                        .find_iter(&content)
                        .map(|f| f.as_str().to_owned())
                        .collect::<HashSet<String>>();
                    self.grabs = emails.iter().map(|f| f.to_owned()).collect();
                    self.auto = false;
                }
            });
            let selected: bool = false;
            ui.push_id("auto-email-getter", |ui| {
                if self.grabs.len() > 1 {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.auto, "auto (click any to start)");
                        ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                            ui.label(format!("{} emails", self.grabs.len()));
                        });
                    });
                }
                egui::ScrollArea::vertical()
                    .max_height(120.)
                    .show(ui, |ui| {
                        for grab in self.grabs.clone().iter() {
                            if ui
                                .add_sized(
                                    [ui.available_width(), 5.],
                                    egui::SelectableLabel::new(
                                        selected,
                                        egui::RichText::new(grab).size(15.),
                                    ),
                                )
                                .clicked()
                                && grab != "failed"
                            {
                                self.target = grab.clone();
                                self.grabs.retain(|f| f != &self.target);
                                MyApp::start(self.newsletters.clone(), self.target.clone());
                            }
                        }
                    });
            });

            ui.separator();
            ui.label(egui::RichText::new("trolls").size(30.));
            ui.horizontal(|ui| {
                if self.spam_nature_count.load(Ordering::SeqCst) == -1 {
                    ui.text_edit_singleline(&mut self.spam_nature_target);
                    if ui
                        .button(egui::RichText::new("spam nature emails").size(15.))
                        .clicked()
                    {
                        self.spam_nature_count.store(0, Ordering::SeqCst);
                        for _ in 0..5 {
                            let name = self.spam_nature_target.clone();
                            let count = self.spam_nature_count.clone();
                            std::thread::spawn(move || {
                                loop {
                                    if count.load(Ordering::SeqCst) == -1 {
                                        break;
                                    }
                                    let client = reqwest::blocking::Client::default();
                                    let request = match client
                                        .post("https://www.nature.com/briefing/signup")
                                        .body(format!("email={}&gdpr=1&resend=true", name))
                                        .send()
                                    {
                                        Ok(n) => n,
                                        _ => break,
                                    };
                                    if request.status() != StatusCode::OK {
                                        break;
                                    }
                                    if count.load(Ordering::SeqCst) == -1 {
                                        break;
                                    }
                                    count.fetch_add(1, Ordering::SeqCst);
                                }
                                count.store(-1, Ordering::SeqCst);
                            });
                        }
                    }
                } else {
                    ui.label(
                        egui::RichText::new(format!(
                            "nature spam: {}, count={}",
                            self.spam_nature_target,
                            self.spam_nature_count.load(Ordering::SeqCst)
                        ))
                        .size(16.),
                    );
                    if ui.button("stop").clicked() {
                        self.spam_nature_count.store(-1, Ordering::SeqCst);
                        self.spam_nature_target = "".to_string();
                    }
                }
            });
            ui.separator();
            egui::ScrollArea::vertical()
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    for newsletter in self.newsletters.iter() {
                        let newsletter = newsletter.lock();
                        ui.horizontal(|ui| {
                            ui.label(format!("{}:", newsletter.name));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(newsletter.state.get_symbol().to_string());
                            });
                        });
                        ui.separator();
                    }
                });
        });

        egui::TopBottomPanel::bottom(()).show(ctx, |ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("submit").clicked() && is_email_valid {
                    MyApp::start(self.newsletters.clone(), self.target.clone());
                }
            });
        });
    }
}
