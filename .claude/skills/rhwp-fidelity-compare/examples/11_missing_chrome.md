# 예제 — Chrome 없음

이슈 #5329. 실 에이전트 경로. gym 아님.

## 증상

```
Chrome/Chromium을 찾을 수 없습니다. CHROME_BIN을 지정하세요.
$ echo $?
2
```

## 처방

```bash
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 2 \
  --text-only --out-dir /tmp/rhwp-fidelity-plan-text
# 또는
CHROME_BIN=/usr/bin/google-chrome \
  venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 2 \
  --out-dir /tmp/rhwp-fidelity-plan
```

설치 스크립트를 이 스킬에 추가하지 않는다.

관련: `references/13_missing_chrome.md`.
전사: `fixtures/transcripts/missing_chrome.txt`.
정지 F10.
