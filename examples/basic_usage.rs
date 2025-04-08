use ibuf::{MBuf, MPool};

fn main() {
    // 基本MBuf使用示例
    let mut buf = MBuf::with_capacity(1024);
    buf.append(b"hello world");
    println!("Basic MBuf usage: {:?}", &buf[..]);
    
    // 内存池使用示例
    let pool = MPool::new(10, 1024);
    let mut buf = pool.alloc();
    buf.append(b"pool allocated data");
    println!("MPool usage: {:?}", &buf[..]);
    pool.free(buf);
    
    // 网络数据处理示例
    let packet_data = b"network packet data";
    handle_packet(&pool, packet_data);
}

fn handle_packet(pool: &MPool, packet_data: &[u8]) {
    let mut buf = pool.alloc();
    buf.append(packet_data);
    process_data(&buf);
    pool.free(buf);
}

fn process_data(buf: &MBuf) {
    println!("Processing packet: {:?}", &buf[..]);
}