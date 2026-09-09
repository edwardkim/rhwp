# Task #1595 — Stage 1+2 완료보고서 (RED + 수정)

**브랜치**: `local/task1595`

## Stage 1 (RED)
"CLICKHERE" 기대 테스트(field.rs:217, section.rs:2427)를 올바른 "CLICK_HERE" 기대로 갱신 → RED.

## Stage 2 (수정)
`field.rs:180` `ClickHere => "CLICK_HERE"` (언더스코어 교정).

## 검증
- `field_begin_emits_type_attr` GREEN, `cargo test --lib` 1969/0, baseline 4/4, clippy 무경고.

## 다음
Stage 3 — fidelity 통제 비교 + 한글 오라클 붕괴 해소율 측정 + opengov 가드.
