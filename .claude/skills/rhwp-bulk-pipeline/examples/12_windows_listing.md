# 예제 12 — Windows 목록과 파이프

```powershell
Get-ChildItem -Path .\samples -Recurse -Include *.hwp,*.hwpx -File |
  ForEach-Object FullName |
  Set-Content -Encoding utf8 .\목록.txt
Get-Content -Encoding utf8 .\목록.txt |
  rhwp batch info --json 1> .\meta.ndjson 2> .\meta.err
```

게이트는 `29_windows_powershell.md` 의 `ConvertFrom-Json` 루프.
`Select-String error` 금지.

이슈 #5311. gym 아님. 새 CLI 아님.
