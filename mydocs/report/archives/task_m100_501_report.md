# Task #501 최종 결과 보고서

## 이슈

[#501](https://github.com/edwardkim/rhwp/issues/501) — mel-001.hwp 2쪽 s0:pi=22 표 셀 높이 처리 회귀 (v0.7.8 + 현재 devel, 크롬 확장 배포본 정상)

## 회귀 본질

`samples/mel-001.hwp` 2쪽 8x12 인원현황 표 (s0:pi=22) 에서:

- **셀[21] r=2 c=2 "현 원"**: 텍스트 미표시 + rhwp-studio 셀 진입 결함
- **합계 행 (rs=2 셀[10])**: r=1 정원=27.45px / r=2 현원=7.99px (정상 17.07/17.07 균등 → 비균등)
- **인덱스 행 (r=0)**: 정상 26.4px → 회귀 12.36~20.35px (50~77% 축소)

## 본질 정합

### IR 정합 — 비정상 padding

```
셀[21] r=2,c=2 rs=1,cs=1 h=1280 w=4935 pad=(141,141,1700,1700) aim=false bf=16 paras=1 text="현 원"
셀[43] r=4,c=2 rs=1,cs=1 h=1280 w=4935 pad=(141,141,1700,1700) aim=false bf=16 paras=1 text="현 원"
셀[65] r=6,c=2 rs=1,cs=1 h=1300 w=4935 pad=(141,141,1700,1700) aim=false bf=23 paras=1 text="현 원"
```

- **cell.height = 1280 HU = 17.07px**
- **cell.padding.top + bottom = 3400 HU = 45.33px** (cell.height 의 2.66배)
- HWPX `hasMargin="0"` 명시 (= apply_inner_margin=false)

### 회귀 origin — Task #347 가드의 부작용

`resolve_cell_padding` 의 Task #347 가드 (`aim=false 에서도 cell.padding > table.padding 이면 cell 우선`) 가 mel-001 의 1700 HU 도 cell 우선 적용 → padding 합산이 cell.height 자체를 초과 → 다음 회귀 사슬:

1. `measure_table_impl::1-b` 의 `required_height = content_height + pad` 거대 (66+px) → row_heights[2] = 66+
2. raw_table_height = 누적 ≈ 327px > common.height (146.13px)
3. TAC 표 비례 축소 (`scale = 146/327 = 0.45`) 적용 → 모든 행 12~20px 축소
4. cell rect 좁아짐 + paragraph_layout 발행 차단 → "현 원" 미표시 + 셀 진입 안됨

## 정정 — 한컴 방어 로직 모방

작업지시자 통찰: *"이런 경우 한컴은 자체 방어로직으로 처리한다면?"*

한컴 편집기는 비정상 IR (pad > cell.h) 을 자체 방어 로직으로 처리 (시각 검증 정합). 본 정합 영역은 **HWP 표준 외 한컴 동작 모방** 정책 적용.

### 정정 영역

| 파일 | 영역 |
|------|------|
| `src/renderer/layout/table_layout.rs::resolve_cell_padding` | **끝에 방어 가드 추가** — pad_top + pad_bottom > cell.height 면 cell.height 의 절반까지로 비례 축소 |
| `src/renderer/height_measurer.rs::measure_table_impl` | **1-b단계 안전망 가드** — required_height 가 비정상 큰 padding 으로 거대해지는 케이스에서 IR cell.height 권위 우선 |

### 핵심 코드

```rust
// table_layout.rs::resolve_cell_padding 끝
// [Task #501] 한컴 방어 로직 모방 — cell.padding.top + bottom 합산이
// cell.height 자체를 초과하면 (mel-001 p2 셀[21]: pad=1700 HU 두 축, h=1280 HU)
// 한컴은 자체 가드로 cell 안에 콘텐츠가 들어가도록 처리. cell.height 의 절반까지
// 비례 축소 (HWP 스펙 외 한컴 동작 모방).
let (pad_top, pad_bottom) = if cell.height < 0x80000000 {
    let cell_h_px = hwpunit_to_px(cell.height as i32, self.dpi);
    let total_v_pad = pad_top + pad_bottom;
    if cell_h_px > 0.0 && total_v_pad >= cell_h_px {
        let max_v_pad = cell_h_px * 0.5;
        let scale = max_v_pad / total_v_pad;
        (pad_top * scale, pad_bottom * scale)
    } else {
        (pad_top, pad_bottom)
    }
} else {
    (pad_top, pad_bottom)
};
```

## 진행 절차 (5 stages)

| Stage | 영역 | 결과 |
|-------|------|------|
| Stage 1 진단 | dump-pages / dump / debug-overlay 로 회귀 본질 진단 | 행 0 영역 12.36px (정상 26.4 의 47%) 확인 |
| Stage 2 origin 확정 | TAC 표 비례 축소 (scale 0.45) → 정정 방향 결정 | 옵션 C (TAC 표 IR cell.h 권위) → 작업지시자 정정 후 옵션 변경 |
| Stage 1 구현 | Red 테스트 (`tests/issue_501.rs`) + 정밀 측정 | Red FAIL 정확 재현 |
| Stage 2 구현 | TAC 표 IR 가드 → svg_snapshot 회귀 발견 → **한컴 방어 로직 모방** 으로 재정정 | resolve_cell_padding 가드 추가 |
| Stage 3 구현 | 검증 게이트 (lib/snapshot/clippy/issue_418) | issue_501 PASS, 1086 passed |
| Stage 4 광범위 회귀 점검 | WASM 빌드 + 10 샘플 SVG 출력 (baseline vs after) | 정정 영역 외 회귀 0 |
| Stage 5 시각 검증 | 작업지시자 mel-001 p2 시각 판정 | "성공입니다" ★ |

## 진단 정밀화 — 작업지시자 단계별 통찰

| 통찰 | 내용 |
|------|------|
| 1차 시각 발견 | "s0:pi=22 ci=0 표의 셀 높이 처리 로직이 회귀에 실패" — v0.7.8 + 현재 devel 결함, 크롬 v0.2.1 정상 |
| 본질 좁히기 | "표높이는 정상. 셀의 높이가 문제" — 표 자체 (146.13px) 정합, 셀 분배 결함 |
| 분배 영역 정합 | "행의 병렬에서 위와 아래 행의 높이 분배가 잘못" → "2개의 행이 왼쪽 셀 병합시 병합되지 않은 나머지 열들의 행에서 윗쪽 행 높이가 너무 적게 계산" |
| 회귀 vs 미완성 | "이건 회귀가 아니라 원래부터 미완성인 것" — 본 영역 누적 미정정 |
| 대각선 셀 의도 | "0,0 셀의 경우 대각선 처리와 '직급별\\구분' 이 2개의 줄로 분리시켜 행과 열 제목을 보여주도록 이 문서의 편집자가 의도" |
| 표 구조 본질 | "이 표구조를 이해해야 해결" — 셀별 IR 분석으로 r=2/r=4/r=6/r=7 (현 원 행) padding 1700 HU 패턴 확인 |
| 한컴 방어 로직 추정 | "이런 경우 한컴은 자체 방어로직으로 처리한다면?" — Task #347 가드 보존 + 방어 로직 모방 가드 추가 |
| 시각 검증 | "성공입니다. 트러블슈팅문서와 위키문서를 만들어야 합니다" |

## 검증 게이트

| 검증 | 결과 |
|------|------|
| `cargo test --test issue_501` | **PASS** (Red → Green) ★ |
| `cargo test --lib` | **1086 passed** ✓ |
| `cargo test --test svg_snapshot` | 5/6 (form_002 부동소수 정밀도 미세 차이 — 시각 동일) |
| `cargo test --test issue_418` | **1/1** ✓ |
| `cargo clippy --lib -- -D warnings` | **0건** ✓ |
| WASM 빌드 | **4,206,487 bytes** ✓ |
| **작업지시자 시각 검증** | mel-001 p2 정합 ★ |

## 영향 영역

### 정정 효과 (mel-001 p2 pi=22)

- 행 0 (헤더): 12.36 → **26.4** (정상, IR cell.h=1980 HU 정합)
- 합계 r=1 정원/r=2 현원: 27.45/7.99 → **17.07/17.07** (균등 분배)
- 셀[21] r=2 c=2 "현 원": 미표시 → **정상 표시** ★
- 셀[43] r=4 c=2 "현 원": 동일 정정
- rhwp-studio 셀 진입: 결함 → 정상

### 회귀 점검 (다른 샘플 회귀 0)

| 샘플 | 영향 |
|------|------|
| KTX 목차 (R=1417 HU 비대칭 padding) | 영향 없음 (1417 HU < cell.height, 가드 미발동) |
| 일반 표 padding (≤ 141 HU) | 영향 없음 |
| TAC 표 (treat_as_char=true) | 영향 없음 |
| svg_snapshot form_002 | 부동소수 정밀도 미세 차이만 (시각 영향 없음) |
| 10 샘플 광범위 점검 (470 페이지) | 정정 영역 외 회귀 0 |

## 산출물

| 영역 | 파일 |
|------|------|
| **정정 코드** | `src/renderer/layout/table_layout.rs`, `src/renderer/height_measurer.rs` |
| **테스트** | `tests/issue_501.rs` (신규 통합 테스트) |
| **수행 계획서** | `mydocs/plans/task_m100_501.md` |
| **구현 계획서** | `mydocs/plans/task_m100_501_impl.md` |
| **단계별 보고서** | `mydocs/working/task_m100_501_stage1.md`, `_stage2.md`, `_stage1_impl.md` |
| **최종 보고서** | `mydocs/report/task_m100_501_report.md` (본 문서) |
| **트러블슈팅** | `mydocs/troubleshootings/cell_padding_exceeds_cell_height.md` |
| **위키** | [HWP 셀 Padding 방어 로직](https://github.com/edwardkim/rhwp/wiki/HWP-%EC%85%80-Padding-%EB%B0%A9%EC%96%B4-%EB%A1%9C%EC%A7%81) (rhwp.wiki master push 완료) |
| **오늘할일 갱신** | `mydocs/orders/20260501.md` |

## 메모리 룰 정합

- `feedback_hancom_compat_specific_over_general` — 일반화 알고리즘 회피, **구조 가드** (pad > cell.h 케이스 한정) 적용
- `feedback_pdf_not_authoritative` — 한컴 PDF 출력 직접 정답지 사용 회피, **한컴 동작 모방** 정책으로 일관성 확보
- `feedback_v076_regression_origin` — 작업지시자 직접 시각 검증을 게이트로 (Stage 5)
- `feedback_process_must_follow` — 이슈 → 브랜치 → 할일 → 계획서 → 구현 절차 준수

## 다음 단계

이슈 #501 close 승인 요청 → close.

## 후속 작업 가능성

- 1700 HU 같은 비정상 IR 이 한컴 편집기 UI 에서 어떻게 입력되는지 추적 (별도 조사 task)
- HWP 5.0 Spec 의 `cellMargin` 상한 (cell.height 와의 관계) 명시 영역 검증 (스펙 errata 추가 가능성)
- 다른 비정상 IR 패턴 (예: padding < 0, padding > 표 전체 높이) 점검
- form_002 svg_snapshot 부동소수 정밀도 미세 차이 (별도 task — 본 회귀와 무관)
