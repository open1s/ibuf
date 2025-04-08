// MIT License
//
// Copyright (c) 2023 gaosg
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use super::MBuf;

/// MBuf池结构体
pub struct MPool {
    free_list: Mutex<Vec<MBuf>>,
    allocated_count: AtomicUsize,
    capacity: usize,
}

impl MPool {
    /// 创建指定容量和初始大小的MBuf池
    pub fn new(initial_size: usize, capacity: usize) -> Self {
        let mut free_list = Vec::with_capacity(initial_size);
        for _ in 0..initial_size {
            free_list.push(MBuf::with_capacity(capacity));
        }

        Self {
            free_list: Mutex::new(free_list),
            allocated_count: AtomicUsize::new(0),
            capacity,
        }
    }

    /// 从池中分配一个MBuf
    pub fn alloc(&self) -> MBuf {
        let mut free_list = self.free_list.lock().unwrap();
        self.allocated_count.fetch_add(1, Ordering::Relaxed);

        if let Some(buf) = free_list.pop() {
            return buf;
        }

        // 如果空闲列表为空，创建新的MBuf
        MBuf::with_capacity(self.capacity)
    }

    /// 将MBuf释放回池中
    pub fn free(&self, buf: MBuf) {
        let mut free_list = self.free_list.lock().unwrap();
        free_list.push(buf);
        self.allocated_count.fetch_sub(1, Ordering::Relaxed);
    }

    /// 获取当前分配的MBuf数量
    pub fn allocated_count(&self) -> usize {
        self.allocated_count.load(Ordering::Relaxed)
    }

    /// 获取当前空闲的MBuf数量
    pub fn free_count(&self) -> usize {
        let a =self.free_list.lock().unwrap().len();
        println!("free_count: {}", a);
        a
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mbuf_pool() {
        let pool = MPool::new(2, 1024);
        assert_eq!(pool.free_count(), 2);
        assert_eq!(pool.allocated_count(), 0);

        let buf1 = pool.alloc();
        assert_eq!(pool.free_count(), 1);
        assert_eq!(pool.allocated_count(), 1);

        let buf2 = pool.alloc();
        assert_eq!(pool.free_count(), 0);
        assert_eq!(pool.allocated_count(), 2);

        pool.free(buf1);
        assert_eq!(pool.free_count(), 1);
        assert_eq!(pool.allocated_count(), 1);

        pool.free(buf2);
        assert_eq!(pool.free_count(), 2);
        assert_eq!(pool.allocated_count(), 0);
    }
}