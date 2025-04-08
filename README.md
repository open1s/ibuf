# ibuf - High-performance Message Buffer Library

ibuf is a high-performance message buffer library similar to Linux mbuf, providing thread-safe memory management and efficient data processing capabilities.

## Features

- **MBuf**: High-performance message buffer with dynamic expansion
- **MPool**: Thread-safe memory pool implementation to reduce memory allocation overhead
- **Zero-copy**: Supports direct access to underlying data, avoiding unnecessary memory copies
- **Thread-safe**: All core operations are thread-safe

## Quick Start

### Add Dependency

Add to `Cargo.toml`:

```toml
[dependencies]
ibuf = { version = "0.2" }
```

### Basic Usage

```rust
use ibuf::{MBuf, MPool};

// Create a single MBuf
let mut buf = MBuf::with_capacity(1024);
buf.append(b"hello world");

// Use memory pool
let pool = MPool::new(10, 1024); // Initial 10 buffers, each 1024 bytes
let buf = pool.alloc();
// ... Use the buffer
pool.free(buf);
```

### Advanced Usage Examples

#### Network Data Processing
```rust
use ibuf::{MBuf, MPool};

// Simulate receiving network packets
fn handle_packet(pool: &MPool, packet_data: &[u8]) {
    // Allocate buffer from pool
    let mut buf = pool.alloc();
    
    // Write data
    buf.append(packet_data);
    
    // Process data...
    process_data(&buf);
    
    // Release buffer
    pool.free(buf);
}

fn process_data(buf: &MBuf) {
    // Zero-copy data access
    let data: &[u8] = &buf;
    println!("Processing {} bytes of data", data.len());
}
```

#### Batch Operations
```rust
use ibuf::{MBuf, MPool};

// Create memory pool
let pool = MPool::new(100, 4096); // 100 buffers of 4KB each

// Batch allocation
let mut buffers = Vec::new();
for _ in 0..10 {
    buffers.push(pool.alloc());
}

// Batch write data
for buf in &mut buffers {
    buf.append(b"batch data");
}

// Batch release
for buf in buffers {
    pool.free(buf);
}
```

#### Stream Processing
```rust
use ibuf::{MBuf, MPool};
use std::io::{Read, Write};

let pool = MPool::new(5, 1024);

// Simulate data stream
let mut stream = pool.alloc();
stream.write_all(b"stream data part 1").unwrap();
stream.write_all(b"stream data part 2").unwrap();

// Read stream data
let mut read_buf = [0u8; 32];
let bytes_read = stream.read(&mut read_buf).unwrap();
println!("Read {} bytes: {:?}", bytes_read, &read_buf[..bytes_read]);

pool.free(stream);
```

## Core Components

### MBuf

- Dynamic expansion: Automatically grows by 1.5x when space is insufficient
- Reference counting: Supports multi-thread sharing
- Zero-copy access: Direct access to underlying data via Deref

### MPool

- Thread-safe: Implemented with Arc+Mutex
- Statistics: Can query allocated and free buffer counts
- Auto-expansion: Creates new buffers when pool is empty

## Performance Recommendations

1. Prefer MPool for frequent allocation/release scenarios
2. Estimate initial capacity to avoid frequent expansions
3. Consider zero-copy access for large data operations

## License

MIT License