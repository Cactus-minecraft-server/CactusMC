# SLP

- [X] Parse server.properties file for info

- [X] Globals like server version

- [ ] Implement SLP interface

# Players


- [ ] Read map file and send data

- [ ] Somehow get a player to connect

# Issues
- [ ] resolve this memory leak
spectre@fedora:~/RustProject/CopperMC$ valgrind --tool=memcheck --leak-check=full ./target/release/copper_server 
==20441== Memcheck, a memory error detector
==20441== Copyright (C) 2002-2024, and GNU GPL'd, by Julian Seward et al.
==20441== Using Valgrind-3.23.0 and LibVEX; rerun with -h for copyright info
==20441== Command: ./target/release/copper_server
==20441== 
[2024-09-25T07:26:15Z INFO  copper_server] Starting minecraft server version 1.21.1
[2024-09-25T07:26:15Z INFO  copper_server] Hello, world from Copper!
[2024-09-25T07:26:15Z ERROR copper_server::fs_manager] Cannot start the server, please agree to the 'eula.txt'
[2024-09-25T07:26:15Z WARN  copper_server] [ SERVER SHUTDOWN WITH CODE: -1]
==20441== 
==20441== HEAP SUMMARY:
==20441==     in use at exit: 103,256 bytes in 172 blocks
==20441==   total heap usage: 289 allocs, 117 frees, 118,951 bytes allocated
==20441== 
==20441== 280 bytes in 1 blocks are possibly lost in loss record 42 of 60
==20441==    at 0x4843866: malloc (vg_replace_malloc.c:446)
==20441==    by 0x1455A3: hashbrown::raw::RawTable<T,A>::reserve_rehash (in /home/spectre/RustProject/CopperMC/target/release/copper_server)
==20441==    by 0x36DF4B: tokio::runtime::blocking::pool::Spawner::spawn_task (in /home/spectre/RustProject/CopperMC/target/release/copper_server)
==20441==    by 0x363AA8: tokio::runtime::builder::Builder::build (in /home/spectre/RustProject/CopperMC/target/release/copper_server)
==20441==    by 0x16EFF4: copper_server::main (in /home/spectre/RustProject/CopperMC/target/release/copper_server)
==20441==    by 0x17A502: std::sys::backtrace::__rust_begin_short_backtrace (in /home/spectre/RustProject/CopperMC/target/release/copper_server)
==20441==    by 0x16ECAC: main (in /home/spectre/RustProject/CopperMC/target/release/copper_server)
==20441== 
==20441== 304 bytes in 1 blocks are possibly lost in loss record 43 of 60
==20441==    at 0x484B133: calloc (vg_replace_malloc.c:1675)
==20441==    by 0x4011FE3: UnknownInlinedFun (rtld-malloc.h:44)
==20441==    by 0x4011FE3: allocate_dtv (dl-tls.c:370)
==20441==    by 0x4012A71: _dl_allocate_tls (dl-tls.c:629)
==20441==    by 0x4FA2233: allocate_stack (allocatestack.c:429)
==20441==    by 0x4FA2233: pthread_create@@GLIBC_2.34 (pthread_create.c:655)
==20441==    by 0x3526A9: std::sys::pal::unix::thread::Thread::new (in /home/spectre/RustProject/CopperMC/target/release/copper_server)
==20441==    by 0x16E031: copper_server::init_ctrlc_handler (in /home/spectre/RustProject/CopperMC/target/release/copper_server)
==20441==    by 0x16F674: copper_server::main (in /home/spectre/RustProject/CopperMC/target/release/copper_server)
==20441==    by 0x17A502: std::sys::backtrace::__rust_begin_short_backtrace (in /home/spectre/RustProject/CopperMC/target/release/copper_server)
==20441==    by 0x16ECAC: main (in /home/spectre/RustProject/CopperMC/target/release/copper_server)
==20441== 
==20441== 304 bytes in 1 blocks are possibly lost in loss record 44 of 60
==20441==    at 0x484B133: calloc (vg_replace_malloc.c:1675)
==20441==    by 0x4011FE3: UnknownInlinedFun (rtld-malloc.h:44)
==20441==    by 0x4011FE3: allocate_dtv (dl-tls.c:370)
==20441==    by 0x4012A71: _dl_allocate_tls (dl-tls.c:629)
==20441==    by 0x4FA2233: allocate_stack (allocatestack.c:429)
==20441==    by 0x4FA2233: pthread_create@@GLIBC_2.34 (pthread_create.c:655)
==20441==    by 0x3526A9: std::sys::pal::unix::thread::Thread::new (in /home/spectre/RustProject/CopperMC/target/release/copper_server)
==20441==    by 0x36DEB0: tokio::runtime::blocking::pool::Spawner::spawn_task (in /home/spectre/RustProject/CopperMC/target/release/copper_server)
==20441==    by 0x181CC8: copper_server::commands::command_line::handle_input::{{closure}} (in /home/spectre/RustProject/CopperMC/target/release/copper_server)
==20441==    by 0x1814C7: tokio::runtime::task::core::Core<T,S>::poll (in /home/spectre/RustProject/CopperMC/target/release/copper_server)
==20441==    by 0x149521: tokio::runtime::task::raw::poll (in /home/spectre/RustProject/CopperMC/target/release/copper_server)
==20441==    by 0x355454: tokio::runtime::scheduler::multi_thread::worker::Context::run_task (in /home/spectre/RustProject/CopperMC/target/release/copper_server)
==20441==    by 0x3594C3: tokio::runtime::task::raw::poll (in /home/spectre/RustProject/CopperMC/target/release/copper_server)
==20441==    by 0x35BBB6: std::sys::backtrace::__rust_begin_short_backtrace (in /home/spectre/RustProject/CopperMC/target/release/copper_server)
==20441== 
==20441== 1,216 bytes in 4 blocks are possibly lost in loss record 51 of 60
==20441==    at 0x484B133: calloc (vg_replace_malloc.c:1675)
==20441==    by 0x4011FE3: UnknownInlinedFun (rtld-malloc.h:44)
==20441==    by 0x4011FE3: allocate_dtv (dl-tls.c:370)
==20441==    by 0x4012A71: _dl_allocate_tls (dl-tls.c:629)
==20441==    by 0x4FA2233: allocate_stack (allocatestack.c:429)
==20441==    by 0x4FA2233: pthread_create@@GLIBC_2.34 (pthread_create.c:655)
==20441==    by 0x3526A9: std::sys::pal::unix::thread::Thread::new (in /home/spectre/RustProject/CopperMC/target/release/copper_server)
==20441==    by 0x36DEB0: tokio::runtime::blocking::pool::Spawner::spawn_task (in /home/spectre/RustProject/CopperMC/target/release/copper_server)
==20441==    by 0x363AA8: tokio::runtime::builder::Builder::build (in /home/spectre/RustProject/CopperMC/target/release/copper_server)
==20441==    by 0x16EFF4: copper_server::main (in /home/spectre/RustProject/CopperMC/target/release/copper_server)
==20441==    by 0x17A502: std::sys::backtrace::__rust_begin_short_backtrace (in /home/spectre/RustProject/CopperMC/target/release/copper_server)
==20441==    by 0x16ECAC: main (in /home/spectre/RustProject/CopperMC/target/release/copper_server)
==20441== 
==20441== LEAK SUMMARY:
==20441==    definitely lost: 0 bytes in 0 blocks
==20441==    indirectly lost: 0 bytes in 0 blocks
==20441==      possibly lost: 2,104 bytes in 7 blocks
==20441==    still reachable: 101,152 bytes in 165 blocks
==20441==         suppressed: 0 bytes in 0 blocks
==20441== Reachable blocks (those to which a pointer was found) are not shown.
==20441== To see them, rerun with: --leak-check=full --show-leak-kinds=all
==20441== 
==20441== For lists of detected and suppressed errors, rerun with: -s
==20441== ERROR SUMMARY: 4 errors from 4 contexts (suppressed: 0 from 0)
spectre@fedora:~/RustProject/CopperMC$ 


