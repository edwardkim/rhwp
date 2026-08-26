# 예제 — 두부 가득 하네스

이슈 #5329. 실 에이전트 경로. gym 아님.

## 상황

`cmp-p001.png` 본문 한글이 전부 □. glyph-risk 는 비어 있거나 U+25A1.

## 행동

랭킹을 폐기한다. 글꼴 경로를 넣고 **새 out-dir** 으로 다시 돈다.
같은 폴더의 PNG 캐시가 오염 시트를 재사용한다.

```bash
RHWP_FONT_PATH_DIR=/opt/hancom/fonts \
  venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 2 \
  --out-dir /tmp/rhwp-fidelity-plan-fonts
```

본문이 살아난 뒤에만 범위를 넓힌다. 원문자만 네모면 09장 (문서 후보).

관련: `references/17_tofu_harness.md`, `09_tofu.md`.
전사: `fixtures/transcripts/tofu_harness.txt`.
정지 F14.
