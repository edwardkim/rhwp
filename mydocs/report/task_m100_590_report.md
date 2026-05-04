# Task #590: exam_kor.hwp 17쪽 [A] 글상자 우측 치우침 정정 — 최종 보고서

## 이슈

[#590](https://github.com/edwardkim/rhwp/issues/590)

## 사용자 보고

`exam_kor.hwp` 17페이지 [35~36] 지문의 [A] 글상자 위치가 오른쪽으로 치우침.

## 분석 결과

### 측정 (SVG 좌표)

| 항목 | 값 |
|---|---|
| body / 단 0 좌측 | 117.17 px |
| [A] 표 셀 실제 좌측 | 151.28 px |
| 단 좌측 기준 옵셋 실측 | 34.11 px ≈ 9.0 mm |
| HWP IR 기대 옵셋 (`horz=단(708 HU)`) | 9.44 px = 2.5 mm |
| **차이** | **24.7 px ≈ 6.5 mm 우측 시프트** |

### 원인

`src/renderer/layout.rs:2285-2300` (Issue #480 / Task #295 도입 분기):

```rust
} else if !is_tac && tbl_is_square {
    let area_x = col_area.x + effective_margin;
    ...
}
```

이 분기는 모든 Square-wrap 표를 **무조건 문단 좌측 가장자리 기준**으로 강제 배치하며 `horz_rel_to` 속성을 무시했음. 문단 2 ParaShape `margin_left=1700, indent=+2000` → effective_margin 24.67 px → 우측 24.7 px (=6.5mm) 시프트.

수식 검증:

```
table_x = col_area.x + effective_margin + h_offset
        = 117.17    + 24.67            + 9.44
        = 151.28 px ← SVG 실측치와 정확히 일치
```

## 패치

`src/renderer/layout.rs:2285-2287` 분기 가드 추가:

```diff
-                } else if !is_tac && tbl_is_square {
+                } else if !is_tac && tbl_is_square
+                    && matches!(t.common.horz_rel_to, crate::model::shape::HorzRelTo::Para) {
                     // [Issue #480 / #590] horz_rel_to=Para 인 Square wrap 표만 paragraph 영역
                     // (col_area + margin) 기준으로 정렬. horz_rel_to=Column/Page/Paper 는
                     // compute_table_x_position 의 기본 분기에서 명세대로 처리한다.
```

본질: Square-wrap 표를 무조건 paragraph 기준으로 배치하던 것을 `horz_rel_to=Para` 인 경우에만 한정. `horz_rel_to=Column/Page/Paper` 는 `compute_table_x_position` (table_layout.rs:975-1005) 의 명세 기반 분기에서 처리.

## 검증

### 단위 테스트

```
cargo test --lib
test result: ok. 1125 passed; 0 failed; 3 ignored
```

### exam_kor.hwp p17 [A] (사용자 보고 케이스)

| | Baseline | Patched | 기대 |
|---|---|---|---|
| 셀 좌측 | 151.28 px | **126.61 px** | 126.61 (= col_left + h_offset) |
| 시프트 | 24.7 px → 0 | ✅ HWP IR 정합 | |

### 광범위 sweep (5 샘플 56 페이지)

| 샘플 | 페이지 | 차분 | 평가 |
|---|---|---|---|
| exam_kor.hwp | 20 | 4 (p14, p17, p18, p19) | 모두 의도된 정정 |
| exam_eng.hwp | 8 | 0 | byte-identical |
| exam_math.hwp | 20 | 0 | byte-identical |
| exam_science.hwp | 4 | 0 | byte-identical |
| exam_social.hwp | 4 | 0 | byte-identical |
| **합계** | **56** | **4** | **52 byte-identical, 4 의도된 정정** |

#### p17/p18/p19 (`halign=Left`)

모두 `horz=단(708 HU), halign=Left` Square wrap 브래킷. 단 좌측 + 2.5mm (h_offset) 위치로 일관 정정.

#### p14 (`halign=Right`)

베이스라인은 브래킷이 단 우측에 ≈ flush (1.9 px). 패치 후 단 우측에서 2.5mm 안쪽 (9.46 px) 으로 이동. HWP 명세 (`ref_x + (ref_w - tbl_w) - h_offset`) 정합. 시각 차이 미세.

이전 동작은 `inline_x_override` 경로가 halign=Right 에서도 h_offset 을 ADD 하던 모순 (Right 정렬은 우측에서 SUBTRACT 해야 함) 에 의한 우연 결과 — 명세상 부정확.

### hancomdocs PDF 시각 검증

| 페이지 | rhwp 결과 | hancomdocs PDF |
|---|---|---|
| p17 [A] | 단 좌측 + 2.5mm 위치 | 단 좌측 거의 flush (≈ 정합) |
| p14 [A] | 단 우측 - 2.5mm inset | 단 우측 ≈ flush (미세 차이) |

p17 (사용자 보고) 정정 명확. p14 미세 차이는 명세 우위.

### clippy

신규 경고 0. 기존 errors (table_ops.rs:1007, object_ops.rs:298 panicking_unwrap) 는 사전 존재 (본 패치 무관).

## 적용 영역 / 미적용 영역

**적용 (위치 변경):**
- Square wrap (`wrap=어울림`) 표
- `treat_as_char=false`
- `horz_rel_to ∈ {Column, Page, Paper}`

**미적용 (이전과 동일):**
- TAC 표 (`treat_as_char=true`) — 인라인 처리
- `horz_rel_to=Para` Square wrap 표 — 기존 #480 분기 유지
- 글뒤로 / 글앞으로 (`BehindText`/`InFrontOfText`) wrap

## 회귀 위험 메모

`feedback_essential_fix_regression_risk` 적용 — 본질 정정.
- 5 샘플 광범위 sweep 회귀 0 (byte-identical 32 페이지, 의도된 정정 4 페이지).
- `compute_table_x_position` 의 명세 기반 분기로 통일 → 단일 룰 (`feedback_rule_not_heuristic` 정합).

## 메모리 정합

- `feedback_essential_fix_regression_risk`: 광범위 샘플 sweep + 한컴 PDF 비교로 검증.
- `feedback_pdf_not_authoritative`: PDF 는 보조 ref. p14 미세 차이는 명세 우위로 판단.
- `feedback_rule_not_heuristic`: 정정은 단일 룰 (`compute_table_x_position` 통합), 분기 가드만 좁힘.

## 변경 파일

| 파일 | 변경 |
|---|---|
| `src/renderer/layout.rs` | 분기 가드 1줄 추가 + 주석 업데이트 |
| `mydocs/plans/task_m100_590.md` | 수행 계획서 신규 |
| `mydocs/plans/task_m100_590_impl.md` | 구현 계획서 신규 |
| `mydocs/working/task_m100_590_stage1.md` | 단계별 보고서 신규 |
| `mydocs/report/task_m100_590_report.md` | 본 보고서 신규 |
| `mydocs/orders/20260504.md` | 오늘할일 갱신 |

## 결론

✅ 사용자 보고 p17 [A] 6.5mm 시프트 정정 완료 (151.28 → 126.61 px)
✅ 1125 단위 테스트 GREEN
✅ 5 샘플 광범위 sweep 회귀 0 (52/56 byte-identical, 4/56 의도된 정정)
✅ 명세 정합도 향상 (`compute_table_x_position` 통합)

`local/task590` → `local/devel` merge → push origin/devel → 이슈 close 진행.
