# Task #702: 최종 결과 보고서 — shortcut.hwp 다단 정의 후속 갱신 누락

## Issue / 브랜치

- GitHub Issue: [#702](https://github.com/edwardkim/rhwp/issues/702)
- 브랜치: `local/task702` (`upstream/devel @ 2fe386c4` 기준)
- 작업 기간: 2026-05-08

## 결함 개요

`samples/basic/shortcut.hwp` (한글 2010 단축키 일람표, A4 가로) 의 rhwp SVG 출력이 한글 2022 편집기 PDF (`pdf/basic/shortcut-2022.pdf`, 7쪽) 와 시각 정합 결함.

- **rhwp 출력 (수정 전)**: 10쪽 (+43% 폭주)
- **PDF 정답지**: 7쪽
- **rhwp 출력 (수정 후)**: 8쪽 (PDF +1쪽, 잔여 결함 별도 이슈로 분리)

## 본질 정정 (2건)

### 본질 1A — Distribute 다단의 짧은 컬럼 vpos-reset 검출 임계값

`src/renderer/typeset.rs:430-446` (수정 전 410-417):

기존 임계값 `pv > 5000` 은 짧은 Distribute (배분) 컬럼 (예: 지우기 3+3 분배) 에서 마지막 paragraph vpos=3000 < 5000 임계값 미달로 column-advance 미발동 → 6항목 1단 적층.

**정정**: `ColumnType::Distribute` (HWPX BalancedNewspaper) 한정 임계값 `pv > 0` 으로 완화. Normal/단단 분기는 기존 `pv > 5000` 유지 → Task #321/#418/#470 회귀 차단.

### 본질 1B — Page/Column break + 새 ColumnDef 미적용

`src/renderer/typeset.rs:396-441`:

shortcut.hwp p2 의 파일/미리보기/편집 sections 는 `[쪽나누기] + 단정의:1단 + 표(header)` → `[단나누기] + 단정의:2단 배분` 패턴 사용. 기존 코드는 `MultiColumn` break 만 ColumnDef 적용 → Page/Column break 동반 ColumnDef 무시 → col_count 가 이전 zone 값 유지 → 페이지 분기 폭주.

**정정**: Page/Column break 처리 시 새 ColumnDef 검출 후 zone 재정의 적용:
- `Column + has_diff_col_def`: `process_multicolumn_break` 호출
- `Column + 동일 ColumnDef`: 기존 `advance_column_or_new_page`
- `Page/Section + has_diff_col_def`: `force_new_page` 후 새 ColumnDef 적용 (col_count, layout, column_type 갱신)

### 보조 변경

- `TypesetState` 에 `current_zone_column_type: ColumnType` 필드 추가
- `TypesetState::new` 시그니처에 `column_type: ColumnType` 인자 추가
- `process_multicolumn_break` 에서 ColumnDef 적용 시 `current_zone_column_type` 갱신
- `typeset_section` 에서 초기 `column_def.column_type` 전달

## 변경 파일

### src/

- `src/renderer/typeset.rs`: +50 / -6
  - `ColumnType` import 추가
  - `TypesetState.current_zone_column_type` 필드 + 초기화
  - `TypesetState::new` 시그니처 변경
  - 메인 루프 `vpos-reset trigger` 분기 (Distribute 완화)
  - 메인 루프 Page/Column break 핸들러 + 새 ColumnDef 적용
  - `process_multicolumn_break` 에서 column_type 전파

### tests/

- `tests/issue_702.rs` (신규):
  - `shortcut_distribute_short_column_split`: 페이지 수 ≤ 8 검증
  - `shortcut_page2_has_three_sections`: 페이지 2 SVG 에 파일/편집 헤더 모두 존재 검증

### mydocs/

- `mydocs/plans/task_m100_702.md`: 수행계획서
- `mydocs/plans/task_m100_702_impl.md`: 구현계획서
- `mydocs/working/task_m100_702_stage2.md`: Stage 2 단계별 보고서
- `mydocs/working/task_m100_702_stage3.md`: Stage 3 단계별 보고서
- `mydocs/report/task_m100_702_report.md`: 본 최종 보고서

## 검증 결과

### `cargo test --release`

```
1248+ tests run
- 단위 테스트 (lib): 1157 passed, 0 failed, 2 ignored
- 통합 테스트 (총 18 그룹, 0 failures)
  - exam_eng_multicolumn: 14 passed (Task #470 회귀 차단)
  - issue_418: 1 passed (단단 partial-table split 잔재)
  - svg_snapshot: 7 passed (golden snapshots 시각 회귀)
  - issue_702 (신규): 2 passed
  - 기타 모두 0 failures
```

### dump-pages 정합

| 항목 | 수정 전 | 수정 후 |
|------|--------|--------|
| 페이지 수 | 10 | 8 |
| LAYOUT_OVERFLOW | 다수 (40~60px) | 1건 (페이지 8 마지막) |
| 페이지 1 지우기 | 1단 6항목 | 2단 3+3 ✓ |
| 페이지 2 섹션 | 파일 header 만 | 파일+미리보기+편집 ✓ |

### 시각 검증 (qlmanage 비교)

- **페이지 1**: 제목 + 커서이동 (2단 14+13) + 지우기 (2단 3+3) ✓ PDF 정합
- **페이지 2**: 파일 (2단 5+5) + 미리보기 (2단 5+4) + 편집 (2단 12+11) ✓ PDF 정합
- **페이지 3 이후**: 컨텐츠 1쪽 시프트 (pi=94 케이스, 별도 이슈 #708)

### 광범위 샘플 회귀

| 샘플 | 페이지 수 | LAYOUT_OVERFLOW | 회귀 |
|------|----------|----------------|------|
| KTX | 1 | 1 | ✓ 무회귀 |
| aift | 77 | 8 | ✓ 무회귀 |
| hwp-multi-001 | 10 | 1 | ✓ 무회귀 |
| kps-ai | 80 | 10 | ✓ 무회귀 |
| exam_eng | 8 | 12 | ✓ 무회귀 |

## 잔여 결함 → 별도 이슈 분리

### Issue #708 — pi=94 bare `[단나누기]` 마지막 col 시프트

shortcut.hwp pi=94 (`<편집 화면 분할에서>`) 의 bare `[단나누기]` (no ColumnDef) at last col 케이스. 1쪽 시프트의 직접 원인.

시도한 정정 (Column+last_col+no_def → process_multicolumn_break) 은 3개 기존 테스트 회귀 발생 → rollback. 회귀 가드 더 정밀하게 분석 후 후속 task 에서 처리.

### Issue #709 — 부수 시각 결함 4건

본질 1 외 잔존 시각 결함:
1. 제목 PUA 글자 (`\u{f53a}`) "한글 2010" → "ㅎ글2010" 표시
2. 탭 leader (점선 가이드) 미렌더링
3. 바탕쪽 자동번호 (페이지 번호 데코) 미렌더
4. 페이지 1 커서이동 right col 단축키 우측 정렬 누락

## 회귀 위험 평가

🟢 **Low**: 정정 범위가 본질 1 (Distribute 한정 + Page/Column break + ColumnDef 검출) 로 한정. Normal/단단 분기 영향 없음. 광범위 회귀 테스트 0 failures.

🟢 **회귀 가드 추가**: tests/issue_702.rs 2건. 향후 회귀 시 조기 검출.

## 정정 효과

- 핵심 결함 정정: 페이지 분기 폭주 정정 (10→8쪽), 지우기 분배 정합, 파일/미리보기/편집 통합 페이지 정합
- 별도 이슈 분리: 잔여 결함 (#708, #709) 후속 처리 명확화
- 광범위 회귀 0 failures, 시각 회귀 0건

## 다음 단계

작업지시자 승인 시:
1. commit (memory rule "내부 task commit 금지" — 명시 요청 시에만)
2. local/devel 머지 (수동 처리)
3. 작업지시자가 mydocs/orders 일일 할일 갱신
