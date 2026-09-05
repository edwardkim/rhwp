# 03 — Windows: venv\Scripts\python.exe

Windows 에이전트가 POSIX 예시를 그대로 붙여 넣으면 실패한다. 이 장은
그 한 줄을 고정한다. 정본 README 도 "Windows 에서는
`venv\\Scripts\\python.exe` 로 바꾼다"고 이미 적는다.

## 인터프리터

| 플랫폼 | 하네스 파이썬 |
| --- | --- |
| POSIX | `venv/bin/python` |
| Windows (cmd/PowerShell) | `venv\Scripts\python.exe` |
| Git Bash on Windows | 여전히 `venv/Scripts/python.exe` (bin 아님) |
| WSL2 | POSIX `venv/bin/python` (별 리눅스 트리) |

`venv\bin\python` 은 Windows venv 에 없다. 실수하면
`can't open file` 또는 `venv\bin\python: not found` 가 난다. 그 순간
시스템 `python` 으로 폴백하지 말 것 — F09 를 우회하게 된다.

## 생성

```powershell
py -3.12 -m venv venv
if (-not $?) { throw "py -3.12 가 없습니다. python.org 3.12 를 설치하세요." }
venv\Scripts\python.exe -m pip install -U pip
venv\Scripts\python.exe -m pip install pypdf pypdfium2 pillow
```

`python -m pip install --break-system-packages pypdf` 는 거절한다 (F15).
회사 이미지가 PEP 668 로 막혀 있어도 우회하지 않는다. 저장소 `venv/` 가
답이다. `conda install` 로 전역 환경을 더럽히지 않는다.

실행 정책(`Activate.ps1` 차단)은 무시해도 된다. 활성화 없이
`venv\Scripts\python.exe` 를 절대 경로로 호출하는 쪽이 재현에 낫다.

## 호출

```powershell
venv\Scripts\python.exe tools\fidelity_compare\fidelity_compare.py plan 0 9 `
  --out-dir $env:TEMP\rhwp-fidelity-plan
```

`--out-dir` 는 worktree 밖 (`$env:TEMP`, `C:\tmp\rhwp-fidelity-...`) 을
권장한다. `output/fidelity/` 도 동작하지만 대용량 PNG 가 worktree 와
디스크를 채운다. 이 저장소 worktree 는 이미 크다.

슬래시 `tools/fidelity_compare/fidelity_compare.py` 도 Python 이 받는다.
백슬래시는 복사-붙여넣기용이다.

## 실행 파일 탐색

하네스 `find_rhwp` / `find_chrome` 는 Windows 에서:

- `rhwp`: `target\release-test\rhwp.exe` → `target\release\rhwp.exe` → PATH
- Chrome: PATH 의 `chrome.exe` →
  `%ProgramFiles%\Google\Chrome\Application\chrome.exe` →
  `%ProgramFiles(x86)%` → `%LocalAppData%\Google\Chrome\Application\chrome.exe`

안 맞으면 환경 변수만 덮는다. 설치 스크립트를 이 스킬에 추가하지 않는다.

```powershell
$env:RHWP_BIN = "C:\build\rhwp.exe"
$env:CHROME_BIN = "C:\Program Files\Google\Chrome\Application\chrome.exe"
venv\Scripts\python.exe tools\fidelity_compare\fidelity_compare.py plan 0 2 `
  --out-dir $env:TEMP\rhwp-fidelity-plan
```

Edge 는 Chrome 이 아니다. `msedge.exe` 를 `CHROME_BIN` 에 넣으면
headless 플래그가 달라 실패할 수 있다. 실패하면 `--text-only` 로 내린다
(F10).

## 글꼴

Windows 는 설치 글꼴을 네이티브로 쓴다. Linux 전용 fontconfig 스텁
(`FONTCONFIG_PATH`) 을 만들지 않는다. 하네스도 `os.name == "nt"` 이면
fontconfig 환경을 `None` 으로 둔다.

`RHWP_FONT_PATH_DIR` 은 넘겨도 된다. 한글이 쓰는 `C:\Windows\Fonts` 와
한컴 설치 디렉터리를 `;` 로 잇는다.

```powershell
$env:RHWP_FONT_PATH_DIR = "C:\Windows\Fonts;C:\Program Files (x86)\Hnc\Shared\Fonts"
```

한컴 설치 경로는 버전마다 다르다. 없는 폴더를 넣어도
`configured_font_paths` 가 디렉터리만 채택한다.

## 한글 argv

배경 cmd 의 cp949 는 한글 경로를 깨뜨린다. REG 키(`plan`) 와 ASCII
글롭을 쓰고, 불가피한 한글 경로는 PowerShell 변수에 담아 넘긴다.

```powershell
$src = Get-ChildItem samples\*.hwp | Select-Object -First 1
$pdf = Get-ChildItem pdf\*-2022.pdf | Select-Object -First 1
venv\Scripts\python.exe tools\fidelity_compare\fidelity_compare.py 0 4 `
  --source $src.FullName --reference-pdf $pdf.FullName --label win-pair `
  --text-only --out-dir $env:TEMP\rhwp-fidelity-win-pair
```

## 에이전트 금지

- POSIX `venv/bin/python` 을 Windows 세션에 붙여 넣기
- `py -m pip install --user` 로 전역 오염
- `--break-system-packages`
- Chrome 포터블 설치 스크립트를 이 스킬에 추가
- `choco install` / `winget install` 을 이 장의 필수 단계로 올리기
  (Chrome 이 이미 있으면 `find_chrome` 이 찾는다)
