# Task #249 단계별 완료보고서: 단계 2 — 문단 border_fill margin 반영

> Issue: [#249](https://github.com/edwardkim/rhwp/issues/249)
> 완료일: 2026-04-22

---

## 구현 내용

`border_fill` 렌더링 시 문단의 `margin_left`/`margin_right`를 반영하여 테두리 박스 위치·폭이 텍스트 영역과 정확히 일치하도록 수정.

## 변경 파일

| 파일 | 변경 내용 |
|------|-----------|
| `src/renderer/layout/paragraph_layout.rs` | border_fill rect 계산에 `margin_left`/`margin_right` 적용 |

## 주요 구현 사항

- 기존: border_fill rect가 문단 전체 폭(여백 무시)으로 계산됨
- 수정: `rect.x += margin_left`, `rect.width -= margin_left + margin_right` 적용
- 텍스트 영역과 테두리 박스의 좌/우 경계가 정확히 일치

## 검증

- `cargo test`: 793개 통과, 0 실패
- Visual Diff: 문단 테두리가 한컴 렌더링과 일치 확인
