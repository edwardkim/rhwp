# Task #672 구현 계획서

## 개요

Issue #672 "TAC 표 비례 축소 시 셀 콘텐츠 클립 — `common.height` vs measured `row_heights` 불일치" 정정 구현. 옵션 B (작은 차이 비례 축소 면제) 기반 3 단계 진행.

## Stage 1: 본질 진단 + 다른 fixture 영향 평가

### 목표

본 환경의 fixture 중 TAC 표 비례 축소가 발동하는 모든 영역을 식별. 각 영역의 raw vs common 차이 분포를 측정하여, 어떤 임계값이 본 case 정정 + 다른 영역 회귀 0 양립하는지 결정.

### 진단 절차

1. **TAC 표 비례 축소 발동 영역 sweep**
   - 진단 도구 (`examples/inspect_task672.rs`) 작성 — 모든 fixture 의 모든 TAC 표에 대해 raw_table_height vs common.height 비교 출력
   - sweep 대상: 187 fixture 전체

2. **차이 분포 분석**
   - 발동 영역의 (raw - common) / common 비율 분포
   - 본 case (`samples/계획서.hwp`) 의 비율: ~1.3% (970/969 ≈ 1.013)
   - 다른 영역: 의도적으로 큰 비례 축소 (예: TAC 표 본문 압축)

3. **임계값 후보 평가**
   - 1.0px (현재 동작 유사)
   - 1px ~ common * 5% (옵션 B-1 후보)
   - common * 5% (사용자 의도 영역만 발동)

### 진단 산출물

`mydocs/working/task_m100_672_stage1.md` — 본질 진단 결과 + 임계값 결정 + Stage 2 정정 방향 제안.

### 승인 요청

Stage 1 진단 결과 + Stage 2 임계값 + 정정 방향 승인 요청.

## Stage 2: 본질 정정 — 옵션 B (작은 차이 비례 축소 면제)

### 목표

`height_measurer.rs:822-830` TAC 표 비례 축소 분기에 임계값 가드 추가. 작은 차이 (예: 1~5%) 는 측정값 우선, 큰 차이만 비례 축소 (사용자 의도 영역 보존).

### 정정 원칙

- **케이스 가드 명시**: 임계값 (Stage 1 에서 결정) 이상 차이만 비례 축소 발동
- **휴리스틱 금지**: 본질 룰 — 한컴 동작 분석 기반 임계값 (가능한 경우)
- **다른 영역 무영향**: 의도적으로 큰 비례 축소 영역 그대로 동작
- **회귀 위험 좁힘**: TAC 표 비례 축소 발동 영역 sweep 결과로 정정 영향 명시

### 구현 방안

`src/renderer/height_measurer.rs:822-830` 정정:

```rust
// [Task #672] 작은 차이 (~1-5%) 는 측정값 우선 — 셀 콘텐츠 측정값과 common.height
// 의 미세한 불일치 시 비례 축소가 셀 콘텐츠 클립을 발생.
// 사용자 의도 영역 (의도적으로 큰 비례 축소) 은 임계값 초과 시에만 발동.
let shrink_threshold = common_h * SHRINK_THRESHOLD_RATIO; // 또는 절대 px
let table_height = if table.common.treat_as_char
    && common_h > 0.0
    && raw_table_height > common_h + shrink_threshold {
    let scale = common_h / raw_table_height;
    for h in &mut row_heights {
        *h *= scale;
    }
    common_h
} else {
    raw_table_height
};
```

### 검증 절차

1. **결정적 검증**
   - cargo test --lib --release 회귀 0 (1155+ passed 유지)
   - svg_snapshot 6/6 + issue_546 1/1 + issue_554 12/12
   - cargo clippy --release 신규 경고 0
2. **시각 검증**
   - `samples/계획서.hwp` 셀 [21] 3줄 / [52] 3 paragraph 정상 표시
3. **진단 도구 재실행**
   - 비례 축소 발동 영역 변화 확인 (어떤 영역이 면제되는지)

### 정정 산출물

- 영향 코드 변경 (`src/renderer/height_measurer.rs`)
- `mydocs/working/task_m100_672_stage2.md` 단계별 보고서

### 승인 요청

Stage 2 정정 결과 + 검증 결과 보고 → Stage 3 광범위 회귀 sweep 진행 승인.

## Stage 3: 광범위 회귀 sweep + 최종 검증

### 목표

광범위 페이지네이션 회귀 sweep 으로 회귀 위험 영역 좁힘 입증 + 시각 판정 게이트웨이 통과.

### 검증 절차

1. **광범위 페이지네이션 회귀 sweep**
   - `samples/` 폴더 전체 187 fixture BEFORE/AFTER 페이지 수 차이 측정
   - 차이 0 보장
2. **결정적 검증 (release 모드)**
   - cargo test --lib --release 1155+ passed
   - cargo test --release 전체 GREEN
   - cargo clippy --release 신규 경고 0
3. **시각 판정 게이트웨이 (작업지시자)**
   - `samples/계획서.hwp` 1 페이지 시각 판정
   - 셀 [21] 3 줄 모두 표시 + 셀 [52] 3 paragraph 모두 표시 확인
   - 다른 셀 영역 회귀 0 확인
4. **TAC 표 fixture 시각 검증** (Stage 1 에서 식별된 영역)
   - 비례 축소 면제된 영역의 표시 변화 확인
   - 의도된 비례 축소 영역 (큰 차이) 그대로 동작 확인

### 산출물

- `mydocs/report/task_m100_672_report.md` 최종 보고서
- `mydocs/orders/20260507.md` Task #672 상태 갱신

### 승인 요청

최종 보고서 + 시각 판정 게이트웨이 통과 후 작업지시자 승인 → fork push + PR 생성 영역.

## 회귀 위험 영역 좁힘 원칙

- **수정 영역 명시**: `height_measurer.rs:822-830` 단일 분기
- **케이스 가드**: 임계값 이상 차이만 비례 축소 발동
- **광범위 sweep 검증**: 187 fixture 페이지 수 차이 0 보장
- **TAC 표 영역 영향 명시**: Stage 1 진단 결과로 정정 영향 영역 좁힘 입증

## 의존성

- **선행 의존**: Task #671 정정 코드 (`local/task671` 브랜치) — 본 task 는 그 위에서 분기
- **후행 의존**: 없음

## 최종 결과 영역

본 task 완료 후:
- Issue #672 close (closes #672 키워드)
- TAC 표 비례 축소 본질 영역 정합 (한컴 권위 영역과 정합화)
- 후속 task 영역 — 한컴 권위 동작 추가 분석 (가능 시)

## 옵션 B 선택 근거 재확인

수행계획서에서 권장한 옵션 B (작은 차이 비례 축소 면제):
- 회귀 위험 좁힘 + 본 case 정정 양립
- 사용자 의도 영역 (큰 비례 축소) 보존
- 단일 분기 정정으로 구현 단순

옵션 A (비례 축소 제거): TAC 표 본질 영역 변경, 회귀 위험 큼.
옵션 C (row_heights 최소값 보장): 추가 가드 복잡, 구현 영역 큼.
옵션 D (cell.clip 완화): 의도적 클립 영향, 회귀 위험 큼.

## 작업지시자 결정 영역

Stage 1 진단 결과에서 임계값 후보 정량 측정 후 작업지시자에게 결정 요청. 임계값은 본질 영역이므로 신중한 결정 필요.
