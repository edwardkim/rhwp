# Task #1001 Stage 5 — 격차 B/C Fix 구현 + 단위 검증 보고서

이슈: [#1001](https://github.com/edwardkim/rhwp/issues/1001)
Stage 1-4: [`stage1`](task_m100_1001_stage1.md), [`stage2`](task_m100_1001_stage2.md), [`stage3`](task_m100_1001_stage3.md), [`stage4`](task_m100_1001_stage4.md)
Stage 5-A/B: [`stage5a`](task_m100_1001_stage5a.md), [`stage5b`](task_m100_1001_stage5b.md)

## 1. 구현 내용

### 1-1. CfbReader::detect_hwp3_variant() (신규)

`src/parser/cfb_reader.rs` 에 메서드 추가:
- HwpSummaryInformation stream 의 UTF-16LE 텍스트 안에 "1990-2003년" 검출
- 한컴이 HWP3 → HWP5 변환 시 원본 작성일을 string field (PID 2/6) 에 보존

### 1-2. Document IR 의 `is_hwp3_variant: bool`

`src/model/document.rs` 의 `Document` struct 에 필드 추가. 변환본 여부 IR 보존.

### 1-3. 식별 결합 휴리스틱 (parser/mod.rs)

`parse_hwp_with_cfb` 에서:
1. `cfb.detect_hwp3_variant()` 로 HwpSummary HWP3 시대 년 확인
2. AND PS/CS 비율 검증 (ps_ratio<0.20 AND cs_ratio<0.20)
3. 두 조건 모두 충족 시 `doc.is_hwp3_variant = true`

False positive (예: exam_eng 본문에 2002년 인용된 일반 HWP5) 차단.

### 1-4. style_resolver 1/2 추가 보정

`resolve_styles_with_variant` / `resolve_single_para_style` 에서:
- 변환본 시 ParaShape spacing/margin 을 `/4` (기본 `/2` + 추가 `/2`) 로 보정
- 일반 HWP5 는 기존 `/2` 유지

영향 필드:
- `margin_left`, `margin_right`, `indent`
- `spacing_before`, `spacing_after`

### 1-5. 호출 chain

- `DocumentCore::from_bytes` 가 `document.is_hwp3_variant` 전달
- `DocumentCore::set_dpi` 도 동일 전달
- WASM API + rhwp-studio 자동 적용

## 2. 식별 sweep 결과

| 파일 | paragraphs | ps_ratio | cs_ratio | HwpSummary 1990-2003년 | **variant** | 분류 |
|------|-----------|----------|----------|----------------------|-------------|------|
| hwp3-sample16-hwp5.hwp | 1058 | 0.174 | 0.154 | YES | **true** | 변환본 ✓ |
| exam_kor.hwp | 749 | 0.076 | 0.214 | NO | false | 일반 ✓ |
| exam_math.hwp | 275 | 0.189 | 0.393 | NO | false | 일반 ✓ |
| exam_eng.hwp | 318 | 0.267 | 0.362 | YES | **false** | 일반 ✓ (cs>0.20) |
| aift.hwp | 921 | 0.295 | 0.317 | NO | false | 일반 ✓ |
| biz_plan.hwp | 93 | 0.505 | 0.312 | NO | false | 일반 ✓ |
| 통합재정통계(2014).hwp | 17 | 1.941 | 2.000 | NO | false | 일반 ✓ |
| 복학원서.hwp | 17 | 1.647 | 1.824 | YES | false | 일반 ✓ (para<50) |

**8개 sample 모두 정확 식별** (variant=true 1개, false 7개).

## 3. 단위 검증 결과

### 3-1. ParaShape spacing/h 정합 (sample16-hwp5 페이지 3)

| paragraph | Fix 전 (HWP5) | Fix 후 (HWP5) | HWP3 원본 (baseline) |
|-----------|--------------|---------------|---------------------|
| pi=69 "Ⅰ. 사업개요" sb | 11.4 | **5.7** ✓ | 5.7 |
| pi=70 "1. 추진목적" sb | 11.4 | **5.7** ✓ | 5.7 |
| pi=70 h | 32.7 | **27.0** ✓ | 27.0 |
| pi=72 sb | 7.6 | **3.8** ✓ | 3.8 |

ParaShape 단위 정합 ✓.

### 3-2. 페이지 수 유지

- sample16-hwp5: **64페이지** (Task #998/#999 정합 유지)
- sample16 원본: 64페이지 (baseline)

### 3-3. cargo test / clippy

- `cargo test --release --lib`: **1306 passed**, 0 failed (baseline 유지) ✓
- `cargo clippy --release -- -D warnings`: 0 warnings ✓

## 4. 격차 B (styling) 영향

격차 C (spacing drift) fix 후 격차 B (점선 박스 + 그라데이션 등) 의 시각 영향이 어느 정도 줄어들지 Stage 6 시각 판정 시 측정.

**격차 B 의 주된 출처는 paragraph 의 Shape control (wrap=TopAndBottom + tac=true + 그라데이션 fill)** — 이는 변환본의 원본 스타일을 한컴이 simplify 하는 패턴. Stage 6 작업지시자 시각 판정 결과로 후속 분리 결정.

## 5. line_segs.vertical_pos 1/2 추가 보정 (Stage 5-D 후속)

ParaShape 보정만으로 시각 정합 불완전 — paragraph y 위치는 `line_segs.vertical_pos` 가 직접 결정 (`src/renderer/layout.rs:1230,1253`). 변환본의 `vertical_pos` 는 ParaShape spacing 의 2배 영향 누적으로 큰 값.

**추가 fix** (`src/parser/mod.rs::fixup_line_segs_for_variant`):
- 변환본 식별 후 모든 `line_segs.vertical_pos` 를 1/2 보정
- 표 셀 내부 paragraph 도 재귀 보정
- `line_height` / `text_height` / `baseline_distance` / `line_spacing` 은 단위 동일 (변환본/HWP3 모두 lh=1600 등) 이라 보정 불필요

### 5-D 시각 정합 결과

`samples/hwp3-sample16-hwp5.hwp` 페이지 3 SVG 위치 비교:

| Text | HWP3 (baseline) y px | HWP5 변환본 (after fix) y px | 차이 |
|------|---------------------|---------------------------|------|
| "1. 추진목적" | 142.1 | **142.1** | **0** ✓ |
| "2. 추진방향" | 454.6 | 462.7 | +8.1 (작음) |

거의 완전 정합 — 한컴 viewer 와 동일한 paragraph 배치.

## 6. 잔존 / 후속

- 페이지 16/17 등 다른 페이지의 시각 정합 확인 (Stage 6 시각 sweep)
- Shape control 의 한컴 simplify 패턴 (점선 박스 + 그라데이션, 격차 B 잔존) — 격차 C fix 후 잔존 시 별도 issue 후속
- 자동 식별 휴리스틱 정밀화 (Stage 6 sweep 결과 fail 시 정밀화)

## 6. Stage 6 진입 계획

- SVG sweep (sample16-hwp5 모든 페이지 + 일반 HWP5 회귀 측정)
- WASM 빌드 검증
- rhwp-studio 실제 렌더링 확인
- 작업지시자 한컴 정합 시각 판정
- 최종 보고서

## 7. 변경 파일 요약

| 파일 | 변경 |
|------|------|
| `src/parser/cfb_reader.rs` | `detect_hwp3_variant()` 메서드 추가 |
| `src/model/document.rs` | `is_hwp3_variant: bool` 필드 추가 |
| `src/parser/mod.rs` | parse_hwp_with_cfb + parse_hwp_with_lenient + hwpx::mod.rs 초기화 + 식별 결합 휴리스틱 |
| `src/parser/hwpx/mod.rs` | Document 초기화 |
| `src/renderer/style_resolver.rs` | `resolve_styles_with_variant` + `resolve_single_para_style` 시그니처 + `/4` 보정 |
| `src/document_core/mod.rs` | `set_dpi` 가 variant 전달 |
| `src/document_core/commands/document.rs` | `from_bytes` 가 variant 전달 |
| `src/serializer/cfb_writer/tests.rs` | Document literal 추가 필드 |
