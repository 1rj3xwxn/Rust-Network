# WebSocket Chat

一个使用 Rust、Tokio 和 Axum 构建的最小 WebSocket 群聊示例。

本示例接在 `03-axum-server` 后面，重点展示：

- HTTP Upgrade 握手与 WebSocket 连接生命周期
- 一个客户端连接对应一个 Tokio Task
- 使用 `tokio::sync::broadcast` 在多个连接之间广播消息
- 使用 `tokio::select!` 同时等待客户端输入和广播消息
- 使用 `StreamExt::split` 将 WebSocket 读写方向拆开

## 运行

在本目录执行：

```bash
cargo run
```

服务监听：

```text
127.0.0.1:8080
```

访问 `http://127.0.0.1:8080/` 可以确认 HTTP 服务正在运行。WebSocket 地址为：

```text
ws://127.0.0.1:8080/ws
```

## 测试

可以使用浏览器开发者工具 Console：

```javascript
const ws = new WebSocket("ws://127.0.0.1:8080/ws");

ws.onopen = () => {
  console.log("connected");
  ws.send("Hello from browser");
};

ws.onmessage = (event) => {
  console.log("message:", event.data);
};

ws.onclose = () => console.log("closed");
ws.onerror = (error) => console.error("WebSocket error:", error);
```

打开两个浏览器标签页，分别执行上面的代码。在一个标签页发送消息，另一个标签页也会收到广播。当前示例会把消息广播回发送者自己。

也可以用支持 WebSocket 的客户端工具连接 `ws://127.0.0.1:8080/ws`。

## 项目结构

```text
04-websocket-chat/
├── Cargo.toml
├── README.md
└── src/
    └── main.rs
```

## 消息流

1. `main` 创建一个共享的 `broadcast::Sender<String>`。
2. Axum 通过 `State` 将 Sender 共享给每次 WebSocket 握手处理函数。
3. 握手成功后，`handle_socket` 开始处理这个连接。
4. 每个连接调用 `tx.subscribe()`，得到自己的 Receiver。
5. 客户端发来的文本消息通过 `tx.send()` 发布到广播通道。
6. 每个连接 Task 的 `rx.recv()` 收到消息，再通过自己的 WebSocket Sink 发回客户端。

## 核心设计说明

### 为什么每个连接都调用 subscribe？

广播通道有一个共享发送端，但每个订阅者需要独立的接收端和读取进度。A、B、C 各自持有自己的 Receiver，因此不需要互相持有对方的 Socket。

### 为什么 split WebSocket？

一个连接需要同时处理两个方向：

- 从客户端读取消息
- 向客户端写入广播消息

`socket.split()` 将它拆成 Stream 和 Sink。这样 `tokio::select!` 可以分别等待读取事件和广播事件，同时避免在两个分支中争用同一个可变借用。

### broadcast 的容量与 Lagged

通道容量是 100。它是有界缓冲区，不是永久消息队列。如果某个 Receiver 太慢，旧消息可能被覆盖，接收端会收到 `RecvError::Lagged(n)`，其中 `n` 是跳过的消息数。

### 为什么发送者也会收到自己的消息？

所有活跃订阅者都会收到发布的消息，包含发送者自己的 Receiver。真实聊天室可以给消息附加发送者 ID，再决定是否回显。

## 当前示例的边界

这是用于理解架构的教学项目，不是生产级聊天室：

- 只广播文本消息
- 没有用户身份、房间和权限管理
- 没有消息持久化
- 没有限流、最大消息大小和慢客户端隔离策略
- 没有完整的心跳、重连和关闭管理
- 广播缓冲区发生 Lagged 时只记录日志，不做历史消息补偿

生产系统需要根据业务要求补齐这些机制。

## 建议阅读顺序

1. 从 `main` 看共享广播通道如何创建。
2. 看 `ws_handler` 如何完成 HTTP Upgrade。
3. 看 `handle_socket` 如何为每个连接订阅。
4. 分别追踪 `receiver.next()` 和 `rx.recv()` 两个分支。
5. 最后研究 `split`、`select!` 和 `Lagged` 的行为。
