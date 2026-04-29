# Task #445 구현 계획서

**이슈**: [#445](https://github.com/edwardkim/rhwp/issues/445)
**브랜치**: `local/task445`
**수행계획서**: [`task_m100_445.md`](./task_m100_445.md)
**작성일**: 2026-04-29

---

## 단계 구성 (4 단계)

### Stage 1 — 원인 확정 (진단)

**목표**: 1차 가설(A/B/C) 중 정확한 원인 식별

**작업**:
1. `paragraph_layout.rs:2535` `para_border_ranges.push` 호출 직전에 임시 `eprintln!` 추가하여, 페이지 8 col 0 의 모든 push 호출에 대해 `bf_id, y_start, y_end, start_line, end_line, composed.lines.len()` 출력
2. `layout.rs:1614` 의 merge 로직 직전에 모든 ranges 와 merge 결과 출력
3. `expand_clip` 호출 전후의 `clip` 값을 layout.rs:442 부근에서 출력
4. `cargo run -- export-svg samples/exam_kor.hwp 2> /tmp/diag.log` 실행 후 page 8 분석
5. 결과를 바탕으로 원인 (A/B/C) 확정

**산출물**: `mydocs/working/task_m100_445_stage1.md` (원인 확정 + 데이터 첨부)

**완료 조건**:
- 어느 paragraph(또는 어떤 merge 결과) 의 y_end 가 비정상인지 명확히 식별
- 비정상 y_end 가 어떤 계산 경로로 산출되는지 코드 라인 단위로 추적

---

### Stage 2 — 구현

**목표**: Stage 1 에서 확정된 원인을 최소 침습으로 수정

**예상 수정 위치 (가설 의존)**:
- (A) PartialParagraph bbox 계산: `paragraph_layout.rs:2526` 의 `y` 값 산출 경로
- (B) border 그리기 루프: `paragraph_layout.rs:2535` push 시점의 y 값
- (C) expand_clip 정책: `layout.rs:442-474`

**작업**:
1. Stage 1 결과를 바탕으로 수정 코드 작성
2. Stage 1 의 임시 eprintln 제거
3. 단위 테스트(있을 시) 갱신
4. `cargo build --release` 통과
5. `cargo test` 통과

**산출물**: `mydocs/working/task_m100_445_stage2.md` (수정 내용 + 테스트 결과)

**완료 조건**:
- 빌드/테스트 통과
- exam_kor.hwp page 8 의 vertical line y_end 가 col_bottom 이하로 줄어듦 (육안 확인)

---

### Stage 3 — 검증

**목표**: 회귀 없이 모든 영향 페이지가 정상화되었는지 확인

**작업**:
1. `./target/release/rhwp export-svg samples/exam_kor.hwp` 재실행
2. Page 2, 5, 8, 15 의 세로선 길이 측정 → col_bottom 이하 확인
3. body-clip 의 y/h 가 페이지 영역 내로 들어왔는지 확인
4. 다른 회귀 후보 샘플 검증:
   - `samples/exam_eng.hwp`, `samples/exam_math.hwp`, `samples/exam_science.hwp`, `samples/exam_social.hwp` (구조 유사)
   - `samples/k-water-rfp*` (장문 표 + 페이지 분할 케이스)
   - HWPX 가 있는 샘플 → `ir-diff` 로 확인
5. 표 외곽 테두리 회귀 케이스: 표만 있는 샘플의 border 확인
6. PDF 비교 (스크린샷 비교 또는 길이 수치 비교)

**산출물**: `mydocs/working/task_m100_445_stage3.md` (검증 결과 + 비교표)

**완료 조건**:
- exam_kor 4페이지 모두 정상화
- 회귀 후보에서 새로운 비정상 사항 없음
- 기존 표 테두리 정상 유지

---

### Stage 4 — 최종 보고

**목표**: 결과 정리 및 문서화

**작업**:
1. 최종 보고서 작성 (`mydocs/report/task_m100_445_report.md`)
2. 오늘할일(`mydocs/orders/20260429.md`) 갱신 — Task #445 항목 완료 표시
3. 단계별 보고서 + 최종 보고서 + orders 갱신을 task 브랜치에 커밋
4. merge 전 `git status` 로 미커밋 파일 없음 확인

**산출물**: `mydocs/report/task_m100_445_report.md`

**완료 조건**:
- 작업지시자 승인 후 `local/devel` merge

---

## 검증 항목 체크리스트

- [ ] page 2: 세로선 y_end ≤ 1423 (col_bottom)
- [ ] page 5: 세로선 y_end ≤ 1423
- [ ] page 8: 세로선 y_end ≤ 1423 (현재 1671 → 약 248px 단축)
- [ ] page 15: 세로선 y_end ≤ 1423
- [ ] body-clip h 가 페이지 높이(1587) 를 넘지 않음
- [ ] 페이지 1 (표 닫힘) 정상 유지
- [ ] `cargo test` 통과
- [ ] 다른 샘플에서 회귀 없음

---

자동 승인으로 Stage 1 진단을 시작합니다.
