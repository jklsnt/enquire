use std::fmt::Display;
use unicode_segmentation::UnicodeSegmentation;

use termion::event::Key;

use crate::{
    answer::OptionAnswer,
    config::{self},
    error::{InquireError, InquireResult},
    renderer::Renderer,
    terminal::Terminal,
    utils::paginate,
};

pub struct CustomOption<T>
where
    T: Display + Clone,
{
    selected: bool,
    value: T,
}

pub type OptionGenerator<'a, T> =
    &'a mut dyn FnMut(&Option<String>, &Vec<CustomOption<T>>) -> Vec<CustomOption<T>>;

/// Presents a message to the user and a list of options for the user to choose from.
/// The user is able to choose multiple options.
pub struct InteractiveMultiSelect<'a, T>
where
    T: Display + Clone,
{
    /// Message to be presented to the user.
    pub message: &'a str,

    /// Function that generates options displayed to the user.
    pub option_generator: OptionGenerator<'a, T>,

    /// Help message to be presented to the user.
    pub help_message: Option<&'a str>,

    /// Page size of the options displayed to the user.
    pub page_size: usize,

    /// Whether vim mode is enabled. When enabled, the user can
    /// navigate through the options using hjkl.
    pub vim_mode: bool,

    /// Starting cursor index of the selection.
    pub starting_cursor: usize,

    /// Whether the current filter typed by the user is kept or cleaned after a selection is made.
    pub keep_filter: bool,
}

impl<'a, T> InteractiveMultiSelect<'a, T>
where
    T: Display + Clone,
{
    /// Default page size, equal to the global default page size [config::DEFAULT_PAGE_SIZE]
    pub const DEFAULT_PAGE_SIZE: usize = config::DEFAULT_PAGE_SIZE;
    /// Default value of vim mode, equal to the global default value [config::DEFAULT_PAGE_SIZE]
    pub const DEFAULT_VIM_MODE: bool = config::DEFAULT_VIM_MODE;
    /// Default starting cursor index.
    pub const DEFAULT_STARTING_CURSOR: usize = 0;
    /// Default behavior of keeping or cleaning the current filter value.
    pub const DEFAULT_KEEP_FILTER: bool = true;
    /// Default help message.
    pub const DEFAULT_HELP_MESSAGE: Option<&'a str> =
        Some("↑↓ to move, space to select one, → to all, ← to none, type to filter");

    /// Creates a [MultiSelect] with the provided message and options, along with default configuration values.
    pub fn new(message: &'a str, option_generator: OptionGenerator<'a, T>) -> Self {
        Self {
            message,
            option_generator,
            help_message: Self::DEFAULT_HELP_MESSAGE,
            page_size: Self::DEFAULT_PAGE_SIZE,
            vim_mode: Self::DEFAULT_VIM_MODE,
            starting_cursor: Self::DEFAULT_STARTING_CURSOR,
            keep_filter: Self::DEFAULT_KEEP_FILTER,
        }
    }

    /// Sets the help message of the prompt.
    pub fn with_help_message(mut self, message: &'a str) -> Self {
        self.help_message = Some(message);
        self
    }

    /// Removes the set help message.
    pub fn without_help_message(mut self) -> Self {
        self.help_message = None;
        self
    }

    /// Sets the page size.
    pub fn with_page_size(mut self, page_size: usize) -> Self {
        self.page_size = page_size;
        self
    }

    /// Enables or disabled vim_mode.
    pub fn with_vim_mode(mut self, vim_mode: bool) -> Self {
        self.vim_mode = vim_mode;
        self
    }

    /// Sets the keep filter behavior.
    pub fn with_keep_filter(mut self, keep_filter: bool) -> Self {
        self.keep_filter = keep_filter;
        self
    }

    /// Sets the starting cursor index.
    pub fn with_starting_cursor(mut self, starting_cursor: usize) -> Self {
        self.starting_cursor = starting_cursor;
        self
    }

    /// Parses the provided behavioral and rendering options and prompts
    /// the CLI user for input according to them.
    pub fn prompt(self) -> InquireResult<Vec<T>> {
        let terminal = Terminal::new()?;
        let mut renderer = Renderer::new(terminal)?;
        self.prompt_with_renderer(&mut renderer)
    }

    pub(in crate) fn prompt_with_renderer(self, renderer: &mut Renderer) -> InquireResult<Vec<T>> {
        InteractiveMultiSelectPrompt::new(self).prompt(renderer)
    }
}

struct InteractiveMultiSelectPrompt<'a, T>
where
    T: Display + Clone,
{
    message: &'a str,
    options: Vec<CustomOption<T>>,
    option_generator: OptionGenerator<'a, T>,
    help_message: Option<&'a str>,
    vim_mode: bool,
    cursor_index: usize,
    page_size: usize,
    keep_filter: bool,
    filter_value: Option<String>,
    error: Option<String>,
}

impl<'a, T> InteractiveMultiSelectPrompt<'a, T>
where
    T: Display + Clone,
{
    fn new(mso: InteractiveMultiSelect<'a, T>) -> Self {
        Self {
            message: mso.message,
            options: (mso.option_generator)(&None, &vec![]),
            option_generator: mso.option_generator,
            help_message: mso.help_message,
            vim_mode: mso.vim_mode,
            cursor_index: mso.starting_cursor,
            page_size: mso.page_size,
            keep_filter: mso.keep_filter,
            filter_value: None,
            error: None,
        }
    }

    fn move_cursor_up(&mut self) {
        self.cursor_index = self
            .cursor_index
            .checked_sub(1)
            .or(self.options.len().checked_sub(1))
            .unwrap_or_else(|| 0);
    }

    fn move_cursor_down(&mut self) {
        self.cursor_index = self.cursor_index.saturating_add(1);
        if self.cursor_index >= self.options.len() {
            self.cursor_index = 0;
        }
    }

    fn toggle_cursor_selection(&mut self) {
        let idx = match self.options.get_mut(self.cursor_index) {
            Some(val) => val,
            None => return,
        };

        idx.selected = !idx.selected;

        if !self.keep_filter {
            self.filter_value = None;
        }
    }

    fn on_change(&mut self, key: Key) {
        let mut dirty = false;

        match key {
            Key::Up => self.move_cursor_up(),
            Key::Char('k') if self.vim_mode => self.move_cursor_up(),
            Key::Char('\t') | Key::Down => self.move_cursor_down(),
            Key::Char('j') if self.vim_mode => self.move_cursor_down(),
            Key::Char(' ') => {
                self.toggle_cursor_selection();
                dirty = true;
            }
            Key::Char('\x17') | Key::Char('\x18') => {
                self.filter_value = None;
                dirty = true;
            }
            Key::Backspace => {
                if let Some(filter) = &self.filter_value {
                    let len = filter[..].graphemes(true).count();
                    let new_len = len.saturating_sub(1);
                    self.filter_value = Some(filter[..].graphemes(true).take(new_len).collect());
                    dirty = true;
                }
            }
            Key::Right => {
                for val in &mut self.options {
                    val.selected = true;
                }

                if !self.keep_filter {
                    self.filter_value = None;
                }

                dirty = true;
            }
            Key::Left => {
                for val in &mut self.options {
                    val.selected = false;
                }

                if !self.keep_filter {
                    self.filter_value = None;
                }

                dirty = true;
            }
            Key::Char(c) => {
                match &mut self.filter_value {
                    Some(val) => val.push(c),
                    None => self.filter_value = Some(String::from(c)),
                };
                dirty = true;
            }
            _ => {}
        }

        if dirty {
            self.options = (self.option_generator)(&self.filter_value, &self.options);
        }
    }

    fn get_final_answer(&self) -> Result<Vec<T>, String> {
        let selected_options = self
            .options
            .iter()
            .filter_map(|opt| match opt.selected {
                true => Some(opt.value.clone()),
                false => None,
            })
            .collect::<Vec<T>>();

        return Ok(selected_options);
    }

    fn render(&mut self, renderer: &mut Renderer) -> InquireResult<()> {
        let prompt = &self.message;

        renderer.reset_prompt()?;

        if let Some(err) = &self.error {
            renderer.print_error_message(err)?;
        }

        renderer.print_prompt(&prompt, None, self.filter_value.as_deref())?;

        let choices = self
            .options
            .iter()
            .enumerate()
            .map(|(idx, opt)| OptionAnswer::new(idx, &format!("{}", opt.value)))
            .collect::<Vec<OptionAnswer>>();

        let (paginated_opts, rel_sel) = paginate(self.page_size, &choices, self.cursor_index);

        for (idx, opt) in paginated_opts.iter().enumerate() {
            renderer.print_multi_option(
                rel_sel == idx,
                self.options.get(opt.index).unwrap().selected,
                &opt.value,
            )?;
        }

        if let Some(help_message) = self.help_message {
            renderer.print_help(help_message)?;
        }

        renderer.flush()?;

        Ok(())
    }

    fn prompt(mut self, renderer: &mut Renderer) -> InquireResult<Vec<T>> {
        let final_answer: Vec<T>;

        loop {
            self.render(renderer)?;

            let key = renderer.read_key()?;

            match key {
                Key::Ctrl('c') => return Err(InquireError::OperationCanceled),
                Key::Char('\n') | Key::Char('\r') => match self.get_final_answer() {
                    Ok(answer) => {
                        final_answer = answer;
                        break;
                    }
                    Err(err) => self.error = Some(err),
                },
                key => self.on_change(key),
            }
        }

        let formatted = final_answer
            .iter()
            .map(|o| format!("{}", o))
            .collect::<Vec<String>>()
            .join(", ");

        renderer.cleanup(&self.message, &formatted)?;

        Ok(final_answer)
    }
}
