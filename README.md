# ibuf - 高性能消息缓冲区库

ibuf是一个类似Linux mbuf的高性能消息缓冲区库，提供了线程安全的内存管理和高效的数据处理能力。

## 功能特性

- **MBuf**: 高性能消息缓冲区，支持动态扩容
- **MPool**: 线程安全的内存池实现，减少内存分配开销
- **零拷贝**: 支持直接访问底层数据，避免不必要的内存拷贝
- **线程安全**: 所有核心操作都是线程安全的

## 快速开始

### 添加依赖

在`Cargo.toml`中添加:

```toml
[dependencies]
ibuf = { git = "https://github.com/your-repo/ibuf.git" }
```

### 基本使用

```rust
use ibuf::{MBuf, MPool};

// 创建单个MBuf
let mut buf = MBuf::with_capacity(1024);
buf.append(b"hello world");

// 使用内存池
let pool = MPool::new(10, 1024); // 初始10个缓冲区，每个1024字节
let buf = pool.alloc();
// ... 使用缓冲区
pool.free(buf);
```

### 高级使用示例

#### 网络数据处理
```rust
use ibuf::{MBuf, MPool};

// 模拟接收网络数据包
fn handle_packet(pool: &MPool, packet_data: &[u8]) {
    // 从内存池分配缓冲区
    let mut buf = pool.alloc();
    
    // 写入数据
    buf.append(packet_data);
    
    // 处理数据...
    process_data(&buf);
    
    // 释放缓冲区
    pool.free(buf);
}

fn process_data(buf: &MBuf) {
    // 零拷贝访问数据
    let data: &[u8] = &buf;
    println!("Processing {} bytes of data", data.len());
}
```

#### 批量操作
```rust
use ibuf::{MBuf, MPool};

// 创建内存池
let pool = MPool::new(100, 4096); // 100个4KB缓冲区

// 批量分配
let mut buffers = Vec::new();
for _ in 0..10 {
    buffers.push(pool.alloc());
}

// 批量写入数据
for buf in &mut buffers {
    buf.append(b"batch data");
}

// 批量释放
for buf in buffers {
    pool.free(buf);
}
```

#### 数据流处理
```rust
use ibuf::{MBuf, MPool};
use std::io::{Read, Write};

let pool = MPool::new(5, 1024);

// 模拟数据流
let mut stream = pool.alloc();
stream.write_all(b"stream data part 1").unwrap();
stream.write_all(b"stream data part 2").unwrap();

// 读取流数据
let mut read_buf = [0u8; 32];
let bytes_read = stream.read(&mut read_buf).unwrap();
println!("Read {} bytes: {:?}", bytes_read, &read_buf[..bytes_read]);

pool.free(stream);
```

## 核心组件

### MBuf

- 动态扩容: 当空间不足时自动按1.5倍增长
- 引用计数: 支持多线程共享
- 零拷贝访问: 通过Deref直接访问底层数据

### MPool

- 线程安全: 使用Arc+Mutex实现线程安全
- 统计功能: 可查询已分配和空闲缓冲区数量
- 自动扩容: 当池为空时自动创建新缓冲区

## 性能建议

1. 对于频繁分配/释放的场景，优先使用MPool
2. 预估好初始容量避免频繁扩容
3. 大块数据操作时考虑使用零拷贝访问

## 许可证

MIT License