# Task #1152 구현 계획서 — TAC 표 intra-paragraph vpos-reset 가드

- 이슈: [#1152](https://github.com/edwardkim/rhwp/issues/1152)
- 브랜치: `local/task1152`
- 작성일: 2026-05-28

## 1. 변경 범위

### 코드
- `src/renderer/typeset.rs` `typeset_tac_table()` (line 2234~2279) 진입부에 intra-paragraph vpos-reset 가드 추가.

### 테스트
- `tests/issue_1152_intra_para_vpos_reset.rs` 신규.
  - 입력: `samples/2022년 국립국어원 업무계획.hwp`
  - 검증: page 32 에 pi=586 ci=1 (1×3 TAC 박스) 없음, page 33 첫 PageItem 으로 존재.

### 문서
- 본 계획서 + Stage 2~4 보고서 + 최종 결과보고서.

## 2. 구현 단계 (총 4 단계)

### Stage 2: 코드 패치 + 단위 검증

**작업**:
1. `src/renderer/typeset.rs:2234 typeset_tac_table()` 진입부에 아래 가드 추가:
   ```rust
   // [Task #1152] 호스트 문단의 intra-paragraph vpos-reset (ls[i].vpos == 0, i > 0)
   // 신호 — HWP 가 "이 TAC 표를 새 페이지 상단부터" 라고 명시한 케이스.
   // 기존 fit 검사는 표 크기가 잔여 영역에 들어가면 통과시키지만, HWP 의
   // 명시 분할 신호를 존중하려면 fit 검사 이전에 advance.
   //
   // 가드 조건: empty-text host paragraph + N controls + N line_segs 1:1 매핑
   // (다른 라인 구성에서는 보수적으로 미적용).
   //
   // 케이스: 2022년 국립국어원 업무계획.hwp pi=586 ci=1 (별첨 박스).
   if !st.current_items.is_empty()
       && ctrl_idx > 0
       && para.text.is_empty()
       && para.line_segs.len() == para.controls.len()
       && para
           .line_segs
           .get(ctrl_idx)
           .map(|s| s.vertical_pos)
           .unwrap_or(-1) == 0
   {
       st.advance_column_or_new_page();
   }
   ```
2. 빌드 (`cargo build --release`).
3. `rhwp dump-pages "samples/2022년 국립국어원 업무계획.hwp" -p 31` 로 페이지 32 → 1×3 박스 사라지는지 확인.
4. `-p 32` 로 페이지 33 첫 PageItem 으로 1×3 박스(또는 1×3 박스 → 빈 문단 → 본문 표 순) 확인.

**완료 조건**: page 32 items=1 (12×5 PartialTable만), page 33 의 1×3 박스가 빈 문단보다 앞에 위치 (한컴 PDF 와 정합).

### Stage 3: 회귀 테스트 + 폭넓은 검증

**작업**:
1. `cargo test --release` 전체 통과.
2. 인접 케이스 5건 (kps-ai, 2025 기부, 비공개 sample A) 페이지 수 변동 측정 — 0 변동 목표.
3. 변경 파일 `cargo fmt` + `cargo clippy --all-targets -- -D warnings`.
4. 신규 회귀 테스트 `tests/issue_1152_intra_para_vpos_reset.rs` 추가.

**완료 조건**: 전체 테스트 통과 + 인접 5 케이스 페이지 수 변동 0.

### Stage 4: 시각 검증 + 최종 검토

**작업**:
1. `rhwp export-svg samples/2022년 국립국어원 업무계획.hwp -p 31 --debug-overlay` 로 페이지 32 SVG 확인 (별첨 박스 없음).
2. `-p 32` 로 페이지 33 SVG 확인 (별첨 박스 상단 + 본문 표).
3. 한컴 PDF page 30, 31 (= page_num 기준) 과 시각 정합 비교.
4. 변경 통계 (코드/테스트 라인 수, 페이지 수 영향 sample 수).

**완료 조건**: 한컴 PDF 와 시각 정합.

### Stage 5: 최종 보고서 + merge

**작업**:
1. `mydocs/report/task_m100_1152_report.md` 작성.
2. 커밋 (`Task #1152: ...`).
3. `local/devel` 로 merge.

**완료 조건**: 보고서 작성 + merge 완료.

## 3. 위험 분석

| 위험 | 가능성 | 방어 |
|------|--------|------|
| 인접 케이스 false positive (이미 자연 advance 인데 추가 advance) | 저 | `!st.current_items.is_empty()` 조건으로 빈 페이지 추가 방지. Stage 3 회귀로 확인 |
| 다른 sample 에서 line_seg.vpos=0 이 자연 신호 (column reset 등) 로 사용되는 케이스 | 저 | empty-text + N:N 매핑 가드로 협소 범위 |
| `place_table_with_text` 의 pre-table 텍스트 처리와 충돌 | 저 | empty-text 가드로 사전 차단 |

## 4. 롤백 절차

문제 발생 시 가드 블록 (5~6 줄) 삭제만 하면 기존 동작 복원.

## 5. 일정 가늠

| Stage | 시간 |
|-------|------|
| 2. 코드 패치 + 단위 | 30 분 |
| 3. 회귀 + 테스트 | 1 시간 |
| 4. 시각 검증 | 30 분 |
| 5. 보고/merge | 30 분 |

총 약 2.5 시간.
