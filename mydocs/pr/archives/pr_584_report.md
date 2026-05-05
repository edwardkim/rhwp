# PR #584 처리 보고서 — 핀셋 cherry-pick (3 commits) + 옵션 C 충돌 통합 + 시각 판정 ★ 통과

**PR**: [#584 Task #574: HY견명조 heavy display 오분류 정정 (closes #574)](https://github.com/edwardkim/rhwp/pull/584)
**작성자**: @planet6897 (PR 등록) / Jaeook Ryu = @jangster77 (commit author)
**처리 결정**: ✅ **핀셋 cherry-pick (3 commits) + 옵션 C 충돌 통합 + devel merge + push + PR close + Issue #574 close**
**처리일**: 2026-05-05

## 1. 처리 결과 요약

| 항목 | 결과 |
|------|------|
| 결정 | ✅ 핀셋 cherry-pick (3 commits) + 옵션 C 충돌 통합 + devel merge + push + PR close |
| 시각 판정 | ★ **통과** (Canvas + SVG 양쪽) |
| Devel merge commit | `4c44a27` |
| Cherry-pick 충돌 | 1건 (`integration_tests.rs`) → 옵션 C 통합으로 해결 |
| Author 보존 | ✅ Jaeook Ryu (@jangster77) 보존 |
| Issue #574 | CLOSED (수동 close + 안내 댓글) |
| 광범위 페이지네이션 sweep | 164 fixture / 1,614 페이지 / 페이지 수 회귀 0 |
| SVG `font-weight="bold"` 정량 측정 | 4 fixture 합계 1,664 → 423 (**-1,241** 제거) |

## 2. 본질 결함 (PR 진단)

### 2.1 결함 가설

`is_heavy_display_face` (`src/renderer/style_resolver.rs:601`) 의 hardcoded list 에 `"HY견명조"` 가 잘못 포함되어 CharShape.bold=false 가 무시되고 SVG 에 `font-weight="bold"` 강제 적용됨.

### 2.2 Stage 0 정밀 진단의 통찰

PR 본문이 명시한 이슈 본문 가설 정정:
- 쪽번호 "1" 출처는 바탕쪽이 아닌 **본문 [6] 표 셀 paragraph[0] Shape (사각형, InFrontOfText) TextBox** 내부 literal text "1"
- IR 색상 #000000 (검정), 한컴 PDF 도 검정 — "회색" 가설 잘못. **본질은 굵기만**

→ 표면 증상 → 실제 결함 본질 추적 패턴 (`feedback_v076_regression_origin` 정합).

### 2.3 본질 정정 (단일 줄)

```diff
 matches!(primary,
     "HY헤드라인M" | "HYHeadLine M" | "HYHeadLine Medium"
-    | "HY견고딕" | "HY견명조" | "HY견명조B"
+    | "HY견고딕" | "HY견명조B"
     | "HY그래픽" | "HY그래픽M"
 )
```

**보존**: HY헤드라인M (Task #146 v4 본질 케이스), HY견고딕, HY견명조B (명시 Bold variant), HY그래픽
**제거**: HY견명조 (한컴 일반 두께 명조 — heavy 아님)

## 3. PR 의 7 commits 분석 (cherry-pick 대상 식별)

| commit | 설명 | 본 환경 처리 |
|--------|------|-------------|
| `a7b15d2f` Stage 0 — 수행 계획서 | 컨트리뷰터 fork plans | 무관 |
| **`6d193ec9` Stage 0 — 정밀 진단 + 진단 스크립트** | `examples/inspect_574.rs` (보존 가치) + working stage0 | ⭐ cherry-pick |
| `fde4b4f6` Stage 1 — 구현 계획서 | 컨트리뷰터 fork plans | 무관 |
| **`ef6583ce` Stage 2 — TDD 통합 테스트 + 단위 테스트 갱신 (RED)** | `integration_tests.rs` +74 + `tests.rs` 갱신 + working stage2 | ⭐ cherry-pick |
| **`0002b496` Stage 3 — fix (RED → GREEN)** | `style_resolver.rs` 단일 줄 정정 + working stage3 | ⭐ cherry-pick |
| `e795f164` Stage 4 — 광범위 회귀 sweep | 컨트리뷰터 fork working | 무관 |
| `a3ed7c42` Stage 5 — 최종 보고서 + orders | 컨트리뷰터 fork report + orders | 무관 |

→ **본질 cherry-pick = 3 commits** (PR #561~#580 의 단일 본질 commit 패턴과 다름 — TDD Stage 2 RED + Stage 3 GREEN 분리 + Stage 0 진단 스크립트 보존 가치).

## 4. cherry-pick 진행 + 옵션 C 충돌 통합

### 4.1 대상 commits (3개)

```
c9cc04a Task #574 Stage 0: 정밀 진단 + 본질 확정 보고서 (코드 무수정/진단 스크립트만)
0f0dd5e Task #574 Stage 2: TDD 통합 테스트 + 단위 테스트 갱신 (RED 확인)
24b1211 Task #574 Stage 3: is_heavy_display_face HY견명조 제거 (RED→GREEN)
```

`Jaeook Ryu <jaeook.ryu@gmail.com>` author 보존.

### 4.2 충돌 영역 (1건) + 옵션 C 통합

`src/renderer/layout/integration_tests.rs` 의 mod tests 끝부분에서 충돌:
- HEAD: `test_521`/`test_552` 영역 (PR #561/#562/#564 cherry-pick 시 들어옴)
- ef6583c (Stage 2): `test_574` 신규 추가

**옵션 C 통합 해결**:
- HEAD 의 `test_521_tac_table_outer_margin_bottom_p2` 보존 (assert closing + function closing)
- ef6583c 의 `test_574_page_number_not_force_bold_for_hy_kyun_myeongjo` 추가
- 의미상 독립적 영역 — 같은 모듈 끝부분에 순차 추가

### 4.3 변경 영역 (3 commits 합)

| 파일 | 변경 |
|------|------|
| `src/renderer/style_resolver.rs` | +5 / -1 (HY견명조 제거 + 주석 추가) |
| `src/renderer/layout/integration_tests.rs` | +71 (test_574 신규) |
| `src/renderer/layout/tests.rs` | +6 / -3 (test_is_heavy_display_face 갱신) |
| `examples/inspect_574.rs` | +197 (진단 스크립트, 보존) |
| `mydocs/working/task_m100_574_stage{0,2,3}.md` | +295 (단계별 보고서) |

## 5. 결정적 검증 결과

| 게이트 | 결과 |
|--------|------|
| `cargo build --release` | ✅ Finished |
| `cargo test --lib --release` | ✅ **1132 passed** / 0 failed / 2 ignored (test_574 RED → GREEN, baseline +1) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_546` | ✅ 1 passed (Task #546 회귀 0) |
| `cargo test --test issue_554` | ✅ 12 passed |
| `cargo clippy --release --lib` | ✅ 0건 |
| Docker WASM 빌드 | ✅ **4,581,498 bytes** (1m 28s, PR #580 baseline +9,855 bytes) |

## 6. 광범위 페이지네이션 회귀 sweep

본 환경 `samples/` 폴더 전체 자동 sweep:

| 통계 | 결과 |
|------|------|
| 총 fixture | **164** (158 hwp + 6 hwpx) |
| 총 페이지 (devel baseline) | **1,614** |
| 총 페이지 (cherry-pick 후) | **1,614** |
| **fixture 별 페이지 수 차이** | **0** |

→ font-weight 변경 (HY견명조 사용 텍스트의 bold 제거) 이 페이지네이션에 영향 없음.

## 7. SVG byte 차이 + 정량 측정

### 7.1 byte 차이 (HY견명조 사용 영역 광범위 영향)

| Fixture | 페이지 수 | byte 차이 |
|---|---|---|
| **exam_science** | 4 | **4 / 4** (모든 페이지) |
| **exam_kor** | 20 | **20 / 20** (모든 페이지) |
| **exam_eng** | 8 | **8 / 8** (모든 페이지) |
| **exam_math** | 20 | **20 / 20** (모든 페이지) |

### 7.2 `font-weight="bold"` 출현 횟수 정량 측정

| Fixture | before (devel) | after (cherry-pick) | 제거된 bold |
|---|---|---|---|
| exam_science | 116 | 34 | **-82** |
| exam_kor | 878 | 119 | **-759** |
| exam_eng | 276 | 120 | **-156** |
| exam_math | 394 | 150 | **-244** |
| **합계** | **1,664** | **423** | **-1,241** |

→ HY견명조 사용 텍스트의 `font-weight="bold"` 강제 적용이 4 fixture 합계 **1,241건** 제거. 다른 heavy face (HY헤드라인M / HY견고딕 / HY견명조B / HY그래픽) 의 bold 적용은 보존 (after 의 잔존 423건은 다른 폰트 영역).

### 7.3 페이지 1 쪽번호 "1" 권위 케이스 검증

`exam_science_001.svg` 우상단 (x≈924, y≈115, font-size=44, HY견명조):

| 상태 | SVG element |
|---|---|
| **Before** | `<text ... font-family="HY견명조,..." font-size="44" `**`font-weight="bold"`**` fill="#000000">1</text>` |
| **After** | `<text ... font-family="HY견명조,..." font-size="44" fill="#000000">1</text>` |

→ `font-weight="bold"` 제거 (CharShape.bold=false 권위 회복).

## 8. 시각 판정 (★ 게이트)

### 8.1 SVG 자료 + WASM 환경

- `output/svg/pr584_before/{exam_science,exam_kor,exam_eng,exam_math}/` (devel 기준, 52 페이지)
- `output/svg/pr584_after/{exam_science,exam_kor,exam_eng,exam_math}/` (cherry-pick 후, 52 페이지)
- WASM: `pkg/rhwp_bg.wasm` 4,581,498 bytes (다양한 hwp 직접 검증용)

### 8.2 작업지시자 시각 판정 결과

작업지시자 시각 검증:
- **Canvas (웹 캔바스)**: ★ 통과 (HY견명조 일반 두께 정상)
- **SVG**: ★ 통과 (HY견명조 단독 정정 영역)

추가 발견 — Canvas 와 SVG 의 코드 경로 차이 + 시각 판정 정합:
- **SVG** (`svg.rs`): `is_visually_bold()` 사용 → `is_heavy_display_face` 호출 → 본 PR 의 hardcoded list 에 영향
- **Canvas** (`web_canvas.rs`): `style.bold` 만 사용 → `is_heavy_display_face` 미호출 → hardcoded list 영향 없음 (애초에 강제 bold 없음)

→ 본 PR 의 fix 효과는 **SVG 경로 단독 적용** (Canvas 는 처음부터 정상). 메모리 룰 `feedback_image_renderer_paths_separate` (renderer 별 별도 image 함수) 의 자연스러운 발현.

### 8.3 SVG 시각 검증 중 발견 — 별도 task 영역

작업지시자 SVG 시각 검증 중 HY견고딕 의 "홀수형" 텍스트도 `font-weight="bold"` 강제 적용 발견 — 이는 본 PR 이 의도적으로 보존한 영역 (Task #146 v4 본질). HY견고딕 / HY헤드라인M / HY견명조B / HY그래픽 모두 heavy 로 분류 보존. 한컴 PDF 와의 비교 결과에 따라 별도 task 후보로 검토 가능.

## 9. PR / Issue close 처리

### 9.1 PR #584 close
- 댓글 등록 (cherry-pick 결과 + 결정적 검증 + 광범위 sweep + SVG 정량 측정 + 페이지 1 쪽번호 권위 케이스 + Canvas/SVG 경로 차이 분석 + 별도 task 영역 안내 + 컨트리뷰터 협업 인정)
- close 처리

### 9.2 Issue #574 수동 close
- closes #574 키워드는 PR merge 가 아닌 close 로 자동 처리 안 됨 (PR #564/#570/#575/#580 와 동일 패턴)
- 수동 close + 안내 댓글 (Canvas/SVG 경로 차이 + feedback_image_renderer_paths_separate 정합)

## 10. 메모리 정합

- ✅ `feedback_visual_regression_grows` — 시각 판정 게이트 정합 운영 (Canvas + SVG 양쪽 통과)
- ✅ `feedback_v076_regression_origin` — Stage 0 정밀 진단으로 이슈 본문 가설 정정 (사용자 보고 "회색" → 실제 본질 "굵기만") 정합
- ✅ `feedback_hancom_compat_specific_over_general` — 케이스별 명시 가드 (HY견명조 vs HY견명조B 구분, 단일 face 만 제거)
- ✅ `feedback_rule_not_heuristic` — hardcoded list 의 명시 face 룰 (휴리스틱 아님)
- ✅ `feedback_pdf_not_authoritative` — PR 본문이 한컴 PDF 도 검정임을 명시 (회색 가설 정정의 근거)
- ✅ `feedback_per_task_pr_branch` — Task #574 단일 본질 PR 정합
- ✅ `feedback_pr_comment_tone` — close 댓글 차분/사실 중심 + 컨트리뷰터 협업 인정
- ✅ `feedback_check_open_prs_first` — PR OPEN 상태 확인 후 진행
- ✅ `feedback_small_batch_release_strategy` — 본 사이클 (5/1 ~ 5/5) 활발한 외부 기여의 빠른 회전 (18번째 PR 처리)
- ✅ `feedback_image_renderer_paths_separate` — Canvas 와 SVG 의 코드 경로 차이 식별 (작업지시자 시각 판정 추적 결과)

## 11. 본 PR 의 우수성 — 본 사이클 가장 세분화된 흐름

본 PR 의 처리 본질에서 가장 우수한 점:

1. **7-stage 하이퍼-워터폴 + TDD Stage 2/3 분리** — 본 사이클 가장 세분화된 흐름 (Stage 0 수행 + Stage 0 진단 + Stage 1 구현계획 + Stage 2 TDD RED + Stage 3 fix GREEN + Stage 4 sweep + Stage 5 보고)
2. **Stage 0 정밀 진단으로 이슈 본문 가설 정정** — 사용자 보고 "회색" → 실제 본질 "굵기만" 추적 (`feedback_v076_regression_origin`)
3. **TDD 흐름의 명시적 RED → GREEN 전환** — 이전 PR 들의 단일 본질 commit 보다 더 명확한 검증 흐름
4. **진단 스크립트 보존 가치** — `examples/inspect_574.rs` 향후 재사용 도구
5. **단일 줄 정정 + 회귀 위험 영역 좁힘** — 17 글자 변경 (`HY견명조 |` 제거)
6. **Canvas/SVG 경로 차이 자연 발현** — 메모리 룰 `feedback_image_renderer_paths_separate` 정합 (작업지시자 시각 판정 추적 결과)

## 12. 본 사이클 사후 처리

- [x] PR #584 close (cherry-pick 머지 + push)
- [x] Issue #574 수동 close (안내 댓글)
- [x] 처리 보고서 (`mydocs/pr/archives/pr_584_report.md`, 본 문서)
- [ ] 검토 보고서 archives 이동 (`mydocs/pr/pr_584_review.md` → `mydocs/pr/archives/pr_584_review.md`)
- [ ] 5/5 orders 갱신 (PR #584 항목 추가)
