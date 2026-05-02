# PR #443 처리 보고서 — Task #439 Square wrap 표 직후 col 0 over-fill 정정 (exam_kor 22→20)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#443](https://github.com/edwardkim/rhwp/pull/443) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) |
| 이슈 | [#439](https://github.com/edwardkim/rhwp/issues/439) (closes) |
| 처리 결정 | **cherry-pick 머지** (Task #439 본질 4 commits 분리) |
| 처리 일자 | 2026-04-29 |

## 처리 절차

### Stage 0: 검토 (옵션 A 결정)

PR 의 11 commits 중 Task #435 (4) 는 PR #442 흡수 완료, merge commit (3) 제외, **Task #439 본질 4 commits** 분리 cherry-pick. PR #442 와 같은 패턴.

### Stage 1: cherry-pick

`local/pr443` 브랜치 (`local/devel` 분기) 에서 4 commits cherry-pick — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `93348d9` (← `3884305`) | @planet6897 | Stage 1: 베이스라인 측정 + 진단 (활성 엔진 확인) |
| `eeca209` (← `4ea2746`) | @planet6897 | Stage 2: 원인 확정 + 구현 계획서 |
| `f37f616` (← `99f1596`) | @planet6897 | Stage 3: Square wrap 표 누적 정책 max 적용 (32 lines) |
| `033781c` (← `df2aff3`) | @planet6897 | Stage 4: 최종 결과보고서 + 오늘할일 갱신 |

cherry-pick 결과:
- Stage 1, 2, 3 자동 적용
- Stage 4 의 `mydocs/orders/20260429.md` add/add 충돌 → HEAD 유지 후 `--continue`

### Stage 2: 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1066 passed** (회귀 0) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 45s, 4,178,523 bytes |

### Stage 3: 광범위 byte 단위 비교

10 샘플 / 307 페이지 SVG 비교: **9 / 307 차이 (exam_kor 한정, 의도된 정정)**.

차이 분포: exam_kor 9 페이지 (페이지 14+15 통합 + 후속 페이지 재조판). 다른 9 샘플 회귀 0건.

전체 SVG 수: 307 → 305 (exam_kor 22→20).

### Stage 4: 시각 판정 (작업지시자 직접)

**비교 자료**: 한컴 정답지 3종 (samples/2010-exam_kor.pdf / 2020-exam_kor.pdf / hancomdocs-exam_kor.pdf) + hwp 원본

| 경로 | 시각 판정 결과 |
|------|--------------|
| SVG 내보내기 | ✅ 통과 |
| Canvas (rhwp-studio 웹 에디터) | ✅ 통과 |

작업지시자 코멘트:
> "시각적 판정은 svg, canvas 모두 통과입니다. 점점 개선이 되어가고 있는게 보입니다. 좋은 공략 전략입니다."

→ 정정의 누적 개선 흐름 평가. 잔여 작업 분리 (#439/#440/#441) + 즉시 처리 전략 검증.

## 변경 요약

### 본질 — `typeset.rs::place_table_with_text` 정정 (32 라인)

Square wrap (어울림) 표 누적 정책: `pre_height + table_total` → `max(pre_height, v_off + table_total)`. HWP layout 에서 어울림 표는 본문이 표 옆을 흐름 (같은 수직 영역 공유).

| 메트릭 | Before | After |
|---|---|---|
| exam_kor.hwp 페이지 수 | 22 | **20** |
| 페이지 14 col 0 used | 1225.8 px (+14.5 over) | **1036.1 px** (under) |
| 페이지 14 col 1 items | 2 | **18** |
| 페이지 14 col 1 used | 64.3 px | **1016.9 px** |
| 페이지 14 + 15 통합 | 분리 | **통합** |

### 부수 발견 — 활성 엔진 정확 식별

이슈 본문 추정 (`engine.rs::Paginator`) 이 활성 엔진 아님 (RHWP_USE_PAGINATOR=1 일 때만). 작성자가 활성 엔진 (`typeset.rs::TypesetEngine`) 정확히 식별 후 정정.

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1066 + svg_snapshot 6/6 + clippy 0 + WASM |
| 시각 판정 게이트 (push 전 필수) | ✅ SVG + Canvas 양 경로 통과 |
| 한컴 호환은 일반화보다 케이스별 명시 가드 | ✅ `is_wrap_around_table` 명시 + Square wrap 만 max 정책 |
| 작은 단위 PATCH 회전 | ✅ 22→20 정합을 #439 단독 처리 (#440/#441 별도) |
| output 폴더 가이드라인 | ✅ `output/svg/pr443-{visual,regression-baseline,regression-test}/` |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr443` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | ✅ 머지 + push 후 close |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr443` → `local/devel` → `devel` 머지 + push
3. PR #443 close + 작성자 댓글 (이슈 #439 자동 close)

## 참고

- 검토 문서: `mydocs/pr/pr_443_review.md`
- PR: [#443](https://github.com/edwardkim/rhwp/pull/443)
- 이슈: [#439](https://github.com/edwardkim/rhwp/issues/439)
- 이전 PR: [#442](https://github.com/edwardkim/rhwp/pull/442) (Task #435, exam_kor 24→22)
- 잔여 작업: [#440](https://github.com/edwardkim/rhwp/issues/440), [#441](https://github.com/edwardkim/rhwp/issues/441) (다음 PR 후보)
