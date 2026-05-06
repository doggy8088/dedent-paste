# dedent-paste

`dedent-paste` 是供 Karabiner-Elements 使用的 macOS 輔助工具。它會以純文字讀取目前的剪貼簿內容，移除非空白行共通的前置空白／定位字元縮排，將轉換後的純文字寫回剪貼簿，然後送出 `Command+V`。

預期使用的快捷鍵是 `Option+V`。

## 安裝

```sh
./install.sh
```

安裝腳本會建置 release binary、安裝 Karabiner complex modification asset，並將 `Option+V` 規則加入目前作用中的 Karabiner profile。

## 建置

```sh
cargo build --release
```

Karabiner 已設定為呼叫：

```text
$HOME/projects/dedent-paste/target/release/dedent-paste
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
          "shell_command": "$HOME/projects/dedent-paste/target/release/dedent-paste"
        }
      ],
      "type": "basic"
    }
  ]
}
```
