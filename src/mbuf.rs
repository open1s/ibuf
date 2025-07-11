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

#![allow(dead_code)]

use std::borrow::{Borrow, BorrowMut};
use std::ptr;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::io::{Read, Write, Result};

/// 类似Linux mbuf的高性能消息缓冲区
pub struct MBuf {
    data: *mut u8,
    len: usize,
    capacity: usize,
    ref_count: *mut AtomicUsize,
}

impl MBuf {
    /// 创建指定容量的新缓冲区
    pub fn with_capacity(capacity: usize) -> Self {
        let layout = std::alloc::Layout::from_size_align(capacity, 1).unwrap();
        let data = unsafe { std::alloc::alloc(layout) };
        let ref_count = Box::into_raw(Box::new(AtomicUsize::new(1)));
        Self {
            data,
            len: 0,
            capacity,
            ref_count,
        }
    }
    
    /// 获取当前数据长度
    pub fn len(&self) -> usize {
        self.len
    }
    
    /// 获取缓冲区容量
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    
    /// 追加数据
    pub fn append(&mut self, data: &[u8]) -> usize {
        let current_len = self.len;
        let needed = current_len + data.len();
        
        if needed > self.capacity {
            self.resize(needed);
        }
        
        let to_copy = data.len();
        unsafe {
            ptr::copy_nonoverlapping(
                data.as_ptr(),
                self.data.add(current_len),
                to_copy
            );
        }
        
        self.len = current_len + to_copy;
        to_copy
    }

    pub fn is_unique(&self) -> bool {
        unsafe { (*self.ref_count).load(Ordering::Acquire) == 1 }
    }


    /// 调整缓冲区大小
    pub fn resize(&mut self, new_capacity: usize) {
        let current_len = self.len;
        let new_capacity = std::cmp::max(new_capacity, self.capacity + self.capacity / 2); // 按1.5倍增长
        
        let new_layout = std::alloc::Layout::from_size_align(new_capacity, 1).unwrap();
        let new_data = unsafe { std::alloc::alloc(new_layout) };

        unsafe {
            ptr::copy_nonoverlapping(
                self.data,
                new_data,
                current_len
            );
            
            //如果只有一个引用，释放旧内存
            if (*self.ref_count).fetch_sub(1, Ordering::AcqRel) == 1 {
                let old_layout = std::alloc::Layout::from_size_align(self.capacity, 1).unwrap();
                std::alloc::dealloc(self.data, old_layout);
                //释放旧引用
                drop(Box::from_raw(self.ref_count));
            }
        }
        
        self.data = new_data;
        self.capacity = new_capacity;
        self.ref_count = Box::into_raw(Box::new(AtomicUsize::new(1)));
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }
}

impl Clone for MBuf {
    fn clone(&self) -> Self {
        unsafe {
            (*self.ref_count).fetch_add(1, Ordering::Relaxed);
        }
        Self {
            data: self.data,
            len: self.len,
            capacity: self.capacity,
            ref_count: self.ref_count,
        }
    }
}

impl Drop for MBuf {
    fn drop(&mut self) {
        unsafe {
            if (*self.ref_count).fetch_sub(1, Ordering::AcqRel) == 1 {
                let layout = std::alloc::Layout::from_size_align(self.capacity, 1).unwrap();
                std::alloc::dealloc(self.data, layout);
                drop(Box::from_raw(self.ref_count));
            }
        }  
    }
}

impl Deref for MBuf {
    type Target = [u8];
    
    fn deref(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data, self.len) }
    }
}

impl DerefMut for MBuf {
    fn deref_mut(&mut self) -> &mut [u8] {
        let len = self.len;
        unsafe { std::slice::from_raw_parts_mut(self.data, len) }
    }
}

impl AsRef<[u8]> for MBuf  {
    fn as_ref(&self) -> &[u8] {
        &*self
    }
}

impl AsMut<[u8]> for MBuf {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut *self
    }
}

impl Borrow<[u8]> for MBuf {
    fn borrow(&self) -> &[u8] {
        &*self
    }
}

impl BorrowMut<[u8]> for MBuf {
    fn borrow_mut(&mut self) -> &mut [u8] {
        &mut *self
    }
}


unsafe impl Send for MBuf {}
unsafe impl Sync for MBuf {}

/// 用于遍历MBuf数据的游标结构
pub struct Cursor<'a> {
    buf: &'a MBuf,
    pos: usize,
}

impl<'a> Cursor<'a> {
    /// 创建一个新的游标
    pub fn new(buf: &'a MBuf) -> Self {
        Self { buf, pos: 0 }
    }
    
    /// 获取当前位置
    pub fn position(&self) -> usize {
        self.pos
    }
    
    /// 获取下一个字节，如果到达末尾返回None
    pub fn next(&mut self) -> Option<u8> {
        if self.pos >= self.buf.len() {
            return None;
        }
        
        let byte = unsafe { *self.buf.data.add(self.pos) };
        self.pos += 1;
        Some(byte)
    }
    
    /// 获取下一个切片，长度为size，如果剩余数据不足返回None
    pub fn next_slice(&mut self, size: usize) -> Option<&'a [u8]> {
        if self.pos + size > self.buf.len() {
            return None;
        }
        
        let slice = unsafe { std::slice::from_raw_parts(self.buf.data.add(self.pos), size) };
        self.pos += size;
        Some(slice)
    }
    
    /// 重置游标位置
    pub fn reset(&mut self) {
        self.pos = 0;
    }
}

impl Read for MBuf {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let len = self.len;
        let to_read = std::cmp::min(buf.len(), len);
        
        unsafe {
            ptr::copy_nonoverlapping(
                self.data,
                buf.as_mut_ptr(),
                to_read
            );
            ptr::copy(
                self.data.add(to_read),
                self.data,
                len - to_read
            );
        }
        
        self.len = len - to_read;
        Ok(to_read)
    }
}

impl Write for MBuf {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let written = self.append(buf);
        Ok(written)
    }
    
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mbuf_basic() {
        let mut buf = MBuf::with_capacity(1024);
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.capacity(), 1024);
        
        let data = b"hello world";
        let copied = buf.append(data);
        assert_eq!(copied, data.len());
        assert_eq!(buf.len(), data.len());
        
        let slice: &[u8] = &buf;
        assert_eq!(slice, data);
    }
    
    #[test]
    fn test_cursor() {
        let mut buf = MBuf::with_capacity(1024);
        let data = b"hello world";
        buf.append(data);
        
        let mut cursor = Cursor::new(&buf);
        assert_eq!(cursor.next(), Some(b'h'));
        assert_eq!(cursor.next_slice(4), Some(b"ello".as_ref()));
        assert_eq!(cursor.position(), 5);
        
        cursor.reset();
        assert_eq!(cursor.position(), 0);
        assert_eq!(cursor.next_slice(data.len()), Some(data.as_ref()));
    }
    
    #[test]
    fn test_read() {
        let mut buf = MBuf::with_capacity(1024);
        let data = b"test data";
        buf.append(data);
        
        let mut read_buf = [0u8; 9];
        let read_len = buf.read(&mut read_buf).unwrap();
        assert_eq!(read_len, data.len());
        assert_eq!(&read_buf, data);
        assert_eq!(buf.len(), 0);
    }
    
    #[test]
    fn test_write() {
        let mut buf = MBuf::with_capacity(1024);
        let data = b"write test";
        
        let written = buf.write(data).unwrap();
        assert_eq!(written, data.len());
        assert_eq!(buf.len(), data.len());
        assert_eq!(&*buf, data);
    }

    #[test]
    fn test_fetch_sub() {
       let mm = AtomicUsize::new(1); 
       let q = mm.fetch_sub(1, Ordering::Relaxed);
       println!("q: {}", q);
       let q = mm.fetch_sub(1, Ordering::Relaxed);
       println!("q: {}", q);
    }
}