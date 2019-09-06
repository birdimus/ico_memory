# ico_memory
Multi-threaded memory management for games and graphics in Rust.  This is an experimental memory allocator, and thread safe resource handle manager implemented in Rust.

Presently, this release should be considered for research purposes only - as tests are incomplete, much of the code relies on unsafe functionality, and there are existing allocators (such as jemalloc) that generally perform as well or better and are much more robustly used and tested.