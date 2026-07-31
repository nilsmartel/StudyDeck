/// Module containg datastructures of actual questions.

pub struct QuestionScenario {
    /// Introductory text, because one QuestionScenario can accomodate multiple questions.
    /// May be empty, if the question description already suffices
    scenario_description: String,
    questions: Vec<Question>,
    /// optional tag to identify questions by examen topic
    tags: Vec<String>,
}

pub enum Question {
    /// Offer multiple options to answer from, only some are correct.
    /// Can represent yes and no questions.
    MultipleChoice {
        /// Description of the question itself.
        question_text: String,
        /// Text of options to choose from.
        options: Vec<String>,
        /// Indexes of correct answers. For some question more than one option may be correct.
        correct_answers: Vec<usize>,
    },
    /// A Question where you are presented ith multiple blocks / options that need to be sorted into the
    /// correct order. It may be, that more options exist than slots they can be sorted into.
    /// In other word, false herrings are allowed in the options.
    OrderingTask {
        /// Description of the question itself.
        question_text: String,
        /// Options
        options: Vec<String>,
        /// The correct order of items. The number of available slots to sort the options into is
        /// derived by the length of this array.
        correct_order: Vec<usize>,
    },
}
