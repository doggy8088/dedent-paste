# dedent-paste

`dedent-paste` 是供 Karabiner-Elements 使用的 macOS 輔助工具。它會以純文字讀取目前的剪貼簿內容，移除非空白行共通的前置空白／定位字元縮排，將轉換後的純文字寫回剪貼簿，然後送出 `Command+V`。

預期使用的快捷鍵是 `Option+V`。

## 安裝

需求：

- curl
- Python 3
- Karabiner-Elements

```sh
curl -fsSL https://raw.githubusercontent.com/doggy8088/dedent-paste/main/install.sh | bash
```

安裝腳本會：

- 下載 latest GitHub Release 裡的單一可執行檔到：

  ```text
  ~/.local/bin/dedent-paste
  ```

- 寫入 Karabiner complex modification asset：

  ```text
  ~/.config/karabiner/assets/complex_modifications/paste-dedent-plain-text.json
  ```

- 備份目前的 Karabiner 設定檔：

  ```text
  ~/.config/karabiner/karabiner.json.bak-YYYYMMDDHHMMSS
  ```

- 將 `Option+V` 規則加入目前作用中的 Karabiner profile
- 如果系統有 `karabiner_cli`，會檢查 complex modification JSON 格式

如果是在原始碼 checkout 內直接執行 `./install.sh`，腳本會改用 `cargo build --release` 從本機原始碼建置。

## 建置

```sh
cargo build --release
```

一行安裝後，Karabiner 規則會呼叫 release binary：

```text
$HOME/.local/bin/dedent-paste
```

## 行為

- 計算最小縮排時，會忽略空白行與只包含空白字元的行。
- 空格與定位字元都會計為一個前置空白字元。
- 貼上後，轉換後的純文字會保留在剪貼簿中。
- 貼上動作是透過 AppleScript/System Events 觸發，因此可能需要 macOS 輔助使用權限。

## Karabiner-Elements

複合修改資源安裝於：

```text
~/.config/karabiner/assets/complex_modifications/paste-dedent-plain-text.json
```

`~/.config/karabiner/karabiner.json` 中作用中的 Karabiner 設定檔也包含相同的 `Option+V` 規則。

作用中的規則內容：

```json
{
  "description": "Option+V：執行 dedent-paste",
  "manipulators": [
    {
      "from": {
        "key_code": "v",
        "modifiers": { "mandatory": ["option"] }
      },
      "to": [
        {
          "shell_command": "$HOME/.local/bin/dedent-paste"
        }
      ],
      "type": "basic"
    }
  ]
}
```
