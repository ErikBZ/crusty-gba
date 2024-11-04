use core::fmt;
use std::str::SplitWhitespace;

#[derive(Debug, PartialEq)]
pub enum DebuggerCommand {
    BreakPoint(usize),
    Continue(ContinueSubcommand),
    Info,
    ReadMem(usize),
    Next,
    Quit,
}



#[derive(Debug, PartialEq, Clone)]
pub enum CommandParseError {
    NoCommandGiven,
    CommandNotRecognized(String),
    CommandMissingArguments(String),
}

impl fmt::Display for CommandParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NoCommandGiven => write!(f, ""),
            Self::CommandNotRecognized(s) => write!(f, "Command not recognized: {s}"),
            Self::CommandMissingArguments(s) => write!(f, "Command missing arguements: {s}"),
        }
    }
}

impl DebuggerCommand {
    pub fn parse(command: &str) -> Result<DebuggerCommand, CommandParseError> {
        let mut cmd_iter = command.split_whitespace();
        let cmd = match cmd_iter.next() {
            Some(l) => l,
            None => return Err(CommandParseError::NoCommandGiven),
        };

        let debug_cmd = match cmd {
            "b" | "break" => {
                let point: Result<u32, _> = match cmd_iter.next() {
                    Some(s) => u32::from_str_radix(s, 16),
                    None => return Err(
                        CommandParseError::CommandMissingArguments(command.to_string())
                    ),
                };

                let point = match point {
                    Ok(n) => n,
                    Err(_) => return Err(
                        CommandParseError::CommandNotRecognized(command.to_string())
                    )
                };

                DebuggerCommand::BreakPoint(point as usize)
            },
            "r" | "read" => {
                let point: Result<u32, _> = match cmd_iter.next() {
                    Some(s) => u32::from_str_radix(s, 16),
                    None => return Err(
                        CommandParseError::CommandMissingArguments(command.to_string())
                    ),
                };

                let point = match point {
                    Ok(n) => n,
                    Err(_) => return Err(
                        CommandParseError::CommandNotRecognized(command.to_string())
                    )
                };
                DebuggerCommand::ReadMem(point as usize)
            }
            "c" | "continue" => {
                let lines: Result<u32, _> = match cmd_iter.next() {
                    Some(s) => u32::from_str_radix(s, 10),
                    None => return Ok(DebuggerCommand::Continue(ContinueSubcommand::Endless))
                };
                
                let lines = match lines {
                    Ok(n) => n,
                    Err(_) => return Err(
                        CommandParseError::CommandNotRecognized(command.to_string())
                    )
                };

                DebuggerCommand::Continue(ContinueSubcommand::For(lines as usize))
            }
            "i" | "info" => DebuggerCommand::Info,
            "n" | "next" => DebuggerCommand::Next,
            "q" | "quit" => DebuggerCommand::Quit,
            _ => return Err(CommandParseError::CommandNotRecognized(command.to_string())),
        };

        if cmd_iter.count() > 0 {
            Err(CommandParseError::CommandNotRecognized("Extra arguements not supported for command".to_string()))
        } else {
            Ok(debug_cmd)
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum ContinueSubcommand {
    Endless,
    For(usize)
}

mod test {
    use super::*;

    #[test]
    fn test_parse_break_full() {
        let cmd = DebuggerCommand::parse("break 32");
        assert_eq!(cmd, Ok(DebuggerCommand::BreakPoint(50)));
    }

    #[test]
    fn test_parse_break_short() {
        let cmd = DebuggerCommand::parse("b 32");
        assert_eq!(cmd, Ok(DebuggerCommand::BreakPoint(50)));
    }

    #[test]
    fn test_parse_continue() {
        let cmd = DebuggerCommand::parse("continue");
        assert_eq!(cmd, Ok(DebuggerCommand::Continue));
    }

    #[test]
    fn test_parse_next() {
        let cmd = DebuggerCommand::parse("n");
        assert_eq!(cmd, Ok(DebuggerCommand::Next));
    }
}
