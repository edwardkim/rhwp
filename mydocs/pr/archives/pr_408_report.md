# PR #408 처리 보고서 — Task #404 heading-orphan vpos 기반 보정

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#408](https://github.com/edwardkim/rhwp/pull/408) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) |
| 이슈 | [#402](https://github.com/edwardkim/rhwp/issues/402) (PR #406 로 close), [#404](https://github.com/edwardkim/rhwp/issues/404) (closes) |
| 처리 결정 | **cherry-pick 머지** (Task #404 핵심 4 commits 분리) |
| 처리 일자 | 2026-04-29 |

## 처리 절차

### Stage 0: 검토 (옵션 A 결정)

PR 의 14 commits 분석:
- Task #398 (3): PR #401 v2 로 흡수 완료
- Task #402 (3): PR #406 으로 흡수 완료
- 샘플 PDF: devel 와 동일 결과 (no-op)
- **Task #404 (4)**: 본질 변경, 분리 cherry-pick 대상

→ PR #406 / #401 / #415 와 같은 분리 cherry-pick 패턴 채택.

### Stage 1: cherry-pick

`local/pr408` 브랜치 (`local/devel` 분기) 에서 4 commits cherry-pick — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `571758a` (← `36a457c`) | @planet6897 | Task #404: 수행계획서 + 구현계획서 작성 |
| `d3ab7b5` (← `25b27b2`) | @planet6897 | Stage 1: vpos 진단 + 가설 확정 |
| `1575ede` (← `42b5136`) | @planet6897 | Stage 2: heading-orphan trigger 구현 |
| `25c3150` (← `3a1c9d6`) | @planet6897 | Stage 3: 회귀 검증 통과 + 최종 보고서 |

cherry-pick 결과:
- 앞 3 commits 자동 적용
- Stage 3 의 `mydocs/orders/20260428.md` add/add 충돌 → HEAD 유지 후 `--continue`

### Stage 2: 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1050 passed** (회귀 없음) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 18s, 4,124,155 bytes (+8,904 from PR #406) |

### Stage 3: 시각 판정 (작업지시자 직접)

**비교 자료**: 한컴 hwpx + 한컴 origin PDF (한컴 2010 / 2022 정답지) vs rhwp 출력

| 비교 | devel baseline | PR #408 적용 |
|------|---------------|--------------|
| 9쪽 | pi=83 헤딩 잔류 (orphan) | pi=83 미표시 (push) |
| 10쪽 | 후속 표만 시작 | pi=83 헤딩 + pi=84/85 표 함께 배치 (한컴 일치) |
| 전체 페이지 수 | 30 | 30 (재배치만) |

| 경로 | 시각 판정 결과 |
|------|--------------|
| SVG 내보내기 | ✅ 통과 |
| Canvas (rhwp-studio) | ✅ 통과 |

## 변경 요약

### Task #404 핵심 — heading-orphan vpos 기반 trigger

| 파일 | 변경 |
|------|------|
| `src/renderer/typeset.rs::typeset_section` | 메인 루프에 5 조건 AND trigger 추가 (current fit + vpos overflow + next substantial + next doesn't fit + single column non-wrap) |

5 조건 AND 의 의미:

| # | 조건 | 차단하는 false positive |
|---|------|------------------------|
| A | `!current_items.is_empty()` | 페이지 첫 item 자기참조 |
| B | `wrap_around_cs < 0 && col_count == 1` | 단일 단 + non-wrap (다단/wrap 회피) |
| C | `current_height + para_h_px <= avail` | 현재 fit (이미 overflow 케이스 회피) |
| D | `vpos_end > page_bottom_vpos + 283` | vpos 1mm 초과 (정상 fit 회피) |
| E | `next_h > 30 && current+para+next > avail` | 다음 substantial + fit 불가 (text-only 페이지 + wrap-around 압축 회피) |

작성자 Stage 1 진단: vpos overflow 41건 중 40건은 wrap-around 페이지에서 vpos↔px 비율 어긋남 (페이지 8 pi=57 TAC 그림 + 빈 문단 19개) → 조건 E 가 핵심 필터.

### 회귀 검증 정황

- 10개 대표 샘플 LAYOUT_OVERFLOW: 회귀 0건, 타겟 -15 (57→42), kps-ai -1 (5→4) 개선
- 페이지 수 변화 없음 (재배치만)

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1050 + svg_snapshot 6/6 + clippy 0 + WASM |
| 시각 판정 게이트 (push 전 필수) | ✅ 작업지시자 SVG + Canvas 양 경로 직접 판정 통과 |
| 한컴 호환은 일반화보다 케이스별 명시 가드 | ✅ 5 조건 AND trigger (단순 vpos check 회피) |
| output 폴더 가이드라인 | ✅ `output/svg/pr408-visual/`, `output/svg/pr408-devel-baseline/` |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr408` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | ✅ 머지 + push 후 close |

## 비범위 (작성자 명시, 별도 후속)

- 본 샘플 21쪽 pi=192 표 위치 차이 — TopAndBottom wrap 그림 + 빈 문단 line-height 압축 누적이 origin. **사전 존재 이슈** (Task #404 회귀 아님). 별도 등록 예정.
- `engine.rs::paginate_with_measured` (fallback 경로, `RHWP_USE_PAGINATOR=1`) — 동일 보정 미적용. 별도 후속.

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr408` → `local/devel` → `devel` 머지 + push
3. PR #408 close + 작성자 댓글 (이슈 #404 자동 close)

## 참고

- 검토 문서: `mydocs/pr/pr_408_review.md`
- PR: [#408](https://github.com/edwardkim/rhwp/pull/408)
- 이슈: [#404](https://github.com/edwardkim/rhwp/issues/404)
- 같은 작성자 머지 PR: [#401 v2](https://github.com/edwardkim/rhwp/pull/401), [#406](https://github.com/edwardkim/rhwp/pull/406), [#415](https://github.com/edwardkim/rhwp/pull/415)
