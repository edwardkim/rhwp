# Task #311 최종 보고서: 페이지네이션에서 LINE_SEG vpos-reset을 단/페이지 경계로 강제

상위 Epic: #309
브랜치: `task311`
커밋: `a047cdc` (1단계), `a6f710e` (2단계), 본 보고서 커밋 (3단계)

## 결과 요약

**가설 부정.** Task #310 분석에서 권장한 "vpos-reset 강제 분리"를 실험 구현하고 4개 샘플로 검증한 결과 21_언어 페이지 수가 19→20으로 1쪽 증가. **가설은 부분적이며 별개의 column 가용 공간 정확도 문제와 결합되어야 효과 발생**.

산출물은 부정적 결론이지만 다음 의의가 있음:
1. Task #310 가설을 실측으로 검증/기각 → 다음 작업 방향 명확화
2. `--respect-vpos-reset` 실험 플래그 + `paginate_with_forced_breaks` 인프라를 코드베이스에 보존 → column 정확도 작업 후 결합 검증 가능
3. `PaginationOpts` 구조체 도입(1단계)은 향후 옵션 추가 시 재사용 가능한 리팩토링

## 단계별 결과

### 1단계: PaginationOpts 구조체 도입 (커밋 `a047cdc`)

`paginate_with_measured_opts(..., hide_empty_line: bool)` → `(..., opts: PaginationOpts)` 마이그레이션. 회귀 0 리팩토링.

- `cargo test`: 992 passed
- 4개 샘플 페이지 수 무변화

### 2단계: vpos-reset 강제 분리 + 실험 플래그 (커밋 `a6f710e`)

- `paginate_with_forced_breaks` 메서드 신설 (LINE_SEG vpos-reset 위치에서 PartialParagraph 강제 분리)
- `--respect-vpos-reset` CLI 플래그 (export-svg/dump-pages)
- `set_respect_vpos_reset` 셋터 (변경 시 즉시 재페이지네이션)

**검증 결과**:

| 샘플 | OFF | ON | 평가 |
|------|-----|----|----|
| 21_언어 | 19 | **20** | ❌ +1 (가설 부정) |
| exam_math | 20 | 20 | 무변화 |
| exam_kor | 25 | 25 | 무변화 |
| exam_eng | 11 | 11 | 무변화 |

### 3단계: 보고서 + 가설 검증 결과 문서화 (본 커밋)

- `mydocs/tech/line_seg_vpos_analysis.md` 부록 A 추가 (가설 검증 결과)
- 본 최종 보고서 작성
- Epic #309 코멘트로 결과 게시 + 다음 sub-issue 후보 제안

## 진짜 원인 (재추정)

우리 엔진의 column 가용 공간 계산이 HWP의 실제 사용 공간보다 관대.

페이지 7 21_언어 사례:
- HWP: `pi=117 line 0` 까지만 단에 넣음 (단 사용 공간 ≈ vpos=89700 HWPUNIT ≈ 1196px)
- 우리: `pi=117 전체` 채움 (단 가용 공간 1226.4px 로 약 30px 더 큼)
- 차이의 후보 원인: trailing line_spacing 처리, spacing-after 누적, 줄간격 보정 등

이 차이를 좁히면 우리 엔진이 자연 흐름 페이지네이션으로도 HWP 의도 위치에서 단을 끊게 되어 vpos-reset 강제 분리가 자연스럽게 일치할 가능성.

## 권장 후속 작업

### Epic #309 — Sub-issue #N (제안)

**제목**: `column 가용 공간 계산 정확도 조사 (21_언어 +4쪽 진짜 원인)`

**작업**:
1. 21_언어 페이지 7 단 0의 우리 엔진 vs HWP 단 사용 공간 측정
2. 차이의 origin 식별:
   - trailing line_spacing 처리 시 `available_now` 계산 (engine.rs:602~620)
   - line_height 누적의 spacing 분리 모델
   - spacing-after 마지막 문단 처리
3. 보정 후 4샘플 검증:
   - 21_언어: 19쪽 → ?쪽 (단독)
   - 21_언어 + `--respect-vpos-reset`: 19쪽 → **15쪽** (PDF 일치 목표)
4. 회귀 검증: exam_math/exam_kor/exam_eng 무변화

**도구**: 본 #311의 실험 플래그(`--respect-vpos-reset`) 활용. column 정확도 개선 + vpos-reset 결합 시 PDF 일치 가능성을 실측으로 빠르게 검증 가능.

### 본 sub-issue #311 클로즈 절차

작업지시자 승인 시:
1. `gh issue close 311` (가설 검증 부정 결과 사유 명시)
2. Epic #309 에 결과 코멘트 + 다음 sub-issue 제안
3. 차후 Epic 후속 작업에서 본 #311 의 실험 플래그 활용

## 회귀 검증

- `cargo test`: 992 passed, 0 failed (1·2·3단계 모두)
- 옵션 OFF (기본): 4개 샘플 페이지 수 무변화
- 옵션 ON: 21_언어만 +1 (의도된 실험), 다른 3샘플 무변화

## 비범위 확인

- column 가용 공간 정확도 개선 (다음 sub-issue)
- vpos 우선 모드 전면 재설계 (분석 결과 불필요)

## 학습

1. **가설은 데이터로 검증되어야 한다**. Task #310의 분석은 vpos-reset 패턴 통계까지는 정확했으나 "이를 강제하면 페이지 수 감소"라는 인과 가설은 다른 변수(column 정확도)를 간과했음.
2. **실험 플래그 + 부정적 결과**도 의미 있는 산출물. 다음 작업의 방향을 명확히 하고 결합 검증 도구를 남김.
3. **분리만으로는 column이 압축되지 않는다**. 페이지네이션 엔진은 흐름 기반이므로 분리 신호와 압축 신호(column 정확도)가 동시에 충족되어야 의도된 결과.
