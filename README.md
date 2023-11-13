# TINYQ 预期

1. 轻轻轻/快快快/简单

# Develop

## Protocol
tip: parameters separated by 0x20

### Input: INSTRUCTION BODY_SIZE BODY
> example: `sub 11 first_topic`
- INSTRUCTION: Instruction, fixed at 3 bytes, refer to [Instruction Definition](./src/instruction.rs)
- BODY_SIZE: Byte length of the body, [0, MAX_BODY_SIZE]
- BODY: Content of the instruction parameters

### Output: CODE [DATA]
> example: `0 f42bb9c2-a5e5-49f8-977a-8ff91bf239f3`
- CODE: Fixed 4-byte status code, refer to [Ecode Definition](./src/ecode.rs)
- DATA: Returned based on the instruction and status code (has a value for instructions requiring content - and without errors)


# usage

### server
- simple start
./tinyq --bind=0.0.0.0:7800 --auth admin:123456 --enable-tls --debug

- config start
./tinyq --config config.toml

### client
```go
import (
  "github.com/xxx/tinyq"
)

func connection() *tinyq.Client {
// connection and auth
  client, err := tinyq.NewClient(&config)
  if err != nil {
    panic(err)
  }
  return client
}

func pub(client *tinyq.Client, data any) {
  body, _ := json.Marshal(data)
  messageId, err := client.Pub("xxx", body, tinyq.QueueMode)
  // ...
}

func sub(client *tinyq.Client) {
  channel, err := tinyq.NewChannel("client-hostname", "namespace")
  if err != nil {
    panic(err)
  }
  err = client.Sub("xxx", channel)
  for message := range channel.Q {
    // ...
  }
}
```
