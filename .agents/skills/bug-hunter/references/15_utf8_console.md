# 15 — UTF-8 파일 비교만

playbook / 기존 스킬: 이 PC 콘솔이 cp949 면 한글이 깨져 보인다.
문자열 비교와 검증은 **UTF-8 파일 기반**으로 수행한다. 콘솔
인코딩 착시를 결함으로 오인하지 않는다.

정지 **F10**. 콘솔 스크린샷은 이슈 근거가 아니다.

## 하는 일

```bash
rhwp export-tables --json 작성본.hwp > /tmp/after.json
# PowerShell
rhwp export-tables --json 작성본.hwp | Out-File -Encoding utf8 after.json
```

비교:

```bash
venv/bin/python -c "from pathlib import Path; a=Path('expect.txt').read_text(encoding='utf-8'); b=Path('after.json').read_text(encoding='utf-8'); raise SystemExit(0 if 값 in b else 1)"
```

파일 두 개를 읽어서 비교한다. `Write-Host` 로 본 물음표는 버린다.

## 하지 않는 일

- bash 인라인에 한글 리터럴을 넣고 콘솔 출력을 이슈에 붙이기
- chcp 결과를 rhwp 버그로 등록
- cp949 왕복이 깨진 파이프를 텍스트층 소실로 분류

## 분류

C10 `console_mojibake` → `not-a-defect`, `issueReady=false`.
봉투: `fixtures/envelopes/console_mojibake.json`.

UTF-8 파일에서는 맞고 콘솔에서만 깨지면 절차 오류다. 헌팅 산출이
아니다.

예제: [15_console_encoding.md](../examples/15_console_encoding.md)
