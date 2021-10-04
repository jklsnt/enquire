//! Utilities used to wrap user selections in [Select](crate::Select) and
//! [`MultiSelect`](crate::MultiSelect) prompts.

use std::fmt::{self, Display};

/// Represents a selection made by the user when prompted to select one or several
/// options among those presented.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectedOption<T> {
    /// Index of the selected option relative to the original (full) list passed to the prompt.
    pub index: usize,

    /// Value of the selected option.
    pub value: T,
}

impl<T> SelectedOption<T> {
    /// Constructor for `SelectedOption`.
    ///
    /// # Arguments
    ///
    /// * `index` - Index of the option.
    /// * `value` - String value of the option
    ///
    /// # Examples
    ///
    /// ```
    /// use inquire::selected_option::SelectedOption;
    ///
    /// let answer = SelectedOption::new(0, "a");
    /// ```
    pub fn new(index: usize, value: T) -> Self {
        Self { index, value }
    }

    /// Converts from `&SelectedOption<T>` to `SelectedOption<&T>`.
    pub fn as_ref(&self) -> SelectedOption<&T> {
        SelectedOption::new(self.index, &self.value)
    }

    #[allow(unused)]
    pub(in crate) fn from_list(vals: Vec<T>) -> Vec<SelectedOption<T>> {
        vals.into_iter()
            .enumerate()
            .map(|(index, value)| Self { index, value })
            .collect()
    }

    #[allow(unused)]
    pub(in crate) fn from_enumerated_list(vals: Vec<(usize, T)>) -> Vec<SelectedOption<T>> {
        vals.into_iter()
            .map(|(index, value)| Self { index, value })
            .collect()
    }
}

impl<T> fmt::Display for SelectedOption<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.value.fmt(f)
    }
}
