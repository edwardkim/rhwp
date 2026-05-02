# PR #406 검토 — 동일 문단 inline TAC 그림 페이지네이션 정정 (#402)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#406](https://github.com/edwardkim/rhwp/pull/406) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) — 신뢰 컨트리뷰터 |
| 이슈 | [#402](https://github.com/edwardkim/rhwp/issues/402) (closes #402) |
| base / head | `devel` ← `planet6897:local/task402` |
| 변경 규모 | +1,343 / -25, 16 files (8 commits) |
| mergeable | `CONFLICTING` (DIRTY) — devel 와 충돌 |
| CI | (작성자 확인) cargo test 1023 passed, 0 failed |
| 검토 일자 | 2026-04-29 |

## 본질

같은 paragraph 의 `treat_as_char=true` inline 컨트롤이 2개 이상 서로 다른 line_seg 에 배치된 구조 (예: ls[0]=표, ls[1]=그림) 에서 두 가지 누락:

1. **레이아웃 y 좌표 결정 누락** (`layout.rs::layout_shape_item`)
   - 두 번째 이후 그림의 `pic_y` 가 `para_start_y[para_index]` 단일값에 고정 → 표 시작 위치에 겹침
   - 정정: `control_index` 보다 앞선 인덱스에 같은 paragraph 의 TAC 컨트롤이 존재하면 `para_start_y` 를 진행된 `y_offset` 으로 갱신
2. **페이지네이션 높이 누적 누락** (`typeset.rs::typeset_table_paragraph`)
   - inline TAC 그림이 `PageItem::Shape` 로 push 만 되고 `current_height` 누적 없음 → 페이지 분할 미트리거 + viewBox 초과
   - 정정: 선행 TAC 가 있는 inline TAC 그림의 `line_segs[prior_tac_count]` 높이 산출 + 페이지 초과 시 `advance_column_or_new_page()`

## 충돌 origin

PR 의 8 commits 누적 = Task #398 (3) + Task #402 (3) + devel merge (2). PR #401 v2 머지로 Task #398 commits 가 이미 devel 에 흡수 + Task #398 v2 정정 (`0d7e776`) 적용 → PR 측 base 와 분기. 본질 변경은 Task #402 의 3 commits 만.

## 처리 방향

**옵션 A — Task #402 핵심 3 commits 만 분리 cherry-pick** (권장).

이유:
1. Task #398 commits 는 이미 PR #401 v2 cherry-pick 으로 흡수 (회귀 정정 포함)
2. 작성자 변경의 본질은 Task #402 (3 commits): `0054a27`, `38bea10`, `f710732`
3. 작성자 attribution 보존 가능

같은 작성자의 PR #401 / #415 와 동일 패턴 (다른 PR 변경 누적 → 핵심 commits 만 분리 cherry-pick).

## dry-run cherry-pick 결과

`local/pr406` 브랜치 (`local/devel` 분기) 에서:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `97eeaf9` (← `0054a27`) | @planet6897 | Stage 1: 진단 로깅으로 가설 확정 |
| `65907ae` (← `38bea10`) | @planet6897 | Stage 2: inline TAC 그림 페이지네이션 수정 |
| `517b10a` (← `f710732`) | @planet6897 | Stage 3: 회귀 검증 통과 + 최종 보고서 |

cherry-pick 결과:
- Stage 1, 2 자동 적용
- Stage 3 의 `mydocs/orders/20260428.md` add/add 충돌 (다른 PR 통합 결과 누적) → HEAD (devel 통합 orders) 유지 후 `--continue` (PR #401 v2 와 같은 패턴)

## 검증 (cherry-pick 후)

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1050 passed** (PR #400 시점 동일, 회귀 없음) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 21s, 4,115,251 bytes (PR #400 시점 4,114,661 → +590) |

## 시각 판정

**한컴 정답지 (한컴 2010 / 2022) vs rhwp 출력 비교** — 작업지시자 직접 판정.

| 비교 | devel (PR #406 미적용) | PR #406 적용 |
|------|----------------------|--------------|
| 전체 페이지 수 | 27 | **30** (+3, 분할 정상화) |
| 7쪽 | 표 위 파이 차트 겹침 + 페이지 초과 (회귀) | 표만 정상 (한컴 PDF 일치) |
| 8쪽 | (다음 표 PartialTable 연속) | 파이 차트 정상 배치 |

산출물:
- SVG (devel): `output/svg/pr406-devel-baseline/2025년 기부·답례품 실적 지자체 보고서_양식_{007,008}.svg`
- SVG (PR #406): `output/svg/pr406-visual/2025년 기부·답례품 실적 지자체 보고서_양식_{007,008}.svg`
- Canvas (rhwp-studio): WASM `pkg/rhwp_bg.wasm` (4,115,251 bytes)

**작업지시자 시각 판정 결과:**
- SVG 내보내기: ✅ **통과**
- Canvas 경로 (rhwp-studio): ✅ **통과**

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1050 + svg_snapshot 6/6 + clippy 0 + WASM |
| 시각 판정 게이트 (push 전 필수) | ✅ 작업지시자 SVG + Canvas 직접 판정 통과 |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr406` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | ✅ 머지 + push 후 close 예정 |
| PR 댓글 톤 — 차분하고 사실 중심 | ✅ |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr406` → `local/devel` → `devel` 머지 + push
3. PR #406 close + 작성자 댓글 (이슈 #402 자동 close)

## 참고

- PR: [#406](https://github.com/edwardkim/rhwp/pull/406)
- 이슈: [#402](https://github.com/edwardkim/rhwp/issues/402)
- 같은 작성자 머지 PR: [#401 v2](https://github.com/edwardkim/rhwp/pull/401), [#415](https://github.com/edwardkim/rhwp/pull/415)
- 작성자 비범위 정황: 9쪽 orphan heading → #404 분리 등록 (PR #408 후속 처리)
