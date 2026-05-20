# Task #1035 최종 결과 보고서 — HWP3 vs HWP5 변환본 페이지 alignment fix

**Issue**: [#1035 HWP3 vs HWP5 변환본 페이지별 paragraph alignment 차이](https://github.com/edwardkim/rhwp/issues/1035)
**Branch**: `local/task1035`
**Milestone**: v1.0.0
**관련**: PR #1009 (Task #1007, closed — base 부정합 회귀로 close) follow-up

---

## 1. 결과 요약

PR #1009 (Task #1007, closed) 의 cross-paragraph vpos reset 감지 휴리스틱 base 적용 + **Task #1035 narrow 가드** (high_threshold 0.85→0.95, aux_trigger 제거) 로 sample16-hwp5 페이지 수 64 유지 + alignment **37.5% → 93.75%** 달성. 작업지시자 한컴 한글 정답지 시각 검증 통과.

**잔존**: 4 미정합 페이지 (p21 등) 및 p23 overflow 는 **HWP5 변환본 paragraph height 과대 측정** (HWP3 대비 약 2배) 본질 영역 — 본 task 범위 외, **별도 issue 등록 예정**.

---

## 2. PR #1009 와의 관계

PR #1009 의 분석 + 코드 자산 재활용:
- 변환본 encoder 의 `LineSeg.vertical_pos` page-reset 신호 감지
- variant 가드 (is_hwp3_variant) 한정 동작
- engine.rs + typeset.rs 동일 로직 두 경로 정합

PR #1009 close 사유 (메인테이너 sweep): sample16-hwp5 64 → 65 (+1 over-split 회귀). **본 Task #1035 의 narrow 가드 2 항목** 으로 over-split 회피 + alignment 더 큰 향상:

| 시도 | sample16-hwp5 | alignment |
|------|---------------|-----------|
| devel baseline | 64 | 24/64 (37.5%) |
| PR #1009 그대로 (0.85 + aux) | 65 (+1 회귀) | 23/64 (악화) |
| **Task #1035 (0.95 + main만)** | **64** ✓ | **60/64 (93.75%)** ✓ |

---

## 3. Fix 본질

PR #1009 base + 2 narrow 가드:

```rust
// 변경 A: high_threshold 0.85 → 0.95 (보수적)
let high_threshold = body_height_hu_for_variant * 95 / 100;

// 변경 B: aux_trigger 제거 (empty bridge 휴리스틱 false positive 다수)
if main_trigger {  // 종전 main_trigger || aux_trigger
    variant_vpos_reset_break = true;
}
```

`src/renderer/pagination/engine.rs` + `src/renderer/typeset.rs` 두 경로 모두 동일 narrow.

---

## 4. 변경 위치

| 파일 | 변경 |
|------|------|
| `src/renderer/pagination/engine.rs` | +variant_vpos_reset_break (PR #1009 base + Task #1035 narrow) |
| `src/renderer/typeset.rs` | 동일 로직 두 경로 정합 |
| `src/renderer/pagination.rs` | `PaginationOpts::is_hwp3_variant` 필드 |
| `src/document_core/queries/rendering.rs` | `is_hwp3_variant` 전달 |

variant 식별 인프라 (cfb_reader, model/document.rs, parser/mod.rs, hwpx/mod.rs) 는 **#1005 이미 머지** — 재활용.

---

## 5. 검증

### 5.1 자동 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✓ warning 0 |
| `cargo clippy --release --lib -- -D warnings` | ✓ clean |
| **`cargo fmt --all -- --check`** | ✓ clean (feedback_cargo_fmt_all_required 정합) |
| `cargo test --release --lib` | ✓ 1308 passed (variant 인프라 테스트 +1) |
| `cargo test --release --tests` | ✓ FAILED 0 (전체 integration) |
| `cargo test --release --test issue_1035_alignment` | ✓ 1 passed (회귀 가드) |

### 5.2 회귀 sweep (25 fixture)

- 변환본 9 종 (sample/4/5/10/11/13/14/16/19-hwp5 + .hwpx): 페이지 수 무변동
- HWP3 native 9 종: 무변동
- 일반 HWP5 (exam_kor/eng/math, aift, biz_plan): 무변동

→ **모든 fixture 회귀 0**.

### 5.3 단위 테스트 추가

`tests/issue_1035_alignment.rs::hwp3_sample16_hwp5_page_count_64`:
- sample16-hwp5 페이지 수 = 64 단언 (PR #1009 over-split 65 회귀 재발 방지)

---

## 6. 성공 기준 충족

| 조건 | 기준 | 결과 |
|------|------|------|
| C1: alignment 정합률 향상 | 37.5% → ≥80% | ✓ **93.75%** (목표 초과) |
| C2: sample16-hwp5 64 유지 | over-split 회피 | ✓ |
| C3: 변환본 9 종 회귀 0 | 페이지 수 무변동 | ✓ |
| C4: 일반 HWP5/HWP3 회귀 0 | exam_*, aift, biz_plan | ✓ |
| C5: cargo test 1307+ passed | clean | ✓ (1308) |
| C6: clippy + fmt --all | clean | ✓ |
| C7: 작업지시자 시각 검증 | 한컴 정답 비교 | ✓ (p21 영역 alignment 정합 확인) |

---

## 7. 잔존 미정합 — 별도 issue 등록 예정

### 7.1 본질

본 Task #1035 의 60/64 alignment 잔존 4 미정합 페이지 + p23 overflow 의 본질:
- HWP5 변환본의 paragraph height 가 HWP3 의 약 **2배** (font/spacing metric 차이)
- 같은 11 paragraphs 가 HWP3 에서 fit, HWP5 변환본 에서 overflow
- Task #1008 격차 D (폰트 매핑) 영역 연장

### 7.2 한컴 정답 단언

작업지시자 한컴 한글 viewer 정답지 단언:
- 한컴 p21 (footer "-21-") = rhwp HWP3 p23 (idx=22) — 11 items + "나." Table subheader **alignment 정합** ✓
- 한컴 정답 페이지 수 = 64 또는 근처 (이전 38 페이지 추정은 한컴 viewer 로딩 시 근사값 — 무효)

### 7.3 새 issue 본질

"**HWP5 변환본 paragraph height 과대 측정 — HWP3 대비 약 2배**":
- p21 alignment 잔존 (pi=440 force-break 시 cumulative 누적 over)
- p23 overflow (pi=460 PartialParagraph split + 시각적 overflow)
- 근본 fix: paragraph height 측정 / font metric / spacing 처리 정합

---

## 8. 커밋 history

| 커밋 | 단계 |
|------|------|
| 4b7ec69f | Stage 1 — 진단 + 수행/구현 계획서 |
| 51041cfa | Stage 2 — PR #1009 base + narrow 가드 (alignment 37.5% → 93.75%) |
| 1f2d582f | Stage 3 — case-specific 시도 + fundamental tension 진단 |
| (Stage 4) | 최종 보고서 + orders 갱신 |

---

## 9. 한계 + 후속 작업 권고

### 9.1 한계

- 잔존 4 미정합 페이지 (p21 등) + p23 overflow — **HWP5 변환본 paragraph height 과대 측정** 본질, 단순 paginator 휴리스틱으로 해결 불가
- Task #1008 격차 D (폰트 매핑) 와 영역 겹침 — 추가 폰트/spacing 매핑 필요

### 9.2 후속 작업 권고

별도 issue 등록 + 새 task 진행:
- "HWP5 변환본 paragraph height 과대 측정 (HWP3 대비 약 2배) — p21 alignment + p23 overflow"
- 분석: font_ids 매핑 (Task #1008 격차 D 확장) + line_height/line_spacing/spacing_before/after 산출 정합

closes #1035 (alignment 부분 해결)
