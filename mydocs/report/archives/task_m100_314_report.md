# Task #314 최종 보고서: HWPX 어댑터 페이지 수 차이 보정 (부분 완료)

상위 Epic: 잔존 사안 (#309 외부)
브랜치: `task314`
커밋: `73bfc7c` (수행계획서), `45d1402` (1단계), `f87314e` (2단계), 본 보고서 커밋

## 결과 요약

**부분 완료**: HWPX → HWP 라운드트립의 일부 IR 차이를 해소(`char_shapes` empty default + `control_mask` 재계산)했으나, **페이지 수 +1쪽 잔존**. 격리된 3개 테스트는 #[ignore] 유지. 잔존 origin은 별도 sub-issue 후보.

## 단계별 결과

### 1단계: 차이 정량 분석 (커밋 `45d1402`)
- LINE_SEG 100% 보존 확인 (기존 가정 정확)
- 7가지 paragraph 필드 차이 식별 (`char_shapes_len` 59, `control_mask` 27, `raw_header_extra` 130 등)
- 페이지 3에서 +1쪽 차이 시작 (Table pi=51 처리 분기)

### 2단계: HWPX normalize 적용 (커밋 `f87314e`)
- `normalize_hwpx_paragraphs` 신규 함수
- char_shapes 빈 paragraph → `[(0, 0)]` default
- control_mask 재계산 (controls 기반)
- 셀 paragraph 재귀 처리

**효과**: char_shapes_len 59건 + control_mask 27건 차이 해소. 그러나 페이지 수 +1쪽 차이는 그대로.

### 3단계: 종료 처리 (본 커밋)
- 진단 테스트 (`tests/task314_diag.rs`) 제거
- normalize 코드는 보존 (IR 정합성 개선 가치)
- 격리 테스트 #[ignore] 유지

## 변경 파일

- `src/document_core/commands/document.rs` — `normalize_hwpx_paragraphs` 추가, HWPX 로드 시 호출

## 검증

- `cargo test`: 992 passed (lib), integration 모두 통과 (3 ignored = 격리 유지)
- 4샘플 (21_언어/exam_math/exam_kor/exam_eng) 페이지 수 무변화: 15/20/24/9
- normalize 자체는 회귀 0

## 잔존 사안

페이지 수 +1쪽 차이 origin 미식별. 잔존 후보:
- `char_count` 차이 3건 (paragraph 0.0, 0.34, 1.0 — 0.34는 +1쪽 발생 페이지와 일치)
- `raw_break_type` 4건
- `raw_header_extra` 130건 (전체)
- 표 cell paragraphs 의 다른 미식별 필드

**다음 sub-issue 후보**: `HWPX 어댑터 페이지 수 +1쪽 잔존 origin 조사`

작업 방향:
1. char_count 차이 paragraph 의 typeset/composer 처리 추적
2. 페이지 3 단 0 내부 paragraph 의 used_height 누적 차이 line-by-line 분석
3. 표 cell paragraphs 의 IR 차이 추가 식별
4. 직렬화/파싱 손실 trace

## 학습

1. **이론적으로 영향 없는 차이가 실제로 영향**. char_shapes / control_mask는 typeset 코드 검토상 미사용이지만, 정합성 측면에서 채워주는 게 안전. 다만 직접적인 +1쪽 origin은 아님.
2. **부분 진전도 가치 있음**. normalize 코드는 IR 정합성을 개선하여 향후 다른 회귀 방지.
3. **시간 박싱의 중요성**. origin이 더 깊은 곳에 있을 가능성 → 본 sub-issue 범위 내에서 미해결, 새 sub-issue로 분리.

## 종료 절차 (작업지시자 승인 후)

1. `gh issue close 314` (부분 완료 사유 명시)
2. 새 sub-issue 등록 (잔존 origin 조사)
3. `task314` → `devel` merge (작업지시자 직접)

## 참고

- `mydocs/plans/task_m100_314.md`, `task_m100_314_impl.md`
- `mydocs/working/task_m100_314_stage1.md` (LINE_SEG 비교)
- `mydocs/working/task_m100_314_stage2.md` (normalize 적용)
