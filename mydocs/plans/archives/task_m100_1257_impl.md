# 구현계획서 — Task #1257: 미주 between-notes 7mm — typeset vpos 통일

- 이슈: edwardkim/rhwp#1257 · 브랜치: `local/task1257` (from `local/task1256`)
- 수행계획서: `task_m100_1257.md` · 접근: A (typeset vpos 통일)

## 0. 정밀 지점 (코드 확정)

`src/renderer/typeset.rs` 미주 방출 루프 between-notes 주입(2200-2233):
```rust
let extra_gap = (between_notes - prev_spacing).max(0);   // 7mm - prev 자연 trailing
if extra_gap > 0 {
    let pagination_gap = endnote_between_notes_pagination_margin(shape); // (between-1984)*3/4 = 0 for 7mm
    if pagination_gap > 0 { vpos_offset += pagination_gap; }   // ← 7mm base는 vpos 미반영
    last_seg.line_spacing = between_notes;                     // ← render y_offset용 주입(desync 원인)
}
let endnote_start = vpos_offset;
// 각 para: ls.vertical_pos += endnote_start
```
→ 제목 stored vpos 가 7mm 미포함 = render 절대-vpos(end_y) 누락의 근원.

**A안 핵심:** `vpos_offset += extra_gap`(full) 로 제목 vpos 에 7mm 반영. 그러면 render end_y 가 7mm
포함 → 모든 분기 자동 정합. 단 render y_offset(line_spacing 주입 기반)과 **이중가산** 가능 →
주입/복원 단일화 필요.

## 1. 단계 계획

### Stage 1 — POC/타당성 (env-gated 측정)
- `RHWP_EN_VPOS_UNIFY=1` 가드로 `vpos_offset += extra_gap`(full) 프로토타입.
- 측정: 전 미주 문서 페이지 수(2022/2023/10월/미주사이20/구분선아래20/3-11), 대표 헤더(문5/26/29,
  10월 문7/8/11/28, 2023 문18/21) 정합, 오버플로우.
- 이중가산 여부(line_spacing 주입 유지 시 render 갭 과다) 확인 → 주입 제거 조합도 측정.
- **산출물**: 페이지 수 영향표 + 정합 영향 + 설계 결론(주입 제거 범위, pagination 보정 필요성).
  → `task_m100_1257_stage1.md`. **A안 페이지 회귀 시 설계 재논의(승인).**

### Stage 2 — typeset vpos 반영 (정식)
- POC 결론대로 `vpos_offset += extra_gap` 정식화 + 이중가산 제거(line_spacing 주입 또는 render
  복원 중 단일화). pagination height-pass 일관화. env 가드 제거.

### Stage 3 — render 7mm 특례 정리
- #1256 복원 분기·#1246 rescue·column-bottom cap 7mm 특례가 vpos 통일 후 불필요/중복이면
  단순화(제거 또는 가드 축소). 단위테스트 동반 갱신.

### Stage 4 — 광범위 회귀
- 정합표(한컴 PDF): 2022(문5/24/26/29), 2023(문18/21), 10월(문7/8/11/28).
- 페이지·오버플로우: 3-11(0, 21p), 미주사이20(24p), 구분선아래20(23p), 2022(23p).
- `cargo test` 전체 + #1189/#1209→#1256/#1246.

### Stage 5 — 단위테스트 갱신 + 최종 보고서
- typeset 레벨 단위테스트(vpos 7mm 반영) + render 단순화 회귀 테스트. 최종 보고서.

## 2. 리스크/가드
- **R1 pagination 회귀**: Stage1 POC 가 게이트. 페이지 수 변동 시 Stage2 진행 전 승인.
- **R2 이중가산**: vpos+line_spacing 동시 → 갭 2배. Stage1 에서 조합 측정해 단일화 결정.
- **R3 #1256 중복**: vpos 통일 후 #1256 복원이 무발동(end_y≥y_offset)이어야 정상. Stage3 정리.

## 3. 검증 커맨드
```
RHWP_EN_VPOS_UNIFY=1 rhwp dump-pages samples/<doc> -p 0   # 페이지 수
RHWP_EN_VPOS_UNIFY=1 rhwp export-svg samples/<doc> -p N   # 정합
cargo test
```

---
승인 불요(수행계획 승인됨). Stage 1 POC 즉시 착수.
