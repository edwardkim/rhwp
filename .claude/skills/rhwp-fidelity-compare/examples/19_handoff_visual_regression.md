# 예제 — visual-regression 인계

이슈 #5329. 실 에이전트 경로. gym 아님.

## 상황

공식 PDF 없이 편집 전후만 있다. 또는 사용자가 render-diff 를 원한다.

## 문장

"한컴 공식 PDF 가 없어 fidelity_compare 는 정직하지 않습니다.
`rhwp-visual-regression` 스킬의 render-diff 를 엽니다.
그 스킬 파일은 수정하지 않았습니다."

```bash
rhwp render-diff before.hwp after.hwp
```

관련: `references/21_vs_visual_regression.md`, `26_handoff.md`.
정지 F01, F07.
