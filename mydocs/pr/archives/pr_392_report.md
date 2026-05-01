# PR #392 처리 보고서 — 정상 머지 (cherry-pick)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#392](https://github.com/edwardkim/rhwp/pull/392) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) |
| 이슈 | [#391](https://github.com/edwardkim/rhwp/issues/391) |
| 처리 결정 | **정상 머지 (cherry-pick) + 작업지시자 시각 판정 통과** |
| 처리 일자 | 2026-04-28 |

## 결함 요약

**v0.7.7 의 Task #359 정정 후속 회귀**:
- `samples/exam_eng.hwp` (2단 다단 섹션): 8p → **11p 회귀**
- 단 채움 비대칭 + 단독 1-item 단 다수 발생 (p3 단 1, p5 단 0, p7 단 1)

원인: Task #359 의 누적 공식 변경 (`current_height += total_height`) 이 다단 섹션의 layout (vpos 기반 stacking) 와 정합 안됨. 다단에서는 `trailing_ls` 인플레이션 발생 → 단 조기 종료.

## 정황 — 컨트리뷰터 작업 대규모 머지 후속 파장

본 PR 의 결함 (다단 회귀) 은 **2026-04-26 ~ 2026-04-27 의 컨트리뷰터 작업 대규모 머지 + 회귀 정정 + 추가 회귀 유발** 의 연쇄 결과:

1. **v0.7.6 사이클 (2026-04 초~중반)**: 컨트리뷰터 PR 다수 머지 (PR #320 Task #313, PR #327 Task #324, PR #341 Task #340, PR #343 Task #321~#332, PR #351 Task #347)
2. **회귀 발생** (메인테이너 환경): 작업지시자가 v0.7.6 후 다수 페이지네이션 회귀 발견
3. **PDF 기준 비교의 한계 발견** (2026-04-27, PR #360 정황):
   - 컨트리뷰터 환경 (macOS 한글 Viewer + macOS 인쇄): exam_eng 등 다른 페이지 분할 결과
   - 작업지시자 환경 (Windows 한컴 2010 + 2022 편집기): 페이지 분할 정답지
   - 외부 컨트리뷰터들이 자기 환경 PDF 를 기준으로 검증한 코드가 작업지시자 환경에서는 회귀로 보임
4. **v0.7.7 긴급 정정** (2026-04-26 ~ 04-27): Task #359 (k-water-rfp 311px drift), Task #361 (page_num), Task #362 (kps-ai PartialTable + Square wrap 8 항목)
5. **Task #359 정정 부작용 발견**: 단단 (k-water-rfp) 정정에 사용한 누적 공식이 다단 (exam_eng) 에서 회귀 유발 — 본 PR (#392) 정정 대상

회귀 정정 → 부작용 발생 → 또 정정의 연쇄 패턴. 진짜 origin 은 **PDF 환경 의존성 (한글 Viewer + macOS 인쇄 vs 한컴 편집기)** 의 차이가 처음에 인지 안 됐던 것.

본 PR 머지로 다단 섹션 회귀 정정 + Task #359/#361/#362 의 단단 효과 보존.

## 변경 내용

### `src/renderer/typeset.rs` — 누적 공식 분기 (3 줄, 2 곳)

```rust
// [Task #391] 다단/단단 분기:
//   - 단단 (col_count == 1): total_height (k-water-rfp p3 311px drift 차단, #359)
//   - 다단 (col_count > 1): height_for_fit (exam_eng 8p 정상 단 채움 복원)
st.current_height += if st.col_count > 1 { fmt.height_for_fit } else { fmt.total_height };
```

라인 805 (`fits → place 전체배치`) + 815 (`line_count == 0` 폴백) 두 곳.

### `tests/exam_eng_multicolumn.rs` (신규) — 회귀 테스트

```rust
#[test]
fn exam_eng_page_count_after_359_fix() {
    assert_eq!(doc.page_count(), 8, "exam_eng.hwp 8 페이지 기대 (Task #391 / #359 회귀 복원)");
}
```

### 문서

- task_m100_391 수행/구현계획서 + stage1-3 + 최종 보고서 + baseline
- CHANGELOG, orders 갱신

## 처리 절차

### Stage 1: cherry-pick
- `local/pr392` 브랜치 (`local/devel` 분기)
- PR 의 4 commit cherry-pick — Jaeook Ryu attribution 보존
  - `14011e2` Stage 1 재현 정량 진단 + Red 테스트
  - `567475e` Stage 2 다단/단단 누적 공식 분기 (Green)
  - `3f90774` Stage 3 회귀 검증
  - `6194f64` Stage 4 통합 검증 + 최종 보고서 + WASM
- 충돌 없이 자동 머지

### Stage 2: 자동 회귀

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | **1016 passed** (이전 1014 → +2 신규 테스트) |
| `cargo test --test svg_snapshot` | 6/6 |
| `cargo test --test exam_eng_multicolumn` | **1/1** (신규 회귀 테스트) |
| `cargo test --test page_number_propagation` | 2/2 |
| `cargo clippy --lib -- -D warnings` | 통과 |
| `cargo check --target wasm32-unknown-unknown --lib` | 통과 |
| WASM Docker 빌드 | 1m 24s, 4.1 MB |

### Stage 3: 11 샘플 회귀

| 샘플 | 단수 | 현재 devel | **PR #392 후** | 판정 |
|---|---|---|---|---|
| **exam_eng** | 2단 | **11p, 0** | **8p, 0** | ✓ 본 task 핵심 |
| exam_kor | 2단 | 24p, 30 | 24p, 30 | 무변화 |
| **k-water-rfp** | 1단 | 27p, 0 | **27p, 0** | ✓ #359 보존 |
| kps-ai | 1단 | 79p, 5 | 79p, 5 | 무변화 |
| aift | 1단 | 77p, 3 | 77p, 3 | 무변화 |
| form-01 | - | 1p, 0 | 1p, 0 | 무변화 |
| KTX | 1단 | 27p, 1 | 27p, 1 | 무변화 |
| hwp-multi-001 | 1단 | 10p, 0 | 10p, 0 | 무변화 |
| exam_math | - | 20p, 0 | 20p, 0 | 무변화 |

→ 단단 (1단) 모든 샘플 무변화. Task #359/#361/#362 효과 완벽 보존. exam_eng 만 정정.

### Stage 4: 작업지시자 시각 판정 ✅
- SVG (debug overlay) 출력 전/후 비교: `output/svg/task391-debug/{before,after}/`
- WASM 빌드 후 시각 확인
- **작업지시자 시각 판정 통과** (2026-04-28)

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|---|---|
| `feedback_hancom_compat_specific_over_general.md` (구조 명시 가드 우선) | ✅ `col_count > 1` 구조 분기, 측정 의존 없음 |
| `feedback_v076_regression_origin.md` (작업지시자 시각 판정 게이트) | ✅ 시각 판정 통과 후 머지 |
| `feedback_pdf_not_authoritative.md` (PDF 환경 의존성) | ✅ 본 정정의 진짜 origin 인지 |
| `reference_authoritative_hancom.md` (한컴 2010/2022 편집기 정답지) | ✅ 작업지시자 환경 시각 판정 기준 |

## 작성자 기여

@planet6897 (Jaeook Ryu) — Task #321~#332, #340, #347 등 v0.7.6 의 핵심 컨트리뷰터. 본 PR 은 v0.7.7 의 Task #359 정정 후속의 다단 회귀를 정확히 진단 + 정정.

PR 본문의 진단 (다단 layout vpos 기반 stacking vs 단단 layout total_height 정합) 이 정확하고, pre-#359 baseline 까지 worktree 빌드로 비교 검증한 점이 양호.

## 다음 단계

1. local/devel merge → devel push
2. PR #392 댓글 (정황 설명 추가) + close
3. 이슈 #391 close

## 참고

- 검토 문서: `mydocs/pr/pr_392_review.md`
- 시각 판정 산출물: `output/svg/task391-debug/{before,after}/`
- 메모리: `feedback_v076_regression_origin.md`, `feedback_pdf_not_authoritative.md`
