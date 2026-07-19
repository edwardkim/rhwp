# Stage 1 완료 보고서 — Task #1238: between-notes margin 누락 원인 특정

- **이슈**: #1238 (M100)
- **브랜치**: `feature/issue-1238-between-notes-margin`
- **단계**: Stage 1 / 3 (코드 무변경, 조사)
- **작성일**: 2026-06-02

## 핵심 결론

1. **증상 실측**: 문22 above-gap 11.3px (정상 제목 27~38px). 직전이 **다줄 문단**일 때만 발생.
2. **between-notes 적용**: `pagination_gap = 1984-1984 = 0`(vpos 미증가). between-notes 는
   **오직 직전 미주 마지막 문단의 `last_seg.line_spacing = 1984` 주입**으로만 적용됨(진단 실측).
   → 가설(extra_gap=0)은 반증. 주입은 항상 발생, render 가 무시가 진짜 원인.
3. **근본 원인**: `paragraph_layout.rs:4156` — 다줄 미주 문단(`endnote_line_vpos_base`) 경로에서
   **마지막 줄 trailing 을 `0.0` 으로 버림** → 주입된 line_spacing(between-notes) 미반영.
   단일줄 경로(L4173)는 line_spacing 포함 → 정상.

## 산출물

- `mydocs/tech/investigations/issue-1238/between_notes_multiline_1238.md` (실측 표 + 분기 + 회귀 게이트 설계).

## Stage 2 방향 (승인 요청)

`endnote_para_has_different_endnote_successor()` 헬퍼로 **다른 미주가 뒤따르는 마지막 문단**
(=between_notes 주입 위치)에만 다줄 경로 마지막 줄 trailing 복원. 단일줄 경로·같은 미주 내부·
문서 마지막 미주 무영향 → issue_1139/1189 이중가산 회피.

원인과 Stage 2 게이트 방향 확정 시 착수.
