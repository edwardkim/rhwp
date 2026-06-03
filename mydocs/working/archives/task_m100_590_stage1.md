# Task #590 Stage 1 — 패치 + 검증 완료 보고서

## 단계 범위

구현 계획서 1단계(패치 + 빌드) ~ 6단계(clippy) 일괄 수행.

## 패치

`src/renderer/layout.rs:2285-2300` 분기 가드 추가:

```diff
-                } else if !is_tac && tbl_is_square {
+                } else if !is_tac && tbl_is_square
+                    && matches!(t.common.horz_rel_to, crate::model::shape::HorzRelTo::Para) {
                     // [Issue #480] Square wrap 표는 paragraph 영역 (col_area + margin) 기준으로 정렬.
+                    // [Issue #590] horz_rel_to=Column/Page/Paper 는 compute_table_x_position 의
+                    //              기본 분기에서 명세대로 처리한다.
```

`HorzRelTo` 는 layout.rs:7 에 이미 import 되어 있어 추가 use 불필요.

## 검증 결과

### 1. 빌드

```
$ cargo build --release --bin rhwp
   Compiling rhwp v0.7.9
    Finished `release` profile [optimized] target(s) in 1m 02s
```

### 2. 단위 테스트

```
$ cargo test --lib
test result: ok. 1125 passed; 0 failed; 3 ignored
```

회귀 0.

### 3. clippy

신규 경고 0. 기존 errors (table_ops.rs:1007, object_ops.rs:298 의 panicking_unwrap) 는 본 패치 무관 사전 존재 (devel 동일 errors).

### 4. 핵심 검증 — exam_kor.hwp p17 [A]

| 항목 | Baseline | Patched | 기대값 |
|---|---|---|---|
| 셀 좌측 (x px) | 151.28 | **126.61** | 126.61 (= col_left 117.17 + h_offset 9.44) |
| 단 좌측 기준 옵셋 | 34.11 px (9.0mm) | **9.44 px (2.5mm)** | 2.5mm (`horz=단(708)`) |

✅ HWP IR 속성과 정확히 일치.

### 5. 광범위 sweep (5 샘플 56 페이지)

```
diff -rq /tmp/rhwp590_baseline /tmp/rhwp590_patched
└── 차분 4 파일 (모두 exam_kor):
    ├── exam_kor_014.svg  (p14 [A] 박스 4개, halign=Right)
    ├── exam_kor_017.svg  (p17 [A] 박스, halign=Left ← 사용자 보고)
    ├── exam_kor_018.svg  (p18 [B] 박스, halign=Left)
    └── exam_kor_019.svg  (p19 [B] 박스, halign=Left)

다른 4 샘플 (exam_eng, exam_math, exam_science, exam_social) 32 페이지: byte-identical
```

### 5-1. exam_kor p17/p18/p19 (halign=Left, 사용자 보고 케이스 동류)

모두 `horz=단(708), halign=Left` Square wrap 브래킷 표.

| 페이지 | Baseline x | Patched x | 변화 |
|---|---|---|---|
| p17 [A] (단 0) | 151.28 | 126.61 | -24.67 px (= effective_margin) |
| p18 [B] 1번 (단 1) | 602.83 | 591.49 | -11.33 px (= margin_left only, indent<0) |
| p18 [B] 2번 (단 1) | 602.83 | 591.49 | -11.33 px |
| p19 [B] (단 0/1) | (similar) | (similar) | -11.33 px |

→ 모두 단 좌측 + 2.5mm (h_offset) 위치로 일관 정정.

### 5-2. exam_kor p14 (halign=Right) 시각 변화

| 항목 | Baseline | Patched | 명세 동작 |
|---|---|---|---|
| 셀 우측 (x+w px) | 538.56 | 531.01 | col_right - h_offset |
| 단 우측 기준 옵셋 | 1.91 px (≈ flush) | 9.46 px (2.5mm inset) | 2.5mm |

베이스라인의 "≈ flush" 위치는 `inline_x_override` 경로가 halign=Right 에서도 h_offset 을 더하던 모순(우측 정렬 시 우측에서 빼야 함)에 의해 우연히 그렇게 보였던 것. **HWP 명세상 baseline 도 부정확** — `compute_table_x_position` 의 Column/Right 산식 (`ref_x + (ref_w - tbl_w) - h_offset`) 과 어긋났음.

패치 후: `compute_table_x_position` 명세 정합 (한 코드패스로 일관).

### 5-3. hancomdocs PDF 시각 비교

**p17 (사용자 보고)**: `samples/hancomdocs-exam_kor.pdf` p24 ([35-36] 페이지) 와 비교 — 베이스라인은 [A] 브래킷이 본문 안쪽으로 들어가 텍스트와 겹침, **패치 후는 단 좌측에 정확히 위치 (PDF 와 정합)** ✅

**p14**: 한컴 PDF 의 [A] 브래킷이 단 우측에 거의 flush. 패치 후 ≈ 2.5mm 안쪽으로 이동. 시각 차이 미세 (PNG 해상도에서 거의 식별 불가). 단, 명세 정합도는 패치 후가 우위.

## 회귀 위험 평가

- 다단/단일 단/표분할 상호작용: 차분 페이지 4개 모두 다단 페이지에서 발생. 다른 4 샘플 32 페이지에서 회귀 없음.
- 본 정정의 영향 영역: `wrap=Square` + (`tac=false`) + `horz_rel_to=Column/Page/Paper` 인 표만. `horz_rel_to=Para` 는 분기 발동 유지 (변경 없음).
- TAC 표, 글뒤로/글앞으로 표: 분기 미진입 (변경 없음).

## 결론

✅ 1125 단위 테스트 GREEN
✅ p17 [A] 좌측 시프트 6.5mm 정정 (사용자 보고 해결)
✅ p18/p19 [B] 동일 케이스 일괄 정정
⚠️ p14 [A] (halign=Right) 위치 미세 변경 (명세 정합도는 향상)
✅ 다른 4 샘플 byte-identical (회귀 0)
✅ clippy 신규 경고 0

**최종 결과 보고서 작성 단계로 진행 가능.**
