use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::fmt;
use std::io::prelude::*;
use std::collections::HashMap;
use chrono::prelude::*;
use colored::*;


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
    start: DateTime::<Utc>,
    days: HashMap<i64, WorkType>,
    soldiers: Vec<String>
}

#[derive(Debug, Serialize, Deserialize)]
struct Break {
    name: String,
    from: DateTime::<Utc>,
    to: DateTime::<Utc>,
    reason: Option<String>,
}

#[derive(Debug)]
struct TurnusDay {
    turnus_name: String,
    work_type: WorkType,
    day: Date::<Utc>,
    soldiers: Vec<String>,
    away_soldiers: Vec<String>,
}

impl TurnusDay {
    fn print(&self, break_people: &[String]) {
        println!("{} | {} | {}", self.day, self.turnus_name.blue(), self.work_type.to_string().cyan());
        for soldier in itertools::sorted(self.soldiers.iter()) {
            if break_people.iter().any(|x| x == soldier) {
                println!(" - {} {}", soldier.red(), "(away)".red())
            } else {
                println!(" - {}", soldier)
            }
        }
    }
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
    file.read_to_string(&mut contents).expect("Failed to read file");
    let breaks: Vec<Break> = serde_yaml::from_str(&contents).expect("Failed to read fri.yml");
    let mut free_people = vec![];
    for b in breaks.iter() {
        if b.from < date && b.to > date {
            free_people.push(b.name.clone());
        }
    }
    return free_people;
}


fn main() -> Result<(), std::io::Error> {
    let mut turnuses = vec![];
    let files = fs::read_dir("turnus")?;
    println!("{}", "Leser input...".white().on_blue().bold());
    for file in files {
        let file_path = file?.path().to_str().unwrap().to_string();
        let turnus = read_turnus_file(&file_path)?;
        turnuses.push(turnus);
    }
    println!("");
    
    let today = Utc::now().date();
    let perm_people = read_break(Utc::now());
    println!("{}", "Disse personene har fri i dag:".white().on_blue().bold());
    println!("{:#?}", &perm_people);
    
    println!("{}", "Dagens grupper".white().on_blue().bold());
    for turnus in turnuses {
        let turnus_day = turnus_at_day(&turnus, today);
        turnus_day.print(&perm_people);
    }
    
    Ok(())
}
