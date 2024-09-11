#[derive(Debug, PartialEq)]
pub enum DebuggerCommand {
    BreakPoint(u32),
    Continue,
    Next,
    Quit,
}

#[derive(Debug, PartialEq)]
pub enum CommandParseError {
    NoCommandGiven,
    CommandNotRecognized(String),
    CommandMissingArguments(String),
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
                    Some(n) => n.parse(),
                    None => return Err(
                        CommandParseError::CommandMissingArguments("Break command missing instruction address".to_string())
                    ),
                };

                let point = match point {
                    Ok(n) => n,
                    Err(_) => return Err(
                        CommandParseError::CommandNotRecognized("Break command needs unsigned integer input".to_string())
                    )
                };

                DebuggerCommand::BreakPoint(point)
            },
            "c" | "continue" => DebuggerCommand::Continue,
            "n" | "next" => DebuggerCommand::Next,
            "q" | "quit" => DebuggerCommand::Quit,
            _ => return Err(CommandParseError::CommandNotRecognized(cmd.to_string())),
        };

        if cmd_iter.count() > 0 {
            Err(CommandParseError::CommandNotRecognized("Extra arguements not supported for command".to_string()))
        } else {
            Ok(debug_cmd)
        }
    }
}

mod test {
    use super::*;

    #[test]
    fn test_parse_break_full() {
        let cmd = DebuggerCommand::parse("break 32");
        assert_eq!(cmd, Ok(DebuggerCommand::BreakPoint(32)));
    }

    #[test]
    fn test_parse_break_short() {
        let cmd = DebuggerCommand::parse("b 32");
        assert_eq!(cmd, Ok(DebuggerCommand::BreakPoint(32)));
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
