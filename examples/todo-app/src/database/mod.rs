use std::sync::Mutex;

use time::OffsetDateTime;
use uuid::Uuid;

use crate::models::Todo;

pub async fn get_todos() -> Vec<Todo> {
    static TODOS: Mutex<Vec<Todo>> = Mutex::new(Vec::new());

    let mut todos = TODOS.lock().unwrap();

    if todos.is_empty() {
        *todos = vec![
            Todo {
                id: Uuid::new_v4(),
                title: String::from("Finish CS Project"),
                done: false,
                description: Some(String::from(
                    "Complete the final assignment for the CS course.",
                )),
                created_at: OffsetDateTime::from_unix_timestamp(1651747200).unwrap(),
                updated_at: OffsetDateTime::from_unix_timestamp(1651747200).unwrap(),
            },
            Todo {
                id: Uuid::new_v4(),
                title: String::from("Go grocery shopping"),
                done: false,
                description: None,
                created_at: OffsetDateTime::from_unix_timestamp(1651900800).unwrap(),
                updated_at: OffsetDateTime::from_unix_timestamp(1651900800).unwrap(),
            },
            Todo {
                id: Uuid::new_v4(),
                title: String::from("Attend doctor's appointment"),
                done: false,
                description: Some(String::from("Visit the doctor for a routine check-up.")),
                created_at: OffsetDateTime::from_unix_timestamp(1651990800).unwrap(),
                updated_at: OffsetDateTime::from_unix_timestamp(1651990800).unwrap(),
            },
            Todo {
                id: Uuid::new_v4(),
                title: String::from("Call mom"),
                done: false,
                description: None,
                created_at: OffsetDateTime::from_unix_timestamp(1652102400).unwrap(),
                updated_at: OffsetDateTime::from_unix_timestamp(1652102400).unwrap(),
            },
            Todo {
                id: Uuid::new_v4(),
                title: String::from("Plan vacation"),
                done: false,
                description: Some(String::from("Research and plan a vacation to Hawaii.")),
                created_at: OffsetDateTime::from_unix_timestamp(1652300800).unwrap(),
                updated_at: OffsetDateTime::from_unix_timestamp(1652300800).unwrap(),
            },
        ];
    }

    todos.clone()
}
