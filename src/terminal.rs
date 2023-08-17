use crossterm::{
    execute,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
};

use crate::error;

#[derive(Clone, Copy)]
pub enum MessageType {
    Warning,
    Error,
    Note,
}

impl MessageType {
    fn get_color(self) -> Color {
        match self {
            Self::Warning => Color::Magenta,
            Self::Error => Color::Red,
            Self::Note => Color::Cyan,
        }
    }
    fn get_title(self) -> String {
        match self {
            Self::Warning => String::from("Warning"),
            Self::Error => String::from("Error"),
            Self::Note => String::from("Note"),
        }
    }
}

pub fn print(r#type: MessageType, msg: String) -> error::Result<()> {
    execute!(
        std::io::stdout(),
        SetAttribute(Attribute::Bold),
        SetForegroundColor(r#type.get_color()),
        Print(r#type.get_title()),
        SetForegroundColor(Color::White),
        Print(": ".to_string()),
        Print(msg),
        ResetColor,
    )?;

    Ok(())
}
