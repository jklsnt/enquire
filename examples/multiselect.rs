use inquire::{
    formatter::MultiOptionFormatter,
    type_aliases::{DynamicOption, DynamicOptionCondition, OptionCreator},
    validator::{MultiOptionValidator, Validation},
    MultiSelect,
};

fn main() {
    let options = vec![
        "Banana",
        "Apple",
        "Strawberry",
        "Grapes",
        "Lemon",
        "Tangerine",
        "Watermelon",
        "Orange",
        "Pear",
        "Avocado",
        "Pineapple",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    let validator: MultiOptionValidator<String> = &|a| {
        if a.len() < 2 {
            return Ok(Validation::Invalid("This list is too small!".into()));
        }

        let x = a.iter().any(|o| *o.value == "Pineapple");

        match x {
            true => Ok(Validation::Valid),
            false => Ok(Validation::Invalid("Remember to buy pineapples".into())),
        }
    };

    let dyn_option_condition: DynamicOptionCondition<String> = &|inp, opts| {
        if inp.is_empty() || opts.iter().any(|opt| opt.value == inp) {
            Ok(false)
        } else {
            Ok(true)
        }
    };

    let dyn_option_creator: OptionCreator<String> = &|inp| Ok(inp.to_string());

    let formatter: MultiOptionFormatter<String> = &|a| format!("{} different fruits", a.len());

    let ans = MultiSelect::new("Select the fruits for your shopping list:", options)
        .with_validator(validator)
        .with_formatter(formatter)
        .with_dynamic_option(DynamicOption::Enabled(
            dyn_option_condition,
            dyn_option_creator,
        ))
        .prompt();

    match ans {
        Ok(_) => println!("I'll get right on it"),
        Err(_) => println!("The shopping list could not be processed"),
    }
}
