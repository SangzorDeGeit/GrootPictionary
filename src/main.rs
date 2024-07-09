use std::collections::HashMap;
use std::{env, fs};
use std::cell::RefCell;
use gtk::gio::ActionEntry;
use rand::seq::SliceRandom;

use gdk4::prelude::*;
use gtk::{prelude::*, Button, CheckButton, Grid, Label};
use gtk::{glib, ApplicationWindow};

const APP_ID: &str = "pictionary.groot";

const REGELS: [&str; 26] = 
[
 "Achteruit, 2 mogelijkheden, of we spelen de ronde achteruit, of jouw team verplaatst zich achter dat van Ruit",
 "Buur schuur! Deel een biertje met elke buur!",
 "",
 "Dick schoof wilt dat je belasting betaalt: drink 21% van je huidige drankje",
 "regel 5",
 "regel 6",
 "regel 7",
 "regel 8",
 "regel 9",
 "regel 10",
 "regel 11",
 "regel 12",
 "regel 13",
 "regel 14",
 "regel 15",
 "regel 16",
 "regel 17",
 "regel 18",
 "regel 19",
 "regel 20",
 "regel 21",
 "regel 22",
 "regel 23",
 "regel 24",
 "regel 25",
 "regel 26"
 ];

fn main() -> glib::ExitCode {
    // Create a new application
    let app = adw::Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(select_monitor);

    app.set_accels_for_action("app.generate", &["g"]);
    // Run the application
    app.run()
}

    
/// Generates the window to select the monitor and calls main_program on button select
fn select_monitor(app: &adw::Application) {
    let container = Grid::new();
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Groot Pictionary")
        .child(&container)
        .build();

    let label = Label::builder()
        .label("Kies het scherm waar je de woorden op wil laten zien")
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();

    let display = match gdk4::Display::default() {
        Some(d) => d,
        None => todo!(),
    };

    let mut i: u32 = 0;
    while let Some(monitor) = display.monitors().item(i) {
        let monitor = monitor.to_value()
            .get::<gdk4::Monitor>()
            .expect("Value needs to be monitor");

        let monitor_button = Button::builder()
            .label(format!("{}cm x {}cm", monitor.height_mm()/10, monitor.width_mm()/10).as_str())
            .margin_top(6)
            .margin_bottom(6)
            .margin_start(6)
            .margin_end(6)
            .build();

        monitor_button.connect_clicked(glib::clone!(@strong app, @strong window, @weak monitor => move |_| {
            window.destroy();
            main_program(&app, monitor);
        }));

        let column = match i32::try_from(i) {
            Ok(c) => c,
            Err(_) => todo!("break here and display an error message (too many monitors)"),
        };
        container.attach(&monitor_button, column, 1, 1, 1);

        i = i+1;
    }
    let monitor_amount = match i32::try_from(i) {
        Ok(c) => c,
        Err(_) => todo!("break here and display an error message (too many monitors)"),
    };
    if monitor_amount == 0 {
        label.set_text("Kon geen monitors vinden");
        container.attach(&label, 0, 0, 0, 0);
    } else if monitor_amount%2 == 1 {
        container.attach(&label, monitor_amount/2, 0, 1, 1);
    } else {
        container.attach(&label, (monitor_amount-1)/2, 0, 2, 1);
    }

    window.present()
}

/// The main program does the following things
/// * Make the display window and display it on the correct monitor
/// * Make the manager window and display it on the correct monitor
fn main_program(app: &adw::Application,  display_monitor: gdk4::Monitor) {
    // Make the word list variable
    let mut path = env::current_dir().expect("Could not get current directory");
    path.push("word_list.txt");
    let contents = fs::read_to_string(&path).expect("Could not read file contents to string");
    let word_list: Vec<String> = contents
        .split("\n")
        .map(|s| String::from(s))
        .collect();

    // generate display window
    let (display_window, word_label) = generate_display_window(app);
    display_window.present();
    display_window.fullscreen_on_monitor(&display_monitor);

    // generate the manager window
    let manager_window = generate_manager_window(app, word_list, word_label);
    manager_window.present();

    // display window should close if manager window closes
    manager_window.connect_destroy(move |_| {
        display_window.destroy();
    });
    
}


/// Makes the manager window
fn generate_manager_window(app: &adw::Application, word_list: Vec<String>, display_label: Label) -> ApplicationWindow{
    let chars = ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z"];
    let recent_words: RefCell<Vec<String>> = RefCell::new(Vec::new());
    let container = Grid::new();


    let window = ApplicationWindow::builder()
        .application(app)
        .child(&container)
        .title("Groot Pictionary Manager")
        .build();

    let mut rule_array: HashMap<CheckButton, String> = HashMap::new();
    let mut i = 0;
    for letter in chars {
        let checkbox = CheckButton::builder()
            .label(String::from(letter))
            .margin_end(6)
            .margin_bottom(6)
            .margin_top(6)
            .margin_start(6)
            .build();
        let regel = REGELS[usize::try_from(i).unwrap()];
        container.attach(&checkbox, i/5, i%5, 1, 1);
        rule_array.insert(checkbox, String::from(regel));
        i+=1;
    }

    let action_generate = ActionEntry::builder("generate")
        .activate(move |_app: &adw::Application,_,_| {
            loop {
                let random_word = word_list.choose(&mut rand::thread_rng()).unwrap();
                if recent_words.borrow().contains(random_word) {
                    continue;
                }
                if recent_words.borrow().len() >= 50 {
                    recent_words.borrow_mut().remove(0);
                }
                recent_words.borrow_mut().push(String::from(random_word));
                let mut label_text: String;
                if random_word.len() >= 20 {
                    label_text = format!("<span font=\"120\">{}\n</span>", &random_word);
                } else {
                    label_text = format!("<span font=\"150\">{}\n</span>", &random_word);
                }

                for entry in &rule_array {
                    if random_word.starts_with(entry.0.label().unwrap().as_str()) && entry.0.is_active() {
                        label_text = format!("{}<span font=\"70\">({})</span>", label_text, entry.1);
                        break;
                    }
                }
                display_label.set_label(&label_text);
                break;
            }
        })
        .build();
    app.add_action_entries([action_generate]);

    let generate_button = Button::builder()
        .label("Generate word")
        .margin_end(6)
        .margin_bottom(6)
        .margin_top(6)
        .margin_start(6)
        .action_name("app.generate")
        .build();

    container.attach(&generate_button, 0, 10, 3, 1);

    window
}

/// Makes the display window
fn generate_display_window (app: &adw::Application) -> (ApplicationWindow, Label) {
    let label = Label::builder()
        .use_markup(true)
        .label("<span font=\"120\">Tap om een woord\n te genereren!</span>")
        .hexpand_set(true)
        .vexpand_set(true)
        .justify(gtk::Justification::Center)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .wrap(true)
        .wrap_mode(gtk::pango::WrapMode::Word)
        .build();
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Groot pictionary")
        .child(&label)
        .build();

    (window, label)
}
