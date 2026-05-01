# PR #408 검토 — Task #404 heading-orphan vpos 기반 보정

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#408](https://github.com/edwardkim/rhwp/pull/408) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) — 신뢰 컨트리뷰터 |
| 이슈 | [#402](https://github.com/edwardkim/rhwp/issues/402) (PR #406 로 close 완료), [#404](https://github.com/edwardkim/rhwp/issues/404) |
| base / head | `devel` ← `planet6897:local/task404` |
| 변경 규모 | +1,217 / -12, 13 files (14 commits) |
| mergeable | `MERGEABLE` (BEHIND) |
| 검토 일자 | 2026-04-29 |

## PR 의 14 commits 누적 정황

| 영역 | commits | 처리 |
|------|---------|------|
| Task #398 (3) | `4454573`, `d570ab7`, `50f4164` | PR #401 v2 로 흡수 완료 (devel) |
| Task #402 (3) | `0054a27`, `38bea10`, `f710732` | **PR #406 으로 흡수 완료 (devel)** |
| 샘플 PDF 갱신 (1) | `c43f6c8` | devel `c2944a4` 와 동일 결과 (1474284 bytes) → no-op |
| **Task #404 (4)** | `36a457c`, `25b27b2`, `42b5136`, `3a1c9d6` | **본 PR 의 본질 변경 — 분리 cherry-pick 대상** |
| devel merge (3) | `e99e88e`, `3d10ccb`, `cc1ceef` | merge commit (skip) |

## 본질 — 이슈 #404: heading-orphan 패턴

### 원인

- HWP 원본은 paragraph 단위 vpos (LineSeg.vertical_pos) 가 페이지 본문 영역을 넘으면 다음 페이지로 push
- rhwp 는 누적 height 기반 fit 결정 → vpos 미세 초과 (886 HU = 0.31mm) 케이스에서 누적 height 는 fit 으로 판정 → 헤딩만 페이지 끝에 잔류, 후속 표는 다음 페이지로 → orphan 패턴

### 수정 (`typeset.rs::typeset_section` 메인 루프)

5 조건 AND trigger (false positive 차단):

| # | 조건 | 의미 |
|---|------|------|
| A | `!current_items.is_empty()` | 페이지 첫 item 자기참조 회피 |
| B | `wrap_around_cs < 0 && col_count == 1` | 단일 단 + non-wrap |
| C | `current_height + para_h_px <= avail` | 현재 fit |
| D | `vpos_end > page_bottom_vpos + 283` | vpos 기준 1mm 초과 |
| E | `next_h > 30 && current+para+next > avail` | 다음 substantial + fit 불가 |

발동 시 `st.advance_column_or_new_page()` 호출.

### False Positive 차단 효과

작성자 Stage 1 진단: vpos overflow paragraph 41건 중 1건만 진짜 orphan. 조건 E 가 핵심 필터.

### 설계 노트

`page_top_vpos` 는 `TypesetState` 필드 추적 대신 `current_items` 첫 item 의 `para_index` 로 매 iteration 즉시 계산 (typeset_paragraph 내부 페이지 flush 와 필드 setter 가 동기 안 되는 문제 회피).

## 처리 방향

**옵션 A — Task #404 4 commits 만 분리 cherry-pick** (PR #406 와 같은 패턴).

이유:
1. Task #398, #402 commits 는 이미 devel 에 흡수
2. 샘플 PDF 갱신은 devel 에 동일 결과로 존재 (no-op)
3. 본질 변경은 Task #404 의 4 commits

## dry-run cherry-pick 결과

`local/pr408` 브랜치 (`local/devel` 분기) 에서 4 commits cherry-pick — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `571758a` (← `36a457c`) | @planet6897 | Task #404: 수행계획서 + 구현계획서 작성 |
| `d3ab7b5` (← `25b27b2`) | @planet6897 | Stage 1: vpos 진단 + 가설 확정 |
| `1575ede` (← `42b5136`) | @planet6897 | Stage 2: heading-orphan trigger 구현 |
| `25c3150` (← `3a1c9d6`) | @planet6897 | Stage 3: 회귀 검증 통과 + 최종 보고서 |

cherry-pick 결과:
- 앞 3 commits 자동 적용
- Stage 3 의 `mydocs/orders/20260428.md` add/add 충돌 (PR #406 와 같은 패턴) → HEAD 유지 후 `--continue`

## 검증 (cherry-pick 후)

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1050 passed** (PR #406 동일, 회귀 없음) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 18s, 4,124,155 bytes (PR #406 시점 4,115,251 → +8,904) |

## 시각 판정

**한컴 정답지 (한컴 2010 / 2022 + origin PDF) vs rhwp 출력 비교** — 작업지시자 직접 판정.

| 비교 | devel baseline (Task #404 미적용) | PR #408 적용 |
|------|-----------------------------------|--------------|
| 9쪽 | pi=83 "(7) 다수 기부자 현황" 헤딩 잔류 (orphan) | pi=83 헤딩 미표시 (다음 페이지 push) |
| 10쪽 | (후속 표 pi=84/85 만 시작) | pi=83 헤딩 + pi=84/85 표 함께 배치 (한컴 일치) |
| 전체 페이지 수 | 30 | 30 (페이지 수 변화 없음, 재배치만) |

산출물:
- SVG (devel): `output/svg/pr408-devel-baseline/2025년 기부·답례품 실적 지자체 보고서_양식_{009,010}.svg`
- SVG (PR #408): `output/svg/pr408-visual/2025년 기부·답례품 실적 지자체 보고서_양식_{009,010}.svg`
- Canvas (rhwp-studio): WASM `pkg/rhwp_bg.wasm` (4,124,155 bytes)

**작업지시자 시각 판정 결과:**
- SVG 내보내기: ✅ **통과**
- Canvas 경로 (rhwp-studio): ✅ **통과**

## 비범위 (작성자 명시)

- 본 샘플 21쪽 pi=192 표 위치 차이 — TopAndBottom wrap 그림 + 빈 문단 line-height 압축 누적이 origin. **사전 존재 이슈** (Task #404 회귀 아님). 별도 등록 예정.
- `engine.rs::paginate_with_measured` (fallback 경로, `RHWP_USE_PAGINATOR=1`) — 동일 보정 미적용. 별도 후속.

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1050 + svg_snapshot 6/6 + clippy 0 + WASM |
| 시각 판정 게이트 (push 전 필수) | ✅ 작업지시자 SVG + Canvas 양 경로 직접 판정 통과 |
| 한컴 호환은 일반화보다 케이스별 명시 가드 | ✅ 5 조건 AND trigger (false positive 차단) — 단순 vpos check 회피 |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr408` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | ✅ 머지 + push 후 close 예정 |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr408` → `local/devel` → `devel` 머지 + push
3. PR #408 close + 작성자 댓글 (이슈 #404 자동 close)

## 참고

- PR: [#408](https://github.com/edwardkim/rhwp/pull/408)
- 이슈: [#404](https://github.com/edwardkim/rhwp/issues/404)
- 같은 작성자 머지 PR: [#401 v2](https://github.com/edwardkim/rhwp/pull/401), [#406](https://github.com/edwardkim/rhwp/pull/406), [#415](https://github.com/edwardkim/rhwp/pull/415)
