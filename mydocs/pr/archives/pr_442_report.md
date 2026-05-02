# PR #442 처리 보고서 — Task #435 exam_kor.hwp 24→22 페이지 정합

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#442](https://github.com/edwardkim/rhwp/pull/442) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) |
| 이슈 | [#435](https://github.com/edwardkim/rhwp/issues/435) (closes), [#393](https://github.com/edwardkim/rhwp/issues/393) (closes 옵션 A) |
| 처리 결정 | **cherry-pick 머지** (4 commits 분리, merge commit 1 제외) |
| 처리 일자 | 2026-04-29 |

## 처리 절차

### Stage 0: 검토 (옵션 A 결정)

PR 의 5 commits 중 merge commit 1 제외 본질 4 commits cherry-pick. PR #406/#408/#410/#424/#434 와 같은 분리 cherry-pick 패턴.

### Stage 1: cherry-pick

`local/pr442` 브랜치 (`local/devel` 분기) 에서 4 commits cherry-pick — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `e05957c` (← `e0cdc2e`) | @planet6897 | Stage 1: 베이스라인 측정 + 진단 데이터 수집 |
| `2493c94` (← `0879f11`) | @planet6897 | Stage 2: col 1 reserve 정정 (#393 옵션 A, 32 lines) |
| `e791d35` (← `bd08d6b`) | @planet6897 | Stage 3: 일반 페이지 누적 부족 조사 (코드 변경 없음) |
| `1218167` (← `0a096fc`) | @planet6897 | Stage 5: 최종 결과보고서 + 오늘할일 갱신 |

cherry-pick 결과:
- Stage 1, 2, 3 자동 적용
- Stage 5 의 `mydocs/orders/20260429.md` add/add 충돌 → HEAD 유지 후 `--continue`

### Stage 2: 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1066 passed** (회귀 0) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 44s, 4,174,855 bytes |

### Stage 3: 광범위 byte 단위 비교

10 샘플 / 309 페이지 SVG 비교: **24 / 309 차이 (exam_kor 한정, 의도된 정정)**.

차이 분포: exam_kor 24 페이지 모두 (정정 대상). 다른 9 샘플 회귀 0건.

전체 SVG 수: 309 → 307 (exam_kor 24→22 정합).

### Stage 4: 시각 판정 (작업지시자 직접)

**비교 자료**: 한컴 정답지 3종 (samples/2010-exam_kor.pdf / 2020-exam_kor.pdf / hancomdocs-exam_kor.pdf) + hwp 원본

| 경로 | 시각 판정 결과 |
|------|--------------|
| Canvas (rhwp-studio 웹 에디터) | ✅ 통과 |

작업지시자 코멘트:
> "wasm 으로 확인했습니다. 이후 연결된 버그들도 이어서 컨트리뷰터가 처리하겠군요. 집요합니다."

## 변경 요약

### 본질 — `typeset.rs::compute_body_wide_top_reserve_for_para` 정정 (32 라인)

`VertRelTo::Paper` 분기에서 paper-rel `vertical_offset` 을 body-rel 변환 없이 reserve 에 누적하던 버그 정정. body-rel 기준 시작/끝 y 계산 (Paper: body_top 차감).

| 메트릭 | Before | After |
|---|---|---|
| exam_kor.hwp 페이지 수 | 24 | **22** |
| Orphan 페이지 (page 2, 15) | 2개 | **0** |
| pi=0.30 / pi=1.25 split | PartialParagraph | **FullParagraph** |
| col 1 reserve | 306.1 px | **94.4 px** (HWP 실측 ±0.2) |

### 잔여 작업 분리 (작은 단위 회전 정책 부합)

22→20 정합 미달성 정황을 3 별도 이슈로 분리 등록 (작성자):
- **#439** Square wrap 표 직후 col 0 over-fill (페이지 14)
- **#440** 다단 [단나누기] 후 새 페이지 단일 컬럼 (페이지 15)
- **#441** 다단 col 0 cur_h HWP vpos 대비 over-advance (섹션 2 페이지 18)

→ 메인테이너 운영 철학 ([feedback_small_batch_release_strategy](https://github.com/edwardkim/rhwp/blob/main/mydocs/manual/memory/feedback_small_batch_release_strategy.md)) 정확 인지.

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1066 + svg_snapshot 6/6 + clippy 0 + WASM |
| 시각 판정 게이트 (push 전 필수) | ✅ 작업지시자 Canvas 직접 판정 통과 |
| 한컴 호환은 일반화보다 케이스별 명시 가드 | ✅ `VertRelTo::Paper` 명시 분기 + body_top 차감 |
| 작은 단위 PATCH 회전 | ✅ 22→22 우선 정합 + 잔여 22→20 별도 이슈 분리 |
| output 폴더 가이드라인 | ✅ `output/svg/pr442-{regression-baseline,regression-test}/` |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr442` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | ✅ 머지 + push 후 close |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr442` → `local/devel` → `devel` 머지 + push
3. PR #442 close + 작성자 댓글 (이슈 #435, #393 자동 close)

## 참고

- 검토 문서: `mydocs/pr/pr_442_review.md`
- PR: [#442](https://github.com/edwardkim/rhwp/pull/442)
- 이슈: [#435](https://github.com/edwardkim/rhwp/issues/435), [#393](https://github.com/edwardkim/rhwp/issues/393)
- 잔여 작업 (별도 이슈): [#439](https://github.com/edwardkim/rhwp/issues/439), [#440](https://github.com/edwardkim/rhwp/issues/440), [#441](https://github.com/edwardkim/rhwp/issues/441)
- 같은 작성자 머지 PR (v0.7.8 + v0.7.9-dev 사이클): [#401 v2](https://github.com/edwardkim/rhwp/pull/401), [#406](https://github.com/edwardkim/rhwp/pull/406), [#408](https://github.com/edwardkim/rhwp/pull/408), [#410](https://github.com/edwardkim/rhwp/pull/410), [#415](https://github.com/edwardkim/rhwp/pull/415), [#424](https://github.com/edwardkim/rhwp/pull/424), [#434](https://github.com/edwardkim/rhwp/pull/434)
