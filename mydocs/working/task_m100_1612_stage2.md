# Stage 2 완료보고서 — Task #1612 (메트릭 정정)

**단계**: `compute_hwp_used_height` per-page 정정 + 테스트 · **브랜치**: `local/task1612`

## 수정 (`src/document_core/queries/rendering.rs`)

`compute_hwp_used_height` 에 **페이지 시작 vpos 오프셋(`base_top`) 차감** 추가:
- `base_top` = 이 페이지(cc) 첫 항목(문단/표 모두, host 문단 line_seg)의 top vpos.
- 두 반환점(reset 케이스·last-item 케이스)에서 `(bottom_hwpu − base_top).max(0)` 로 per-page 화.

표 항목도 host 문단 vpos 로 base 산출(첫 항목이 표인 페이지에서 base 가 하단 문단으로 잘못
잡히는 것 방지).

## 검증 (수동 dump-pages)

```
36398709 (정정 후): p1 diff −1.0(불변), p2 +71.7, p3 +166.0, p4 +109.3, p5 +3.8
  → "−3300px" 아티팩트 제거, per-page 수십~166px 정상
36387725 (단일): p1 diff −13.2(본문 vpos 640.7 vs 흐름 627.5), p2 footer 단독
```

## 단위 테스트
`rendering.rs` tests: `task1612_hwp_used_height_is_per_page_not_cumulative` — 누적 vpos(66568+)
문단을 page2 단으로 넣어 per-page 높이(<100px) 반환 단언(누적이면 ~933px). **GREEN**.
