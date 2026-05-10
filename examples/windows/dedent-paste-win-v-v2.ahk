#Requires AutoHotkey v2.0
#SingleInstance Force

dedentPaste := A_Home "\.local\bin\dedent-paste.exe"

; Win+V：執行 dedent-paste。
; 注意：這會覆蓋 Windows 內建的剪貼簿歷程記錄快捷鍵。
#v::{
    global dedentPaste

    KeyWait "LWin"
    KeyWait "RWin"

    if !FileExist(dedentPaste) {
        MsgBox "找不到：`n" dedentPaste "`n`n請先安裝 dedent-paste，或修改腳本中的路徑。", "dedent-paste", "Icon!"
        return
    }

    try exitCode := RunWait Format("""{1}""", dedentPaste),, "Hide"
    catch Error as err {
        MsgBox "啟動 dedent-paste 失敗。`n`n" err.Message, "dedent-paste", "Iconx"
        return
    }

    if (exitCode != 0)
        MsgBox "dedent-paste 執行失敗，exit code: " exitCode, "dedent-paste", "Iconx"
}
