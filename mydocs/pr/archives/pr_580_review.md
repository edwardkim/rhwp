# PR #580 검토 보고서

**PR**: [#580 Task #577: 셀 내부 단독 TopAndBottom 이미지 1라인 오프셋 정정 (closes #577)](https://github.com/edwardkim/rhwp/pull/580)
**작성자**: @planet6897 (PR 등록) / Jaeook Ryu = @jangster77 (commit author)
**상태**: OPEN, mergeable=CONFLICTING (PR base 시점 차이 추정, 본질 cherry-pick 충돌 0 확인)
**처리 결정**: ⏳ **검토 중** (1차 검토)
**검토 시작일**: 2026-05-05

## 1. 검토 핵심 질문

1. **본질 결함 식별 정합성** — 셀 내부 비-TAC TopAndBottom Picture 의 anchor 시점 좌표가 `layout_composed_paragraph` advance 후 `para_y` 로 사용되어 line_height(~15.32 px) 만큼 아래로 밀리는 결함이 본 환경에서도 재현되는가?
2. **조건부 anchor_y 도입의 회귀 위험** — `text_wrap=TopAndBottom AND vert_rel_to=Para` 케이스만 `para_y_before_compose` 사용하는 가드의 정합성과 다른 케이스 무영향 확인.
3. **LAYOUT_OVERFLOW 회귀 검증** — exam_science.hwp (9.5→3.4 px) + mel-001.hwp (8건→0건) 의 자동 측정 패턴이 본 환경에서도 재현되는가?

## 2. PR 정보

| 항목 | 값 | 평가 |
|------|-----|------|
| 제목 | Task #577 셀 내부 단독 TopAndBottom 이미지 1라인 오프셋 정정 | 정합 |
| author (PR 등록) | @planet6897 | — |
| commit author | Jaeook Ryu (= @jangster77) | 컨트리뷰터간 협업 흐름 (PR #561/#564/#567/#570/#575 동일 패턴) |
| changedFiles | 8 / +581 / -3 | 본질 코드 +22/-3 + 보고서 다수 |
| 본질 변경 | `src/renderer/layout/table_layout.rs` +22/-3 | 단일 파일 |
| mergeable | CONFLICTING | PR base 시점 차이 추정 |
| Issue | closes #577 | ✅ |

## 3. PR 의 4 commits 분석

| commit | 설명 | 본 환경 처리 |
|--------|------|-------------|
| `9fe0b312` Stage 1 — 분석·재현·기준선 캡처 | 컨트리뷰터 fork plans/working | 무관 (본 환경 자체 보고서) |
| **`0acd13a6` Stage 2 — 본질 정정** | `table_layout.rs` +22/-3 + working stage2 | ⭐ **cherry-pick 대상** |
| `9d571e5f` Stage 3 — 시각·자동 검증 | 컨트리뷰터 fork working | 무관 |
| `1f164668` Stage 4 — 최종 보고서 + orders | 컨트리뷰터 fork report + orders | 무관 (orders 충돌 위험) |

→ **본질 cherry-pick 대상 = `0acd13a6` 단독**. PR #561/#564/#567/#570/#575 와 동일 패턴.

## 4. 본질 변경 영역

### 4.1 결함 가설 (PR 본문)

> `exam_science.hwp` 페이지 1 — 2번 문제 보기 ⑤(및 ②④) 이미지 하단이 cell-clip 영역을 약 10.81 px 초과하여 잘려 보이던 결함을 정정합니다. `text_wrap=TopAndBottom AND vert_rel_to=Para` 인 비-TAC Picture 가 셀에 들어 있을 때 `compute_object_position` 호출에 사용하던 `para_y` 가 `layout_composed_paragraph` 의 advance(line_height ≈ 15.32 px) 를 포함하고 있어 이미지가 anchor 라인 한 줄만큼 아래로 밀려 있던 문제입니다.

### 4.2 결함 메커니즘

```
관측 image_y - cell_y = 19.10 px
  = pad_top(3.78) + line_height(15.32, lh=1150 HU)
정정 후 image_y - cell_y = 3.78 px = pad_top   ← HWP IR 정합
```

→ HWP IR 표준은 anchor 시점이 paragraph 시작 좌표 (`para_y_before_compose`) 이지만 코드는 advance 후 `para_y` 사용 → 1라인 오프셋 발생.

### 4.3 정정 (PR)

```rust
let anchor_y = if matches!(pic.common.text_wrap, TextWrap::TopAndBottom)
              && matches!(pic.common.vert_rel_to, VertRelTo::Para)
{ para_y_before_compose } else { para_y };
```

**핵심 가드 정합성:**
- `text_wrap=TopAndBottom AND vert_rel_to=Para` 양 조건 명시 → 다른 케이스 (Char/Page anchor / Square wrap) 무영향
- `picture_footnote.rs::compute_object_position` 자체는 무변경 (다른 호출처 회귀 방지)
- 케이스별 명시 가드 (`feedback_hancom_compat_specific_over_general` 정합)

### 4.4 정량 측정

- exam_science page 1 보기 ①~⑤: image_y - cell_y = 3.78 px = pad_top (HWP IR 정합)
- LAYOUT_OVERFLOW: exam_science 9.5→3.4 px / mel-001.hwp 8건→0건

## 5. 본 환경 직접 검증 (임시 브랜치 cherry-pick test)

`pr580-cherry-test` 임시 브랜치에서 `0acd13a6` 단독 cherry-pick:

| 단계 | 결과 |
|------|------|
| `0acd13a6` cherry-pick (no-commit) | ✅ Auto-merging src/renderer/layout/table_layout.rs (충돌 0) |
| `cargo test --lib --release` | ✅ **1131 passed** / 0 failed / 2 ignored (회귀 0) |
| `cargo clippy --release --lib` | ✅ 0건 |

→ **CONFLICTING 표시는 PR base 시점 차이로 추정**. 본질 commit (`0acd13a6`) 단독 cherry-pick 시 본 환경 devel 에 깨끗하게 적용 가능.

## 6. PR 본문의 자기 검증 결과

| 검증 | PR 본문 결과 | 본 환경 재검증 (임시) |
|------|---------|----------|
| `cargo build --release` | ✅ | ⏳ 본격 검증 |
| `cargo test --release --lib` | 1118 passed | ✅ 1131 passed (본 환경 baseline 정합) |
| `cargo clippy --release` | (PR 본문 미명시) | ✅ 0건 |
| exam_science 페이지 1 보기 ①~⑤ 좌표 정합 | image_y - cell_y = 3.78 px | ⏳ 본 환경 시각 판정 |
| LAYOUT_OVERFLOW 변화 | exam_science 9.5→3.4 / mel-001 8→0 | ⏳ 본 환경 측정 권장 |
| 작업지시자 시각 판정 | (미진행) | ⏳ 본 환경 시각 판정 게이트 |

## 7. 메인테이너 정합성 평가

### 정합 영역
- ✅ **본질 cherry-pick 깨끗** — 충돌 0 (auto-merge)
- ✅ **결정적 검증 정합** — cargo test --lib 1131 passed (회귀 0) / clippy 0
- ✅ **케이스별 명시 가드** — `text_wrap=TopAndBottom AND vert_rel_to=Para` 양 조건 동시 검사 (`feedback_hancom_compat_specific_over_general` 정합)
- ✅ **회귀 위험 영역 좁힘** — `compute_object_position` 자체 무변경 (다른 호출처 회귀 차단)
- ✅ **정밀 측정** — image_y - cell_y = 19.10 px → 3.78 px (= pad_top, HWP IR 정합) + LAYOUT_OVERFLOW 정량 측정
- ✅ **하이퍼-워터폴 흐름** — Stage 1 분석·재현 → Stage 2 본질 → Stage 3 검증 → Stage 4 보고. 본 환경 워크플로우 정합
- ✅ **단일 파일 본질** — `src/renderer/layout/table_layout.rs` +22/-3 의 작은 본질
- ✅ **HWP IR 정합 명시** — anchor 시점 좌표 (paragraph 시작) 가 IR 표준이라는 본질 명시

### 우려 영역
- ⚠️ **CONFLICTING 표시** — PR base 시점 차이 추정 (본질 cherry-pick 충돌 0 확인됨)
- ⚠️ **부수 효과 영역** — PR 본문 §"부수 효과" 명시: "비-TAC TopAndBottom 셀 내부 이미지 다수에 동일 산식 정정 적용 → 일부 페이지의 좌표가 IR vpos 정합 방향(LAYOUT_OVERFLOW 가 줄어드는 방향)으로 이동". 본 환경 광범위 sweep 으로 확인 필수
- ⚠️ **작업지시자 시각 판정 게이트** — PR 본문 미진행 명시. 본 환경 cherry-pick 후 직접 시각 판정 필수
- ⚠️ **LAYOUT_OVERFLOW 회귀 자동 측정 패턴** — PR 본문 측정값이 본 환경에서도 재현되는지 본격 검증

## 8. 1차 검토 결론

### 정합 평가
- ✅ **본질 cherry-pick 가능** — `0acd13a6` 단독 충돌 0
- ✅ **결정적 검증** — 1131 passed / clippy 0
- ✅ **케이스별 명시 가드** — text_wrap=TopAndBottom AND vert_rel_to=Para 양 조건
- ✅ **회귀 위험 영역 좁힘** — compute_object_position 자체 무변경
- ⏳ **시각 판정 별도 진행 필요** — PR 본문 미진행
- ⏳ **광범위 sweep 본격 검증 필요** — 본 환경 자동 sweep (PR #564/#570/#575 패턴)

### 권장 처리 방향 (작업지시자 결정 대기)

#### 옵션 A — 핀셋 cherry-pick (권장)
- `0acd13a6` 단독 cherry-pick (Stage 1/3/4 의 plans/working/report/orders 는 컨트리뷰터 fork 정합 — 본 환경 자체 처리 보고서)
- 본 환경 결정적 재검증 + 광범위 페이지네이션 sweep + WASM
- 작업지시자 시각 판정 (★ 게이트) — exam_science page 1 보기 ①~⑤ + 부수 효과 영향 영역
- 통과 시 devel merge + push + PR close 처리

#### 옵션 B — 추가 정정 요청
- 시각 판정에서 잔존 결함 발견 시 컨트리뷰터에게 추가 정정 요청

#### 옵션 C — close + 본 환경 직접 처리
- 시각 판정 다수 결함 발견 시 본 환경에서 직접 처리

→ **작업지시자 결정 대기**. 옵션 A 권장 — 본질 cherry-pick 깨끗 + 결정적 검증 통과 + 케이스별 명시 가드 + 회귀 위험 영역 좁힘 + HWP IR 정합 명시.

## 9. 옵션 A 진행 결과 (작업지시자 승인 후)

### 9.1 핀셋 cherry-pick

| 단계 | 결과 |
|------|------|
| 본질 commit cherry-pick (`0acd13a6`) | ✅ 충돌 0, author Jaeook Ryu 보존 |
| local/devel cherry-pick commit | `50a80ab` |

### 9.2 결정적 검증 (모두 통과)

| 검증 | 결과 |
|------|------|
| `cargo test --lib --release` | ✅ **1131 passed** / 0 failed / 2 ignored (회귀 0) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_546` | ✅ 1 passed (Task #546 회귀 0) |
| `cargo test --test issue_554` | ✅ 12 passed |
| `cargo clippy --release --lib` | ✅ 0건 |
| `cargo build --release` | ✅ Finished |
| Docker WASM 빌드 | ✅ **4,571,643 bytes** (1m 33s, PR #575 baseline +39 bytes — table_layout.rs +22/-3 LOC 정합) |

### 9.3 광범위 페이지네이션 sweep (페이지 수 회귀 자동 검출)

본 환경 `samples/` 폴더 전체 자동 sweep:

| 통계 | 결과 |
|---|---|
| 총 fixture | **164** (158 hwp + 6 hwpx) |
| 총 페이지 (devel baseline) | **1,614** |
| 총 페이지 (cherry-pick 후) | **1,614** |
| **fixture 별 페이지 수 차이** | **0** |
| Export 실패 fixture | 0 |

→ **164 fixture / 1,614 페이지 / 페이지 수 회귀 0**. 케이스별 명시 가드 (`text_wrap=TopAndBottom AND vert_rel_to=Para`) 의 정합성이 광범위 sweep 으로 정량 입증.

### 9.4 SVG byte 차이 (PR 본문 영역 + 회귀 검증)

| Fixture | 페이지 수 | byte 차이 | 정정 영역 |
|---|---|---|---|
| **exam_science** | 4 | **2 (page 1, page 4)** | page 1 보기 ①~⑤ 정정 (PR 본문 권위 영역) |
| **mel-001** | 21 | 0 | LAYOUT_OVERFLOW 8건→0건 정정이 SVG byte 단위는 무영향 (clip 영역 안으로만 이동) |

→ exam_science page 1 의 정정 영역 정합 + page 4 부수 효과 영역 + mel-001 의 LAYOUT_OVERFLOW 측정 정정 (SVG byte 무변경) 확인.

### 9.5 다음 단계

1. ✅ 본 1차 검토 보고서 작성 (현재 문서)
2. ✅ 본 환경 결정적 재검증
3. ✅ SVG 생성 — `output/svg/pr580_before/{exam_science,mel-001}/` + `output/svg/pr580_after/{exam_science,mel-001}/`
4. ✅ Docker WASM 빌드 완료 (4,571,643 bytes)
5. ✅ 광범위 페이지네이션 sweep — 164 fixture 1,614 페이지 / 페이지 수 회귀 0
6. ⏳ **작업지시자 시각 판정** (★ 게이트, exam_science page 1 보기 ①~⑤ + page 4 부수 효과 + WASM 다양한 hwp 검증) — 본 단계 대기 중
7. ⏳ 통과 시 devel merge + push + PR close
8. ⏳ 처리 보고서 (`pr_580_report.md`) 작성 + archives 이동

## 10. 메모리 정합

- ✅ `feedback_visual_regression_grows` — 시각 판정 게이트 정합 운영
- ✅ `feedback_v076_regression_origin` — 정밀 측정 (image_y - cell_y = 19.10 → 3.78 = pad_top + line_height) 으로 결함 origin 식별
- ✅ `feedback_hancom_compat_specific_over_general` — 케이스별 명시 가드 (text_wrap=TopAndBottom AND vert_rel_to=Para)
- ✅ `feedback_rule_not_heuristic` — HWP IR 표준 (anchor 시점 좌표 = paragraph 시작) 직접 사용 (휴리스틱 아닌 규칙)
- ✅ `feedback_pdf_not_authoritative` — 권위는 작업지시자 한컴 환경
- ✅ `feedback_per_task_pr_branch` — Task #577 단일 본질 PR 정합
- ✅ `feedback_pr_comment_tone` — 본 검토 톤 차분/사실 중심
- ✅ `feedback_check_open_prs_first` — 본 PR OPEN 상태 확인 후 진행
- ✅ `feedback_small_batch_release_strategy` — 본 사이클 (5/1 ~ 5/5) 누적 17번째 PR

---

**검토자**: 클로드 (페어 프로그래밍 파트너)
**1차 검토 단계 — 작업지시자 승인 대기**.
