use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

struct Task {
   // id: u32,
    description: String,
}

fn main() {
    // Create a list of tasks
    let tasks: Vec<Task> = (0..10)
        .map(|i| Task {
            // id: i,
            description: format!("Task number {}", i),
        })
        .collect();

    // Shared counter for completed tasks
    let completed_tasks = Arc::new(Mutex::new(0));

    tasks.par_iter().for_each(|task| {
        // Simulate task processing
        println!("Processing {}", task.description);
        thread::sleep(Duration::from_millis(500));

        // Increment the counter for completed tasks
        let mut completed = completed_tasks.lock().unwrap();
        *completed += 1;
        println!("Completed {} tasks", *completed);
    });

    println!("All tasks completed!");
}
