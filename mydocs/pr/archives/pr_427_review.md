# PR #427 검토 — SvgRenderer defs 중복 방지를 HashSet 으로 통합 (#423)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#427](https://github.com/edwardkim/rhwp/pull/427) |
| 작성자 | [@oksure](https://github.com/oksure) (Hyunwoo Park) — 신뢰 컨트리뷰터, PR #395/#396 머지 통과 |
| 이슈 | [#423](https://github.com/edwardkim/rhwp/issues/423) (closes), **PR #395 리뷰 시 메인테이너가 후속 PR 후보로 안내한 사안** |
| base / head | `devel` ← `oksure:contrib/svg-defs-hashset-dedup` |
| 변경 규모 | +8 / -10, **1 file** (단일 파일), 1 commit |
| mergeable | `MERGEABLE` (BEHIND) |
| 검토 일자 | 2026-04-29 |

## 본질

PR #395 (그림 밝기/대비 효과) 리뷰 시 메인테이너가 안내한 후속 PR 후보 — `SvgRenderer` 의 defs 중복 방지를 `HashSet` 으로 통합하여 O(n) → O(1) 개선.

### 변경

| 항목 | Before | After |
|------|--------|-------|
| 필드 | `arrow_marker_ids: HashSet<String>` (arrow 전용) | `defs_ids: HashSet<String>` (범용) |
| `ensure_image_effect_filter()` | `defs.iter().any()` O(n) 선형탐색 | `defs_ids.insert()` O(1) |
| `ensure_brightness_contrast_filter()` | `defs.iter().any()` O(n) 선형탐색 | `defs_ids.insert()` O(1) |
| `ensure_arrow_marker()` | `arrow_marker_ids` contains+insert 2줄 | `defs_ids.insert()` 1줄 |
| `begin_page()` | `arrow_marker_ids.clear()` | `defs_ids.clear()` |

## 처리 방향

**옵션 A — 단일 commit cherry-pick** (PR 가 이미 단순/단일 commit, 작성자 attribution 보존).

이유:
1. 단일 파일 단순 리팩토링 (+8/-10)
2. 작성자 신뢰성 확정 (PR #395/#396 머지 통과)
3. 출력 변화 없음 정황 (HashSet 자료구조 변경만)

## dry-run cherry-pick 결과

`local/pr427` 브랜치 (`local/devel` 분기) 에서 단일 commit cherry-pick — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `ada647a` (← `1972b78`) | @oksure | refactor: SvgRenderer defs 중복 방지를 HashSet<String>으로 통합 (#423) |

cherry-pick 결과: 충돌 없이 자동 적용.

## 검증 (cherry-pick 후)

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1062 passed** (회귀 없음) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 23s, 4,178,840 bytes (-5,800 from PR #424, HashSet 통합 정리 효과) |

## 광범위 byte 단위 무회귀 검증

10 샘플 / 309 페이지 SVG 비교 (PR #424 머지 후 devel ↔ PR #427 적용):

**결과: 309/309 byte 단위 동일 (100%)** ✅

→ 자료구조 변경만으로 출력 영향 없음 검증 완료.

## 시각 판정 정황

본 PR 은 **단순 자료구조 리팩토링** (HashSet 통합) 으로 출력 변화 없음. 309/309 byte 단위 동일이 결정적 검증.

→ **시각 판정 불필요** (PR #405, #411, #400, #419 와 같은 패턴).

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1062 + svg_snapshot 6/6 + clippy 0 + WASM + byte 단위 무회귀 |
| 시각 판정 게이트 (push 전 필수) | ✅ 단순 리팩토링 (309/309 byte 동일) → 불필요 |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr427` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | ✅ 머지 + push 후 close |
| PR 댓글 톤 | ✅ |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr427` → `local/devel` → `devel` 머지 + push
3. PR #427 close + 작성자 댓글 (이슈 #423 자동 close)

## 참고

- PR: [#427](https://github.com/edwardkim/rhwp/pull/427)
- 이슈: [#423](https://github.com/edwardkim/rhwp/issues/423)
- 같은 작성자 머지 PR: [#395](https://github.com/edwardkim/rhwp/pull/395) (밝기/대비), [#396](https://github.com/edwardkim/rhwp/pull/396) (수식 렌더링)
- origin: PR #395 리뷰 시 메인테이너 안내 (HashSet dedup 별도 PR 후보)
