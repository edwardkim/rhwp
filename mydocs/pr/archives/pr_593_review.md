# PR #593 검토 보고서

**PR**: [#593 fix: Square wrap 표 horz_rel_to=단 속성 정합 (closes #590)](https://github.com/edwardkim/rhwp/pull/593)
**작성자**: @planet6897 (PR 등록) / Jaeook Ryu = @jangster77 (commit author)
**상태**: OPEN, **mergeable=MERGEABLE** (PR #592 와 함께 본 사이클 두 번째 케이스)
**처리 결정**: ⏳ **검토 중** (1차 검토)
**검토 시작일**: 2026-05-05

## 1. 검토 핵심 질문

1. **본질 결함 식별 정합성** — Issue #480 도입 분기가 모든 Square-wrap 표를 무조건 문단 좌측 가장자리 기준으로 강제 배치하며 `horz_rel_to` 속성을 무시하는 결함이 본 환경에서도 재현되는가?
2. **분기 가드 1줄 추가의 회귀 위험** — `horz_rel_to=Para` 한정 가드가 다른 케이스 (Column/Page/Paper) 를 `compute_table_x_position` 명세 기반 분기로 정합하게 보내는가?
3. **단일 commit 의 처리** — 본질 (`layout.rs` +5 LOC) + plans + working + report + orders 갱신이 모두 1 commit 에 묶여있음. 본 환경 cherry-pick 처리 방식?

## 2. PR 정보

| 항목 | 값 | 평가 |
|------|-----|------|
| 제목 | Square wrap 표 horz_rel_to=단 속성 정합 | 정합 |
| author (PR 등록) | @planet6897 | — |
| commit author | Jaeook Ryu (= @jangster77) | 컨트리뷰터간 협업 흐름 (PR #561~#592 동일 패턴) |
| changedFiles | 6 / +531 / -4 | 본질 코드 +5/-4 + 보고서 다수 + orders 갱신 |
| 본질 변경 | `src/renderer/layout.rs` +5/-4 (분기 가드 1줄 + 주석 갱신) | 단일 파일 |
| **mergeable** | **MERGEABLE** | PR #592 와 함께 본 사이클 두 번째 케이스 |
| Issue | closes #590 | ✅ |

## 3. PR 의 1 commit 분석 (특이 사례)

| commit | 설명 | 본 환경 처리 |
|--------|------|-------------|
| **`5d3b3e2d` fix + 보고서 + orders 갱신** | `layout.rs` +5/-4 + plans/working/report + orders | ⭐ **cherry-pick 대상** (본질만 + 보고서 분리 검토) |

**특이 사항**: 이전 PR 들 (PR #561~#592) 은 본질 commit + plans/working/report/orders 를 **별도 commit 으로 분리**했지만, 본 PR 은 모든 것이 1 commit. **orders 충돌 우려** 있었으나 본 환경 임시 cherry-pick test 결과 auto-merge 정합 (충돌 0).

## 4. 본질 변경 영역

### 4.1 결함 가설

PR 본문:
> `src/renderer/layout.rs:2285-2300` (Issue #480 도입 분기) 가 모든 Square-wrap 표를 무조건 문단 좌측 가장자리(`col_area.x + effective_margin`) 기준으로 강제 배치하며 `horz_rel_to` 속성을 무시했음.

### 4.2 정량 측정 + 수식 검증

문단 2 ParaShape `margin_left=1700, indent=+2000` → effective_margin = 24.67 px → 우측 24.7 px (=6.5mm) 시프트:

```
table_x = col_area.x + effective_margin + h_offset
        = 117.17    + 24.67            + 9.44
        = 151.28 px ← SVG 실측치와 정확히 일치
```

→ 사용자 보고 "오른쪽으로 6.5mm 치우침" 의 본질 = effective_margin (24.67 px = 6.5 mm) 강제 적용.

### 4.3 정정 (분기 가드 1줄 추가)

```diff
-                } else if !is_tac && tbl_is_square {
+                } else if !is_tac && tbl_is_square
+                    && matches!(t.common.horz_rel_to, crate::model::shape::HorzRelTo::Para) {
                     // [Issue #480 / #590] horz_rel_to=Para 인 Square wrap 표만 paragraph 영역
                     // (col_area + margin) 기준으로 정렬. horz_rel_to=Column/Page/Paper 는
                     // compute_table_x_position 의 기본 분기에서 명세대로 처리한다.
```

**핵심 가드 정합성:**
- `horz_rel_to=Para` 한정 → Issue #480 분기는 Para 기준 표만 처리
- `Column/Page/Paper` → `compute_table_x_position` 명세 기반 분기로 위임 (HWP 표준)
- 케이스별 명시 가드 (`feedback_hancom_compat_specific_over_general` 정합)

### 4.4 적용 영역 / 미적용 영역

**적용 (위치 변경):**
- Square wrap (`wrap=어울림`) 표
- `treat_as_char=false`
- `horz_rel_to ∈ {Column, Page, Paper}`

**미적용 (이전 동작 유지):**
- TAC 표 (`treat_as_char=true`)
- `horz_rel_to=Para` Square wrap 표 (#480 분기 동작 유지)
- 글뒤로 / 글앞으로 wrap

## 5. 본 환경 직접 검증 (임시 브랜치 cherry-pick test)

`pr593-cherry-test` 임시 브랜치에서 `5d3b3e2d` 단독 cherry-pick:

| 단계 | 결과 |
|------|------|
| `5d3b3e2d` cherry-pick (no-commit) | ✅ Auto-merging src/renderer/layout.rs + mydocs/orders/20260504.md (충돌 0) |
| `cargo test --lib --release` | ✅ **1134 passed** / 0 failed / 2 ignored (PR #592 baseline 정합, 회귀 0) |
| `cargo clippy --release --lib` | ✅ 0건 |

→ **MERGEABLE 표시 정합** — 본질 commit (`5d3b3e2d`) cherry-pick 시 본 환경 devel 에 깨끗하게 적용 + 충돌 0. orders 갱신은 본 환경 5/4 orders 의 PR #587 처리 후속 영역 다음에 자연스럽게 추가됨.

## 6. PR 본문의 자기 검증 결과

| 검증 | PR 본문 결과 | 본 환경 재검증 (임시) |
|------|---------|----------|
| `cargo test --lib` | 1125 passed | ✅ 1134 passed (본 환경 baseline 정합) |
| `cargo clippy` | 신규 경고 0 | ✅ 0건 |
| `cargo build --release` | ✅ | ⏳ 본격 검증 |
| `svg_snapshot` | 6/6 passed | ⏳ 본격 검증 |
| 5 샘플 56 페이지 sweep | 32 byte-identical + 4 의도된 정정 (exam_kor) | ⏳ 본 환경 sweep 권장 |
| p17 [A] 셀 좌측 | 151.28 → 126.61 px (col_left + h_offset, IR 정합) | ⏳ 본 환경 시각 판정 |
| hancomdocs PDF p24 | 시각 정합 | ⚠️ `feedback_pdf_not_authoritative` — 작업지시자 시각 판정으로 보정 |
| 메인테이너 시각 판정 | 미진행 | ⏳ 본 환경 시각 판정 게이트 |

## 7. 차분 페이지 4 의 의미

| 페이지 | halign | 변화 |
|---|---|---|
| **p17 [A]** | Left | 151.28 → 126.61 (-24.7 px, **사용자 보고 정정**) |
| p18 [B] x2 | Left | 602.83 → 591.49 (-11.33 px, 동류 정정) |
| p19 [B] | Left | (동류 정정) |
| p14 [A] x4 | Right | 515.60 → 508.05 (-7.55 px, 명세 정합 향상) |

**p14 (halign=Right) 미세 변경의 의미:**
- 이전 동작: `inline_x_override` 경로가 Right 정렬 시에도 h_offset 을 ADD 하던 모순 (Right 는 SUBTRACT 해야 함)
- `compute_table_x_position` 의 명세 기반 분기로 통일
- → **명세 정합 향상** (회귀 아님)

## 8. 메인테이너 정합성 평가

### 정합 영역
- ✅ **MERGEABLE 표시** — PR #592 와 함께 본 사이클 두 번째 케이스. PR base 정합
- ✅ **본질 cherry-pick 깨끗** — 충돌 0 (auto-merge), orders 갱신도 정합
- ✅ **결정적 검증 정합** — cargo test --lib 1134 passed (회귀 0) / clippy 0
- ✅ **단일 줄 본질 정정** — 분기 가드 1줄 + 주석 갱신 (회귀 위험 영역 좁힘)
- ✅ **케이스별 명시 가드** — `horz_rel_to=Para` 한정 + Column/Page/Paper 위임 (`feedback_hancom_compat_specific_over_general` 정합)
- ✅ **HWP 표준 룰 명시** — `compute_table_x_position` 명세 기반 분기로 위임 (`feedback_rule_not_heuristic`)
- ✅ **수식 검증** — `table_x = col_area.x + effective_margin + h_offset = 117.17 + 24.67 + 9.44 = 151.28 px` 정량 측정으로 결함 origin 식별 (`feedback_v076_regression_origin`)
- ✅ **명세 정합 향상** — p14 (halign=Right) 의 `inline_x_override` ADD/SUBTRACT 모순도 정정

### 우려 영역
- ⚠️ **PR 본문 hancomdocs PDF 시각 정합** — `feedback_pdf_not_authoritative` 메모리 룰 잠재 우려. 작업지시자 시각 판정으로 보정 필요
- ⚠️ **단일 commit 처리** — 본질 + 보고서 + orders 가 모두 1 commit 에 묶여있음. 본 환경 cherry-pick 시 컨트리뷰터 fork 의 plans/working/report 도 함께 들어옴 — 본 환경 자체 처리 보고서 패턴과 차이
- ⚠️ **작업지시자 시각 판정 게이트** — PR 본문 미진행 명시. 본 환경 cherry-pick 후 직접 시각 판정 필수

## 9. 1차 검토 결론

### 정합 평가
- ✅ **본질 cherry-pick 가능 + 충돌 0** — `5d3b3e2d` 단독 + MERGEABLE 표시 정합
- ✅ **결정적 검증** — 1134 passed / clippy 0
- ✅ **단일 줄 본질 정정** — 분기 가드 1줄 추가
- ✅ **케이스별 명시 가드** — horz_rel_to=Para 한정
- ⏳ **시각 판정 별도 진행 필요** — 본 환경 cherry-pick 후 작업지시자 직접 시각 판정 필수
- ⏳ **광범위 sweep 본격 검증 필요** — 본 환경 자동 sweep (PR #564~#592 패턴)

### 권장 처리 방향 (작업지시자 결정 대기)

#### 옵션 A — 핀셋 cherry-pick (권장)
- `5d3b3e2d` 단독 cherry-pick (1 commit 통합 — 본 환경 자체 보고서는 처리 단계에서 작성)
- 본 환경 결정적 재검증 + 광범위 페이지네이션 sweep + WASM
- 작업지시자 시각 판정 (★ 게이트) — exam_kor p17 [A] / p18 [B] / p19 [B] / p14 [A] (Right 정렬)
- 통과 시 devel merge + push + PR close 처리

#### 옵션 B — 추가 정정 요청
- 시각 판정에서 잔존 결함 발견 시 컨트리뷰터에게 추가 정정 요청

#### 옵션 C — close + 본 환경 직접 처리
- 시각 판정 다수 결함 발견 시 본 환경에서 직접 처리

→ **작업지시자 결정 대기**. 옵션 A 권장 — 본질 cherry-pick 깨끗 + MERGEABLE + 결정적 검증 통과 + 단일 줄 본질 정정 + 케이스별 명시 가드 + 명세 정합 향상.

## 10. 옵션 A 진행 결과 (작업지시자 승인 후)

### 10.1 핀셋 cherry-pick

| 단계 | 결과 |
|------|------|
| 본질 commit cherry-pick (`5d3b3e2d`) | ✅ 충돌 0, author Jaeook Ryu 보존 |
| local/devel cherry-pick commit | `3682cff` |

### 10.2 결정적 검증 (모두 통과)

| 검증 | 결과 |
|------|------|
| `cargo test --lib --release` | ✅ **1134 passed** / 0 failed / 2 ignored (회귀 0) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_546` | ✅ 1 passed (Task #546 회귀 0) |
| `cargo test --test issue_554` | ✅ 12 passed |
| `cargo clippy --release --lib` | ✅ 0건 |
| `cargo build --release` | ✅ Finished |
| Docker WASM 빌드 | ✅ **4,581,465 bytes** (1m 33s, PR #592 baseline -62 bytes — 가드 1줄 추가의 LLVM 최적화 효과) |

### 10.3 광범위 페이지네이션 sweep (페이지 수 회귀 자동 검출)

| 통계 | 결과 |
|---|---|
| 총 fixture | **164** (158 hwp + 6 hwpx) |
| 총 페이지 (devel baseline) | **1,614** |
| 총 페이지 (cherry-pick 후) | **1,614** |
| **fixture 별 페이지 수 차이** | **0** |

→ Square wrap 표 horz_rel_to 정정이 페이지네이션에 영향 없음.

### 10.4 SVG 차이 (PR 본문 100% 재현)

| Fixture | 페이지 수 | byte 차이 | 평가 |
|---|---|---|---|
| **exam_kor** | 20 | **4 (page 14, 17, 18, 19)** | PR 본문 명시 정확 재현 |

**상세:**
- **page 17 [A]**: Left 정렬 — 151.28 → 126.61 (-24.7 px, **사용자 보고 정정**)
- **page 18 [B] x2**: Left 정렬 — 동류 정정
- **page 19 [B]**: Left 정렬 — 동류 정정
- **page 14 [A] x4**: Right 정렬 — 515.60 → 508.05 (-7.55 px, **명세 정합 향상**)

→ PR 본문 "32 byte-identical + 4 의도된 정정" 정확 재현. 다른 fixture (exam_eng / exam_math / exam_science / exam_social 등) 무영향 — 케이스별 명시 가드 정합 입증.

### 10.5 다음 단계

1. ✅ 본 1차 검토 보고서 작성 (현재 문서)
2. ✅ 본 환경 결정적 재검증
3. ✅ SVG 생성 — `output/svg/pr593_before/exam_kor/` + `output/svg/pr593_after/exam_kor/` (4 페이지 의도된 차이)
4. ✅ Docker WASM 빌드 완료 (4,581,465 bytes)
5. ✅ 광범위 페이지네이션 sweep — 164 fixture 1,614 페이지 / 페이지 수 회귀 0
6. ⏳ **작업지시자 시각 판정** (★ 게이트, exam_kor p14/p17/p18/p19 + WASM 다양한 hwp 검증) — 본 단계 대기 중
7. ⏳ 통과 시 devel merge + push + PR close
8. ⏳ 처리 보고서 (`pr_593_report.md`) 작성 + archives 이동

## 11. 메모리 정합

- ✅ `feedback_visual_regression_grows` — 시각 판정 게이트 정합 운영
- ✅ `feedback_v076_regression_origin` — 수식 검증 (table_x = 117.17 + 24.67 + 9.44 = 151.28) 으로 결함 origin 정량 식별
- ✅ `feedback_hancom_compat_specific_over_general` — 케이스별 명시 가드 (horz_rel_to=Para 한정)
- ✅ `feedback_rule_not_heuristic` — `compute_table_x_position` 명세 기반 분기로 위임
- ⚠️ `feedback_pdf_not_authoritative` — hancomdocs PDF 비교는 참고이지만 권위는 작업지시자 한컴 환경
- ✅ `feedback_per_task_pr_branch` — Task #590 단일 본질 PR 정합
- ✅ `feedback_pr_comment_tone` — 본 검토 톤 차분/사실 중심
- ✅ `feedback_check_open_prs_first` — 본 PR OPEN 상태 확인 후 진행
- ✅ `feedback_small_batch_release_strategy` — 본 사이클 (5/1 ~ 5/5) 누적 20번째 PR

---

**검토자**: 클로드 (페어 프로그래밍 파트너)
**1차 검토 단계 — 작업지시자 승인 대기**.
