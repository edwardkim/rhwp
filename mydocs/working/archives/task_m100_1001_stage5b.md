# Task #1001 Stage 5-B — 변환본 식별 신호 + 영향 필드 카탈로그

이슈: [#1001](https://github.com/edwardkim/rhwp/issues/1001)
Stage 5-A: [`task_m100_1001_stage5a.md`](task_m100_1001_stage5a.md)

## 1. 변환본 식별 신호 후보

### 1-1. HwpSummaryInformation 비교

| 파일 | 작성일 | Version 필드 | 작성자 |
|------|--------|------------|--------|
| `hwp3-sample16-hwp5.hwp` (변환본) | **1998년** | 13, 0, 0, 3457 | Kowaco |
| `exam_kor.hwp` (일반 HWP5) | 2021년 | 12, 0, 0, 4204 | User |
| `biz_plan.hwp` (일반 HWP5) | (sweep 필요) | (sweep 필요) | (sweep 필요) |
| `aift.hwp` (일반 HWP5) | (sweep 필요) | (sweep 필요) | (sweep 필요) |

### 1-2. 식별 신호 카탈로그

| 신호 | 신뢰도 | 자동 검출 | 비고 |
|------|--------|----------|------|
| Summary creation_date 가 HWP3 시대 (~2003 이전) | 보통 | 가능 | 오래된 HWP5 misid risk |
| Summary 의 Version 필드 (예: major=13) | 미확인 | 가능 | 일반 HWP5 sweep 필요 |
| 파일명 패턴 `*-hwp5.hwp` | 매우 높음 | 가능 | 파일명 규약 의존 |
| 짝 .hwpx 파일 존재 + `hp:switch HwpUnitChar` | 매우 높음 | 가능 | 같은 디렉토리 의존 |
| ParaShape spacing 통계 (모두 2배 패턴) | 보통 | 가능하나 복잡 | HWP3 원본 비교 필요 |
| **사용자 명시 옵트인 (환경변수/CLI 옵션)** | **100%** | N/A | **사용자 인지 기반, production 안전** |

## 2. 권고 식별 전략

**Stage 5-C 구현 전략**:

1. **Primary**: 환경변수 `RHWP_HWP5_CONVERT_VARIANT_HALF=1` 명시 옵트인 (사용자 인지)
2. **Secondary (선택)**: 자동 휴리스틱 (Summary creation_date + paired .hwpx hp:switch) — Stage 6 이후 후속

**이유**:
- 자동 식별이 위험 (일반 HWP5 misid 시 회귀)
- 사용자가 변환본임을 명확히 인지 (파일명 등) → 명시 활성화가 가장 안전
- 환경변수는 rhwp-studio 에서 노출하여 사용자 선택 가능
- 자동 식별은 추후 휴리스틱 통계 + sweep 검증 후 안전성 확인 후 도입

## 3. 영향 필드 카탈로그 (1/2 보정 대상)

**ParaShape (Primary)**:
- `spacing_before` (HWPUNIT) — 본 격차의 주요 원인
- `spacing_after` (HWPUNIT)
- `margin_left` (HWPUNIT)
- `margin_right` (HWPUNIT)
- `indent` (HWPUNIT)
- `line_spacing` (단, type=FIXED/MINIMUM 만 — PERCENT 는 보정 제외)

**ParaShape.border_spacing** (HWPUNIT16):
- left, right, top, bottom
- 확인 필요 — 시각 검증

**TabDef.tabs.position** (HWPUNIT):
- 탭 위치 — 확인 필요

**ParaShape.tabs 가 ID 참조라면 TabDef 단위는 별도 처리**

**불확실 (Stage 5-D 단위 검증 필요)**:
- ColumnDef margin / spacing
- PageDef margin
- BorderFill spacing
- Shape position / size

## 4. Stage 5-C 구현 계획

### 4-1. 환경변수 + 파서 보정

`src/parser/body_text.rs::parse_para_shape` 등에서:
```rust
let half_scale = std::env::var("RHWP_HWP5_CONVERT_VARIANT_HALF").is_ok();
let scale = if half_scale { 2 } else { 1 };
ps.spacing_before = raw_value / scale;
// ... 다른 필드 동일 ...
```

또는 더 깔끔하게:
- 파싱 후 ParaShape struct 전체에 일괄 1/2 보정 함수 호출
- 변환본 식별 시 적용

### 4-2. 적용 단위

**일관 적용**: ParaShape 의 모든 spacing/margin 필드를 1/2 보정 (Primary)

**확장**: 단위 검증 후 다른 필드 (TabDef, BorderFill, ColumnDef) 추가 적용

### 4-3. 단위 검증 시나리오

- `RHWP_HWP5_CONVERT_VARIANT_HALF=1 rhwp export-svg samples/hwp3-sample16-hwp5.hwp -p 2` 시각 비교
- 사업개요 페이지의 paragraph spacing 이 한컴 정합 — 1차 확인
- 다른 페이지 (16, 17 등) 도 정합 확인

### 4-4. 회귀 차단

- 환경변수 미설정 시 동작 변경 없음 (기존 baseline)
- 일반 HWP5 sample 들은 환경변수 영향 없음

## 5. Stage 5-C 작업 단위

1. `src/parser/body_text.rs::parse_para_shape` 에 환경변수 보정 추가
2. 필요 시 `src/parser/hwpx/header.rs` 의 HwpUnitChar 처리와 일관성 유지
3. 단위 검증 (Stage 5-D)
4. 작업지시자 시각 판정
5. 추가 필드 확장 (필요 시)

## 6. 잔존 분리

자동 식별 (Summary creation_date / paired .hwpx 등) 는 본 Task 에서는 환경변수 옵트인으로 시작. 자동 식별 도입은:
- Stage 6 의 시각 판정 통과 후
- 휴리스틱 sweep 검증 (일반 HWP5 회귀 0 확인) 후
- 후속 task 또는 본 Task 의 v3 확장

## 7. Stage 5-C 진입 결정

권고 전략 확정 시 Stage 5-C 진입:
1. 환경변수 `RHWP_HWP5_CONVERT_VARIANT_HALF` 추가
2. ParaShape spacing/margin 1/2 보정
3. 단위 검증 (sample16-hwp5 페이지 1-5 시각 정합)
