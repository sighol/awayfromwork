use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;
use chrono::prelude::*;


#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
enum WorkType {
    Education,
    Perm,
    Day,
    Night,
}

#[derive(Debug, Serialize, Deserialize)]
struct Turnus {
    name: String,
    start: DateTime::<Utc>,
    days: HashMap<i64, WorkType>,
    soldiers: Vec<String>
}

#[allow(dead_code)]
struct Free {
    name: String,
    from: Date::<Utc>,
    to: Date::<Utc>,
}

#[derive(Debug)]
struct TurnusDay {
    work_type: WorkType,
    day: Date::<Utc>,
    soldiers: Vec<String>,
    away_soldiers: Vec<String>,
}

impl TurnusDay {
    fn combine(a: Self, b: Self) -> Self {
        if a.work_type != b.work_type {
            panic!("Not the same work type")
        }
        a
    }
}

fn turnus_at_day(turnus: &Turnus, day: Date<Utc>) -> TurnusDay {

    let number_of_days = (day - turnus.start.date()).num_days() % 28;
    println!("This is {} days ago", number_of_days);
    let work_type = turnus.days.get(&number_of_days).unwrap();
    
    TurnusDay {
        work_type: *work_type,
        away_soldiers: vec![],
        soldiers: vec![],
        day: day,
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


fn main() -> Result<(), std::io::Error> {
    let turnus1 = read_turnus_file("turnus/turnus1.yml")?;
    let turnus2 = read_turnus_file("turnus/turnus2.yml")?;
    let turnus = turnus2;

    let today = Utc::now().date();
    let turnus_day = turnus_at_day(&turnus, today);
    println!("Turnus day: {:?}", turnus_day);


    let s = serde_yaml::to_string(&turnus).unwrap();

    Ok(())
}
