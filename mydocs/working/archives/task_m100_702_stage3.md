# Task #702: Stage 3 단계별 보고서 — 광범위 회귀 검증 + 시각 판정

## 작업 개요

Stage 2 본질 1 정정 (Distribute 다단 + Page/Column break + 새 ColumnDef 케이스) 의 광범위 회귀 차단 검증 + shortcut.hwp 시각 정합 페이지별 비교.

## 광범위 회귀 검증

### `cargo test --release` 전체 회귀

```
1248+ tests run
- 단위 테스트 (lib): 1157 passed, 0 failed, 2 ignored
- 통합 테스트 (총 18 그룹, 0 failures)
  - exam_eng_multicolumn: 14 passed (Task #470 회귀 차단 ✓)
  - issue_418: 1 passed (단단 partial-table split 잔재 ✓)
  - issue_630: 1 passed (aift.hwp 4페이지 우측 정렬)
  - svg_snapshot: 7 passed (golden snapshots 시각 회귀)
  - issue_702 (신규): 2 passed
  - 기타 issue_301/501/505/514/516/530/546/554/598/630, hwpx_*, page_number_*, tab_cross_run: 모두 0 failures
```

### 광범위 샘플 export-svg

| 샘플 | 페이지 수 | LAYOUT_OVERFLOW |
|------|----------|----------------|
| KTX | 1 | 1 (기존 유지) |
| aift | 77 | 8 |
| hwp-multi-001 | 10 | 1 |
| kps-ai | 80 | 10 |
| exam_eng | 8 | 12 |

## shortcut.hwp 시각 정합 페이지별 비교

### 페이지 수

- **PDF 정답지**: 7쪽 (한컴 2022 출력)
- **rhwp 출력 (Stage 1 시점)**: 10쪽
- **rhwp 출력 (Stage 2 정정 후)**: 8쪽

1쪽 잔여 차이 = 페이지 3 이후 컨텐츠 1쪽 분량 시프트.

### 페이지별 정합 (qlmanage 비교)

| Page | rhwp SVG | PDF | 정합도 |
|------|----------|-----|-------|
| 1 | 제목 + 커서이동 (2단 14+13) + 지우기 (2단 3+3) | 동일 | ✅ 정합 |
| 2 | 파일 (2단 5+5) + 미리보기 (2단 5+4) + 편집 (2단 12+11) | 동일 | ✅ 정합 |
| 3 | 보기 (2단 6+6) | 보기 + <편집 화면 분할에서> + 입력 + <그림 넣기에서> + 그림 | ⚠️ 컨텐츠 부족 |
| 4 | <편집 화면 분할에서> + 화면이동 + 입력 + <그림 넣기에서> + 그림 + 글상자 + 상용구 + 서식 + 스타일 | <글상자> + 상용구 + 서식 + <스타일> + <글자 속성> | ⚠️ 컨텐츠 시프트 |
| 5~8 | (시프트된 후속 컨텐츠) | (5~7만, 7쪽 종료) | ⚠️ 1쪽 시프트 |

### 페이지 3 이후 1쪽 시프트 원인

**pi=94 케이스 — bare `[단나누기]` 미적용**:

shortcut.hwp pi=94 = "<편집 화면 분할에서>" 헤더 라인. column_type = `[단나누기]` (Column break) **without ColumnDef 컨트롤**.

처리 시나리오 (현재 정정 후):
- 보기 right col 끝 (col 1 of 2) 직후 pi=94 [단나누기] 도달
- `has_diff_col_def = false` (no ColumnDef)
- `advance_column_or_new_page` 호출 → 마지막 col → **새 페이지 강제**
- pi=94 가 페이지 4 로 시프트

PDF 정답: pi=94, 95 가 페이지 3 의 보기 content 아래 같은 2단 zone 에 자리. 즉 **HWP 의 bare `[단나누기]` 가 마지막 col 에서 새 zone (col_count 유지) 시작 신호로 사용**된 패턴.

### 시도한 정정 + 회귀 발견

`[단나누기] + 마지막 col + no ColumnDef → process_multicolumn_break` 호출 추가 시도.

**결과**: shortcut.hwp 7쪽 PDF 정합 달성, 그러나 **3개 기존 테스트 회귀**:
- `test_539_partial_paragraph_after_overlay_shape`
- `test_548_cell_inline_shape_first_line_indent_p8`
- `test_exam_math_page_count`

해당 테스트들은 다른 다단 패턴에서 페이지 수 / 컨텐츠 위치 검증. bare `[단나누기]` at last col 의 회귀 위험이 너무 큼 → **정정 취소 (rollback)** 후 1쪽 차이 잔존.

### 잔여 결함 (별도 이슈 후보)

#### 본질 영역 (pi=94 시프트)

본 사이클 범위 외, 별도 task 분리:
- bare `[단나누기]` at last col 정합 — 회귀 가드 더 정밀하게 분석 필요
- 후보: pi=94 의 다음 paragraph (pi=95) 가 content (text_len > 0) 인지 확인하여 분기

#### 시각 결함 (본 사이클 범위 외)

1. **PUA 글자 (제목 "한글 2010" → "ㅎ글2010")** — char_shape `\u{f53a}` PUA 글리프 / spacing -5% / ratio 95% 적용 결함
2. **탭 leader 미렌더링** — PDF 의 점선 가이드 (key 와 description 사이) 누락
3. **바탕쪽 자동번호 (각 페이지 우하단 큰 회색 1~7) 미렌더** — `tb_ctrl[0]: 자동번호(Page)` 의 글상자 렌더 부재
4. **page 1 커서이동 right col 단축키 우측 정렬 누락** — 텍스트는 있으나 키 부분 표시 안 됨

## Stage 2 + Stage 3 종합 결과

### 정정 영향

| 항목 | 수정 전 | Stage 2 정정 후 |
|------|--------|----------------|
| shortcut.hwp 페이지 수 | 10 | 8 (PDF 7 +1쪽) |
| LAYOUT_OVERFLOW | 다수 (40~60px) | 1건 (페이지 8 마지막) |
| 페이지 1 지우기 분할 | 1단 6항목 | 2단 3+3 ✓ |
| 페이지 2 섹션 통합 | 파일 header 만 | 파일+미리보기+편집 ✓ |
| `cargo test` | — | **0 failed** (1248+ tests) |
| 회귀 가드 (issue_702) | — | 2 passed (페이지 수 ≤ 8 + 페이지 2 섹션 정합) |

### 핵심 결함 정정

1. **본질 1A**: `ColumnType::Distribute` (HWPX BalancedNewspaper) 한정 vpos-reset 임계값 `pv > 0` 으로 완화. Normal/단단 분기 영향 없음.
2. **본질 1B**: `[쪽나누기]` / `[단나누기]` 가 새 ColumnDef 동반 시 zone 재정의 적용 (이전: ColumnDef 무시).

## 승인 요청

Stage 3 검증 결과대로 Stage 4 (최종 보고 + 부수 결함 별도 이슈 분리 + commit 승인) 진입 가능 여부 승인 부탁드립니다.

Stage 4 작업 항목:
1. 잔여 결함 (pi=94 시프트, PUA 글자, 탭 leader, 바탕쪽 자동번호) 별도 이슈 등록
2. 최종 결과 보고서 작성 (`mydocs/report/task_m100_702_report.md`)
3. 작업지시자 명시 승인 시 commit (memory rule "내부 task commit 금지" 정합 — 명시 요청 시에만)
