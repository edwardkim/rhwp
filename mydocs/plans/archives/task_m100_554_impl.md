# 구현계획서 — Task #554

> **이슈**: [#554](https://github.com/edwardkim/rhwp/issues/554)
> **수행계획서**: `mydocs/plans/task_m100_554.md`
> **Stage 1 보고서**: `mydocs/working/task_m100_554_stage1.md`
> **방안**: C — 한컴 변환본 식별 휴리스틱 + 조건부 `-1600 HU` 보정
> **작성일**: 2026-05-03

---

## 1. 구현 개요

HWP3 → HWP5/HWPX 변환본은 한글97의 "마지막 줄 tolerance" 동작 (`-1600 HU`) 이 누락되어 페이지 수가 +1~+4 증가한다. 한컴 변환본을 robust하게 식별 후 조건부로 `-1600 HU` 보정을 적용한다.

**식별 휴리스틱** (Stage 1 검증):
- **HWPX**: `<hh:head version="1.4">` 체크
- **HWP5**: `(ParaShape/Paragraph < 0.05) AND (CharShape/Paragraph < 0.15) AND (Paragraph > 50)` 체크

**적용 위치**:
- `src/parser/hwpx/section.rs:103` (HWPX page margin)
- `src/parser/body_text.rs:764` (HWP5 page margin)

## 2. 단계 (4 단계)

### Stage 2-1: HWPX 휴리스틱 + 조건부 보정 구현

**대상 파일**:
- `src/parser/hwpx/header.rs` (또는 header 파싱 위치) — hwpml 버전 추출
- `src/parser/hwpx/mod.rs` 또는 section.rs — `is_hwp3_origin` 플래그 전달
- `src/parser/hwpx/section.rs:103` — 조건부 `-1600` 적용

**구현**:
1. HWPX header 파싱 시 `<hh:head version="X.Y">` 의 X.Y 추출
2. `version == "1.4"` 이면 `is_hwp3_origin = true` 플래그 설정
3. PageDef 파싱 시 플래그가 true이면 `margin_bottom -= 1600`

**검증**:
- `hwp3-sample-hwpx.hwpx`: 17 → 16 ✓ (정답 16)
- `hwp3-sample5-hwpx.hwpx`: 68 → 64 ✓ (정답 64)
- `2025년 기부·답례품.hwpx`: 30 → 30 (회귀 0)
- `표-텍스트.hwpx`, `table-vpos-01.hwpx`, `tac-img-02.hwpx`: 변화 없음

**산출물**: `mydocs/working/task_m100_554_stage2.md`

### Stage 2-2: HWP5 휴리스틱 + 조건부 보정 구현

**대상 파일**:
- `src/parser/mod.rs` (또는 HWP5 파싱 진입점) — 휴리스틱 계산
- `src/parser/body_text.rs:764` — 조건부 `-1600` 적용

**구현**:
1. HWP5 파싱 후 (DocInfo + BodyText 파싱 완료 후) 다음 계산:
   - `total_paragraphs = sections.iter().map(|s| s.paragraphs.len()).sum()`
   - `ps_ratio = doc_info.para_shapes.len() as f64 / total_paragraphs as f64`
   - `cs_ratio = doc_info.char_shapes.len() as f64 / total_paragraphs as f64`
   - `is_hwp3_origin = ps_ratio < 0.05 && cs_ratio < 0.15 && total_paragraphs > 50`
2. `is_hwp3_origin == true` 이면 모든 SectionDef의 `page_def.margin_bottom -= 1600` (post-process)

**대안 구현** (더 깨끗): `parse_page_def`에 플래그를 받지 않고, 파싱 완료 후 별도 fixup 패스. 휴리스틱은 paragraph 수가 필요하므로 page_def 파싱 시점에는 알 수 없음 → **post-process 패턴** 채택.

**검증**:
- `hwp3-sample-hwp5.hwp`: 17 → 16 ✓
- `hwp3-sample4-hwp5.hwp`: 38 → 36 ✓
- `hwp3-sample5-hwp5.hwp`: 68 → 64 ✓
- 다른 24 fixture: 변화 없음 (회귀 0)

**산출물**: `mydocs/working/task_m100_554_stage3.md`

### Stage 2-3: 회귀 검증 + 신규 회귀 테스트

**대상 파일**:
- `tests/issue_554.rs` (신규)
- (필요 시) 기존 golden test 갱신

**구현**:
1. `tests/issue_554.rs` 작성:
   - HWP3 변환본 5개 fixture 페이지 수 정합 (16/36/64/16/64)
   - 직접 작성본 fixture 회귀 0 (exam_kor 20, exam_science 4 등)
2. 전체 회귀 검증:
   - `cargo test --lib` 1113+ passed
   - `cargo clippy --lib -- -D warnings` 0건
   - `cargo test --test svg_snapshot` 6/6
   - `cargo test --test issue_546` 통과 (Task #546 정합 유지)
   - `cargo test --test issue_418/501/505/530` 통과
3. 광범위 fixture sweep (시간 허용 시):
   - `samples/*.hwp`, `samples/*.hwpx` 모두 페이지 수 측정
   - baseline (방안 C 적용 전) vs 적용 후 차이 0 확인 (HWP3 변환본 외)

**산출물**: `mydocs/working/task_m100_554_stage4.md`

### Stage 2-4: 정리 + 최종 보고서

**대상 파일**:
- `src/bin/check_compat.rs` 삭제 또는 정식 명령(`dump-records`)으로 통합
- `mydocs/orders/20260503.md` 갱신 (Task #554 완료 표시)
- `mydocs/report/task_m100_554_report.md` 작성

**구현**:
1. `src/bin/check_compat.rs` 처리 결정 (작업지시자 승인):
   - 옵션 A: 삭제 (임시 도구)
   - 옵션 B: 영구 도구로 정리 (예: `rhwp dump-info` 명령에 hwp3-origin 식별 필드 추가)
2. `mydocs/orders/20260503.md`에서 Task #554 항목 "완료" 표시 + Stage 결과 요약
3. `mydocs/report/task_m100_554_report.md` 최종 보고서:
   - 본질, 해결 접근, 검증 결과
   - 잔존 사항 (hwp3-sample4 HWP3 자체 회귀, sample 변환본 -1 over-correct)
   - 향후 후속 task 권고

**산출물**: `mydocs/report/task_m100_554_report.md`

## 3. 위험 관리

### 3.1 회귀 위험

- **광범위 회귀 검출**: Stage 2-3에서 모든 HWP5/HWPX fixture 페이지 수 baseline 비교
- **issue_546 회귀**: exam_science.hwp 4페이지 정합 유지 필수
- **휴리스틱 false positive**: paragraph 수가 매우 적은 (< 50) 단순 문서는 휴리스틱 미적용 (가드)

### 3.2 잔존 결함 (의도된 trade-off)

- `hwp3-sample.hwp` 변환본의 -1 over-correct (16 → 15)
   - 단일 -1600 값으로 sample/sample5 모두 정합 어려움
   - 향후 보정 정밀화 (예: paragraph 수 기반 가변 tolerance) 별도 task로 분리
- `hwp3-sample4.hwp` HWP3 자체 회귀 (39 vs 정답 36)
   - Task #554 범위 밖, 별도 issue 등록

### 3.3 PR #553 의존성

- 변환본 fixture 5개 (`hwp3-sample-hwp5.hwp` 등) 가 PR #553에 포함됨
- task554 브랜치는 fixture를 임시 추가하여 검증 가능
- task554 PR 시 PR #553 머지 후 의존성 정리 (또는 task554 PR이 PR #553을 머지 base로 사용)

## 4. 작업 일정

| Stage | 예상 시간 | 산출물 |
|-------|-----------|--------|
| 2-1 (HWPX 구현) | 1시간 | stage2.md |
| 2-2 (HWP5 구현) | 1.5시간 | stage3.md |
| 2-3 (검증) | 1시간 | stage4.md |
| 2-4 (정리/보고) | 30분 | report.md |

총 예상: 4시간

## 5. 작업지시자 승인 요청

본 구현계획서 검토 후:

1. **단계 분할 승인** (4 단계: HWPX 구현 → HWP5 구현 → 검증 → 정리)
2. **방안 C 구체적 구현 방식 승인**:
   - HWPX: hwpml 버전 파싱 + 조건부 보정 (parse 시점)
   - HWP5: 파싱 완료 후 post-process 휴리스틱 (paragraph 수 필요)
3. **임시 도구 처리 방향**: Stage 2-4에서 삭제 vs 영구 도구 통합 결정
4. **Stage 2-1 (HWPX 구현) 진행 승인**

승인 후 Stage 2-1로 진행한다.
