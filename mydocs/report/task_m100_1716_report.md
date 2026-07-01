# Task #1716 최종 결과보고서 — 반복 제목행 overhead 페이지당 1행 폭주 수정

- 이슈: edwardkim/rhwp#1716 (M100)
- 브랜치: `local/task1716` (devel 기반)
- 커밋: `8a821046`

## 1. 문제

hwpdocs 표본 3000개 PI↔페이지(한글 OLE) 검증 최악 아웃라이어:
`국토교통부/3031803 건설공사 품질시험기준.hwpx` → rhwp **173쪽** vs 한글 **52쪽** (+121).
표 pi=12(183행×7열, RowBreak)가 행 60부터 **페이지당 1행**씩 배치되어 폭증.

## 2. 근본 원인

반복 제목행(repeat-header) overhead 를 `cursor_row`(렌더러는 `start_row`) 아래의 **모든
`is_header` 행 높이 합**으로 계산(typeset.rs, table_partial.rs). 이 문서는 `<hp:tc header="1">`
가 4265셀 중 364개로 **본문 행 전반에 흩어져** 있어(파서는 정상 반영), cursor 전진 시 overhead
가 단조 증가(cursor40→589px, 76→1350px>페이지) → `avail_for_rows=0` → 1행/쪽 폭주.

Task #1022 가 "행 0만" → "cursor 아래 모든 is_header 행"으로 확대(다중 머리행 pi=111 대응)한
것이 흩어진 header 문서에서 역효과를 낸 것. 한글은 상단 제목행만 반복 → 52쪽.

## 3. 수정

**정의**: 반복 제목행 = 표 상단(행 0)부터 연속인 제목행 블록 `rows 0..H`
(행 r 제목여부는 header 셀 rowspan 덮개 반영). 흩어진 하위 is_header 행 제외.

- `model/table.rs`: `Table::leading_header_rows()` 공유 헬퍼 + 단위테스트 4개.
- `renderer/typeset.rs`: `header_overhead` 를 `leading_header_rows ∩ {r<cursor_row}` 로 교체.
- `renderer/layout/table_partial.rs`(2곳): 렌더러 반복 제목행 수집도 동일 헬퍼로 통일
  → 페이지네이터·렌더러 정합(desync=오버플로 차단).

## 4. 검증 결과

| 항목 | 수정 전 | 수정 후 | 한글 |
|------|--------|--------|------|
| 대표 파일 총 페이지 | 173 | **53** | 52 |

- pi=12 표 배치: 페이지당 1행 → **46/46/44/7행** 정상 회복.
- 한글 OLE 재검증: PAGE_DELTA 173→**53**(한글 52, +1). A 유형 폭주 해소.
  - 잔여 +1쪽/37 PI off-by-one 은 별개 **B 유형 미세 행높이 표류**(수정이 유발 아님).
- **회귀**:
  - lib 단위테스트 **2042 passed / 0 failed** (신규 4개 포함).
  - 표 통합 테스트 **19 passed / 0 failed** (7 크레이트).
  - 의미적 무회귀: 일반/다중 머리행(연속) 표는 `leading_header_rows` 결과가 기존과 동일.

## 5. 산출물

- 소스: `model/table.rs`, `renderer/typeset.rs`, `renderer/layout/table_partial.rs`, table/tests.rs
- 문서: 수행계획서/구현계획서, stage1~4, 본 보고서
- 샘플: `samples/task1716/table_scattered_header_rowbreak.hwpx` + README

## 6. 잔여/후속

- **B 유형 +1쪽 표류**(별개 이슈): 대표 파일 잔여 및 표본 다수 PAGE_DELTA 소규모 off-by-1.
- 전체 `cargo test`(통합 162 크레이트, 릴리즈 LTO)·hwpdocs 3000 재검증은 환경 제약으로 CI/후속 권장.

## 7. 결론

A 유형(표 행높이 폭주) 대표 케이스를 **173→53쪽으로 정정**, 회귀 없음. 반복 제목행을 상단
연속 블록으로 한정하는 최소·안전 수정. devel 머지 후보.
