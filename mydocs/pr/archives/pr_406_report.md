# PR #406 처리 보고서 — 동일 문단 inline TAC 그림 페이지네이션 정정 (#402)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#406](https://github.com/edwardkim/rhwp/pull/406) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) |
| 이슈 | [#402](https://github.com/edwardkim/rhwp/issues/402) (closes #402) |
| 처리 결정 | **cherry-pick 머지** (Task #402 핵심 3 commits 분리) |
| 처리 일자 | 2026-04-29 |

## 처리 절차

### Stage 0: 검토 (옵션 A 결정)

PR 의 8 commits 누적 분석 → Task #398 (3 commits) 은 이미 PR #401 v2 cherry-pick 으로 devel 흡수 (v2 회귀 정정 포함). 본질 변경은 Task #402 의 3 commits.

→ PR #401 / #415 와 같은 분리 cherry-pick 패턴 채택.

### Stage 1: cherry-pick

`local/pr406` 브랜치 (`local/devel` 분기) 에서 3 commits cherry-pick — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `97eeaf9` (← `0054a27`) | @planet6897 | Stage 1: 진단 로깅으로 가설 확정 |
| `65907ae` (← `38bea10`) | @planet6897 | Stage 2: inline TAC 그림 페이지네이션 수정 |
| `517b10a` (← `f710732`) | @planet6897 | Stage 3: 회귀 검증 통과 + 최종 보고서 |

cherry-pick 결과:
- Stage 1, 2 자동 적용
- Stage 3 의 `mydocs/orders/20260428.md` add/add 충돌 → HEAD (devel 통합 orders) 유지 후 `--continue`

### Stage 2: 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1050 passed** (회귀 없음) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 21s, 4,115,251 bytes |

### Stage 3: 시각 판정 (작업지시자 직접)

**비교 자료**: 한컴 hwpx + 한컴 origin PDF (한컴 2010 / 2022 정답지) vs rhwp 출력

| 비교 | devel (미적용) | PR #406 적용 |
|------|---------------|--------------|
| 전체 페이지 수 | 27 | **30** (+3 분할 정상화) |
| 7쪽 | 표 위 파이 차트 겹침 + 페이지 초과 | 표만 정상 (한컴 일치) |
| 8쪽 | (다음 표 연속) | 파이 차트 정상 배치 |

| 경로 | 시각 판정 결과 |
|------|--------------|
| SVG 내보내기 | ✅ 통과 |
| Canvas (rhwp-studio) | ✅ 통과 |

## 변경 요약

### Task #402 핵심 — 동일 문단 inline TAC 그림 y 좌표 + 페이지 분할

| 파일 | 변경 |
|------|------|
| `src/renderer/layout.rs::layout_shape_item` | 선행 TAC 컨트롤이 같은 paragraph 에 존재하면 `para_start_y` 를 진행된 `y_offset` 으로 갱신 (단순 y_offset 비교 대신 **선행 TAC 존재 여부** 가드) |
| `src/renderer/typeset.rs::typeset_table_paragraph` | 선행 TAC 그림의 `line_segs[prior_tac_count]` 높이 산출 + 페이지 초과 시 `advance_column_or_new_page()` + `current_height += line_h` |

**주의**: 기본 페이지네이션 엔진은 `typeset.rs::TypesetEngine` (engine.rs 는 `RHWP_USE_PAGINATOR=1` 일 때만 fallback). engine.rs 의 동일 누락은 본 PR 범위 밖.

### 회귀 검증 정황

- 10개 대표 샘플 LAYOUT_OVERFLOW 카운트 회귀 없음 (작성자 검증)
- 페이지 수 27→30 (분할로 인한 정상 증가)

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1050 + svg_snapshot 6/6 + clippy 0 + WASM |
| 시각 판정 게이트 (push 전 필수) | ✅ 작업지시자 SVG + Canvas 양 경로 직접 판정 통과 |
| output 폴더 가이드라인 | ✅ `output/svg/pr406-visual/`, `output/svg/pr406-devel-baseline/` |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr406` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | ✅ 머지 + push 후 close |
| PR 댓글 톤 | ✅ |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr406` → `local/devel` → `devel` 머지 + push
3. PR #406 close + 작성자 댓글 (이슈 #402 자동 close)

## 참고

- 검토 문서: `mydocs/pr/pr_406_review.md`
- PR: [#406](https://github.com/edwardkim/rhwp/pull/406)
- 이슈: [#402](https://github.com/edwardkim/rhwp/issues/402)
- 같은 작성자 머지 PR: [#401 v2](https://github.com/edwardkim/rhwp/pull/401), [#415](https://github.com/edwardkim/rhwp/pull/415)
- 비범위 (분리 등록): 9쪽 orphan heading → #404 (PR #408 후속)
