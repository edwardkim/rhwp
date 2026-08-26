# 29 — Windows PowerShell

이 저장소의 실측 가이드 숫자는 32코어 Windows release 에서 나왔다.
에이전트가 Linux `find` 만 알면 목록부터 틀린다.

## 목록

```powershell
Get-ChildItem -Path 폴더 -Recurse -Include *.hwp,*.hwpx -File |
  ForEach-Object { $_.FullName } |
  Set-Content -Encoding utf8 목록.txt
```

`Out-File` 기본은 UTF-16 LE. `Set-Content -Encoding utf8` 또는
`utf8NoBOM`(PS 6+)를 쓴다.

파이프:

```powershell
Get-Content -Encoding utf8 목록.txt | rhwp batch info --json 1> meta.ndjson 2> meta.err
```

`1>` / `2>` 를 섞지 않는다. `*>&1` 금지.

## 게이트

```powershell
$lines = Get-Content -Encoding utf8 목록.txt
$inputN = @($lines).Count
$records = Get-Content -Encoding utf8 결과.ndjson | ForEach-Object { $_ | ConvertFrom-Json }
$ok = @($records | Where-Object { -not $_.error }).Count
$bad = @($records | Where-Object { $_.error }).Count
Write-Output "입력 $inputN = 성공 $ok + 실패 $bad"
if ($inputN -ne ($ok + $bad)) { throw "게이트 실패" }
```

`Select-String error` 는 본문에 "error" 가 있는 성공 행을 오탐한다. 쓰지 말 것.

## jq

Windows 에 jq 가 있으면 같은 레시피(`27_gate_recipes.md`)를 그대로 쓴다.
없으면 위의 `ConvertFrom-Json` 한 줄 루프.

## 종료 코드

```powershell
rhwp batch export-text --json < 목록.txt 1> 결과.ndjson 2> 요약.err
$code = $LASTEXITCODE
```

`<` 리다이렉트가 구형 Windows PowerShell 5.1 에서 깨지면:

```powershell
Get-Content -Encoding utf8 목록.txt | rhwp batch export-text --json 1> 결과.ndjson 2> 요약.err
```

fill 은 stdin 을 읽지 않으므로 파이프에 목록을 넣지 말 것.

## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `29_windows_powershell.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
