mod question;

fn main() {
    println!("Hello, world!");
use question::{Question, QuestionScenario};
/// Hand-written example scenarios exported as JSON to feed another agent.
fn example_scenarios() -> Vec<QuestionScenario> {
    vec![
        // Multi-correct MultipleChoice.
        QuestionScenario {
            scenario_description: "A team is choosing memory-safety guarantees for a new \
                systems project written in Rust."
                .to_string(),
            questions: vec![Question::MultipleChoice {
                question_text: "Which of the following are enforced by Rust's ownership \
                    and borrowing rules at compile time?"
                    .to_string(),
                options: vec![
                    "No data races between threads".to_string(),
                    "No use-after-free of heap allocations".to_string(),
                    "Guaranteed absence of logic bugs".to_string(),
                    "No double frees".to_string(),
                ],
                correct_answers: vec![0, 1, 3],
            }],
            tags: vec!["rust".to_string(), "memory-safety".to_string()],
        },
        // Yes/no (single-correct) MultipleChoice.
        QuestionScenario {
            scenario_description: String::new(),
            questions: vec![Question::MultipleChoice {
                question_text: "In Rust, does moving a value out of a variable leave that \
                    variable usable afterwards?"
                    .to_string(),
                options: vec!["Yes".to_string(), "No".to_string()],
                correct_answers: vec![1],
            }],
            tags: vec!["rust".to_string(), "ownership".to_string()],
        },
        // OrderingTask with a red herring option.
        QuestionScenario {
            scenario_description: "You are describing the lifecycle of an HTTP request as it \
                travels through a typical web server."
                .to_string(),
            questions: vec![Question::OrderingTask {
                question_text: "Put the following steps in the order they occur when handling \
                    an incoming request."
                    .to_string(),
                options: vec![
                    "Parse the request headers".to_string(),
                    "Accept the TCP connection".to_string(),
                    "Send the response body".to_string(),
                    "Route to the matching handler".to_string(),
                    "Reboot the server".to_string(), // red herring
                ],
                correct_order: vec![1, 0, 3, 2],
            }],
            tags: vec!["networking".to_string(), "http".to_string()],
        },
        // Multi-question scenario mixing both variants under a shared description.
        QuestionScenario {
            scenario_description: "Consider a database transaction running under the default \
                isolation level of a typical relational database."
                .to_string(),
            questions: vec![
                Question::MultipleChoice {
                    question_text: "Which properties are captured by the ACID acronym?"
                        .to_string(),
                    options: vec![
                        "Atomicity".to_string(),
                        "Consistency".to_string(),
                        "Availability".to_string(),
                        "Isolation".to_string(),
                        "Durability".to_string(),
                    ],
                    correct_answers: vec![0, 1, 3, 4],
                },
                Question::OrderingTask {
                    question_text: "Order the phases of a two-phase commit as coordinated \
                        across participants."
                        .to_string(),
                    options: vec![
                        "Coordinator sends commit".to_string(),
                        "Coordinator sends prepare".to_string(),
                        "Participants vote".to_string(),
                        "Participants acknowledge commit".to_string(),
                    ],
                    correct_order: vec![1, 2, 0, 3],
                },
            ],
            tags: vec!["databases".to_string(), "transactions".to_string()],
        },
    ]
}

fn main() -> std::io::Result<()> {
    Ok(())
}
