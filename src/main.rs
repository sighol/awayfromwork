use chrono::prelude::*;
use clap::App;
use clap::Arg;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs::{self, File};
use std::io;
use std::io::prelude::*;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
enum WorkType {
    Education,
    Perm,
    Day,
    Night,
}

impl fmt::Display for WorkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Turnus {
    name: String,
    start: DateTime<Utc>,
    days: HashMap<i64, WorkType>,
    soldiers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Break {
    name: String,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    reason: Option<String>,
}

#[derive(Debug)]
struct TurnusDay {
    turnus_name: String,
    work_type: WorkType,
    day: Date<Utc>,
    soldiers: Vec<String>,
    away_soldiers: Vec<String>,
}

fn turnus_at_day(turnus: &Turnus, day: Date<Utc>) -> TurnusDay {
    let number_of_days = (day - turnus.start.date()).num_days() % 28;
    let work_type = turnus.days.get(&number_of_days).unwrap();

    TurnusDay {
        work_type: *work_type,
        away_soldiers: vec![],
        soldiers: turnus.soldiers.clone(),
        day: day,
        turnus_name: turnus.name.clone(),
    }
}

fn read_turnus_file(path: &str) -> Result<Turnus, std::io::Error> {
    println!("Reading {}", path);
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let turnus: Turnus = serde_yaml::from_str(&contents).expect("Failed to read yml file");
    return Ok(turnus);
}

fn read_break(date: DateTime<Utc>) -> Vec<String> {
    let mut file = File::open("fri.yml").expect("Could not find fri.yml");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read file");
    let breaks: Vec<Break> = serde_yaml::from_str(&contents).expect("Failed to read fri.yml");
    let mut free_people = vec![];
    for b in breaks.iter() {
        if b.from < date && b.to > date {
            free_people.push(b.name.clone());
        }
    }
    return free_people;
}

fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "\nPress any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

fn main() -> Result<(), std::io::Error> {
    let matches = App::new("myapp")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .long("silent")
                .takes_value(false)
                .help("Do not pause at the end of program execution."),
        )
        .arg(
            Arg::with_name("num_days")
                .short("n")
                .long("number-of-days")
                .takes_value(true)
                .help("Number of future days to print"),
        )
        .get_matches();

    let mut turnuses = vec![];
    let files = fs::read_dir("turnus")?;
    println!("{}", "Leser input...");
    for file in files {
        let file_path = file?.path().to_str().unwrap().to_string();
        let turnus = read_turnus_file(&file_path)?;
        turnuses.push(turnus);
    }
    println!("");

    let number_of_days = matches.value_of("num_days").unwrap_or("1");
    let number_of_days = number_of_days.parse::<i32>().unwrap_or(1);

    let mut day = Utc::now().date();
    for _ in 0..number_of_days {
        print_day(day, &turnuses);
        day = day.succ();
    }

    if !matches.is_present("quiet") {
        pause();
    }

    Ok(())
}

fn print_day(today: Date<Utc>, turnuses: &[Turnus]) {
    let mut t = term::stdout().unwrap();
    t.fg(term::color::GREEN).unwrap();
    t.attr(term::Attr::Bold).unwrap();
    writeln!(t, "DAG: {}", today);
    t.reset().unwrap();

    let perm_people = read_break(today.and_hms(12, 0, 0));

    println!("{}", "Fri:");
    for person in perm_people.iter() {
        println!(" - {}", person);
    }

    t.reset().unwrap();

    for turnus in turnuses {
        let turnus_day = turnus_at_day(&turnus, today);

        t.fg(term::color::BLUE).unwrap();
        t.attr(term::Attr::Bold).unwrap();
        let num_working = turnus_day.soldiers.len();
        writeln!(t, "{}", today);
        t.reset().unwrap();

        turnus_day.print(&perm_people);
    }
}

impl TurnusDay {
    fn print(&self, break_people: &[String]) {
        println!("{} | {} | {}", self.day, self.turnus_name, self.work_type);
        for soldier in itertools::sorted(self.soldiers.iter()) {
            if break_people.iter().any(|x| x == soldier) {
                println!(" - {} {}", soldier, "(away)")
            } else {
                println!(" - {}", soldier)
            }
        }
    }
}
