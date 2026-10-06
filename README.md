# Mini LSM-tree

这是一个用 Rust 从零练习存储引擎的学习项目。目标是实现最小的单线程 LSM-tree：支持 `put`、`get`、`delete`，把内存中的有序数据写成 SSTable，并在正常关闭、重新打开后读回已刷盘的数据。

项目按 M0–M5 逐步完成。当前代码还不是可用的数据库，也没有命令行接口；`src/main.rs` 仍是 Cargo 默认的 Hello World。

## 学习目标

这条路线从 C++ 中熟悉的“容器、指针、文件”出发，逐步练习 Rust 的所有权与借用、`enum` 与模式匹配、迭代器、`Result` 错误传播、`Read`/`Write` I/O trait，以及模块与集成测试。先让每一层独立通过测试，再把它们组合成引擎。

## 目标中的读写路径

```text
put / delete -> 分配 seq -> MemTable (SkipMap) -> flush -> SSTable 文件
get          -> MemTable -> 从新到旧查找 SSTable
```

- `Entry` 保存序列号 `seq` 和操作类型：`Put(Vec<u8>)` 或 `Tombstone`。
- 同一个 key 在 MemTable 中只保留最新的 `Entry`。删除会写入墓碑，而不是直接移除 key。
- 将来的 `get` 会先查 MemTable，再从最新的 SSTable 向旧文件查找。遇到 `Put` 或 `Tombstone` 都立即停止；墓碑不能被当成“这个文件没有该 key”。
- 本阶段的恢复目标是**显式 flush 后**，重新打开仍能读到最新值和删除状态。没有 WAL，尚未刷盘的内存数据不保证在进程崩溃后保留。

## 开发进度

| 阶段 | 内容 | 状态 |
| --- | --- | --- |
| M0 | 定义 `Entry`、`ValueKind`，实现 Put/墓碑构造函数与测试 | 已完成 |
| M1 | 使用 `skiplist::SkipMap` 实现有序 MemTable | 进行中 |
| M2 | 定义错误类型，编解码单条记录并处理截断数据 | 计划中 |
| M3 | 顺序写入、扫描单个 SSTable 文件 | 计划中 |
| M4 | 组合读写路径、flush 和重新打开后的状态恢复 | 计划中 |
| M5 | 测试跨文件覆盖、墓碑、重启与错误路径 | 计划中 |

MVP 暂不实现 WAL、Compaction、Bloom Filter、后台任务、并发读写、快照或 MVCC。

## 当前代码

- `src/entry.rs`：`Entry`、`ValueKind` 和构造函数。
- `src/memtable.rs`：M1 的 `SkipMap<Vec<u8>, Entry>` 包装，提供写入、查询、遍历等接口。
- `src/lib.rs`：导出当前模块的公开类型。
- `tests/entry_func_test.rs`、`tests/memtable_func_test.rs`：现有集成测试。

后续阶段计划添加 `error.rs`、`sstable.rs` 和 `engine.rs`。SSTable 记录格式与每一步的代码练习见项目根目录的学习路线 PPT。

## 运行测试

在项目根目录执行：

```bash
cargo test
```

也可以单独运行某一阶段的测试：

```bash
cargo test --test entry_func_test
cargo test --test memtable_func_test
```

当前测试覆盖 `Entry` 的两种构造方式，以及 MemTable 的写入、覆盖、查询和迭代。实现下一阶段时，先补对应测试，再扩展引擎功能。
