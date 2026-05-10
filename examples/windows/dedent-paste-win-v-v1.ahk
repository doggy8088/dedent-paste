#NoEnv
#SingleInstance Force
SendMode Input
SetWorkingDir %A_ScriptDir%

dedentPaste := A_Home . "\.local\bin\dedent-paste.exe"

; Win+V：執行 dedent-paste。
; 注意：這會覆蓋 Windows 內建的剪貼簿歷程記錄快捷鍵。
#v::
    KeyWait, LWin
    KeyWait, RWin

    if !FileExist(dedentPaste)
    {
        MsgBox, 48, dedent-paste, 找不到：`n%dedentPaste%`n`n請先安裝 dedent-paste，或修改腳本中的路徑。
        return
    }

    quotedPath := Chr(34) . dedentPaste . Chr(34)
    RunWait, %quotedPath%,, Hide UseErrorLevel
    if (ErrorLevel = "ERROR")
    {
        MsgBox, 16, dedent-paste, 啟動 dedent-paste 失敗。
        return
    }

    if (ErrorLevel != 0)
        MsgBox, 16, dedent-paste, dedent-paste 執行失敗，exit code: %ErrorLevel%
return
