use std::collections::HashSet;

use inquire::OptionGenerator;

fn main() {
    let mut persisted_tags: HashSet<String> = ["travel", "food", "fuel", "medicine"]
        .iter()
        .cloned()
        .map(String::from)
        .collect();

    let mut temporary_tags: HashSet<String> = HashSet::new();

    let option_generator: OptionGenerator<String> = &mut |filter, state| {};

    // let _ans = InteractiveMultiSelect::new("Tags:", &options)
    //     .with_help_message("This is a custom help")
    //     .with_page_size(10)
    //     .with_validator(validator)
    //     .with_default(&default)
    //     .with_starting_cursor(1)
    //     .prompt()
    //     .expect("Failed when creating mso");
}
