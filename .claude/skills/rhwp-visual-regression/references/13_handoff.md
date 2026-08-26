# 13 — 인계

이 스킬은 레이아웃 숫자만 책임진다.

## rhwp-form-fill 에서 돌아와 이 스킬

- 언제: 누름틀을 채운 뒤 레이아웃을 본다
- 명령: `render-diff <빈서식> <채움산출>`

## rhwp-table-exchange

- 언제: 표 CSV 왕복 후 칸 너비
- 명령: `render-diff 전후 후 필요하면 export-png`

## rhwp-safe-edit

- 언제: 원본을 계획서로 여러 번 고침
- 명령: `run --verify 후 render-diff`

## rhwp-security-sweep

- 언제: 배포 전 숨은 글·주입
- 명령: `inspect. 비교 전에 먼저`

## rhwp-doc-triage

- 언제: 문서가 뭔지만
- 명령: `info / explain. 비교하지 않음`

## rhwp-work-receipt

- 언제: 작업 영수증이 필요
- 명령: `replay. 이 스킬은 레이아웃만`

이 스킬 안에서 이웃 SKILL.md 를 재작성하지 않는다.
