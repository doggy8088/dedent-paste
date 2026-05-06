# dedent-paste

貼上縮排文字時，自動移除多餘的共同縮排。

`dedent-paste` 是一個搭配 Karabiner-Elements 使用的小型 macOS 工具。按下 `Option+V` 後，它會：

1. 以純文字讀取目前剪貼簿內容。
2. 移除非空白行前方共同的空白或 Tab 縮排。
3. 透過 `Command+V` 貼上整理後的文字。

## 快速安裝

需求：macOS、Karabiner-Elements、`curl` 與 Python 3。

```sh
curl -fsSL https://raw.githubusercontent.com/doggy8088/dedent-paste/main/install.sh | bash
```

安裝完成後，直接按：

```text
Option+V
```

安裝程式會使用最新的 cargo-dist Release installer，將 `dedent-paste` 安裝到：

```text
~/.local/bin/dedent-paste
```

它也會把 `Option+V` 規則加入目前啟用中的 Karabiner-Elements profile，並在修改前備份你的 Karabiner 設定。

## 權限設定

Karabiner-Elements 可能需要 macOS「輔助使用」權限，才能透過 System Events 觸發貼上動作。

請開啟：

```text
系統設定 > 隱私權與安全性 > 輔助使用
```

確認 Karabiner-Elements 已被允許。

## 相關連結

- [Karabiner-Elements](https://karabiner-elements.pqrs.org/)
- [Karabiner-Elements 使用手冊](https://karabiner-elements.pqrs.org/docs/)
- [Karabiner complex modifications](https://karabiner-elements.pqrs.org/docs/manual/configuration/configure-complex-modifications/)
- [dedent-paste GitHub Releases](https://github.com/doggy8088/dedent-paste/releases)
- [cargo-dist](https://opensource.axo.dev/cargo-dist/)
- [macOS 輔助使用權限說明](https://support.apple.com/guide/mac-help/allow-accessibility-apps-to-access-your-mac-mh43185/mac)

## 更多資訊

- 網站：[dedent-paste GitHub Pages](https://dedent-paste.gh.miniasp.com/)
- 開發筆記：[DEVELOPMENT.md](DEVELOPMENT.md)
- 變更紀錄：[CHANGELOG.md](CHANGELOG.md)
- 授權：[MIT](LICENSE)
