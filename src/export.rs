

use {
    crate::Lesson,
    icalendar::{
        Event,
        Calendar,
        Component,
    },
    std::{
        fs,
        io::Write,
    },
};



pub fn to_ics(lessons: Vec<Lesson>) -> Vec<u8>{
    let events = lessons
        .iter()
        .map(|lesson| {
            Event::new()
                .summary(lesson.class().as_str())
                .description(format!("Địa điểm: {}", lesson.place()).as_str())
                .starts(lesson.begin())
                .ends(lesson.end())
                .done()
        }).collect::<Vec<Event>>();

    let mut cal = Calendar::new();
    cal.name("TKBSV");
    cal.extend(events);
   
    format!("{}", cal).into_bytes()
    
}
