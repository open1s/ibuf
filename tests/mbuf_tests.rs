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

use ibuf::{MBuf, Cursor, MPool};
use std::io::{Read, Write};
use std::sync::Arc;
use std::thread;

#[test]
fn test_mbuf_basic_operations() {
    let mut buf = MBuf::with_capacity(1024);
    assert_eq!(buf.len(), 0);
    assert_eq!(buf.capacity(), 1024);
    
    // Test append
    let data = b"hello world";
    let copied = buf.append(data);
    assert_eq!(copied, data.len());
    assert_eq!(buf.len(), data.len());
    
    // Test deref
    let slice: &[u8] = &buf;
    assert_eq!(slice, data);
    
    // Test clone
    let buf2 = buf.clone();
    assert_eq!(buf2.len(), data.len());
    assert_eq!(&*buf2, data);
}

#[test]
fn test_mbuf_resize() {
    let mut buf = MBuf::with_capacity(10);
    let data = b"this is a long string that will trigger resize";
    
    let copied = buf.append(data);
    assert_eq!(copied, data.len());
    assert!(buf.capacity() >= data.len());
    assert_eq!(&*buf, data);
}

#[test]
fn test_mbuf_read_write() {
    let mut buf = MBuf::with_capacity(1024);
    
    // Test Write trait
    let data = b"test data";
    let written = buf.write(data).unwrap();
    assert_eq!(written, data.len());
    assert_eq!(buf.len(), data.len());
    
    // Test Read trait
    let mut read_buf = [0u8; 9];
    let read_len = buf.read(&mut read_buf).unwrap();
    assert_eq!(read_len, data.len());
    assert_eq!(&read_buf, data);
    assert_eq!(buf.len(), 0);
}

#[test]
fn test_cursor_operations() {
    let mut buf = MBuf::with_capacity(1024);
    let data = b"cursor test data";
    buf.append(data);
    
    let mut cursor = Cursor::new(&buf);
    
    // Test next
    assert_eq!(cursor.next(), Some(b'c'));
    assert_eq!(cursor.position(), 1);
    
    // Test next_slice
    assert_eq!(cursor.next_slice(6), Some(b"ursor ".as_ref()));
    assert_eq!(cursor.position(), 7);
    
    // Test reset
    cursor.reset();
    assert_eq!(cursor.position(), 0);
    assert_eq!(cursor.next_slice(data.len()), Some(data.as_ref()));
}

#[test]
fn test_mbuf_thread_safety() {
    let buf = Arc::new(MBuf::with_capacity(1024));
    let data = b"thread test";
    
    let handles: Vec<_> = (0..4).map(|_| {
        let buf = Arc::clone(&buf);
        let data = data.clone();
        thread::spawn(move || {
            let mut buf = (*buf).clone();
            buf.append(&data);
            assert_eq!(buf.len(), data.len());
        })
    }).collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_mpool_basic() {
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

#[test]
fn test_mpool_thread_safety() {
    let pool = Arc::new(MPool::new(2, 1024));
    
    let handles: Vec<_> = (0..4).map(|_| {
        let pool = Arc::clone(&pool);
        thread::spawn(move || {
            let buf = pool.alloc();
            thread::sleep(std::time::Duration::from_millis(10));
            pool.free(buf);
        })
    }).collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    assert_eq!(pool.free_count(), 4);
    assert_eq!(pool.allocated_count(), 0);
}

#[test]
fn test_edge_cases() {
    // Test empty buffer
    let mut buf = MBuf::with_capacity(0);
    assert_eq!(buf.len(), 0);
    assert_eq!(buf.capacity(), 0);
    
    // Test append to empty buffer
    let data = b"test";
    let copied = buf.append(data);
    assert_eq!(copied, data.len());
    assert!(buf.capacity() >= data.len());
    
    // Test read from empty buffer
    let mut read_buf = [0u8; 4];
    let read_len = buf.read(&mut read_buf).unwrap();
    assert_eq!(read_len, data.len());
    assert_eq!(&read_buf, data);
    
    // Test cursor at end
    let mut cursor = Cursor::new(&buf);
    assert_eq!(cursor.next(), None);
    assert_eq!(cursor.next_slice(1), None);
}