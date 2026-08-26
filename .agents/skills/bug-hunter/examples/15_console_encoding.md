# 예제 — 콘솔 인코딩 착시

이슈 #5324. F10. gym 아님. 결함 아님.

## 정답지

UTF-8 파일의 바이트. 콘솔 글리프가 아니다.

## 명령

```bash
rhwp export-tables --json 작성본.hwp > after.json
venv/bin/python -c "from pathlib import Path; t=Path('after.json').read_text(encoding='utf-8'); print('ok' if '시연용' in t else 'missing')"
```

PowerShell 호스트에 물음표가 보여도 파일에 한글이 있으면 통과.

## 읽는 법

C10 `not-a-defect`. 이슈를 올리지 않는다. 비교 시트 스케일
착시도 같은 계열 — 원본 스케일로 다시 연다.
