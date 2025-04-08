use ibuf::{MBuf, MPool};
use std::sync::Arc;
use std::thread;

fn main() {
    // 并发处理示例
    concurrent_processing();
    
    // 大数据流操作示例
    large_data_stream();
}

/// 并发处理示例：多个线程共享内存池处理数据
fn concurrent_processing() {
    // 创建共享内存池(线程安全)
    let pool = Arc::new(MPool::new(100, 4096)); // 100个4KB缓冲区
    
    let mut handles = vec![];
    
    // 创建10个工作线程
    for i in 0..10 {
        let pool = pool.clone();
        
        handles.push(thread::spawn(move || {
            // 从内存池分配缓冲区
            let mut buf = pool.alloc();
            
            // 写入线程特定数据
            buf.append(format!("Thread {} data", i).as_bytes());
            
            // 模拟数据处理
            thread::sleep(std::time::Duration::from_millis(100));
            
            // 释放缓冲区
            pool.free(buf);
        }));
    }
    
    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("并发处理完成");
}

/// 大数据流操作示例：处理超过单个缓冲区容量的数据
fn large_data_stream() {
    // 创建内存池
    let pool = MPool::new(10, 1024); // 10个1KB缓冲区
    
    // 模拟大数据流(5MB)
    let large_data = vec![b'A'; 5 * 1024 * 1024];
    
    // 记录处理时间
    let start = std::time::Instant::now();
    
    // 分块处理数据
    let mut processed = 0;
    while processed < large_data.len() {
        // 计算本次处理的数据块大小
        let chunk_size = std::cmp::min(1024, large_data.len() - processed);
        let chunk = &large_data[processed..processed + chunk_size];
        
        // 从内存池分配缓冲区
        let mut buf = pool.alloc();
        
        // 写入数据
        buf.append(chunk);
        
        // 模拟数据处理
        process_large_data(&buf);
        
        // 释放缓冲区
        pool.free(buf);
        
        processed += chunk_size;
    }
    
    println!("处理完成 {} bytes, 耗时 {:?}", 
        large_data.len(), 
        start.elapsed());
}

fn process_large_data(buf: &MBuf) {
    // 这里可以添加实际的数据处理逻辑
    // 示例中只是简单计算数据长度
    let _len = buf.len();
}