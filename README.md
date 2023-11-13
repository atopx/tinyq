# TINYQ 预期

1. 轻轻轻/快快快/简单


## 使用

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
