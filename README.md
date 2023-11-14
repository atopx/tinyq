# TINYQ

- light, light, light
- fast, fast, fast
- simple, simple, simple

# Develop

## Protocol

![](./docs/protocol.jpg)

### Input: INSTRUCTION BODY_SIZE BODY

- INSTRUCTION: Type is uint8(fixed 1-byte), refer to [Instruction Definition](./src/instruction/mod.rs)
- BODY_SIZE: Type is uint32(fixed 4-byte), Byte length of the body, [0, MAX_BODY_SIZE]
- BODY: Content of the instruction parameters

### Output: CODE [DATA]

- CODE: Type is uint8(fixed 1-byte), refer to [Ecode Definition](./src/instruction/ecode.rs)
- DATA_SIZE: Type is uint32(fixed 4-byte)
- DATA: Returned based on the instruction and status code (has a value for data_size > 0)
