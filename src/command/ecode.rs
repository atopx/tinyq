/// Error code definition,
/// with the first 4-byte of the reply fixed as 'err_code'
pub type ECode = u8;
// Default success
pub const SUCCESS: ECode = 0x00;
// Instruction parsing error.
pub const INS_PARSE_ERR: ECode = 0x10;
// Invalid instruction.
pub const INS_INVAL_ERR: ECode = 0x11;
// The instruction parameters are deficient, lacking the requisite 'body_size'.
pub const INS_PARAM_ERR: ECode = 0x12;
// An error occurred during the parsing of the 'body_size' parameter.
pub const BODY_SIZE_PARSE_ERR: ECode = 0x20;
// Invalid body_size, [0, MAX_BODY_SIZE]
pub const BODY_SIZE_INVAL_ERR: ECode = 0x21;
// Body parsing error.
pub const BODY_PARSE_ERROR: ECode = 0x30;
// The instruction parameters are deficient, lacking the requisite 'body'.
pub const BODY_PARAM_ERR: ECode = 0x31;
