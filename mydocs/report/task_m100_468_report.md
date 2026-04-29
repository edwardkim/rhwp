# Task #468 최종 결과보고서

## 제목

exam_kor 6p 좌측 단 [18~21] passage 박스 하단 줄 — cross-column 박스 연속 미감지

## 개요

`samples/exam_kor.hwp` 6p 좌측 단 [18~21] passage 박스의 하단 stroke 가 컬럼 경계 (col 0 → col 1 → page 7 col 0) 연속에도 불구하고 그려져 박스가 닫혀 보이는 현상. cross-column / cross-page 박스 연속을 검출하여 inner 경계 stroke 를 제거.

## 원인

기존 `paragraph_layout.rs:2527-2528` 의 partial 플래그 (`is_partial_start`, `is_partial_end`) 는 **단일 paragraph 내부 split** (PartialParagraph) 만 검출. 별개 paragraph 가 sequential 하게 같은 bf_id 를 가지며 컬럼/페이지 경계로 분리된 케이스 미감지.

## 수정 내용

### 1. `para_border_ranges` 튜플 확장 (9 → 10 필드)

`para_index` 추가 — merge 후에도 paragraph 인덱스 추적 가능.

- `src/renderer/layout.rs:217-220` (선언)
- `src/renderer/layout/paragraph_layout.rs:2580-2581` (push)

### 2. merge 그룹 구조 확장

groups 튜플에 `first_para_idx`, `last_para_idx` 필드 추가. merge 시 마지막 그룹의 `last_para_idx` 갱신.

- `src/renderer/layout.rs:1633-1672` (build groups + merge)

### 3. Cross-column partial 보정 로직 추가

merge 완료 후 각 그룹의 첫/마지막 paragraph 의 sequential 인접 bf_id 를 검사:

- `composed[first_pi - 1]` 의 bf_id == 그룹 bf_id → partial_start
- `composed[last_pi + 1]` 의 bf_id == 그룹 bf_id → partial_end

bf_id 0 (border 없음) 그룹은 skip.

- `src/renderer/layout.rs:1674-1700`

## 검증 결과

### 시각 검증 (exam_kor.hwp)

| 위치 | Top | Right | Bottom | Left | 결과 |
|------|-----|-------|--------|------|------|
| 6p col 0 [18~21] 박스 | ✓ | ✓ | **✗ (목표)** | ✓ | 하단 미렌더 ✓ |
| 6p col 1 박스 (continuation) | ✗ | ✓ | ✗ | ✓ | 양방향 연속 ✓ |
| 7p col 0 박스 (continuation end) | ✗ | ✓ | ✓ | ✓ | 박스 닫힘 ✓ |
| 14p 단일 박스 | ✓ | ✓ | ✓ | ✓ | 회귀 0 ✓ |

### 테스트

- `cargo test --release --lib` : 1069 passed, 0 failed, 1 ignored
- `cargo test --release --test svg_snapshot` : 6 passed, 0 failed

### 회귀 (5종 샘플)

| 샘플 | 페이지 수 | 결과 |
|------|----------|------|
| 2010-01-06.hwp | 6 | ✓ |
| biz_plan.hwp | 6 | ✓ |
| 21_언어_기출_편집가능본.hwp | 15 | ✓ |
| exam_eng.hwp | 8 | ✓ |
| 2022년 국립국어원 업무계획.hwp | 40 | ✓ |

## 단계 진행

| 단계 | 내용 | 상태 |
|------|------|------|
| Stage 1 | 수행계획서 + 구현계획서 | ✅ |
| Stage 2 | cross-column partial 보정 구현 | ✅ |
| Stage 3 | 시각·테스트·회귀 검증 | ✅ |
| Stage 4 | 최종 보고서 + merge | ✅ |

## 변경 파일

- `src/renderer/layout.rs` (+44 lines)
- `src/renderer/layout/paragraph_layout.rs` (+1, -1)
- `mydocs/plans/task_m100_468.md` (수행계획서)
- `mydocs/plans/task_m100_468_impl.md` (구현계획서)
- `mydocs/working/task_m100_468_stage3.md` (Stage 3 보고서)
- `mydocs/report/task_m100_468_report.md` (최종 보고서)

## 참조

- GitHub Issue: [#468](https://github.com/edwardkim/rhwp/issues/468)
- 관련 Task: #463 Stage 5 (wrap inner edge partial 플래그 도입)
- 샘플: `samples/exam_kor.hwp` (6p, 7p, 14p)
