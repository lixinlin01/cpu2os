## ccc: rust 當中如何做測試？

在 Rust 中，測試是開發流程的核心。Rust 內建了強大的測試架構，你不需要安裝額外的工具就能開始編寫測試。

主要可以分為 **單元測試 (Unit Tests)** 與 **整合測試 (Integration Tests)** 兩大類。

---

## 1. 單元測試 (Unit Tests)

單元測試通常與原始碼放在同一個檔案中。慣例是在檔案末尾建立一個名為 `tests` 的模組，並加上 `#[cfg(test)]` 屬性。

### 範例程式碼

```rust
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

```

* **`#[cfg(test)]`**: 告訴編譯器只有在測試模式下才編譯這段程式碼，能縮小最終執行檔的大小。
* **`#[test]`**: 標記該函式為測試案例。

---

## 2. 整合測試 (Integration Tests)

整合測試旨在測試庫（Library）的各個部分是否能協同工作。它們必須放在專案根目錄下的 **`tests/`** 資料夾中。

* **目錄結構**：
```text
my_project/
├── Cargo.toml
├── src/
│   └── lib.rs
└── tests/
    └── common/      # 共享的工具模組
    └── my_test.rs   # 整合測試檔案

```



在 `tests/my_test.rs` 中，你不需要 `#[cfg(test)]`，且必須像外部使用者一樣匯入你的 crate：

```rust
use my_project;

#[test]
fn test_integration() {
    assert_eq!(my_project::add(3, 2), 5);
}

```

---

## 3. 常用斷言 (Assertions)

Rust 提供幾個核心巨集來驗證結果：

| 巨集 | 說明 |
| --- | --- |
| `assert!(condition)` | 確保條件為 `true` |
| `assert_eq!(left, right)` | 確保兩者相等 (left == right) |
| `assert_ne!(left, right)` | 確保兩者不等 (left != right) |

---

## 4. 如何執行測試

在終端機輸入以下指令：

* **執行所有測試**：
`cargo test`
* **執行特定測試**：
`cargo test test_name`
* **忽略某些測試**：
使用 `#[ignore]` 標記耗時的測試，並用 `cargo test -- --ignored` 執行。
* **查看列印輸出**：
預設情況下，成功的測試不會顯示 `println!`。若要看到輸出，請使用：
`cargo test -- --nocapture`

---

## 5. 文件測試 (Doc Tests)

Rust 甚至支持在文件的範例程式碼中直接執行測試！這能確保你的範例永遠是最新的。

```rust
/// 將兩個數字相加。
///
/// # Examples
///
/// ```
/// let result = my_project::add(1, 2);
/// assert_eq!(result, 3);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

```

執行 `cargo test` 時，Rust 會自動編譯並執行 `/// ``` ` 區塊中的程式碼。

---

**您目前是否正在為某個特定的函式編寫測試，需要我幫您看看測試邏輯嗎？**