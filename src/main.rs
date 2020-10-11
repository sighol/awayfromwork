use chrono::prelude::*;
use clap::App;
use clap::Arg;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

fn turnus_at_day(turnus: &Turnus, day: Date<Utc>) -> Result<TurnusDay, Box<dyn Error>> {
    let number_of_days = (day - turnus.start.date()).num_days() % turnus.days.len() as i64 + 1;
    let work_type = turnus.days.get(&number_of_days).ok_or_else(|| {
        format!(
            "Turnus '{}' har noe feil med dager. Finner ikke dag {}",
            turnus.name, number_of_days
        )
    })?;

    Ok(TurnusDay {
        work_type: *work_type,
        away_soldiers: vec![],
        soldiers: turnus.soldiers.clone(),
        day: day,
        turnus_name: turnus.name.clone(),
    })
}

fn read_turnus_file(path: &str) -> Result<Turnus, std::io::Error> {
    println!("  Leser turnusfil: {}", path);
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let turnus: Turnus = serde_yaml::from_str(&contents).expect("Failed to read yml file");
    return Ok(turnus);
}

fn read_break(date: DateTime<Utc>) -> Vec<Break> {
    let mut file = File::open("fri.yml").expect("Could not find fri.yml");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read file");
    let breaks: Vec<Break> = serde_yaml::from_str(&contents).expect("Failed to read fri.yml");
    let mut free_people = vec![];
    for b in breaks.iter() {
        if b.from < date && b.to > date {
            free_people.push(b.clone());
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

fn ask_for_days() -> i32 {
    print!("Hvor mange dager ønsker du å se? ");
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_n) => input.trim().parse::<i32>().unwrap(),
        Err(error) => {
            println!("Failed to read input from terminal: {}", error);
            1
        }
    }
}

fn main() {
    let matches = App::new("myapp")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(
            Arg::with_name("quiet")
                .short("s")
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

    match run(&matches) {
        Ok(()) => (),
        Err(error) => println!("Failed: {:#?}", error),
    }

    if !matches.is_present("quiet") {
        pause();
    }
}

fn run(matches: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
    let mut turnuses = vec![];
    let files = fs::read_dir("turnus")?;
    println!("{}", "Leser input...");
    for file in files {
        let file_path = file?
            .path()
            .to_str()
            .ok_or("Can't get file path")?
            .to_string();
        let turnus = read_turnus_file(&file_path)?;
        turnuses.push(turnus);
    }

    let number_of_days = match matches.value_of("num_days") {
        Some(value) => value.parse::<i32>()?,
        None => ask_for_days(),
    };

    let mut day = Utc::now().date();
    for _ in 0..number_of_days {
        print_day(day, &turnuses)?;
        day = day.succ();
    }

    Ok(())
}

fn print_day(today: Date<Utc>, turnuses: &[Turnus]) -> Result<(), Box<dyn Error>> {
    let all_breaks = read_break(today.and_hms(12, 0, 0));
    let all_work_people: Vec<String> = turnuses
        .iter()
        .flat_map(|z| z.soldiers.iter())
        .map(|x| x.clone())
        .collect();
    let breaks_work_people: Vec<Break> = all_breaks
        .iter()
        .filter(|x| all_work_people.iter().filter(|y| *y == &x.name).count() > 0)
        .map(|x| x.clone())
        .collect();

    let turnus_days: Vec<TurnusDay> = turnuses
        .iter()
        .map(|x| turnus_at_day(x, today))
        .collect::<Result<Vec<TurnusDay>, Box<dyn Error>>>()?;
    let perm_people: Vec<String> = turnus_days
        .iter()
        .filter(|x| x.work_type == WorkType::Perm)
        .flat_map(|x| x.soldiers.clone())
        .collect();

    let mut perm_count = perm_people.len();
    let perm_borte: Vec<&String> = perm_people
        .iter()
        .filter(|p| breaks_work_people.iter().any(|x| &x.name == *p))
        .collect();
    perm_count -= perm_borte.len();

    let working_count = all_work_people.len() - breaks_work_people.len() - perm_count;
    let away_count = breaks_work_people.len() + perm_count;

    println!("-------------------------------------------------");
    println!("");
    let mut t = term::stdout().ok_or("Could not get stdout")?;
    t.fg(term::color::GREEN)?;
    t.attr(term::Attr::Bold)?;
    writeln!(
        t,
        "{}. På leir: {}. Borte: {}",
        today.naive_local().format("%A %d. %B %Y"),
        working_count,
        away_count
    )?;
    t.reset()?;
    for turnus in turnuses {
        let turnus_day = turnus_at_day(&turnus, today)?;

        t.fg(term::color::BLUE)?;
        t.attr(term::Attr::Bold)?;
        t.reset()?;

        turnus_day.print(&all_breaks)?;
    }

    Ok(())
}

impl TurnusDay {
    fn print(&self, break_people: &[Break]) -> Result<(), Box<dyn Error>> {
        let mut t = term::stdout().ok_or("Failed get stdout")?;
        t.fg(term::color::BLUE)?;
        t.attr(term::Attr::Bold)?;
        writeln!(t, "  {} | {}", self.turnus_name, self.work_type)?;
        t.reset()?;
        for soldier in itertools::sorted(self.soldiers.iter()) {
            if self.work_type == WorkType::Perm {
                t.fg(term::color::RED)?;
                writeln!(t, "   - {} (perm)", soldier)?;
                t.reset()?;
            } else if let Some(break_) = break_people.iter().filter(|x| &x.name == soldier).next() {
                t.fg(term::color::RED)?;
                writeln!(
                    t,
                    "   - {} (borte: {})",
                    soldier,
                    break_
                        .reason
                        .as_ref()
                        .unwrap_or(&"ingen god grunn".to_string())
                )?;
                t.reset()?;
            } else {
                writeln!(t, "   - {}", soldier)?;
            }
        }
        Ok(())
    }
}
