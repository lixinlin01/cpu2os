pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)] // 只有執行 cargo test 時才會編譯此模組
mod tests {
    use super::*; // 匯入外部範疇的函式

    #[test] // 標記這是一個測試函式
    fn it_works() {
        assert_eq!(add(2, 2), 4);
    }

    #[test]
    #[should_panic] // 預期此測試會發生 panic
    fn test_panic() {
        panic!("發生錯誤！");
    }
}