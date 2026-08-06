//! The iced GUI: quizzes the user through every question in every loaded
//! scenario, one question at a time, and tracks a running score.

use std::collections::BTreeSet;
use std::fmt;

use iced::keyboard;
use iced::widget::{
    Column, Row, Space, button, checkbox, column, container, markdown, pick_list, row, scrollable,
    text,
};
use iced::{Alignment, Element, Length, Subscription};
use rand::seq::SliceRandom;

use crate::question::{Question, QuestionScenario};

/// A selectable option for an [`Question::OrderingTask`] slot. Carries the
/// original option index so identical labels stay distinguishable and the
/// answer can be checked against `correct_order`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Choice {
    index: usize,
    label: String,
}

impl fmt::Display for Choice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label)
    }
}

/// The user's in-progress answer for the *current* question.
enum Answer {
    /// One boolean per option: whether the user ticked it.
    MultipleChoice { selected: Vec<bool> },
    /// One entry per slot: which option index (if any) the user placed there.
    Ordering { slots: Vec<Option<usize>> },
    /// No question is active (e.g. the session is finished or empty).
    None,
}

impl Answer {
    /// returns true if the user has filled all needed details
    pub fn is_filled(&self) -> bool {
        match self {
            // If at least one slot has been selected
            Answer::MultipleChoice { selected } => selected.contains(&true),
            // of all slots have been assigned
            Answer::Ordering { slots } => slots.contains(&None) == false,
            Answer::None => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    /// A multiple-choice option checkbox was toggled.
    ToggleOption(usize),
    /// An ordering slot (`.0`) had option index (`.1`) assigned to it.
    SlotSelected(usize, usize),
    /// Grade the current answer.
    Submit,
    /// Advance to the next question.
    Next,
    /// Start the whole session over.
    Restart,
    /// The Enter key was pressed — submit or advance depending on state.
    Enter,
    /// A different UI theme was picked from the theme selector.
    ThemeSelected(iced::Theme),
    /// A link in some rendered Markdown was clicked; carries its target URL.
    LinkClicked(markdown::Uri),
}

pub struct App {
    /// All loaded scenarios — the source of every question.
    scenarios: Vec<QuestionScenario>,
    /// Flattened `(scenario index, question index)` pairs, in order. This is
    /// the sequence the user walks through; `current` indexes into it.
    order: Vec<(usize, usize)>,
    /// Index of the current question within `order`. Equal to `order.len()`
    /// once the session is complete.
    current_question_index: usize,
    /// The user's working answer for the current question.
    answer: Answer,
    /// `Some(is_correct)` once the current question has been graded, else
    /// `None` while it is still being answered.
    graded: Option<bool>,
    /// One entry per answered question over the course of the session:
    /// `true` = a point earned, `false` = a point missed. This is the
    /// failed-vs-correct tracker "over time".
    history: Vec<bool>,
    /// The currently selected UI theme, chosen via the theme picker.
    theme: iced::Theme,
}

impl App {
    pub fn new(mut scenarios: Vec<QuestionScenario>) -> Self {
        // Present scenarios in a random order each session.
        scenarios.shuffle(&mut rand::rng());

        let order = flatten(&scenarios);
        let answer = order
            .first()
            .map(|&(s, q)| fresh_answer(&scenarios[s].questions[q]))
            .unwrap_or(Answer::None);

        Self {
            scenarios,
            order,
            current_question_index: 0,
            answer,
            graded: None,
            history: Vec::new(),
            theme: iced::Theme::CatppuccinLatte,
        }
    }

    /// The theme currently driving the application's look.
    pub fn theme(&self) -> iced::Theme {
        self.theme.clone()
    }

    /// The scenario + question currently on screen, if any.
    fn current_question(&self) -> Option<(&QuestionScenario, &Question)> {
        let &(s, q) = self.order.get(self.current_question_index)?;
        let scenario = &self.scenarios[s];
        Some((scenario, &scenario.questions[q]))
    }

    /// Build a blank answer for whatever question is now current.
    fn fresh_answer_for_current(&self) -> Answer {
        match self.current_question() {
            Some((_, question)) => fresh_answer(question),
            None => Answer::None,
        }
    }

    /// Grade the working answer against the current question.
    fn check_current(&self) -> bool {
        let Some((_, question)) = self.current_question() else {
            return false;
        };
        match (question, &self.answer) {
            (
                Question::MultipleChoice {
                    correct_answers, ..
                },
                Answer::MultipleChoice { selected },
            ) => {
                let chosen: BTreeSet<usize> = selected
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &on)| on.then_some(i))
                    .collect();
                let expected: BTreeSet<usize> = correct_answers.iter().copied().collect();
                chosen == expected
            }
            (Question::OrderingTask { correct_order, .. }, Answer::Ordering { slots }) => {
                match slots.iter().copied().collect::<Option<Vec<usize>>>() {
                    Some(filled) => &filled == correct_order,
                    None => false, // an empty slot can never be correct
                }
            }
            _ => false,
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::ToggleOption(i) => {
                if self.graded.is_some() {
                    return; // locked after grading
                }
                if let Answer::MultipleChoice { selected } = &mut self.answer {
                    if let Some(flag) = selected.get_mut(i) {
                        *flag = !*flag;
                    }
                }
            }
            Message::SlotSelected(slot, option) => {
                if self.graded.is_some() {
                    return;
                }
                if let Answer::Ordering { slots } = &mut self.answer {
                    if let Some(cell) = slots.get_mut(slot) {
                        *cell = Some(option);
                    }
                }
            }
            Message::Submit => {
                if self.graded.is_some() {
                    return;
                }
                let correct = self.check_current();
                self.graded = Some(correct);
                self.history.push(correct);
            }
            Message::Next => {
                if self.graded.is_none() {
                    return; // must grade before advancing
                }
                self.current_question_index += 1;
                self.graded = None;
                self.answer = self.fresh_answer_for_current();
            }
            Message::Restart => {
                self.current_question_index = 0;
                self.history.clear();
                self.graded = None;
                self.answer = self.fresh_answer_for_current();
            }
            Message::Enter => {
                if self.current_question().is_none() {
                    return; // no active question (summary or empty screen)
                }
                // if we have clicked answers, but not yet clicked "submit"
                if self.graded.is_none() {
                    if self.answer.is_filled() {
                        self.update(Message::Submit);
                    }
                } else {
                    self.update(Message::Next);
                }
            }
            Message::ThemeSelected(theme) => {
                self.theme = theme;
            }
        }
    }

    /// Global keyboard handling: Enter submits the current answer, or advances
    /// to the next question once it has been graded.
    pub fn subscription(&self) -> Subscription<Message> {
        keyboard::listen().filter_map(|event| match event {
            keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Enter),
                ..
            } => Some(Message::Enter),
            _ => None,
        })
    }

    pub fn view(&self) -> Element<'_, Message> {
        let body = if self.order.is_empty() {
            self.view_empty()
        } else if self.current_question_index >= self.order.len() {
            self.view_summary()
        } else {
            self.view_question()
        };

        let content = column![self.view_header(), body].spacing(24).padding(24);
        scrollable(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Score line plus the ✓/✗ timeline of every answered question.
    fn view_header(&self) -> Element<'_, Message> {
        let correct = self.history.iter().filter(|&&ok| ok).count();
        let failed = self.history.len() - correct;
        let total = self.order.len();

        let score = text(format!(
            "Score: {correct} correct · {failed} failed — {} of {total} answered",
            self.history.len()
        ))
        .size(18);

        let mut marks: Row<'_, Message> = row![].spacing(4);
        for &ok in &self.history {
            let mark = if ok {
                text("✓").style(text::success)
            } else {
                text("✗").style(text::danger)
            };
            marks = marks.push(mark);
        }

        let theme_picker = pick_list(
            iced::Theme::ALL,
            Some(self.theme.clone()),
            Message::ThemeSelected,
        )
        .text_size(14);

        row![
            column![score, marks].spacing(8),
            Space::new().width(Length::Fill),
            theme_picker,
        ]
        .align_y(Alignment::Start)
        .into()
    }

    fn view_question(&self) -> Element<'_, Message> {
        let (scenario, question) = self.current_question().unwrap();
        let locked = self.graded.is_some();
        let mut items: Vec<Element<'_, Message>> = Vec::new();

        // Progress indicator.
        items.push(
            text(format!(
                "Question {} of {}",
                self.current_question_index + 1,
                self.order.len()
            ))
            .size(14)
            .style(text::secondary)
            .into(),
        );

        // Scenario context.
        if !scenario.scenario_description.is_empty() {
            items.push(
                container(text(scenario.scenario_description.as_str()).size(16))
                    .padding(12)
                    .width(Length::Fill)
                    .style(container::rounded_box)
                    .into(),
            );
        }
        if !scenario.tags.is_empty() {
            items.push(
                text(format!("Topics: {}", scenario.tags.join(", ")))
                    .size(12)
                    .style(text::secondary)
                    .into(),
            );
        }

        // The question itself and its input widgets.
        match (question, &self.answer) {
            (
                Question::MultipleChoice {
                    question_text,
                    options,
                    ..
                },
                Answer::MultipleChoice { selected },
            ) => {
                items.push(text(question_text.as_str()).size(22).into());
                for (i, option) in options.iter().enumerate() {
                    let mut cb = checkbox(selected[i]).label(option.as_str());
                    if !locked {
                        cb = cb.on_toggle(move |_| Message::ToggleOption(i));
                    }
                    items.push(cb.into());
                }
            }
            (
                Question::OrderingTask {
                    question_text,
                    options,
                    correct_order,
                    ..
                },
                Answer::Ordering { slots },
            ) => {
                items.push(text(question_text.as_str()).size(22).into());
                items.push(
                    text("Assign an option to each position:")
                        .size(14)
                        .style(text::secondary)
                        .into(),
                );
                let choices: Vec<Choice> = options
                    .iter()
                    .enumerate()
                    .map(|(index, label)| Choice {
                        index,
                        label: label.clone(),
                    })
                    .collect();
                for slot in 0..correct_order.len() {
                    let selected = slots[slot].map(|oi| choices[oi].clone());
                    let picker = pick_list(choices.clone(), selected, move |choice: Choice| {
                        Message::SlotSelected(slot, choice.index)
                    })
                    .placeholder("choose…");
                    items.push(
                        row![text(format!("{}.", slot + 1)).size(18).width(28), picker]
                            .spacing(10)
                            .align_y(Alignment::Center)
                            .into(),
                    );
                }
            }
            _ => {}
        }

        // Feedback + controls.
        items.push(Space::new().height(8).into());
        match self.graded {
            None => {
                items.push(button(text("Submit")).on_press(Message::Submit).into());
            }
            Some(correct) => {
                let feedback = if correct {
                    text("✓ Correct!").size(18).style(text::success)
                } else {
                    text("✗ Incorrect").size(18).style(text::danger)
                };
                let last = self.current_question_index + 1 >= self.order.len();
                let next_label = if last { "See results" } else { "Next question" };
                items.push(
                    row![
                        feedback,
                        Space::new().width(Length::Fill),
                        button(text(next_label)).on_press(Message::Next),
                    ]
                    .align_y(Alignment::Center)
                    .into(),
                );
                if !correct {
                    if let Some(reveal) = correct_answer_text(question) {
                        items.push(text(reveal).size(14).style(text::secondary).into());
                    }
                }
                // Show the explanation, if any, regardless of whether the
                // answer was right or wrong.
                if let Some(explanation) = question_explanation(question) {
                    items.push(
                        container(text(explanation).size(14))
                            .padding(12)
                            .width(Length::Fill)
                            .style(container::rounded_box)
                            .into(),
                    );
                }
            }
        }

        Column::with_children(items).spacing(14).into()
    }

    fn view_summary(&self) -> Element<'_, Message> {
        let correct = self.history.iter().filter(|&&ok| ok).count();
        let total = self.history.len();
        column![
            text("Session complete").size(28),
            text(format!("You scored {correct} out of {total} points.")).size(20),
            Space::new().height(8),
            button(text("Restart")).on_press(Message::Restart),
        ]
        .spacing(14)
        .into()
    }

    fn view_empty(&self) -> Element<'_, Message> {
        column![
            text("No questions found").size(24),
            text("Put scenario *.json files in the example-questions directory (or pass a directory as the first argument) and restart.")
                .size(16)
                .style(text::secondary),
        ]
        .spacing(12)
        .into()
    }
}

/// Flatten scenarios into an ordered list of `(scenario, question)` indices.
fn flatten(scenarios: &[QuestionScenario]) -> Vec<(usize, usize)> {
    scenarios
        .iter()
        .enumerate()
        .flat_map(|(s, scenario)| (0..scenario.questions.len()).map(move |q| (s, q)))
        .collect()
}

/// A blank working answer sized to the given question.
fn fresh_answer(question: &Question) -> Answer {
    match question {
        Question::MultipleChoice { options, .. } => Answer::MultipleChoice {
            selected: vec![false; options.len()],
        },
        Question::OrderingTask { correct_order, .. } => Answer::Ordering {
            slots: vec![None; correct_order.len()],
        },
    }
}

/// The optional explanation attached to a question, shown after grading
/// whether the answer was right or wrong.
fn question_explanation(question: &Question) -> Option<&str> {
    match question {
        Question::MultipleChoice { explanation, .. }
        | Question::OrderingTask { explanation, .. } => explanation.as_deref(),
    }
}

/// A human-readable description of the correct answer, shown when the user
/// answers incorrectly.
fn correct_answer_text(question: &Question) -> Option<String> {
    match question {
        Question::MultipleChoice {
            options,
            correct_answers,
            ..
        } => {
            let labels: Vec<&str> = correct_answers
                .iter()
                .filter_map(|&i| options.get(i).map(String::as_str))
                .collect();
            Some(format!(
                "Correct answer:{}",
                labels
                    .iter()
                    .map(|answer| format!("\n  · {answer}"))
                    .collect::<String>()
            ))
        }
        Question::OrderingTask {
            options,
            correct_order,
            ..
        } => {
            let labels: Vec<&str> = correct_order
                .iter()
                .filter_map(|&i| options.get(i).map(String::as_str))
                .collect();
            Some(format!(
                "Correct order:{}",
                labels
                    .iter()
                    .map(|answer| format!("\n  · {answer}"))
                    .collect::<String>()
            ))
        }
    }
}
