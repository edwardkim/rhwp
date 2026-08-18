---
kind: guide
status: active
canonical: gym/tutorial/README.md
last_verified: 2026-08-18
---

# 19. Windows PowerShell — 같은 놀이기구, 다른 셸

휴게실의 본문은 bash 관례(`mkdir -p`, `echo '{…}' >`)로 적혀 있다.
Windows 방문자가 그 줄을 그대로 붙이면 폴더가 안 생기거나 JSON 에
BOM·CP949 가 섞인다. 이 페이지는 **같은 동작의 PowerShell 번역**이다.
채점 논리와 과제 JSON 을 바꾸지 않는다.

돌아가기: [README.md](README.md) · 막힘: [18-troubleshooting.md](18-troubleshooting.md)

저장소 루트(`rhwp` 가 있는 곳)에서 실행한다. 작업 폴더가
`C:\Users\…\rhwp-gym-tutorial-park-docs` 같은 worktree 여도 된다.

## 표 끊기

```powershell
python gym/score.py --agent 나 --profile family
```

바이너리를 지목할 때:

```powershell
python gym/score.py --agent 나 --profile family --bin target\debug\rhwp.exe
```

## 폴더 만들기

```powershell
New-Item -ItemType Directory -Force -Path gym\submissions\나\casual-rides\CR01 | Out-Null
New-Item -ItemType Directory -Force -Path gym\submissions\나\casual-rides\CR02 | Out-Null
New-Item -ItemType Directory -Force -Path gym\submissions\나\casual-rides\CR03 | Out-Null
New-Item -ItemType Directory -Force -Path gym\submissions\나\casual-rides\CR04 | Out-Null
```

`mkdir -p` 의 `-p` 는 PowerShell 기본 `mkdir` 에 없다. `-Force` 가
이미 있는 폴더를 허용한다.

## JSON 을 UTF-8 without BOM 으로 쓰기

PowerShell 5.1 의 `Set-Content -Encoding utf8` 은 BOM 을 붙인다.
채점기의 `json.load` 는 BOM 을 거절할 수 있다. PR 본문과 같은
이유로 **BOM 없는 UTF-8** 을 쓴다.

```powershell
$utf8 = New-Object System.Text.UTF8Encoding $false
[System.IO.File]::WriteAllText(
    (Join-Path (Get-Location) 'gym\submissions\나\casual-rides\CR01\answer.json'),
    '{"pages": 3}',
    $utf8)
```

`3` 은 설명용이다. `rhwp info samples/table-001.hwp --json` 이 준
`pageCount` 로 바꾼다.

CR02·CR03·CR04 도 같은 방식으로 키만 바꾼다.

```powershell
# CR02 paragraphs / CR03 tables / CR04 hits — 숫자는 네 오라클
[System.IO.File]::WriteAllText((Join-Path (Get-Location) 'gym\submissions\나\casual-rides\CR02\answer.json'), '{"paragraphs": 0}', $utf8)
[System.IO.File]::WriteAllText((Join-Path (Get-Location) 'gym\submissions\나\casual-rides\CR03\answer.json'), '{"tables": 0}', $utf8)
[System.IO.File]::WriteAllText((Join-Path (Get-Location) 'gym\submissions\나\casual-rides\CR04\answer.json'), '{"hits": 0}', $utf8)
```

## 문서 읽기

```powershell
rhwp info samples\table-001.hwp --json
rhwp explain samples\table-001.hwp --json
rhwp export-tables samples\table-001.hwp --json
rhwp search samples\table-001.hwp --json -- 표
```

경로 구분자는 `\` 나 `/` 둘 다 되는 경우가 많다. 샘플 이름에 한글·
공백이 있으면 작은따옴표로 감싼다.

```powershell
rhwp search 'samples/2022년 국립국어원 업무계획.hwp' 국어 --json
```

## 콘솔이 한글을 `??` 로 바꿀 때

현재 콘솔 코드 페이지가 CP949 면 `rhwp` 의 UTF-8 출력이 깨져 보일
수 있다. 채점기는 파일을 UTF-8 로 읽으므로, **보이는 글자보다
answer.json 바이트**가 중요하다. 출력을 파일로 받아 키를 읽는다.

```powershell
rhwp info samples\table-001.hwp --json | Out-File -Encoding utf8 tmp-info.json
```

이 `Out-File` 에도 BOM 이 붙을 수 있다. 숫자를 눈으로 확인하는 용도로만
쓰고, 제출 파일은 위의 `UTF8Encoding($false)` 로 쓴다.

채점기 자체는 `sys.stdout.reconfigure(encoding="utf-8")` 를 시도한다.
그래도 콘솔이 깨지면 파일(`scorecard.json`, `report.md`)을 편집기로
연다.

## 전당과 초대

```powershell
python gym/tools/leaderboard.py attest --agent 나
python gym/tools/leaderboard.py verify
python gym/tools/leaderboard.py invite --agent 친구이름
```

명령은 bash 와 같다. 경로만 백슬래시로 바꿔도 된다.

## 이 페이지가 바꾸지 않는 것

- 답 키 (`pages`, `paragraphs`, `tables`, `hits`)
- 라이브 오라클 명령
- 프로파일 이름
- `admission.json` 의 `verdict` 계산
- `gym/core/checks.py`
