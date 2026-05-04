# PR #551 Task #544 핀셋 리뷰 — passage 글상자 우측 시프트 정정 (옵션 A2)

**PR**: [#551 (closed, Task #525/#528 cherry-pick 완료)](https://github.com/edwardkim/rhwp/pull/551)
**작성자**: @planet6897 (Jaeuk Ryu)
**처리 결정**: ✅ **옵션 A2** — `05beb208` (Task #544 v2 Stage 2) **단독 cherry-pick** (#547+#544(1) 통합)
**작성일**: 2026-05-04
**관련 이슈**: #544 (재오픈), #547 (closed, 본질 v2 에 통합), #548 (closed, 별개 본질, 제외), #552 (open, **본 devel 에서 발현 안 함, 제외**)

## 결정 근거 요약

본 devel 측정 (페이지 4 [7~9]) 결과:
- 박스 top y = **233.97** ≈ PDF 233.8 (✅ 일치, +0.17 px)
- 박스 left x = **128.51** vs PDF 117.0 (❌ +11.5 px 시프트)
- 박스 width = **402.5** vs PDF 318.9 (❌ +83.6 px 큼)
- 본문 첫 글자 '평' x = **153.12** vs PDF 141.6 (❌ +11.5 px 시프트)

→ **Sub-issue (1) box_x/w 시프트만 발현. (2) box top y 는 본 devel 의 pre-#479 trailing-ls 모델로 이미 정합**.
→ #552 / v3 cherry-pick 은 #479 모델 전환을 강요 → 회귀 위험. **A2 단독 채택**.

## 1. 배경

작업지시자 보고 (2026-05-04):
> 21_언어_기출_편집가능본.hwp - 1 페이지 [1~3], 2 페이지 [4~6], 4 페이지 [7~9], 5 페이지 [10~12], 7 페이지 [13~15], 8 페이지 [16~18], 11 페이지 [22~24], 13 페이지 [25~27], 14 페이지 [28~30] 의 글상자가 오른쪽으로 밀려있음.

- 9개 passage paragraph border 박스 모두 동일 결함
- 이슈 #544 본문의 측정값 (페이지 4 [7~9]): rhwp 박스 left x = 128.5 / PDF = 117.0 → **+11.5 px 우측 시프트**, width = 402.5 / PDF = 318.9 → **+83.6 px 큼**
- 본문 텍스트 첫 글자 '평' 도 동일하게 +11.5 px 우측 시프트

## 2. 본 devel 현재 코드 (`src/renderer/layout/paragraph_layout.rs:2641-2644`)

```rust
let (box_x, box_w) = if let Some((ox, ow)) = self.border_box_override.get() {
    (ox + box_margin_left, ow - box_margin_left)
} else {
    (col_area.x + box_margin_left, col_area.width - box_margin_left - box_margin_right)
};
```

→ paragraph 의 `margin_left` (1704 HU ≈ 22.72 px / 본 케이스 절반인 11.5 px) 가 박스 좌표에 가산 + width 에서 차감되어 시프트 발생.

또한 paragraph 내부 텍스트 위치 산출 (라인 661-)도 `inner_pad_left = box_margin_left` 로 한 번 더 가산하는 분기가 있어, 박스를 col_area 로 옮길 때 이중 inset 부작용 발생 가능 (Task #547 정정 대상).

## 3. 컨트리뷰터 fork 의 정정 chain (`devel-backup` 브랜치)

시간 순:

| # | commit | 타스크 | 본질 |
|---|--------|--------|------|
| 1 | `7b86876c` ~ `5fa10e9f` | **#544 v1** Stage 0~3 | (1) box_x/w col_area 채택 + (2) `paragraph_border_y_correction_px` Cell 도입 + Task #540 Stage 4 가드 revert |
| 2 | `084a9bbe` / `733167fd` | #544 v1 merge | — |
| 3 | `e2b9a711` ~ `418c3be3` / `e0295278` | **#552** | paragraph border 시작 직전 trailing ls 보존 (Task #479 회귀) — **#544 v1 의 (2) y 보정 본질을 더 깔끔하게 흡수** |
| 4 | `ebe7edb7` / `39408de0` / `e4b96e66` | **#547** | inner_pad 분기 제거 — #544 가 box outline 만 col_area 로 옮긴 후 텍스트 inset 이중 적용 부작용 정정 |
| 5 | `c17efa27` | #544 v2 Stage 1 | TDD RED 복원 + #552 양립 사전 검증 |
| 6 | `05beb208` | **#544 v2 Stage 2** | **#547 + #544 (1) box_x/w 통합 재적용**. #544 (2) y 보정은 #552 가 흡수했으므로 skip |
| 7 | `9dc40ddb` | **#548 Phase C** | 셀 inline TAC Shape margin_left + indent (별개 본질, [푸코] 페이지) |
| 8 | `e3824a35` ~ `2c39f244` | #544 v2 Stage 4 + merge | — |
| 9 | `584e0644` ~ `9e341bb6` / `2f6261b1` / `2c39f244` | **#544 v3** Stage 1~4 + merge | 박스 안 sequential paragraph trailing-ls 보존 — `next_para_continues_visible_border` Cell 신설 (Task #479 보강, #552 와 의미 분리) |
| 10 | `0341a2a7` | #517/#518/#544 v3/#552 다중 | **단일 commit 으로 일괄** — staged 영역 추가 미세 조정. 분해 불가 |

## 4. 핵심 의존 관계

- **#544 v1 (`7ba2ecbe`) 단독 cherry-pick → 권장 안 함**
  - (2) y 보정 (`paragraph_border_y_correction_px` Cell) 은 #552 가 더 본질적으로 흡수
  - inner_pad 이중 inset 부작용 미정정 (#547 별도)
  - 이후 v2 가 v1 + #547 을 통합 재적용한 형태로 표준이 됨

- **#544 v2 Stage 2 (`05beb208`) 가 사실상 표준 fix**
  - #547 + #544 (1) box_x/w 통합
  - #544 (2) y 보정 skip (#552 가 처리)
  - 통합 테스트는 #552 의존 (테스트 파일 conflict)

- **#544 v3 Stage 2 (`84d1d4b2`)**
  - 박스 **안의 sequential paragraph 사이 line spacing 좁음** 정정 — 별개 본질
  - paragraph_layout.rs 에 #552 의 `next_para_starts_visible_border` 가 이미 있음을 전제로 보강
  - 단독 cherry-pick 불가 (3 파일 모두 conflict)

- **#548 Phase C (`9dc40ddb`)**
  - 셀 inline TAC Shape margin_left + first-line indent 정정 ([푸코] 페이지)
  - **사용자 보고 결함과 무관한 별개 본질**

## 5. dry-run 결과

| commit | 충돌 파일 | 비고 |
|--------|----------|------|
| `1934161f` Task #552 Stage 2 | `paragraph_layout.rs` (auto-merge 안됨, 라인 2612) | manual resolve 필요 |
| `05beb208` Task #544 v2 Stage 2 | `integration_tests.rs` (Task #552 테스트 부분 / 라인 755) | #552 가 먼저 들어오면 자동 해소 가능 |
| `84d1d4b2` Task #544 v3 Stage 2 | 3 파일 모두 (`layout.rs` + `paragraph_layout.rs` + `integration_tests.rs`) | #552 + v2 가 먼저 들어오면 자동 해소 가능 |

→ **추천 순서**: #552 → #544 v2 Stage 2 → #544 v3 Stage 2 (광범위 회귀 검증 + 보고서 commits 제외)

## 6. 처리 옵션

| 옵션 | 범위 | 사용자 보고 직접 해결 | 회귀 위험 | 광범위 |
|------|------|---------------------|----------|--------|
| **A1** | `7ba2ecbe` (v1) + `84d1d4b2` (v3) — 사용자가 처음 적은 그대로 | ✅ | ⚠️ #552 와 (2) y 보정 영역 충돌 / #547 미반영으로 inner_pad 이중 inset | 중 |
| **A2** | `05beb208` (v2 Stage 2) 만 | ✅ | 낮음 (#552 테스트 conflict 만 manual) | 낮음 |
| **A3 (권장)** | `1934161f` + `05beb208` + `84d1d4b2` (#552 + v2 + v3) | ✅ | 낮음 (의존 chain 완전, conflict 자동 해소 기대) | 중 |
| **A4** | A3 + `9dc40ddb` (#548 Phase C) | ✅ + 별개 본질 동시 | 중 (#548 은 셀 inline TAC Shape, 별개 영역) | 큼 |

## 7. 옵션 변경: A3 → **A2** (B1 진단 후)

**B1 진단 결과** (본 devel build + 측정):

본 devel 의 paragraph_layout.rs 는 Task #479 미반영 → **trailing ls 항상 가산** 모델. pi=80 의 trailing ls 716 HU(=9.54 px) 가 가산되어 pi=81 시작 = body_area.y + 24.21 ≈ 233.97 px = PDF 233.8 px. **(2) box top y 시프트는 본 devel 에서 발현 안 함**.

이슈 #544 본문의 측정값 (rhwp 224.4) 은 컨트리뷰터 fork (Task #479 반영 상태) 기준. 본 devel 과 다른 baseline.

→ **#552 / v3 cherry-pick 은 본 devel 에 #479-style trailing-ls 제외 분기 신설** (현재 없음) → 광범위 회귀 위험.
→ **A2 (`05beb208` 단독)** 가 본 devel 의 trailing-ls 모델과 정합되며 사용자 보고 결함 (box_x/w 시프트) 을 직접 fix.

**제외**:
- **#552** (`1934161f`): #479 회귀 정정 commit. #479 미적용 본 devel 에서 의미 없음 + 모델 전환 위험.
- **v3** (`84d1d4b2`): #479 본질 보존 가드. 같은 이유로 모델 전환 위험.
- **#548 Phase C** (`9dc40ddb`): 별개 본질 (셀 inline TAC Shape — [푸코] 페이지). 별도 사이클로 분리.

## 8. cherry-pick 절차 (옵션 A2)

```bash
git checkout devel
git checkout -b local/task544_pr551
git cherry-pick 05beb208                    # Task #544 v2 Stage 2
# integration_tests.rs conflict 해소 — #552 의존 테스트 부분 (test_552_passage_box_top_gap_p2_4_6) 제외
cargo test --lib                            # baseline 1118 + #544/547 신규 테스트 GREEN 기대
cargo clippy --release -- -D warnings
./target/release/rhwp export-svg samples/21_언어_기출_편집가능본.hwp -o /tmp/diag
# 9 박스 측정 + 작업지시자 시각 판정
```

**검증 기준**:
- `cargo test --lib` 통과 + 회귀 0건 (issue_546/530/505/418/501)
- `svg_snapshot` 6/6
- `clippy` 0 warnings
- 21_언어_기출 1/2/4/5/7/8/11/13/14 페이지 글상자 PDF 한컴 2010 정합 (±2 px)
- exam_kor / exam_eng / exam_science / 2010-01-06 / synam / 복학원서 광범위 회귀 0
- 작업지시자 시각 판정 (1차 SVG + 2차 web Canvas) 통과

## 9. 메모리 룰 정합

- [feedback_pdf_not_authoritative] — 한컴 2010 PDF 보조 ref + 한컴 2010/2020 직접 비교 권고
- [feedback_essential_fix_regression_risk] — paragraph border 좌표 변경은 광범위 영향, 광범위 샘플 검증 필수
- [feedback_rule_not_heuristic] — box outline = col_area 산식이 HWP 표준 정답인지 자문 (Task #544 본문에서 11.5 px = box_margin_left 절반 발견 — 850 HU 단위)
- [feedback_visual_regression_grows] — 작업지시자 직접 시각 판정 게이트 (1차 SVG + 2차 web Canvas)
- [feedback_local_task_branches_origin_backup] — `devel-backup` 보존 유지

## 10. 작업지시자 결정 사항

1. **옵션 선택**: ✅ **A2** (B1 진단 후 변경 승인 — 2026-05-04)
2. cherry-pick 절차 진행 (브랜치 `local/task544_pr551` → conflict resolution → 검증 → merge → push)
3. 처리 보고서 위치: `mydocs/pr/pr_551_v3_544_a2_report.md`
4. 처리 후 이슈 close 절차:
   - #544 close (본 cherry-pick 으로 (1) box_x/w + 본문 텍스트 inset 해결)
   - #547 본 devel 반영 코멘트 (v2 에 통합되었음)
   - #552 / v3 본 devel 미적용 — 본 devel #479 미적용 모델로 발현 안 함, 추후 #479 모델 도입 결정 시 재검토
   - #548 별도 사이클 결정 대기
