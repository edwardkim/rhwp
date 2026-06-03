# Task #546 최종 보고서 — exam_science.hwp 페이지네이션 회귀 정정

**이슈**: [#546](https://github.com/edwardkim/rhwp/issues/546) — exam_science.hwp 2페이지 페이지네이션 회귀 — PR #506 머지 후 본문 누락 (4페이지 → 6페이지)
**처리 결정**: ✅ **옵션 A (전체 revert)** — `82e41ba` 의 typeset.rs +36 + layout.rs +58 모두 제거
**처리일**: 2026-05-03

## 1. 처리 결과 요약

| 항목 | 결과 |
|------|------|
| 회귀 origin | `82e41ba` (Task #460 보완5: Square wrap 그림 아래 텍스트 y위치 보정 — layout + typeset) |
| 정정 방식 | 옵션 A (전체 revert) — 옵션 C (페이지/단 경계 검사) 시도 후 효과 0 검증으로 작업지시자 결정 |
| 변경 LOC | -94 (typeset.rs -36 + layout.rs -58) |
| exam_science.hwp 페이지 | 6 → **4** (회귀 정정) ✅ |
| p2 단 0 items / used | 2 / 132.7 px → **37 / 1133.6 px** ✅ |
| 결정적 검증 | 모두 통과 |
| 광범위 회귀 검증 (byte) | 105 페이지 byte-identical + exam_science 4 페이지 의도된 정정 |
| WASM 빌드 | ✅ 4,442,504 bytes (-19,366 from PR #538 시점) |
| 시각 판정 (작업지시자) | ✅ 통과 |

## 2. 회귀 본질 분석

### 2.1 회귀 origin (`82e41ba` 변경)

**의도**: HWP3 Square wrap 그림 아래 텍스트가 그림과 겹쳐 출력되는 결함 정정 — wrap zone 종료 시 `current_height` 를 그림 하단으로 advance.

**구현**: `src/renderer/typeset.rs` 에 `wrap_around_pic_bottom_px` 필드 추가 + 그림 하단 y 미리 계산 + wrap zone 종료 시 `current_height = max(current_height, bottom_px)` 보정.

### 2.2 exam_science 의 부작용

exam_science.hwp 의 페이지 1 우상단 Square wrap 그림 (pi=21, 39.7×36.1mm):
- 2단 레이아웃 + 그림이 단 0 끝 근처 + 풍부한 wrap-around paragraph

**double advance 패턴 발생**:

```
1. wrap zone 진입 (current_height = 531.69 px)
2. wrap-around paragraphs 흘러감 (current_height 누적, 그림 옆 통과)
3. wrap zone 종료 시 보정 적용
   current_height = max(current_height, 668.09)
   ← wrap-around 가 이미 그림 옆을 다 통과했는데 추가 advance
4. 후속 paragraph 가 페이지/단 끝 도달
5. 페이지 분리 (4 → 6 페이지)
```

### 2.3 옵션 C 가 효과 없는 이유

옵션 C 의 가드 `bottom_px <= col_h`:

| Square wrap | bottom_px | col_h | 가드 결과 |
|-------------|-----------|-------|----------|
| pi=21 | 668.09 | 1215.15 | 통과 (적용) |
| pi=37 | 752.05 | 1215.15 | 통과 (적용) |
| pi=60 | 1052.87 | 1215.15 | 통과 (적용) |

→ 보정값 자체는 col 영역 내. 결함은 col 경계가 아닌 **wrap-around paragraph 의 누적 height 와의 상호작용** 영역. 옵션 C 검출 불가.

### 2.4 광범위 영향이 작았던 이유

본 환경 8 fixture / 105 페이지 byte-identical — **2단 + 그림이 단 0 끝 + 풍부한 wrap-around** 의 specific 조합에서만 결함 trigger. PR #506 의 다른 fixture 검증에서도 미검출 (작업지시자 환경의 exam_science 가 본 결함의 정밀 fixture).

## 3. 변경 영역 (옵션 A revert)

### 3.1 제거된 변경

#### `src/renderer/layout.rs` (-58 LOC) — 현재 영향 0 (무해 영역)

`wrap_pic_bottom_y` 계산 + `wrap_anchor_shape_seen` 가드 + Square wrap 그림 처리 후 첫 일반 paragraph 의 `y_offset` 보정. Para-relative 분기에서 `wrap_pic_bottom_y = 0.0` 으로 설정되어 현재 모든 exam_science Square wrap 그림에서 영향 0.

#### `src/renderer/typeset.rs` (-36 LOC) — 회귀 직접 origin

- `TypesetState::wrap_around_pic_bottom_px` 필드
- 초기화 코드
- wrap zone 종료 시 `current_height = max(current_height, wrap_around_pic_bottom_px)` 보정
- non-TAC Picture/Shape Square wrap 분기에서 `wrap_around_pic_bottom_px` 계산

### 3.2 신규 회귀 테스트

`tests/issue_546.rs`:

```rust
#[test]
fn issue_546_exam_science_p2_pagination_restored() {
    let bytes = fs::read("samples/exam_science.hwp").unwrap();
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).unwrap();
    
    // v0.7.9 정답지: 4 페이지 (회귀 시: 6)
    assert_eq!(doc.page_count(), 4, "exam_science.hwp 는 4 페이지여야 함");
}
```

## 4. 검증 결과

### 4.1 결정적 검증 (모두 통과)

| 게이트 | 결과 |
|--------|------|
| `cargo test --lib` | ✅ **1113 passed** (PR #538 시점과 동일) |
| `cargo test --test issue_546` (신규) | ✅ 1 passed |
| `cargo test --test issue_505` | ✅ 9/9 (PR #507 회귀 0) |
| `cargo test --test issue_530` | ✅ 1 (PR #531 회귀 0) |
| `cargo test --test issue_418/501` | ✅ 회귀 0 |
| `cargo test --test svg_snapshot` | ✅ **6/6 passed** |
| `cargo clippy --lib -- -D warnings` | ✅ 0 건 |
| `cargo build --release` | ✅ Finished |

### 4.2 광범위 fixture sweep (byte-level)

PR #538 머지 후 (`e585b58`) ↔ revert 후 SVG byte 비교:

| fixture | byte-identical | 결과 |
|---------|---------------|------|
| 2010-01-06.hwp | 6/6 | ✅ 회귀 0 |
| 21_언어_기출_편집가능본.hwp | 15/15 | ✅ 회귀 0 (PR #538 fixture) |
| exam_eng.hwp | 8/8 | ✅ 회귀 0 |
| exam_kor.hwp | 20/20 | ✅ 회귀 0 |
| exam_math.hwp | 20/20 | ✅ 회귀 0 |
| **exam_science.hwp** | **0/4** | ✅ **의도된 정정** |
| synam-001.hwp | 35/35 | ✅ 회귀 0 |
| 복학원서.hwp | 1/1 | ✅ 회귀 0 |

### 4.3 v0.7.9 페이지 수 정합

| fixture | v0.7.9 | revert 후 | 정합 |
|---------|--------|-----------|------|
| 2010-01-06 | 6 | 6 | ✅ |
| 21_언어_기출 | (PR #538) | 15 | ✅ |
| exam_eng | 8 | 8 | ✅ |
| exam_kor | 20 | 20 | ✅ |
| exam_math | 20 | 20 | ✅ |
| **exam_science** | **4** | **4** | ✅ (회귀 정정) |
| synam-001 | 35 | 35 | ✅ |
| 복학원서 | 1 | 1 | ✅ |
| aift | 77 | 77 | ✅ |

### 4.4 WASM 빌드

| 산출물 | 크기 |
|--------|------|
| `pkg/rhwp_bg.wasm` | 4,442,504 bytes (PR #538 시점 4,461,870 -19,366 — 94 LOC 제거 반영) |
| `pkg/rhwp.js` | 변동 없음 |
| `rhwp-studio/public/rhwp_bg.wasm` | ✅ 동기화 |
| `rhwp-studio/public/rhwp.js` | ✅ 동기화 |

### 4.5 작업지시자 시각 판정

작업지시자 인용:
> 시각 회귀 통과입니다.

→ exam_science p2 본문 정상 출력 + 다른 fixture 회귀 0 확인.

## 5. Task #460 보완5 손실 영향

`82e41ba` 의 본 의도 (HWP3 Square wrap 그림 아래 텍스트 y위치 정합) 는 본 revert 로 손실. 작업지시자 시각 판정 통과로 다음 두 가능성 중 하나:

1. **HWP3 fixture 가 본 환경에 부재** → 결함 잠재
2. **다른 commit 이 보완5 의 본질을 이미 흡수** (예: `ab2f4d0` HWP3 paper-relative → column-relative) → 결함 재발 없음
3. **보완5 가 처음부터 부작용 큰 정정** → revert 가 정합한 결정

**향후 처리**: HWP3 fixture 에서 Square wrap 그림 아래 텍스트 결함 재발견 시 별도 task 로 **페이지네이션 안전한 방식** (예: wrap-around paragraph 의 누적 height 추적 + 그림 하단 도달 여부 검사) 으로 재시도 권장.

## 6. 단계별 산출물

| Stage | 산출물 |
|-------|--------|
| 1 | `mydocs/working/task_m100_546_stage1.md` (회귀 origin bisect 식별) |
| 2 | `mydocs/plans/task_m100_546.md` + `task_m100_546_impl.md` (수행 + 구현 계획서) |
| 3 | `mydocs/working/task_m100_546_stage3.md` (옵션 C 시도 결과 0 + 옵션 A 적용) |
| 4 | `mydocs/working/task_m100_546_stage4.md` (회귀 검증 종합) |
| 5 | 본 보고서 + WASM 빌드 + 시각 판정 |

## 7. commit + 머지 절차

```bash
git checkout local/task546
# (이미 commit 됨: 9575667)

git checkout local/devel
git merge local/task546 --no-ff -m "Merge local/task546: Task #546 exam_science.hwp 페이지네이션 회귀 정정 (82e41ba revert) — closes #546"

git checkout devel
git merge local/devel --ff-only
git push origin devel

gh issue close 546 --repo edwardkim/rhwp --comment "..."
```

## 8. 메모리 정합

- ✅ `feedback_v076_regression_origin` — bisect 로 회귀 origin 정확히 식별 (단일 commit `82e41ba`)
- ✅ `feedback_visual_regression_grows` — 광범위 fixture sweep + 작업지시자 시각 판정 게이트
- ✅ `feedback_pdf_not_authoritative` / `reference_authoritative_hancom` — 한컴 2010/2020 직접 비교 (작업지시자 시각 판정)
- ✅ `feedback_image_renderer_paths_separate` — 본 정정은 typeset 단계, renderer 분기 영향 없음
- ✅ `feedback_hancom_compat_specific_over_general` — 옵션 C 의 일반화 정정이 효과 없음 → 옵션 A 의 case-specific revert 로 정합
- ✅ `feedback_search_troubleshootings_first` — Stage 1 직전 troubleshootings 사전 검색 (직접 매핑 영역 없음)
- ✅ `feedback_close_issue_verify_merged` — close 전 devel 머지 검증 진행

## 9. 후속 사항

- [ ] devel 머지 + push
- [ ] 이슈 #546 close
- [ ] orders 갱신 (20260503.md)
- [ ] HWP3 Square wrap fixture 의 시각 결함 점검 (재발견 시 별도 task)
