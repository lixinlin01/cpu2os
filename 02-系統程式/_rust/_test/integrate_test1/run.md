```
(py310) cccimac@cccimacdeiMac _test % cargo new integrate_test1 --lib 
    Creating library `integrate_test1` package
note: see more `Cargo.toml` keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html
(py310) cccimac@cccimacdeiMac _test % cd integrate_test1 
(py310) cccimac@cccimacdeiMac integrate_test1 % cargo teset
error: no such command: `teset`

help: a command with a similar name exists: `test`

help: view all installed commands with `cargo --list`
help: find a package to install `teset` with `cargo search cargo-teset`
(py310) cccimac@cccimacdeiMac integrate_test1 % cargo test 
   Compiling integrate_test1 v0.1.0 (/Users/cccimac/Desktop/ccc/cpu2os/02-系統程式/_rust/_test/integrate_test1)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.70s
     Running unittests src/lib.rs (target/debug/deps/integrate_test1-94e9cafd9f42548e)

running 1 test
test tests::it_works ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests integrate_test1

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```