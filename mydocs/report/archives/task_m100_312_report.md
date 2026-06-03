# Task #312 최종 보고서: column 가용 공간 정확도 조사

상위 Epic: #309
브랜치: `task312`
커밋: `923a4dd` (1단계), `b40b368` (2단계), 본 보고서 커밋 (3단계)

## 결과 요약

당초 가설(단일 column 가용 공간 origin 식별 + 보정)은 데이터로 부정. 페이지/단마다 diff 부호와 크기가 들쭉날쭉하여 단일 보정 불가.

**대신 의외의 발견**: 코드베이스에 이미 존재하는 `TypesetEngine` 이 검증 모드로 동작 중이며, 4샘플 모두에서 Paginator 보다 더 정확한 (PDF에 가까운) 페이지 수를 산출함.

| 샘플 | Paginator | TypesetEngine | PDF | 평가 |
|------|-----------|---------------|-----|------|
| 21_언어 | 19 | **15** | 15 | ✅ TypesetEngine = PDF 정확 일치 |
| exam_math | 20 | 20 | 20 | ✅ 모두 일치 |
| exam_kor | 25 | 24 | (미보유) | typeset 1쪽 적음 |
| exam_eng | 11 | 9 | (미보유) | typeset 2쪽 적음 |

## 단계별 결과

### 1단계: 단별 used_height 측정 도구 (커밋 `923a4dd`)
- `ColumnContent.used_height` 필드 + `dump-pages` 단 헤더 출력 (`used`, `hwp_used`, `diff`)
- 도구 자체는 회귀 0, 향후 페이지네이션 디버깅에 재사용 가능

### 2단계: origin 조사 → TypesetEngine 발견 (커밋 `b40b368`)
- 페이지별·단별 diff 측정 결과: 단일 origin 모델로 설명 불가
- TypesetEngine 발견 (`src/renderer/typeset.rs` + `rendering.rs:837~` 검증 코드)

### 3단계: 결과 정리 + 새 sub-issue 제안 (본 커밋)

## 권장 후속 작업

### Epic #309 — 새 Sub-issue (제안)

**제목**: `TypesetEngine을 main pagination으로 전환 (Paginator 대체)`

**근거**:
- TypesetEngine 결과가 4샘플 모두에서 Paginator보다 정확
- 21_언어 PDF 정확 일치
- 단일 패스 설계로 누적 오차 본질 해결

**범위**:
1. TypesetEngine 호환성 검토 (Paginator API와의 차이)
2. DocumentCore / WASM API 진입점 변경
3. 4샘플 검증 + cargo test 회귀 확인
4. Paginator 코드 정리/제거 또는 보존 결정 (vpos-reset 실험 플래그 등 부속물)

**기대 효과**:
- 21_언어: 19쪽 → 15쪽 (Epic #309 핵심 목표 달성)
- exam_kor: 25쪽 → 24쪽
- exam_eng: 11쪽 → 9쪽

이 sub-issue 완료 시 Epic #309 클로즈 가능.

### 본 sub-issue #312 종료 절차

작업지시자 승인 시:
1. Epic #309 코멘트 게시 (TypesetEngine 발견 + 새 sub-issue 제안)
2. `gh issue close 312` (당초 범위는 의외의 발견으로 대체됨)
3. 새 sub-issue 등록

## 산출물

- `mydocs/plans/task_m100_312.md`, `task_m100_312_impl.md`
- `mydocs/working/task_m100_312_stage1.md`, `task_m100_312_stage2.md`
- `mydocs/report/task_m100_312_report.md` (본 문서)
- 코드: 단별 측정 도구 (1단계 산출, 보존 가치 있음)

## 회귀 검증

- `cargo test`: 992 passed, 0 failed (1단계 변경 후)
- 4샘플 페이지 수 무변화 (Paginator 기반)

## 학습

1. **광범위 조사 전 코드베이스 전체 탐색**의 가치. TypesetEngine은 이미 존재했고, dump-pages 실행 첫 줄에 매번 결과를 출력했으나 본 발견 전까지 인지 못 함.
2. **측정 도구 우선 접근**의 효용 입증. 1단계에서 만든 도구 덕에 2단계에서 단일 origin 가설을 빠르게 부정하고 다른 방향을 찾을 수 있었음.
3. **가설은 데이터로 검증되어야 한다**. #311의 vpos-reset 가설, #312의 column 정확도 가설 모두 실측으로 부정. 가설 부정 자체가 다음 방향을 명확히 하는 산출물.
