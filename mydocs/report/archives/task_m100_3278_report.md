# task_m100_3278 처리결과 보고서 — `export-tables` 표 격자 JSON 추출

- **이슈**: [#3278](https://github.com/edwardkim/rhwp/issues/3278)
- **브랜치**: `pr/task-tables-json` (**upstream/devel 직분기 — 열린 PR 4건과 공유 커밋 없음**)
- **범위**: `src/document_core/queries/table_extract.rs`(신규), `queries/mod.rs`(1행),
  `src/main.rs`(명령 1개·디스패치 1행·help), `tests/table_extract_json_contract.rs`(신규),
  `mydocs/manual/cli_commands.md`
- **분류**: 기능 추가 (읽기 전용 질의)

## 1. 문제

한국 행정문서는 표가 본체인데 표를 기계가 읽을 출구가 없었다.

- `export-text`: 셀 경계 소실 → 평문
- `export-markdown`: `table_to_markdown`(rendering.rs)이 앵커 위치에만 텍스트를 찍어
  **병합을 표현하지 못한다.** 실측 `samples/table-001.hwp` 헤더가 `| 구 분 | 5월 |  |  | 6월 |`
  로 나와, 3열 병합인 "5월" 뒤의 빈 칸을 소비자가 **별개 열로 오독**한다.
- `dump`: rs/cs 는 정확하나 비정형 텍스트라 파싱 계약이 없다.

즉 정보는 IR 에 정확히 있는데(`Cell{row,col,row_span,col_span}`) 출구만 없는 문제였다.

## 2. 분석 — 설계 결정

- **격자를 펼치지 않는다.** `Table.cells` 는 이미 **앵커 셀만** 담으므로 그대로 직역한다.
  rows×cols 로 미리 할당하는 순진한 구현은 손상 문서(`MAX_TABLE_GRID_CELLS`=4M)에서
  OOM 위험이 있는데, 앵커 리스트 방식은 이를 원천 회피한다.
- **컨테이너 재귀**: 본문 최상위만 훑으면 공문서에서 흔한 글상자·머리말·각주 안의 표를
  통째로 놓친다. `Control::{Shape(글상자)/Header/Footer/Footnote/Endnote}` 를 재귀한다.
  실측 근거: `samples/basic/treatise sample.hwp` 는 최상위 기준 1개지만 실제 **3개**,
  `samples/biz_plan.hwp` 는 8 → **9**.
- **방어**: 범위 밖 앵커는 버리고(doclang `convert_table` 의 clamp 와 같은 취지),
  span 0 은 1로 정규화하며, 중첩 깊이 상한 8을 둔다(병적 중첩 스택 보호).
- **주소 부여**: 표마다 `section`/`paragraph` 를 실어 인용·역참조가 성립하게 했다.
- 본체는 `queries/` 모듈에 두고 `main.rs` 에는 인자 파싱만 남겼다(main.rs 6,500줄 규약).

## 3. 변경

- `queries/table_extract.rs` 신설 — `TableGrid`/`GridCell`(serde) + `extract_tables()`
- `export-tables` 명령: `--json` 봉투 / `-o` 파일 저장 / 기본은 사람용 요약
- `cli_commands.md` 신설 항목 (병합 소실 문제·한계 명시)

## 4. 검증

- **계약 테스트 8종 red→green**: 봉투 스키마, **병합 보존**(가로·세로 병합 존재 + 병합 면적
  합이 격자를 넘지 않음=중복 출력 금지), 셀 주소 범위, **중첩 표**, **컨테이너 표 3개+**,
  기본 출력 비-JSON 가드, 종료 코드(없는 파일 1·인자 없음 2)
- `cargo clippy --release --bin rhwp -- -D warnings` 0, `rustfmt` clean
- 실측: `2025 행정업무운영 편람(최종).hwp` 표 323개 추출, 코퍼스 120건 스윕에서
  기존 `info` 대비 표를 더 찾는 문서 2건 확인(위 근거)

## 5. 남긴 것

- 셀 안 **자동번호**는 IR 텍스트에 값이 없다(렌더 단계 주입) — 문서에 한계로 명시.
- 1×1 래퍼 표 평탄화는 정책 주입이라 하지 않았다 — 소비자가 `rows==1&&cols==1` 로 거른다.
- `batch export-tables`(코퍼스 표 스윕)는 batch 축을 건드리는 다른 PR 과 겹치므로 후속.
