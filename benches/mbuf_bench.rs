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

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ibuf::{MBuf, MPool};
use std::io::{Read, Write};
use std::sync::Arc;
use std::thread;

fn mbuf_append_benchmark(c: &mut Criterion) {
    let data = b"this is a test string for benchmarking";
    
    c.bench_function("mbuf_append", |b| {
        b.iter(|| {
            let mut buf = MBuf::with_capacity(1024);
            buf.append(black_box(data));
        })
    });
}

fn mbuf_clone_benchmark(c: &mut Criterion) {
    let mut buf = MBuf::with_capacity(1024);
    buf.append(b"test data");
    
    c.bench_function("mbuf_clone", |b| {
        b.iter(|| {
            black_box(buf.clone());
        })
    });
}

fn mbuf_read_write_benchmark(c: &mut Criterion) {
    let mut buf = MBuf::with_capacity(1024);
    let data = b"test data";
    
    c.bench_function("mbuf_write", |b| {
        b.iter(|| {
            buf.write(black_box(data)).unwrap();
            buf.clear();
        })
    });
    
    c.bench_function("mbuf_read", |b| {
        b.iter(|| {
            buf.write(data).unwrap();
            let mut read_buf = [0u8; 9];
            buf.read(&mut read_buf).unwrap();
            buf.clear();
        })
    });
}

fn mpool_alloc_free_benchmark(c: &mut Criterion) {
    let pool = Arc::new(MPool::new(10, 1024));
    
    c.bench_function("mpool_alloc_free", |b| {
        b.iter(|| {
            let buf = pool.alloc();
            pool.free(buf);
        })
    });
}

fn mpool_threaded_benchmark(c: &mut Criterion) {
    let pool = Arc::new(MPool::new(10, 1024));
    
    c.bench_function("mpool_threaded", |b| {
        b.iter(|| {
            let handles: Vec<_> = (0..4).map(|_| {
                let pool = Arc::clone(&pool);
                thread::spawn(move || {
                    let buf = pool.alloc();
                    thread::sleep(std::time::Duration::from_millis(1));
                    pool.free(buf);
                })
            }).collect();
            
            for handle in handles {
                handle.join().unwrap();
            }
        })
    });
}

criterion_group!(
    benches,
    mbuf_append_benchmark,
    mbuf_clone_benchmark,
    mbuf_read_write_benchmark,
    mpool_alloc_free_benchmark,
    mpool_threaded_benchmark
);
criterion_main!(benches);