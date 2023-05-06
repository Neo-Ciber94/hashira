use std::sync::Mutex;

use crate::models::Todo;

pub async fn get_todos() -> Vec<Todo> {
    static TODOS: Mutex<Vec<Todo>> = Mutex::new(Vec::new());

    let mut todos = TODOS.lock().unwrap();

    if todos.is_empty() {
        *todos = vec![
            Todo {
                id: 1,
                title: String::from("Finish CS Project"),
                done: false,
                description: Some(String::from(
                    "Complete the final assignment for the CS course.",
                )),
            },
            Todo {
                id: 2,
                title: String::from("Go grocery shopping"),
                done: false,
                description: None,
            },
            Todo {
                id: 3,
                title: String::from("Attend doctor's appointment"),
                done: false,
                description: Some(String::from("Visit the doctor for a routine check-up.")),
            },
            Todo {
                id: 4,
                title: String::from("Call mom"),
                done: false,
                description: None,
            },
            Todo {
                id: 5,
                title: String::from("Plan vacation"),
                done: false,
                description: Some(String::from("Research and plan a vacation to Hawaii.")),
            },
        ];
    }

    todos.clone()
}
