# PR #410 처리 보고서 — Task #409 TopAndBottom Picture vert=Para chart 정정 + atomic TAC top-fit

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#410](https://github.com/edwardkim/rhwp/pull/410) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) |
| 이슈 | [#409](https://github.com/edwardkim/rhwp/issues/409) (closes) |
| 처리 결정 | **cherry-pick 머지** (Task #409 본질 3 commits 분리) |
| 처리 일자 | 2026-04-29 |

## 처리 절차

### Stage 0: 검토 (옵션 A 결정)

PR 의 27 commits 중 본질은 Task #409 의 v1/v2/v3 — `368a869`, `d1f19e9`, `fa8f923`. 나머지는 다른 PR (#401 v2 / #406 / #408) 로 흡수 완료 + 샘플/계획서/Stage 보고서/merge.

→ PR #406/#408 와 같은 분리 cherry-pick 패턴 채택.

### Stage 1: cherry-pick

`local/pr410` 브랜치 (`local/devel` 분기) 에서 3 commits cherry-pick — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| (← `368a869`) | @planet6897 | v1: layout.rs prev_has_overlay_shape 가드 확장 (Picture + TopAndBottom/vert=Para) |
| `bd8ea80` (← `d1f19e9`) | @planet6897 | v2: typeset.rs::typeset_section chart 높이 누적 |
| `1f5dcc5` (← `fa8f923`) | @planet6897 | v3: typeset.rs::typeset_paragraph atomic TAC top-fit (60px tolerance) |

cherry-pick 결과: 충돌 없이 자동 적용.

### Stage 2: 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1050 passed** (회귀 없음) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 40s, 4,116,929 bytes |

### Stage 3: 시각 판정 (작업지시자 직접)

**비교 자료**: 한컴 hwpx + 한컴 origin PDF (한컴 2010 / 2022 정답지) vs rhwp 출력

| 페이지 | devel baseline | PR #410 적용 |
|--------|---------------|--------------|
| 21쪽 | 2x1 표 페이지 하단 잘림 | 차트 직하 정상 |
| 22쪽 | 차트로 시작 (헤딩+표 누락) | (4) 헤딩 + 10x5 표 정상 |
| 23쪽 | 차트 24쪽으로 밀림 | 막대 차트 하단 정상 |
| 24쪽 | 차트로 시작 | 2x1 표 → (6) 헤딩 → 파이차트 |

| 경로 | 시각 판정 결과 |
|------|--------------|
| SVG 내보내기 | ✅ 통과 |
| Canvas (rhwp-studio) | ✅ 통과 |

### Stage 4: 회귀 검증

**이슈 #426 영향 확인** (방금 등록한 캡션+다음 문단 오버래핑):
- `aift.hwp` 19페이지 SVG: devel 536,259 bytes ↔ PR #410 적용 536,259 bytes
- **byte 단위 동일** → 회귀 없음 ✅

**회귀 검증 중 발견 (별도 등록)**: `aift.hwp` 41페이지 표 셀 안 이미지 미조판 — **사전 존재 결함** (PR #410 변경 영역과 다른 경로). 이슈 #429 로 등록.

## 변경 요약

### Task #409 v1+v2+v3 (3 단계)

| 단계 | 파일 | 변경 |
|------|------|------|
| v1 | `src/renderer/layout.rs` | `prev_has_overlay_shape` 가드 확장 — Control::Picture (non-TAC) + TextWrap::TopAndBottom + VertRelTo::Para 케이스 포함 |
| v2 | `src/renderer/typeset.rs::typeset_section` | controls 루프에서 비-TAC TopAndBottom Picture/Shape 의 height + margin.bottom 을 current_height 누적 |
| v3 | `src/renderer/typeset.rs::typeset_paragraph` | fit 분기에 atomic TAC top-fit 시멘틱 추가 (단일 라인 + TAC Picture/Shape 60px 이내 초과 시 현재 페이지 배치) |

### LAYOUT_OVERFLOW 정정 정황 (작성자 검증)

| 단계 | 대상 샘플 건수 | 6개 다른 샘플 |
|------|---------------|--------------|
| devel (v0) | 22 | 무회귀 베이스 |
| v1 적용 | 4 | 무회귀 |
| v2 적용 | 1 | 무회귀 |
| **v3 적용** | **1** (잔여 1건은 page=2 449, 본 PR 무관 사전 결함) | 무회귀 |

→ chart 관련 overflow 전건 해소.

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1050 + svg_snapshot 6/6 + clippy 0 + WASM |
| 시각 판정 게이트 (push 전 필수) | ✅ SVG + Canvas 양 경로 통과 |
| 한컴 호환은 일반화보다 케이스별 명시 가드 | ✅ TopAndBottom + vert=Para + 비-TAC 명시, atomic TAC 60px tolerance |
| output 폴더 가이드라인 | ✅ `output/svg/pr410-visual/`, `output/svg/pr410-devel-baseline/`, `output/svg/pr410-426-check/` |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr410` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | ✅ 머지 + push 후 close |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr410` → `local/devel` → `devel` 머지 + push
3. PR #410 close + 작성자 댓글 (이슈 #409 자동 close)
4. 별도 등록한 이슈 #429 (표 셀 내 이미지 미조판) 후속 추적

## 참고

- 검토 문서: `mydocs/pr/pr_410_review.md`
- PR: [#410](https://github.com/edwardkim/rhwp/pull/410)
- 이슈: [#409](https://github.com/edwardkim/rhwp/issues/409)
- 사전 등록 이슈: [#426](https://github.com/edwardkim/rhwp/issues/426) (회귀 영향 없음)
- 회귀 검증 중 발견: [#429](https://github.com/edwardkim/rhwp/issues/429) (사전 결함, 별도 추적)
- 같은 작성자 머지 PR: [#401 v2](https://github.com/edwardkim/rhwp/pull/401), [#406](https://github.com/edwardkim/rhwp/pull/406), [#408](https://github.com/edwardkim/rhwp/pull/408), [#415](https://github.com/edwardkim/rhwp/pull/415)
