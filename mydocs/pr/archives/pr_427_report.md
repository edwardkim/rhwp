# PR #427 처리 보고서 — SvgRenderer defs 중복 방지를 HashSet 으로 통합

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#427](https://github.com/edwardkim/rhwp/pull/427) |
| 작성자 | [@oksure](https://github.com/oksure) (Hyunwoo Park) |
| 이슈 | [#423](https://github.com/edwardkim/rhwp/issues/423) (closes) |
| 처리 결정 | **cherry-pick 머지** (단일 commit) |
| 처리 일자 | 2026-04-29 |

## 처리 절차

### Stage 0: 검토 (옵션 A 결정)

PR #395 리뷰 시 메인테이너가 후속 PR 후보로 안내한 사안 — 이슈 #423 등록 후 작성자가 빠르게 PR 화. 단일 파일 단순 리팩토링 + 작성자 신뢰성 확정 → 옵션 A (단일 commit cherry-pick).

### Stage 1: cherry-pick

`local/pr427` 브랜치 (`local/devel` 분기) 에서 단일 commit cherry-pick — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `ada647a` (← `1972b78`) | @oksure | refactor: SvgRenderer defs 중복 방지를 HashSet<String>으로 통합 (#423) |

cherry-pick 결과: 충돌 없이 자동 적용.

### Stage 2: 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1062 passed** (회귀 없음) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 23s, 4,178,840 bytes (-5,800 from PR #424) |

### Stage 3: 광범위 byte 단위 비교

10 샘플 / 309 페이지 SVG 비교:

**결과: 309/309 byte 단위 동일 (100%)** ✅

→ 자료구조 변경 (HashSet 통합) 으로 출력 영향 없음 검증.

## 변경 요약

### 본질 — `src/renderer/svg.rs` 단일 파일 (+8/-10)

| 항목 | Before | After |
|------|--------|-------|
| 필드 | `arrow_marker_ids: HashSet<String>` | `defs_ids: HashSet<String>` |
| `ensure_image_effect_filter()` | `defs.iter().any()` O(n) | `defs_ids.insert()` O(1) |
| `ensure_brightness_contrast_filter()` | `defs.iter().any()` O(n) | `defs_ids.insert()` O(1) |
| `ensure_arrow_marker()` | contains+insert 2줄 | insert 1줄 |
| `begin_page()` | `arrow_marker_ids.clear()` | `defs_ids.clear()` |

### 영향

- 자료구조 변경만, 출력 byte 단위 동일
- O(n) 선형탐색을 O(1) 으로 개선
- 신규 defs 추가 시 보일러플레이트 감소 (contains+insert → insert)
- WASM 크기 -5,800 bytes (코드 정리 효과)

## 시각 판정 정황

단순 자료구조 리팩토링 패턴 — 309/309 byte 단위 동일이 결정적 검증.

→ **시각 판정 불필요** (PR #405, #411, #400, #419 와 같은 패턴).

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1062 + svg_snapshot 6/6 + clippy 0 + WASM + byte 단위 무회귀 |
| 시각 판정 게이트 (push 전 필수) | ✅ 리팩토링 패턴 (309/309 byte 동일) → 불필요 |
| output 폴더 가이드라인 | ✅ `output/svg/pr427-regression-test/` |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr427` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | ✅ 머지 + push 후 close |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr427` → `local/devel` → `devel` 머지 + push
3. PR #427 close + 작성자 댓글 (이슈 #423 자동 close)

## 참고

- 검토 문서: `mydocs/pr/pr_427_review.md`
- PR: [#427](https://github.com/edwardkim/rhwp/pull/427)
- 이슈: [#423](https://github.com/edwardkim/rhwp/issues/423)
- 같은 작성자 머지 PR: [#395](https://github.com/edwardkim/rhwp/pull/395), [#396](https://github.com/edwardkim/rhwp/pull/396)
- origin: PR #395 리뷰 시 메인테이너 후속 PR 안내 → 작성자가 즉시 PR 화
