# PR #442 검토 — Task #435 exam_kor.hwp 24→22 페이지 정합 (#393 옵션 A)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#442](https://github.com/edwardkim/rhwp/pull/442) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) — 신뢰 컨트리뷰터, v0.7.8 사이클 가장 활발 (8 PR 머지) |
| 이슈 | [#435](https://github.com/edwardkim/rhwp/issues/435) (closes), [#393](https://github.com/edwardkim/rhwp/issues/393) (closes 옵션 A) |
| base / head | `devel` ← `planet6897:local/task435` |
| 변경 규모 | +1,090 / -12, 8 files (5 commits, merge commit 1 제외) |
| mergeable | `MERGEABLE` (BEHIND) |
| 검토 일자 | 2026-04-29 |

## 본질

`samples/exam_kor.hwp` 의 body-wide TopAndBottom 표 (`pi=0` `wrap=TopAndBottom` `vert=Paper(38mm)`) 처리 시 **paper-rel 좌표를 body-rel 변환 없이 reserve 에 누적** 하던 버그 정정.

### 원인

- 표 paper 38mm 시작 (헤더 영역, paper-rel 143.6 px)
- 본문 영역 (margin_top 56mm = 211.7 px) 안으로 79.2 px 침범
- HWP 실제 col 1 시작점: vpos=7085 HU = **94.5 px** (body-rel)
- rhwp 산정 reserve: **306.1 px** (paper-rel 좌표를 body-rel 처럼 누적)
- 차이: **+211.6 px 과대**

이로 인해 col 1 가용 공간 부족 → paragraph split 발생 → page 2 (49px) / page 15 (104px) orphan 페이지 생성.

### 정정

`typeset.rs::compute_body_wide_top_reserve_for_para` 의 `VertRelTo::Paper` 분기:
- **변경 전**: `shape_y_offset = vertical_offset` (paper-rel 직접 사용)
- **변경 후**: body-rel 기준 시작/끝 y 계산 (`Paper`: body_top 차감)

```diff
+let raw_v_offset = ...vertical_offset...;
+let (body_y, body_bottom) = if matches!(common.vert_rel_to, VertRelTo::Paper) {
+    let shape_top_abs = raw_v_offset;
+    let shape_bottom_abs = shape_top_abs + shape_h;
+    if shape_bottom_abs <= body_top { continue; }
+    ((shape_top_abs - body_top).max(0.0), shape_bottom_abs - body_top)
+} else {
+    (raw_v_offset, raw_v_offset + shape_h)
+};
+if body_y > body_h / 3.0 { continue; }
+let bottom = body_bottom + outer_bottom;
```

단일 함수 32 라인 정정. 나머지 1,058 라인은 진단/계획서/단계별 보고서/최종 보고서.

### 결과 메트릭

| 메트릭 | Before | After |
|---|---|---|
| exam_kor.hwp 페이지 수 | 24 | **22** |
| Orphan 페이지 (page 2, 15) | 2개 | **0** |
| pi=0.30 split | PartialParagraph | **FullParagraph** |
| pi=1.25 split | PartialParagraph | **FullParagraph** |
| col 1 reserve | 306.1 px | **94.4 px** (HWP 실측 94.5 와 ±0.2px) |

## 잔여 22→20 정합 작업 — 별도 이슈 분리 (작은 단위 회전 정책 부합)

작성자가 22→20 정합 미달성 정황을 3 가지 별도 메커니즘으로 분리 등록:

- **#439** — Square wrap 표 직후 col 0 over-fill (페이지 14 단 0 1225>1211)
- **#440** — 다단 섹션 [단나누기] 후 새 페이지 단일 컬럼 (페이지 15)
- **#441** — 다단 col 0 cur_h HWP vpos 대비 ~100px over-advance (섹션 2 페이지 18)

→ 메모리 [`feedback_small_batch_release_strategy`](https://github.com/edwardkim/rhwp/blob/main/mydocs/manual/memory/feedback_small_batch_release_strategy.md) 의 작은 단위 회전 정책과 정확히 부합. 작성자가 메인테이너 운영 철학을 정확히 인지하고 PR 분리.

## 처리 방향

**옵션 A — 4 commits 분리 cherry-pick** (PR #406/#408/#410/#424/#434 와 같은 패턴).

PR 의 5 commits 중 merge commit 1 (`a6416bf`) 제외하고 본질 4 commits cherry-pick.

## dry-run cherry-pick 결과

`local/pr442` 브랜치 (`local/devel` 분기) 에서 4 commits cherry-pick — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `e05957c` (← `e0cdc2e`) | @planet6897 | Stage 1: 베이스라인 측정 + 진단 데이터 수집 |
| `2493c94` (← `0879f11`) | @planet6897 | Stage 2: col 1 reserve 정정 (#393 옵션 A, 32 lines) |
| `e791d35` (← `bd08d6b`) | @planet6897 | Stage 3: 일반 페이지 누적 부족 조사 (코드 변경 없음) |
| `1218167` (← `0a096fc`) | @planet6897 | Stage 5: 최종 결과보고서 + 오늘할일 갱신 |

cherry-pick 결과:
- Stage 1, 2, 3 자동 적용
- Stage 5 의 `mydocs/orders/20260429.md` add/add 충돌 (PR #437 와 같은 패턴) → HEAD 유지 후 `--continue`

## 검증 (cherry-pick 후)

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1066 passed** (회귀 0) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 44s, 4,174,855 bytes |

## 광범위 byte 단위 비교

10 샘플 / 309 페이지 SVG 비교 (devel ↔ PR #442):

| 결과 | 카운트 |
|------|------|
| byte 동일 (영향 없음) | **285 / 309** |
| 차이 발생 (의도된 정정) | **24 / 309** |

차이 분포:
- **exam_kor 24 페이지 모두** — 정확히 본 PR 의 정정 대상
- **다른 9개 샘플 회귀 0건** — aift / biz_plan / exam_eng / exam_math / k-water-rfp / kps-ai / 2025년 기부 / synam-001 / equation-lim 모두 byte 단위 동일

→ 작성자 PR 본문 회귀 검증 결과와 정확히 일치. **본 PR 의 영향이 exam_kor 에 정확히 한정** 됨을 byte 단위로 확인.

전체 SVG 수: **309 → 307** (exam_kor 24→22 정합 영향).

## 시각 판정

**한컴 정답지 (한컴 2010 / 2020 / 한컴독스 + hwp 원본) vs rhwp 출력** — 작업지시자 직접 판정.

| 경로 | 시각 판정 결과 |
|------|--------------|
| Canvas (rhwp-studio 웹 에디터, WASM 4,174,855 bytes) | ✅ **통과** |

작업지시자 코멘트:
> "wasm 으로 확인했습니다. 이후 연결된 버그들도 이어서 컨트리뷰터가 처리하겠군요. 집요합니다."

→ Canvas 시각 판정 통과 + 잔여 작업 (#439/#440/#441) 의 컨트리뷰터 후속 처리 인식.

산출물:
- SVG (devel): `output/svg/pr442-regression-baseline/exam_kor_*.svg` (24 SVG)
- SVG (PR #442): `output/svg/pr442-regression-test/exam_kor_*.svg` (22 SVG)
- Canvas: `pkg/rhwp_bg.wasm` (4,174,855 bytes)

## 본 PR 의 좋은 점

1. **정밀 진단**: HWP 실측값 (94.5 px) 과 ±0.2px 까지 일치 — 도메인 이해도 높음
2. **변경 범위 한정**: 단일 함수 32 라인, 영향이 exam_kor 에 정확히 한정 (다른 9 샘플 byte 동일)
3. **잔여 작업 분리**: 22→20 정합은 #439/#440/#441 별도 이슈 — 작은 단위 회전 정책 부합 (메인테이너 운영 철학 정확 인지)
4. **하이퍼-워터폴 절차 정확 준수**: 계획서 + 5 stage + 보고서
5. **Stage 4 생략 의미**: Stage 1-3 후 Stage 5 (최종 보고서) 로 직행 — Stage 4 에 해당하는 회귀 검증을 Stage 3 진단에서 통합 처리한 정황. 작성자 본인의 단계 운영 정합성

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1066 + svg_snapshot 6/6 + clippy 0 + WASM |
| 시각 판정 게이트 (push 전 필수) | ✅ 작업지시자 Canvas 직접 판정 통과 |
| 한컴 호환은 일반화보다 케이스별 명시 가드 | ✅ `VertRelTo::Paper` 분기 명시 + body_top 차감 명시 |
| 작은 단위 PATCH 회전 | ✅ 22→22 우선 정합 + 잔여 22→20 별도 이슈 분리 |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr442` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | ✅ 머지 + push 후 close |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr442` → `local/devel` → `devel` 머지 + push
3. PR #442 close + 작성자 댓글 (이슈 #435, #393 자동 close)

## 참고

- PR: [#442](https://github.com/edwardkim/rhwp/pull/442)
- 이슈: [#435](https://github.com/edwardkim/rhwp/issues/435), [#393](https://github.com/edwardkim/rhwp/issues/393)
- 잔여 작업 (작성자 분리 등록): [#439](https://github.com/edwardkim/rhwp/issues/439), [#440](https://github.com/edwardkim/rhwp/issues/440), [#441](https://github.com/edwardkim/rhwp/issues/441)
- 같은 작성자 머지 PR (v0.7.8 사이클): [#401 v2](https://github.com/edwardkim/rhwp/pull/401), [#406](https://github.com/edwardkim/rhwp/pull/406), [#408](https://github.com/edwardkim/rhwp/pull/408), [#410](https://github.com/edwardkim/rhwp/pull/410), [#415](https://github.com/edwardkim/rhwp/pull/415), [#424](https://github.com/edwardkim/rhwp/pull/424), [#434](https://github.com/edwardkim/rhwp/pull/434)
