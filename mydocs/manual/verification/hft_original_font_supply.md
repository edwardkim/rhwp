---
kind: guide
status: active
canonical: mydocs/manual/verification/hft_original_font_supply.md
last_verified: 2026-10-23
---

# 독립 한글 표본의 글자와 기호 대조

한컴 Print와 실제 공급 글꼴의 글자 모양이나 전진폭이 다를 때 독립 HWPX 표본으로 확인한다.

```bash
venv/bin/python scripts/prepare_hft_hangul_font.py create-specimen \
  --template samples/hwpx/blank_hwpx.hwpx \
  --out output/pr-review/<id>/hangul-alphabet.hwpx
```

표본은 U+AC00–U+D7A3 11,172자를 담는다. 기본 요청 글꼴은 한양신명조다.
다른 글꼴을 비교할 때는 `--face`에 원문의 글꼴 이름을 지정한다. 글머리 기호나 화살표가
필요하면 `--with-bullets`, `--with-arrows`를 추가한다.

[한컴 Print 절차](../mcp_hwp2024Convert_usage.md)에 따라 표본을 출력한다. 입력·PDF의 SHA-256,
한컴 제품과 버전, 실제 선택한 글꼴을 기록한다. PDF는 글자 모양·전진폭 대조에 사용하며
PDF의 글리프를 모아 새 글꼴을 만들거나 설치하지 않는다. 정본 보호 문서에 공급할 글꼴은
[공식 글꼴 공급 목록](../../verification/protected-font-supply/manifest.json)을 확인한다.
