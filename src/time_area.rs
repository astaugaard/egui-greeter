use chrono::{Datelike, Local, Timelike};
use egui::{Color32, Label, RichText, Ui};

static MONTHS: [&str; 12] = [
    "January",
    "Febuary",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

static DAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

pub fn time_area(ui: &mut Ui) {
    let time = Local::now();

    let hour = time.hour();
    let minute = time.minute();
    let day_of_week = DAYS[time.weekday().num_days_from_monday() as usize];
    let month = MONTHS[time.month0() as usize];
    let day = time.day();

    ui.add(Label::new(
        RichText::new(format!("{day_of_week} {month} {day}"))
            .color(Color32::from_rgb(139, 213, 202))
            .size(48.0),
    ));

    ui.add(Label::new(
        RichText::new(format!("{hour}:{:0>2}", minute))
            .color(Color32::from_rgb(139, 213, 202))
            .size(144.0),
    ));
}
