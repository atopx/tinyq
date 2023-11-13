/// Error code definition,
/// with the first 4 bytes of the reply fixed as 'err_code'
pub const SUCCESS: [u8; 4] = 0_u32.to_be_bytes();
// Instruction parsing error.
pub const INS_PARSE_ERR: [u8; 4] = 40000_u32.to_be_bytes();
// Invalid instruction.
pub const INS_INVAL_ERR: [u8; 4] = 40010_u32.to_be_bytes();
// The instruction parameters are deficient, lacking the requisite 'body_size'.
pub const INS_PARAM_ERR: [u8; 4] = 40020_u32.to_be_bytes();
// An error occurred during the parsing of the 'body_size' parameter.
pub const BODY_SIZE_PARSE_ERR: [u8; 4] = 40100_u32.to_be_bytes();
// Invalid body_size, [0, MAX_BODY_SIZE]
pub const BODY_SIZE_INVAL_ERR: [u8; 4] = 40110_u32.to_be_bytes();
// Body parsing error.
pub const BODY_PARSE_ERROR: [u8; 4] = 40200_u32.to_be_bytes();
// The instruction parameters are deficient, lacking the requisite 'body'.
pub const BODY_PARAM_ERR: [u8; 4] = 40210_u32.to_be_bytes();
