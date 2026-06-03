# Stage 2 완료보고서 — #1046: 페이지네이터 force-break 훅 + LayoutOverflow 필드 확장

- 타스크: #1046 (M100), 브랜치 `local/task1046`
- 단계: Stage 2 (순수 메커니즘, 동작 불변)
- 작성일: 2026-05-21

## 1. 변경 내역

### (a) `src/renderer/typeset.rs` — force-break 훅
- `typeset_section`에 `force_break_before: &HashSet<usize>`(para_idx) 파라미터 추가.
- 문단 루프 진입부(기존 `force_page_break` 처리 직후)에 삽입:
  ```rust
  if force_break_before.contains(&para_idx) && !st.current_items.is_empty() {
      st.force_new_page();
  }
  ```
  → 등록된 para_idx가 현재 페이지에 항목이 있으면 새 페이지에서 시작. **빈 셋이면 무동작**.
- 단위테스트 `test_typeset_force_break_before_hint` 추가: 1페이지에 들어갈 3문단을
  hint={1} 시 para0=1페이지 / para1,2=2페이지로 분리됨 검증.

### (b) `src/renderer/layout.rs` — LayoutOverflow 필드 확장
- `LayoutOverflow`에 `section_index: usize`, `is_first_in_column: bool` 추가.
- 항목 루프 `for item in col_content.items.iter()` → `.enumerate()` 로 순번 확보.
- 기록부에서 `section_index = page_content.section_index`, `is_first_in_column =
  (item_ordinal == 0)` 채움.
- Display 포맷에 `sec=`/`first=` 추가 (파싱 의존 테스트 없음 — 안전).

### (c) `src/document_core/queries/rendering.rs` — 호출부
- 빈 셋 `no_force_breaks` 선언 후 typeset_section 호출에 전달 (Stage 2 무동작).
  Stage 3에서 overflow 측정 결과로 채울 예정.

### (d) 미변경: `src/renderer/pagination/engine.rs` (fallback Paginator)
- blast radius 최소화 위해 fallback 엔진은 미변경 (reflow는 기본 typeset 경로 전용).
  `RHWP_USE_PAGINATOR=1` fallback은 reflow 미적용 — 문서화.

## 2. 검증

- `cargo build/clippy --release --lib`: 무경고.
- `cargo test --release`: **1516 passed, 0 failed** (force-break 단위테스트 +1).
- 대상 샘플 overflow **16건 불변** (무동작 확인). 새 로그로 분류 가시화:
  - `first=true` (page-larger, reflow 제외): **1건** — page=61 pi=323.
  - `first=false` (이월 대상): **15건** — pi=567(92쪽 nested)도 first=false라 1회 이월 후
    잔존 예상(수렴 가드로 처리).
  - 섹션 1의 4건(pi=268/354/357/406) 모두 first=false → 이월 대상.

## 3. 다음 단계 (Stage 3)
`paginate()`에 reflow 루프: paginate → 전 페이지 layout 측정 → overflow 중 first=false 를
`(section, para)` hint 로 누적 → 재paginate → 수렴/캡. `no_force_breaks` 를 실제 per-section
셋으로 대체.
