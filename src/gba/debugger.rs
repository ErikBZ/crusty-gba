use core::fmt;
use std::str::SplitWhitespace;
use tracing_subscriber::filter::LevelFilter;

#[derive(Debug, PartialEq)]
pub enum DebuggerCommand {
    BreakPoint(usize),
    Continue(ContinueSubcommand),
    Info,
    ReadMem(usize),
    WriteMem(u32, usize),
    LogLevel(LevelFilter),
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
    // TODO: Add reset
    pub fn parse(command: &str) -> Result<DebuggerCommand, CommandParseError> {
        let mut cmd_iter = command.split_whitespace();
        let cmd = match cmd_iter.next() {
            Some(l) => l,
            None => return Err(CommandParseError::NoCommandGiven),
        };

        let debug_cmd = match cmd {
            "b" | "break" => {
                let inst_addr = parse_number(&mut cmd_iter, cmd, 16)?;
                DebuggerCommand::BreakPoint(inst_addr as usize)
            }
            "w" | "write" => {
                let data = parse_number(&mut cmd_iter, command, 16)?;
                let addr = parse_number(&mut cmd_iter, command, 16)?;
                DebuggerCommand::WriteMem(data, addr as usize)
            }
            "r" | "read" => {
                let addr = parse_number(&mut cmd_iter, command, 16)?;
                DebuggerCommand::ReadMem(addr as usize)
            }
            "c" | "continue" => {
                let lines: Result<u32, _> = match cmd_iter.next() {
                    Some(s) => s.parse::<u32>(),
                    None => return Ok(DebuggerCommand::Continue(ContinueSubcommand::Endless)),
                };

                let lines = match lines {
                    Ok(n) => n,
                    Err(_) => {
                        return Err(CommandParseError::CommandNotRecognized(command.to_string()))
                    }
                };

                DebuggerCommand::Continue(ContinueSubcommand::For(lines as usize))
            }
            "l" | "log_level" => {
                if let Some(l) = cmd_iter.next() {
                    match l {
                        "error" => DebuggerCommand::LogLevel(LevelFilter::ERROR),
                        "warn" => DebuggerCommand::LogLevel(LevelFilter::WARN),
                        "info" => DebuggerCommand::LogLevel(LevelFilter::INFO),
                        "debug" => DebuggerCommand::LogLevel(LevelFilter::DEBUG),
                        "trace" => DebuggerCommand::LogLevel(LevelFilter::TRACE),
                        "off" => DebuggerCommand::LogLevel(LevelFilter::OFF),
                        _ => {
                            return Err(CommandParseError::CommandNotRecognized(
                                command.to_string(),
                            ))
                        }
                    }
                } else {
                    return Err(CommandParseError::CommandMissingArguments(
                        command.to_string(),
                    ));
                }
            }
            "i" | "info" => DebuggerCommand::Info,
            "n" | "next" => DebuggerCommand::Next,
            "q" | "quit" => DebuggerCommand::Quit,
            _ => return Err(CommandParseError::CommandNotRecognized(command.to_string())),
        };

        if cmd_iter.count() > 0 {
            Err(CommandParseError::CommandNotRecognized(
                "Extra arguements not supported for command".to_string(),
            ))
        } else {
            Ok(debug_cmd)
        }
    }
}

fn parse_number(
    cmd_iter: &mut SplitWhitespace,
    cmd: &str,
    radix: u32,
) -> Result<u32, CommandParseError> {
    let point: Result<u32, _> = match cmd_iter.next() {
        Some(s) => u32::from_str_radix(s, radix),
        None => return Err(CommandParseError::CommandMissingArguments(cmd.to_string())),
    };

    let point = match point {
        Ok(n) => n,
        Err(_) => return Err(CommandParseError::CommandNotRecognized(cmd.to_string())),
    };
    Ok(point)
}

#[derive(PartialEq, Debug)]
pub enum ContinueSubcommand {
    Endless,
    For(usize),
}

mod test {
    #![allow(unused)]
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
        assert_eq!(
            cmd,
            Ok(DebuggerCommand::Continue(ContinueSubcommand::Endless))
        );
    }

    #[test]
    fn test_parse_next() {
        let cmd = DebuggerCommand::parse("n");
        assert_eq!(cmd, Ok(DebuggerCommand::Next));
    }
}
