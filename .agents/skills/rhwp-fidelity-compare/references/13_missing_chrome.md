# 13 — 예외: Chrome 없음 (F10)

시트 모드는 Chrome/Chromium 이 필요하다. `--text-only` 는 아니다.

`find_chrome` 실패 stderr:

```
Chrome/Chromium을 찾을 수 없습니다. CHROME_BIN을 지정하세요.
```

또는

```
CHROME_BIN 실행 파일을 찾을 수 없습니다: <값>
```

종료 코드 **2**. 시트를 만들지 않고 종료한다.

## 처방 (이 순서)

1. 사용자가 글자 후보만 필요한가 → `--text-only` 로 내린다. 정지 F02.
2. 시트가 필요한가 → `CHROME_BIN` 을 실제 `chrome`/`chromium` 바이너리로
   지정한다. 설치 프로그램을 이 스킬이 돌리지 않는다.
3. Windows 기본 경로에 Chrome 이 있는데도 실패하면 PATH 와
   `Program Files\Google\Chrome\Application\chrome.exe` 를 확인한다.
4. macOS 는 `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome`.
5. Linux 는 `google-chrome`, `google-chrome-stable`, `chromium`,
   `chromium-browser`.
6. Edge / Firefox / Safari 를 `CHROME_BIN` 에 넣지 않는다.
7. 그래도 없으면 **정지** 하고 텍스트 원장만 제출한다.

## 레시피

```bash
# 진단
command -v google-chrome || command -v chromium || echo none
# 우회
CHROME_BIN=/usr/bin/google-chrome \
  venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 2 \
  --out-dir /tmp/rhwp-fidelity-plan
# 시트 포기
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 2 \
  --text-only --out-dir /tmp/rhwp-fidelity-plan-text
```

Windows:

```powershell
Get-Command chrome.exe -ErrorAction SilentlyContinue
$env:CHROME_BIN = "C:\Program Files\Google\Chrome\Application\chrome.exe"
```

## 캡처 실패와 탐색 실패

탐색 실패는 프로세스 시작 전, exit 2.
탐색은 됐는데 쪽마다 캡처가 실패하면 note `비교 시트 PNG 실패`,
`run-state` incomplete, 종료 코드 1. 그건 F12 에 가깝다.
한 번 재시도는 하네스가 한다. 에이전트가 바깥에서 10번 루프하지 않는다.

## 에이전트 금지

- Chrome 설치 스크립트 (`apt install`, `brew install`, `winget`) 를
  이 스킬의 필수 단계로 올리기
- headless 플래그를 바꿔 가며 새 캡처 도구를 작성
- Playwright/Puppeteer 래퍼를 이 폴더에 추가
- 탐색 실패를 "문서가 깨졌다"로 보고
