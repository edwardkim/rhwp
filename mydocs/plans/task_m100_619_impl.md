# 구현계획서 — Task #619

다단 paragraph 내 vpos-reset 미처리 — TypesetEngine partial-split 에서 `line.vertical_pos == 0` 무시

- 마일스톤: M100 (v1.0.0)
- 브랜치: `local/task619`
- 이슈: https://github.com/edwardkim/rhwp/issues/619
- 수행계획서: `mydocs/plans/task_m100_619.md`

## 1. 단계 구성 (4 단계)

### Stage 1 — vpos-reset forced break 로직 추가 (다단 한정)

**대상**: `src/renderer/typeset.rs::typeset_paragraph` 의 partial-split 루프 (line 1057–1132).

**수정 내용**:
1. 줄 단위 분할 루프 (`while cursor_line < line_count`) 의 inner fit 루프 (`for li in cursor_line..line_count`) 에 vpos-reset 검출 추가.
2. 적용 조건:
   - `st.col_count > 1` (다단 섹션 한정)
   - `li > cursor_line` (현 세그먼트 첫 줄 제외)
   - `para.line_segs.get(li).map(|s| s.vertical_pos == 0).unwrap_or(false)` (vpos-reset 신호)
3. 검출 시: `break` 로 inner 루프 종료 → `end_line = li` 로 forced split.
4. 외부 루프에서 `cursor_line = end_line` 으로 다음 iteration 진입 → `advance_column_or_new_page()` 후 vpos-reset line 부터 재배치.

**수정 위치 의사코드**:
```rust
for li in cursor_line..line_count {
    // [Task #619] 다단 paragraph 내 vpos-reset 강제 분리.
    // line_segs[li].vertical_pos == 0 (li>0) 은 HWP 가 해당 line 을
    // 다음 단/페이지 최상단에 배치하도록 인코딩한 신호. 다단 한정 적용
    // (단일 단은 partial-table split 회귀 차단 위해 미적용).
    if st.col_count > 1
        && li > cursor_line
        && para.line_segs.get(li).map(|s| s.vertical_pos == 0).unwrap_or(false)
    {
        break;
    }
    let content_h = fmt.line_heights[li];
    if cumulative + content_h > avail_for_lines && li > cursor_line {
        break;
    }
    cumulative += fmt.line_advance(li);
    end_line = li + 1;
}
```

**커밋 메시지**: `Task #619: TypesetEngine partial-split 에 다단 vpos-reset forced break 추가`

**완료 조건**:
- `cargo build --release` 성공.
- 코드 변경에 주석으로 의도 명시 (Task #619 참조).

### Stage 2 — 대상 파일 수정 검증

**검증 항목**:
1. `rhwp dump-pages "samples/21_언어_기출_편집가능본.hwp" -p 7` 결과:
   - 단 1: `PartialParagraph pi=181 lines=0..8` (변경 전: `0..9`)
2. `rhwp dump-pages "samples/21_언어_기출_편집가능본.hwp" -p 8` 결과:
   - 단 0: `PartialParagraph pi=181 lines=8..13` (변경 전: `9..13`)
3. `rhwp export-svg "samples/21_언어_기출_편집가능본.hwp" -p 7 -o output/svg/p21_after/`:
   - `LAYOUT_OVERFLOW_DRAW: pi=181 line=8` 미발생.
4. `rhwp export-svg "samples/21_언어_기출_편집가능본.hwp" -p 8 -o output/svg/p21_after/`:
   - 페이지 9 단 0 첫 줄 = pi=181 line 8 의 텍스트 ("거나, 이러한 능력을…").

**한컴 PDF 비교**:
5. `samples/21_언어_기출_편집가능본-2010.pdf` 페이지 8/9 vs SVG: 라인 분포 일치.
6. `samples/21_언어_기출_편집가능본-2020.pdf` 페이지 8/9 vs SVG: 라인 분포 일치.

**완료 조건**:
- 위 1–4 통과.
- 5–6 비교 결과 본문 (단계별 보고서) 에 기록.

### Stage 3 — 회귀 검증

**테스트 항목**:
1. `cargo test --release` 전체 통과.
2. `cargo clippy --all-targets --release -- -D warnings` 통과.
3. 기존 회귀 가드 샘플 SVG 비교 (변경 전/후):
   - `samples/exam_eng*.hwp` (8p 단 채움, Task #470 가드)
   - `samples/exam_kor*.hwp` (partial-table split, issue #418 가드)
   - `samples/kps-ai*.hwp` 또는 유사 다단 샘플 (Task #362 가드)
   - `samples/k-water-rfp*.hwp` p3, p15 (Task #391 / Task #361 가드)
   - 기타 다단 샘플 (samples/ 폴더의 다단 케이스 5–10 개 무작위 검증)
4. 검증 방식:
   - 각 샘플 전 페이지 SVG 생성 (변경 전 백업 → 변경 후 비교).
   - `LAYOUT_OVERFLOW_DRAW` 또는 `LAYOUT_OVERFLOW` 메시지 신규 발생 여부 확인.
   - 페이지 수, 페이지별 단 분포 (`dump-pages`) 변경 여부 비교.

**완료 조건**:
- 1, 2 통과.
- 3 의 회귀 가드 샘플에서 페이지 분포 변화 없음 (또는 변화가 본 변경 의도와 일치하는 합리적 변화임을 단계별 보고서에 명시).

### Stage 4 — 최종 보고서 + 머지

**작업 항목**:
1. 최종 결과 보고서 작성: `mydocs/report/task_m100_619_report.md`.
   - 변경 전/후 dump-pages 출력 비교
   - LAYOUT_OVERFLOW_DRAW 제거 증거
   - 한컴 PDF 비교 스크린샷/요약
   - 회귀 검증 결과 요약
2. `mydocs/orders/{yyyymmdd}.md` 갱신 (Task #619 완료 항목 추가).
3. 단계별 보고서 (`task_m100_619_stage{1,2,3}.md`) 와 최종 보고서를 task 브랜치에 커밋.
4. 작업지시자 승인 후 `local/devel` 머지 진행 (이슈 closes #619).

**완료 조건**:
- 보고서 작성 + 승인.
- task 브랜치에 모든 변경 (소스 + 문서) 커밋 완료.
- `git status` clean 확인 후 머지 준비.

## 2. 변경 파일 목록 (예상)

| 파일 | 변경 사유 |
|------|----------|
| `src/renderer/typeset.rs` | partial-split 루프에 vpos-reset forced break 추가 |
| `mydocs/plans/task_m100_619.md` | 수행계획서 (작성 완료) |
| `mydocs/plans/task_m100_619_impl.md` | 구현계획서 (이 문서) |
| `mydocs/working/task_m100_619_stage1.md` | Stage 1 완료 보고서 |
| `mydocs/working/task_m100_619_stage2.md` | Stage 2 완료 보고서 |
| `mydocs/working/task_m100_619_stage3.md` | Stage 3 완료 보고서 |
| `mydocs/report/task_m100_619_report.md` | 최종 결과 보고서 |
| `mydocs/orders/{날짜}.md` | 오늘 할일 갱신 |

## 3. 위험 요소 및 완화

| 위험 | 영향 | 완화 |
|------|------|------|
| 다단에서 vpos-reset 가 의도와 다른 위치에서 발생하는 케이스 (예: 빈 줄, 표 분할 후 line_segs 잔재) | 페이지 분포 의도와 다르게 변경 | 다단 한정 적용 + Stage 3 회귀 검증으로 차단 |
| Inline shape (pi=181 ci=0 마흐디) 가 forced break 로 잘못된 페이지로 이동 | shape 페이지 누락 / 중복 | Stage 2 SVG 검증 시 shape 위치 확인 |
| 표 분할 + vpos-reset 동시 케이스 | partial-table split 회귀 | 표 분할 경로 (`fn typeset_paragraph` 의 다른 분기) 미수정 → 영향 없음. Stage 3 exam_kor 가드로 검증 |
| `next_will_vpos_reset` (문단 간) 와 충돌 | safety_margin 이중 동작 | 문단 *간* vpos-reset 처리 코드 미수정. 본 변경은 문단 *내부* 만. |

## 4. 승인 요청

본 구현계획서 검토 후 승인 부탁드립니다. 승인 후 Stage 1 부터 순차 진행합니다.
