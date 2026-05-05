# PR #584 검토 보고서

**PR**: [#584 Task #574: HY견명조 heavy display 오분류 정정 (closes #574)](https://github.com/edwardkim/rhwp/pull/584)
**작성자**: @planet6897 (PR 등록) / Jaeook Ryu = @jangster77 (commit author)
**상태**: OPEN, mergeable=CONFLICTING (PR base 시점 차이 + integration_tests.rs 위치 충돌)
**처리 결정**: ⏳ **검토 중** (1차 검토)
**검토 시작일**: 2026-05-05

## 1. 검토 핵심 질문

1. **본질 결함 식별 정합성** — `is_heavy_display_face` hardcoded list 에 "HY견명조" 가 잘못 포함되어 CharShape.bold=false 가 무시되는 결함이 본 환경에서도 재현되는가?
2. **TDD Stage 2/3 전환 정합** — Stage 2 가 실제 RED (테스트 실패) 를 만들고 Stage 3 fix 가 RED → GREEN 으로 정확히 전환되는가?
3. **충돌 해결 정합성** — Stage 2 의 `integration_tests.rs` 추가 영역이 본 환경의 `test_521`/`test_552` 영역과 충돌. 옵션 C 통합 (양 영역 모두 보존) 으로 해결 가능한가?

## 2. PR 정보

| 항목 | 값 | 평가 |
|------|-----|------|
| 제목 | Task #574 HY견명조 heavy display 오분류 정정 | 정합 |
| author (PR 등록) | @planet6897 | — |
| commit author | Jaeook Ryu (= @jangster77) | 컨트리뷰터간 협업 흐름 (PR #561~#580 동일 패턴) |
| changedFiles | 11 / +1,239 / -4 | 본질 코드 +88/-3 (lib +5 + tests +83) + 보고서 + 진단 스크립트 |
| 본질 변경 | `style_resolver.rs` 단일 줄 정정 + tests 갱신 + 신규 통합 테스트 | 단일 라인 본질 |
| mergeable | CONFLICTING | PR base 시점 차이 + 충돌 영역 검증됨 |
| Issue | closes #574 | ✅ |

## 3. PR 의 7 commits 분석

| commit | 설명 | 본 환경 처리 |
|--------|------|-------------|
| `a7b15d2f` Stage 0 — 수행 계획서 | 컨트리뷰터 fork plans | 무관 |
| **`6d193ec9` Stage 0 — 정밀 진단 + 진단 스크립트** | `examples/inspect_574.rs` (보존 가치) + working stage0 | ⭐ **cherry-pick 대상** (진단 도구 보존) |
| `fde4b4f6` Stage 1 — 구현 계획서 | 컨트리뷰터 fork plans | 무관 |
| **`ef6583ce` Stage 2 — TDD 통합 테스트 + 단위 테스트 갱신 (RED)** | `integration_tests.rs` +74 + `tests.rs` 갱신 + working stage2 | ⭐ **cherry-pick 대상** (TDD 흐름) |
| **`0002b496` Stage 3 — fix (RED → GREEN)** | `style_resolver.rs` 단일 줄 정정 + working stage3 | ⭐ **cherry-pick 대상** |
| `e795f164` Stage 4 — 광범위 회귀 sweep | 컨트리뷰터 fork working | 무관 |
| `a3ed7c42` Stage 5 — 최종 보고서 + orders | 컨트리뷰터 fork report + orders | 무관 (orders 충돌 위험) |

→ **본질 cherry-pick 대상 = 3 commits** (`6d193ec9` + `ef6583ce` + `0002b496`). PR #561~#580 의 단일 본질 commit 패턴과 다름 — TDD Stage 2 (RED) + Stage 3 (GREEN) 분리 + Stage 0 진단 스크립트 보존 가치.

## 4. 본질 변경 영역

### 4.1 결함 가설

`is_heavy_display_face` (`src/renderer/style_resolver.rs:601`) 의 hardcoded list 에 `"HY견명조"` 가 잘못 포함되어 CharShape.bold=false 가 무시되고 SVG 에 `font-weight="bold"` 강제 적용됨.

### 4.2 이슈 본문 가설 정정 (Stage 0 정밀 진단)

PR 본문이 명시:
> 이슈 본문 가설 일부 정정:
> - 쪽번호 "1" 출처는 바탕쪽이 아닌 **본문 [6] 표 셀 paragraph[0] Shape (사각형, InFrontOfText) TextBox** 내부 literal text "1"
> - IR 색상 #000000 (검정), 한컴 PDF 도 검정 — "회색" 가설 잘못. **본질은 굵기만**

→ 사용자 보고 (이슈 본문) 의 가설을 정밀 진단으로 정정한 우수 사례 (`feedback_v076_regression_origin` 정합).

### 4.3 정정 (단일 줄)

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

### 4.4 진단 스크립트 보존 (`examples/inspect_574.rs`)

PR 본문 명시: "진단 도구 (보존)". 향후 유사 결함 진단에 재사용 가능 — 본 환경 cherry-pick 가치.

## 5. 본 환경 직접 검증 (임시 브랜치 cherry-pick test)

`pr584-cherry-test` 임시 브랜치에서 3 commits cherry-pick (Stage 0 → 2 → 3):

### 5.1 Stage 0 + Stage 2 (RED) cherry-pick

| 단계 | 결과 |
|------|------|
| `6d193ec9` Stage 0 cherry-pick | ✅ 충돌 0 (`examples/inspect_574.rs` + working stage0 신규) |
| `ef6583ce` Stage 2 cherry-pick | ⚠️ **충돌 발생** — `src/renderer/layout/integration_tests.rs` (HEAD: test_521/test_552 영역 vs ef6583c: test_574 신규 추가) |
| 충돌 해결 | ✅ **옵션 C 통합** — HEAD 의 test_521/test_552 보존 + ef6583c 의 test_574 추가 (의미상 독립적, 같은 모듈 끝부분에 순차 추가) |
| `cargo build --release` | ✅ Finished |
| `cargo test --lib --release test_574` | ✅ **RED** (Stage 2 의도된 RED 상태) |
| `cargo test --lib --release` | ✅ 1130 passed / **2 failed** (test_574 + test_is_heavy_display_face_matches_known_heavy_faces — Stage 2 가 두 테스트 모두 RED 로 갱신) |

→ **TDD Stage 2 RED 정합** (Stage 3 fix 미적용 상태에서 두 테스트 모두 의도된 RED).

### 5.2 Stage 3 fix 적용 (RED → GREEN)

| 단계 | 결과 |
|------|------|
| Stage 3 fix 수동 적용 (`style_resolver.rs` 단일 줄 정정 + 주석 추가) | ✅ |
| `cargo test --lib --release` | ✅ **1132 passed** / 0 failed / 2 ignored (test_574 RED → GREEN, baseline +1) |
| `cargo clippy --release --lib` | ✅ 0건 |

→ **TDD RED → GREEN 전환 정합 확인**. 본 환경 cherry-pick 가능 + Stage 3 fix 가 의도된 결과 도출.

## 6. PR 본문의 자기 검증 결과

| 검증 | PR 본문 결과 | 본 환경 재검증 (임시) |
|------|---------|----------|
| `cargo test --release --lib` | 1120 passed | ✅ 1132 passed (본 환경 baseline +1, Task #574 통합 테스트 정합) |
| 신규 단위 테스트 갱신 | `test_is_heavy_display_face_matches_known_heavy_faces` | ✅ 검증 완료 |
| 신규 통합 테스트 | `test_574_page_number_not_force_bold_for_hy_kyun_myeongjo` | ✅ 검증 완료 |
| `clippy --release` | 신규 경고 0 | ✅ 0건 |
| 7개 샘플 SVG sweep | exam_science 4/4 + exam_kor/eng/math 전체 + 복학원서 1/1 + synam-001 0/35 + text-align 0/1 | ⏳ 본 환경 sweep 권장 |
| 작업지시자 시각 판정 | (미진행) | ⏳ 본 환경 시각 판정 게이트 |

## 7. 메인테이너 정합성 평가

### 정합 영역
- ✅ **본질 cherry-pick 가능 (옵션 C 통합)** — `integration_tests.rs` 충돌은 의미상 독립적 영역 (다른 Task 의 회귀 테스트들이 같은 모듈 끝부분에 순차 추가) 으로 옵션 C 통합 정합
- ✅ **결정적 검증 정합** — cargo test --lib 1132 passed (test_574 RED → GREEN, baseline +1) / clippy 0
- ✅ **TDD Stage 2/3 분리** — RED → GREEN 전환의 명시적 검증 가능 (이전 PR 들과 다른 우수 패턴)
- ✅ **Stage 0 정밀 진단으로 이슈 본문 가설 정정** — 사용자 보고 "회색" 가설 → 실제 본질 "굵기만" 추적 (`feedback_v076_regression_origin` 정합)
- ✅ **단일 줄 정정** — `is_heavy_display_face` hardcoded list 에서 `"HY견명조"` 만 제거 (회귀 위험 영역 좁힘)
- ✅ **HY견명조B (명시 Bold variant) 보존** — Task #146 v4 본질 케이스 + 다른 heavy face 보존 정합
- ✅ **진단 스크립트 (`examples/inspect_574.rs`) 보존 가치** — 향후 유사 결함 진단 재사용 도구
- ✅ **하이퍼-워터폴 + TDD 통합** — Stage 0 수행 → Stage 0 진단 → Stage 1 구현계획 → Stage 2 TDD RED → Stage 3 fix GREEN → Stage 4 sweep → Stage 5 보고. 본 사이클 가장 세분화된 흐름

### 우려 영역
- ⚠️ **CONFLICTING 표시** — `integration_tests.rs` 의 `test_521`/`test_552` 영역과 충돌. 옵션 C 통합으로 해결 가능 확인됨
- ⚠️ **PR 본문 "변경 본질" 명시** — "모든 변경 라인의 100% 가 HY견명조 사용 텍스트, font-weight='bold' 제거만. 좌표/크기/색상/font-family 변경 0건. HY견명조外 폰트 회귀 0건." — 본 환경 광범위 sweep 으로 재현 권장
- ⚠️ **작업지시자 시각 판정 게이트** — PR 본문 미진행 명시. 본 환경 cherry-pick 후 직접 시각 판정 필수
- ⚠️ **font-weight 변경의 시각 영향** — HY견명조 사용 텍스트가 모두 영향받음 (PR 본문: exam_science 4/4 + exam_kor/eng/math 전체 페이지 + 복학원서 1/1) — 광범위 영역 시각 검증 필수

## 8. 1차 검토 결론

### 정합 평가
- ✅ **본질 cherry-pick 가능 (옵션 C 통합)** — 3 commits + 충돌 1 해결
- ✅ **결정적 검증** — 1132 passed (test_574 RED → GREEN) / clippy 0
- ✅ **TDD Stage 2/3 분리** — 명시적 RED → GREEN 전환 검증 가능
- ✅ **이슈 본문 가설 정정** — 정밀 진단의 우수 패턴
- ✅ **단일 줄 정정** — 회귀 위험 영역 좁힘
- ⏳ **시각 판정 별도 진행 필요** — PR 본문 미진행 + 광범위 영향 영역
- ⏳ **광범위 sweep 본격 검증 필요** — 본 환경 자동 sweep + HY견명조 사용 영역 시각 판정

### 권장 처리 방향 (작업지시자 결정 대기)

#### 옵션 A — 핀셋 cherry-pick + 옵션 C 충돌 통합 (권장)
- 3 commits cherry-pick (`6d193ec9` + `ef6583ce` + `0002b496`) — Stage 1/4/5 의 plans/working/report/orders 는 컨트리뷰터 fork 정합
- `integration_tests.rs` 충돌 옵션 C 통합 (HEAD 영역 보존 + ef6583c 의 test_574 추가)
- 진단 스크립트 (`examples/inspect_574.rs`) 보존 (향후 재사용 가치)
- 본 환경 결정적 재검증 + 광범위 sweep + WASM
- 작업지시자 시각 판정 (★ 게이트) — exam_science 4 페이지 + HY견명조 사용 영역 (exam_kor/eng/math + 복학원서)
- 통과 시 devel merge + push + PR close 처리

#### 옵션 B — 추가 정정 요청
- 시각 판정에서 잔존 결함 발견 시 컨트리뷰터에게 추가 정정 요청

#### 옵션 C — close + 본 환경 직접 처리
- 시각 판정 다수 결함 발견 시 본 환경에서 직접 처리

→ **작업지시자 결정 대기**. 옵션 A 권장 — 본질 cherry-pick 가능 + 결정적 검증 통과 + TDD 분리 + 정밀 진단 + 단일 줄 정정 + 진단 스크립트 보존 가치.

## 9. 옵션 A 진행 결과 (작업지시자 승인 후)

### 9.1 핀셋 cherry-pick + 옵션 C 충돌 통합

| 단계 | 결과 |
|------|------|
| 본질 commits cherry-pick (`6d193ec9` + `ef6583ce` + `0002b496`) | ✅ author Jaeook Ryu 보존 |
| `integration_tests.rs` 충돌 | ⚠️ 1건 → ✅ **옵션 C 통합** (HEAD 의 test_521/test_552 보존 + ef6583c 의 test_574 추가) |
| local/devel cherry-pick commits | `0f0dd5e` (Stage 2 RED, 충돌 해결 적용) + `24b1211` (Stage 3 fix) |

### 9.2 결정적 검증 (모두 통과)

| 검증 | 결과 |
|------|------|
| `cargo test --lib --release` | ✅ **1132 passed** / 0 failed / 2 ignored (test_574 RED → GREEN, baseline +1) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_546` | ✅ 1 passed (Task #546 회귀 0) |
| `cargo test --test issue_554` | ✅ 12 passed |
| `cargo clippy --release --lib` | ✅ 0건 |
| `cargo build --release` | ✅ Finished |
| Docker WASM 빌드 | ✅ **4,581,498 bytes** (1m 28s, PR #580 baseline +9,855 bytes) |

### 9.3 광범위 페이지네이션 sweep (페이지 수 회귀 자동 검출)

본 환경 `samples/` 폴더 전체 자동 sweep:

| 통계 | 결과 |
|---|---|
| 총 fixture | **164** (158 hwp + 6 hwpx) |
| 총 페이지 (devel baseline) | **1,614** |
| 총 페이지 (cherry-pick 후) | **1,614** |
| **fixture 별 페이지 수 차이** | **0** |

→ font-weight 변경 (HY견명조 사용 텍스트의 bold 제거) 이 페이지네이션에 영향 없음.

### 9.4 SVG byte 차이 (HY견명조 사용 영역 광범위 영향)

| Fixture | 페이지 수 | byte 차이 | 평가 |
|---|---|---|---|
| **exam_science** | 4 | **4 / 4** (모든 페이지) | PR 본문 권위 영역 (페이지 1 쪽번호 + 본문) |
| **exam_kor** | 20 | **20 / 20** (모든 페이지) | PR 본문 "전체" 정합 |
| **exam_eng** | 8 | **8 / 8** (모든 페이지) | HY견명조 사용 |
| **exam_math** | 20 | **20 / 20** (모든 페이지) | HY견명조 사용 |

→ HY견명조 사용 텍스트가 모든 페이지에 광범위 영향. 변경 본질은 `font-weight="bold"` 제거만 (좌표/크기/색상/font-family 변경 0). PR 본문 100% 재현.

### 9.5 `font-weight="bold"` 출현 횟수 정량 측정

본 PR 의 본질 효과 — `font-weight="bold"` 강제 적용 제거의 정량 측정:

| Fixture | before (devel) | after (cherry-pick) | 제거된 bold |
|---|---|---|---|
| exam_science | 116 | 34 | **-82** |
| exam_kor | 878 | 119 | **-759** |
| exam_eng | 276 | 120 | **-156** |
| exam_math | 394 | 150 | **-244** |
| **합계** | **1,664** | **423** | **-1,241** |

→ HY견명조 사용 텍스트의 `font-weight="bold"` 강제 적용이 4 fixture 합계 **1,241건** 제거. 다른 heavy face (HY헤드라인M / HY견고딕 / HY견명조B / HY그래픽) 의 bold 적용은 보존 (after 의 잔존 423건은 다른 폰트 영역).

### 9.6 페이지 1 쪽번호 "1" 권위 케이스 검증 (PR 본문 명시 영역)

`exam_science_001.svg` 우상단 (x≈924, y≈115, font-size=44, HY견명조):

| 상태 | SVG element |
|---|---|
| **Before** | `<text ... font-family="HY견명조,..." font-size="44" `**`font-weight="bold"`**` fill="#000000">1</text>` |
| **After** | `<text ... font-family="HY견명조,..." font-size="44" fill="#000000">1</text>` |

→ `font-weight="bold"` 제거 (CharShape.bold=false 권위 회복). 다른 함초롬바탕 / 굴림 등 폰트의 `font-weight="bold"` 는 유지 (회귀 0). 케이스별 명시 가드 (`feedback_hancom_compat_specific_over_general`) 정합 입증.

### 9.5 다음 단계

1. ✅ 본 1차 검토 보고서 작성 (현재 문서)
2. ✅ 본 환경 결정적 재검증
3. ✅ SVG 생성 — `output/svg/pr584_before/{exam_science,exam_kor,exam_eng,exam_math}/` + `output/svg/pr584_after/...`
4. ✅ Docker WASM 빌드 완료 (4,581,498 bytes)
5. ✅ 광범위 페이지네이션 sweep — 164 fixture 1,614 페이지 / 페이지 수 회귀 0
6. ⏳ **작업지시자 시각 판정** (★ 게이트, HY견명조 사용 영역 광범위 + WASM 다양한 hwp 검증) — 본 단계 대기 중
7. ⏳ 통과 시 devel merge + push + PR close
8. ⏳ 처리 보고서 (`pr_584_report.md`) 작성 + archives 이동

## 10. 메모리 정합

- ✅ `feedback_visual_regression_grows` — 시각 판정 게이트 정합 운영
- ✅ `feedback_v076_regression_origin` — Stage 0 정밀 진단으로 이슈 본문 가설 정정 (사용자 보고 "회색" → 실제 본질 "굵기만") 정합
- ✅ `feedback_hancom_compat_specific_over_general` — 케이스별 명시 가드 (HY견명조 vs HY견명조B 구분, 단일 face 만 제거)
- ✅ `feedback_rule_not_heuristic` — hardcoded list 의 명시 face 룰 (휴리스틱 아님)
- ✅ `feedback_pdf_not_authoritative` — PR 본문이 한컴 PDF 도 검정임을 명시 (회색 가설 정정의 근거)
- ✅ `feedback_per_task_pr_branch` — Task #574 단일 본질 PR 정합
- ✅ `feedback_pr_comment_tone` — 본 검토 톤 차분/사실 중심
- ✅ `feedback_check_open_prs_first` — 본 PR OPEN 상태 확인 후 진행
- ✅ `feedback_small_batch_release_strategy` — 본 사이클 (5/1 ~ 5/5) 누적 18번째 PR

---

**검토자**: 클로드 (페어 프로그래밍 파트너)
**1차 검토 단계 — 작업지시자 승인 대기**.
