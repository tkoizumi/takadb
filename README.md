# takadb
A lightweight database system written in Rust, focused on learning and implementing core database internals such as buffer pool management, page replacement, and disk scheduling.

## Overview

TakaDB is an project that explores how modern storage engines work under the hood. It implements key components including:

- Buffer Pool Manager (BPM)
- Page abstraction and guards (read/write)
- Disk manager and scheduler
- Page replacement policies (e.g., LRU-K)
- Concurrency-safe access using Rust primitives

The goal of this project is to build intuition around **database systems design**, particularly how memory, disk, and concurrency interact.
