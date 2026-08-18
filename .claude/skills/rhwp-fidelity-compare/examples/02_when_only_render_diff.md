# 예제 — 공식 PDF 가 없을 때 (render-diff 가 정직)

이슈 #5329. 실 에이전트 경로. gym 아님.

## 상황

사용자가 편집 전 `a.hwp` 와 편집 후 `b.hwp` 만 주고 "한컴이랑
같은지" 묻는다. PDF 는 없다.

## 하지 말 것

```bash
# 금지 — b 를 export-pdf 로 뽑아 오라클인 척
rhwp export-pdf b.hwp -o fake-oracle.pdf
venv/bin/python tools/fidelity_compare/fidelity_compare.py 0 3 \
  --source b.hwp --reference-pdf fake-oracle.pdf --label fake
```

자기 자신과 비교가 된다. F01.

## 정직한 다음 수

`rhwp-visual-regression` 을 연다.

```bash
rhwp render-diff a.hwp b.hwp
```

이 스킬 파일을 고치지 않는다 (F07).

관련: `references/01_when_to_use.md`, `21_vs_visual_regression.md`.
정지 F01.
