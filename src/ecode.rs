use std::fmt::Display;

/// Error code definition,
/// with the first 4-byte of the reply fixed as 'err_code'

#[derive(Debug, Default)]
pub enum ECode {
    // Default success
    #[default]
    Success = 0,
    // Instruction parsing error
    CmdParasErr = 10,
    // Invalid command
    CmdInvalErr = 11,
    // An error occurred during the parsing of the 'body_size' parameter
    BodySizeParseErr = 20,
    // Invalid body_size, [0, MAX_BODY_SIZE]
    BodySizeInvalErr = 21,
    // Body parsing error
    BodyParseErr = 30,
    // The command parameters are deficient, lacking the requisite 'body'
    BodyInvalErr = 31,
    // Auth password failed
    AuthErr = 40,
    // Auth password timeout
    AuthTimeout = 41,
    // Server internal panic
    ServerInternalErr = 50,
    // Too many connections to the server.
    ServerBusy = 51,
    // Auth information pertaining to one's identity.
    InputPassword = 100,
}

impl Display for ECode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self))
    }
}

impl ECode {
    pub fn to_byte(self) -> u8 {
        self as u8
    }
}

/// Error returned by most functions.
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// A specialized `Result` type for mini-redis operations.
pub type Result<T> = std::result::Result<T, ECode>;
