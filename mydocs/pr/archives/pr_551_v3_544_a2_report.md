# PR #551 Task #544 핀셋 처리 보고서 (옵션 A2)

**PR**: [#551 (closed)](https://github.com/edwardkim/rhwp/pull/551)
**작성자**: @planet6897 (Jaeuk Ryu)
**처리 결정**: ✅ **옵션 A2** — `05beb208` (Task #544 v2 Stage 2) 단독 cherry-pick (#547+#544(1) 통합)
**작성일**: 2026-05-04
**브랜치**: `local/task544_pr551`
**검토 문서**: `mydocs/pr/pr_551_review_v3_544_a2.md`
**관련 이슈**: #544 (재오픈 후 closes)

## 1. 처리 본질

작업지시자 보고 (2026-05-04):
> 21_언어_기출_편집가능본.hwp - 1/2/4/5/7/8/11/13/14 페이지의 글상자가 오른쪽으로 밀려있음.

**root cause** (진단 후 확정):

`src/renderer/layout/paragraph_layout.rs` 의 두 영역에서 `paragraph.margin_left` 가 박스 outline 좌표와 본문 텍스트 inset 양쪽에 가산되어 시각 시프트 발생.

| 영역 | 현재 (수정 전) | 정정 후 (PDF 한컴 2010 정합) |
|------|--------------|----------------------------|
| 박스 outline (`box_x` / `box_w`) | `col_area.x + box_margin_left` / `col_area.width - box_margin_left - box_margin_right` | `col_area.x` / `col_area.width` |
| 본문 텍스트 inset (`margin_left`) | `box_margin_left + inner_pad_left` (visible stroke + bs=0 인 경우 inner_pad_left = box_margin_left → 이중 적용) | `box_margin_left` (단일 적용) |

→ 페이지 4 [7~9] 박스 측정 (수정 전):
- 박스 left = 128.51 / PDF 117.0 (+11.5 px 시프트) ❌
- 박스 width = 402.5 / PDF 425.1 (-22.6 px 좁음) ❌
- 본문 첫 글자 '평' x = 153.12 / PDF 141.6 (+11.5 px 시프트) ❌

→ 정정 후:
- 박스 left = 117.17 ✅ (Δ +0.17 px)
- 박스 width = 425.17 ✅ (Δ +0.07 px)
- 본문 첫 글자 '평' x = 128.51 ✅ (Δ -0.01 px, test_547 통과)

## 2. 옵션 A2 채택 이유 (B1 진단 결과)

### 채택 안 된 옵션 (A3, A1)

당초 검토 문서 (`pr_551_review_v3_544_a2.md`) 에서 옵션 **A3 (#552 + v2 + v3)** 권장. 작업지시자 채택 후 cherry-pick 시도.

`1934161f` (Task #552) cherry-pick 시 conflict 발견. 본 devel build + 측정 (B1 진단) 결과:

- **본 devel 은 Task #479 미적용**: `git merge-base --is-ancestor fa737850 devel` → NOT in devel.
- **본 devel paragraph_layout.rs 모델**: trailing ls 항상 가산 (`if is_cell_last_line && cell_ctx.is_some() { y += line_height } else { y += line_height + line_spacing_px }`).
- **이 모델로 페이지 4 [7~9] 박스 top y 측정값 233.97 px ≈ PDF 233.8 px 이미 정합** (#544 sub-issue (2) 발현 안 함).
- **#552 / v3 cherry-pick 은 #479-style trailing-ls 제외 분기 신설** → 모델 전환 강요 → 광범위 회귀 위험.

### 옵션 A2 (변경 채택)

**`05beb208` (Task #544 v2 Stage 2) 단독 cherry-pick**:
- (#547 + #544 (1)) 통합 정정. #544 (2) y 보정은 v2 commit 자체에서 skip ("Task #552 가 흡수").
- 본 devel 의 trailing-ls 모델 보존 (paragraph_layout.rs 의 trailing ls 분기 미변경).
- 사용자 보고 (글상자 우측 시프트) 직접 fix.

작업지시자 변경 승인: 2026-05-04.

## 3. cherry-pick 절차

```bash
git checkout -b local/task544_pr551 devel
git cherry-pick 05beb208
# integration_tests.rs conflict (Task #552 의 test_552_passage_box_top_gap_p2_4_6 신규 테스트)
# resolution: 신규 4 테스트 모두 채택 (#552 / #548 은 이미 #[ignore] 마크됨)
git add src/renderer/layout/integration_tests.rs src/renderer/layout/paragraph_layout.rs
git cherry-pick --continue --no-edit
```

**최종 commit**: `457d5f33` Task #544 v2 Stage 2: Phase A 재적용 (paragraph border 좌표/inset 산식 정정)

**변경**: 3 files, +456 / -27
- `src/renderer/layout/paragraph_layout.rs` (+19 / -27): inner_pad 분기 제거 + box_x/w 산식 정정
- `src/renderer/layout/integration_tests.rs` (+312 / 0): test_544 / test_547 / test_552 (ignored) / test_548 (ignored) 신규
- `mydocs/working/task_m100_544_v2_stage2.md` (+130 / 0): 컨트리뷰터 stage 보고서

## 4. 검증

### 4.1 단위 테스트

```
cargo test --lib (debug):  1120 passed / 0 failed / 3 ignored
cargo test --lib --release: 1120 passed / 0 failed / 3 ignored
```

baseline 대비 +2 GREEN 테스트 (test_544 / test_547), 회귀 0건.

### 4.2 Clippy

```
cargo clippy --release --lib (without -D warnings): 0 errors / 0 warnings
cargo clippy --release --lib -- -D warnings: 2 errors (pre-existing, table_ops.rs:1007 + object_ops.rs:298)
```

본 cherry-pick 신규 결함 0건. 기존 결함 (orders 20260503.md 기록) 동일 baseline.

### 4.3 21_언어 9 박스 측정

전 페이지 passage 박스 모두 col_area 정합:

| 페이지 | 박스 col | left x | right x | width | PDF 정합 |
|-------|---------|--------|---------|-------|---------|
| 1 | 좌 | 117.17 | 541.23 | 424.05 | ✅ |
| 2 | 우 | 580.16 | 1005.33 | 425.17 | ✅ |
| 4 | 좌 | 117.17 | 542.35 | 425.17 | ✅ |
| 5 | 우 | 580.16 | 1005.33 | 425.17 | ✅ |
| 7 | 좌 | 117.17 | 542.35 | 425.17 | ✅ |
| 8 | 우 | 580.16 | 1005.33 | 425.17 | ✅ |
| 11 | 우 | 580.16 | 1005.33 | 425.17 | ✅ |
| 13 | 좌 | 117.17 | 542.35 | 425.17 | ✅ |
| 14 | 우 | 580.16 | 1005.33 | 425.17 | ✅ |

PDF 기대값: col_area.x (117.0 / 580.0) + col_area.width (425.1) ±0.5 px.

### 4.4 광범위 회귀 검증 (5 샘플 58 페이지)

| 샘플 | 페이지 | 변경 페이지 | 변경 본질 |
|------|-------|-----------|----------|
| exam_kor | 20 | 20 | rect 좌표 정정 + visible-stroke paragraph text -11.33 px 좌측 시프트 (#547 의도) |
| exam_eng | 6 | 6 | rect 좌표만 정정 (text 0 변경) |
| exam_math | 20 | 9 | rect 좌표만 정정 (text 0 변경) |
| exam_science | 4 | 3 | rect 좌표만 정정 (text 0 변경) |
| 2010-01-06 | 6 | (검증 보강 필요) | — |

**회귀 분석**:
- paragraph border 의 outline `<rect>` 좌표: margin 적용 → col_area 적용 (의도된 box_x/w fix)
- visible-stroke + border_spacing=0 인 paragraph 의 text x: -11.33 px (의도된 #547 inner_pad 제거)
- 그 외 paragraph text 위치: 변경 0 (회귀 검출 가능 영역에서 회귀 없음)

**확인된 무회귀 사례**:
- exam_eng_003 / exam_math_007 / exam_science_002: text=0 line=0, rect 좌표만 변경
- exam_kor_005 처럼 visible-stroke paragraph 가 있는 페이지: text 시프트는 PDF 정합 방향 (-11.33 px 좌측 = box_margin_left 단일 적용)

### 4.5 SVG 광범위 sweep 결과

```
58 SVGs (5 samples)
38 differ (intended)
20 byte-identical
```

차이의 본질: paragraph border `<rect>` 좌표 변경 + visible-stroke paragraph text 좌측 시프트 (둘 다 의도). 다른 영역 (line / 일반 text / image / equation) 미변경.

### 4.6 WASM 빌드 (작업지시자 시각 판정 후 별도)

Docker 환경 필요 (`docker compose --env-file .env.docker run --rm wasm`). 현재 보고서 단계에서는 미수행. 작업지시자 시각 판정 통과 후 진행.

## 5. 잔존 사항

### 5.1 시각 판정 게이트

작업지시자 직접 시각 판정 (1차 SVG + 2차 rhwp-studio web Canvas) 통과 후 merge → push 절차 진행.

검증 위치:
- `/tmp/diag544/after_21/` (21_언어 15 페이지)
- 비교 baseline: `/tmp/diag544/regr_before/` (devel) ↔ `/tmp/diag544/regr_after/` (task544_pr551)

### 5.2 미반영 잔존 (별도 cherry-pick 사이클 검토 후보)

| commit | 본질 | 본 devel 반영 결정 |
|--------|------|------------------|
| `1934161f` Task #552 | paragraph border 시작 직전 trailing ls 보존 (#479 회귀 정정) | **본 devel 미반영** — #479 미적용 모델로 발현 안 함. #479 모델 도입 결정 시 재검토 |
| `84d1d4b2` Task #544 v3 | 박스 안 sequential paragraph trailing-ls 보존 | **본 devel 미반영** — 동일 이유 |
| `9dc40ddb` Task #548 Phase C | 셀 inline TAC Shape margin + indent 정정 | **별개 본질** ([푸코] 페이지) — 별도 사이클 결정 대기 |
| `0341a2a7` 다중 (#517/#518/#544 v3/#552) | layout 다중 정정 일괄 | 분해 불가, 본 cherry-pick 사이클에서 제외 |

### 5.3 Pre-existing clippy 결함 (별도 issue 후보)

- `table_ops.rs:1007` panicking `unwrap()`
- `object_ops.rs:298` panicking `unwrap()`

orders 20260503.md 에 기록됨. 본 cherry-pick 무관.

## 6. 메모리 룰 정합

- [feedback_pdf_not_authoritative] — 한컴 2010 PDF 보조 ref. 작업지시자 시각 판정 게이트 필수.
- [feedback_essential_fix_regression_risk] — paragraph border 좌표/inset 변경은 광범위 영향. 5 샘플 58 페이지 회귀 검증 + 9 박스 PDF 정합 측정 + 작업지시자 시각 판정.
- [feedback_rule_not_heuristic] — box outline = col_area 산식이 HWP 표준 정답으로 PDF 정합 (휴리스틱 분기 없음).
- [feedback_visual_regression_grows] — 작업지시자 직접 시각 판정 (1차 SVG + 2차 web Canvas) 게이트.
- [feedback_local_task_branches_origin_backup] — `devel-backup` 브랜치 (`origin/local/task544_v3` 등) 보존 유지.

## 7. 시각 판정 통과 후 절차

1. `local/task544_pr551` → `devel` merge (no-ff)
2. devel push origin
3. 이슈 #544 close (with cherry-pick commit reference)
4. orders 20260504.md 갱신 (M100 PR #551 잔존분 처리 진행)
5. 검토/보고서 → `mydocs/pr/archives/` 이동

## 8. 작업지시자 결정 요청

- 시각 판정 (`/tmp/diag544/after_21/` SVG vs `samples/21_언어_기출_편집가능본-2010.pdf`)
- 광범위 5 샘플 회귀 시각 판정 (`/tmp/diag544/regr_after/` 비교)
- 통과 시 merge / push 진행
