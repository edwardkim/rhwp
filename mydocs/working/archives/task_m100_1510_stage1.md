# Task #1510 Stage 1 — co-anchored float 표 정렬·흐름 보정 착수

## 1. 이슈

- GitHub: <https://github.com/edwardkim/rhwp/issues/1510>
- 제목: `[HWP] 같은 문단 co-anchored float 표를 vertical_offset 으로 배치`
- 상태: OPEN

## 2. 기준 자료

작업지시자가 합성 샘플을 한컴 2024에서 PDF로 다시 저장했다.
이 샘플은 이슈 #1510 원본 내부 양식이 아니라, 이슈 본문의 최소 재현 조건에 맞춘
합성 재현 문서다. `mydocs/manual/ai_sample_document_authoring_guide.md` 기준으로는
최종 canonical fixture 라기보다 "임시 재현 샘플 + 한컴 2024 PDF 기준"으로 취급한다.

| 파일 | 기준 |
|------|------|
| `samples/issue1510_coanchored_float_tables.hwp` | HWP 재현 입력 |
| `samples/issue1510_coanchored_float_tables.hwpx` | HWPX 참고 입력 |
| `pdf/issue1510_coanchored_float_tables-hwp-2024.pdf` | 로컬 한컴 2024 HWP 기준 PDF |
| `pdf/issue1510_coanchored_float_tables-hwpx-2024.pdf` | 로컬 한컴 2024 HWPX 참고 PDF |

수정 전 `pdfinfo` 기준:

- HWP 2024 PDF: 1페이지
- HWPX 2024 PDF: 2페이지
- rhwp HWP PDF: 2페이지

따라서 이번 Stage의 1차 판정 기준은 HWP 입력과 `hwp-2024.pdf`다.
작업지시자 요청에 따라 HWP/HWPX 샘플과 한컴 2024 PDF, rhwp 산출 PDF를 모두 커밋에 포함한다.

## 3. 관찰

HWP 샘플 첫 문단:

- 가운데 정렬 제목 텍스트: `ISSUE 1510 CENTER TITLE`
- 같은 문단에 비-TAC 표 3개
- 공통 속성: `text_wrap=TopAndBottom`, `vert_rel_to=Para`
- control 순서: A(`vertical_offset=16996`) → B(`vertical_offset=-2000`) → C(`vertical_offset=0`)

수정 전 rhwp `dump-pages`:

```text
PartialParagraph pi=0
Table pi=0 ci=3
Table pi=0 ci=4
Table pi=0 ci=2
```

즉 visible text가 있는 host 문단에서도 `vertical_offset` 정렬이 적용되어 control 순서가 B → C → A로 바뀐다.

한컴 2024 HWP PDF:

- 1페이지에 모든 filler 문단이 들어간다.
- B/C 표는 제목 주변 위쪽에 위치한다.
- A 표는 선언된 양수 offset 위치에 놓이지만, 그 offset만큼 본문 흐름 전체를 앞에서 밀어내지는 않는다.

## 4. 수정 내용

1. visible text가 있는 문단의 para-relative `TopAndBottom` float 표는 `vertical_offset` 정렬 대상에서 제외한다.
2. 같은 조건의 표는 host 문단 처리 시 `vertical_offset`/표 높이를 즉시 current flow 높이에 누적하지 않도록 한다.
3. 빈 host 문단의 다중 para-float lane 경로(`#986`)와 기존 음수 offset 침범 가드(`#712`)는 보존한다.

구현:

- `src/renderer/typeset.rs`
  - 빈 host 문단에서는 기존처럼 para-float 표를 `vertical_offset` 오름차순 안정정렬한다.
  - visible text host 문단에서는 control/document 순서를 유지한다.
  - visible text host 의 para-float 표는 `place_table_with_text`에서 큰 양수 offset을 host 텍스트 앞 공백으로 누적하지 않는다.
- `src/renderer/layout.rs`
  - visible text host 의 para-float 표는 sibling 표 모두 같은 anchor y를 기준으로 배치한다.
  - 양수 offset 표는 layout cursor를 즉시 하단으로 밀지 않는다.
  - 음수/0 offset sibling은 제목 텍스트와 겹치지 않도록 해당 표 하단까지만 cursor를 보정한다.

## 5. 검증 결과

- 신규 테스트: `tests/issue_1510.rs`
  - `samples/issue1510_coanchored_float_tables.hwp` page_count = 1
  - 첫 페이지 render tree에서 pi=0 표 순서 = `[2, 3, 4]`
- 수동 확인:
  - `target/debug/rhwp info samples/issue1510_coanchored_float_tables.hwp` → 1페이지
  - `target/debug/rhwp dump-pages samples/issue1510_coanchored_float_tables.hwp` → `Table pi=0 ci=2`, `ci=3`, `ci=4` 순서, filler 32까지 page 1
  - `target/debug/rhwp export-pdf samples/issue1510_coanchored_float_tables.hwp -o pdf/issue1510_coanchored_float_tables.hwp.pdf` → 1페이지, `LAYOUT_OVERFLOW` 없음
  - `target/debug/rhwp export-pdf samples/issue1510_coanchored_float_tables.hwpx -o pdf/issue1510_coanchored_float_tables.hwpx.pdf` → 1페이지
- cargo:
  - `cargo fmt --check` 통과
  - `cargo test --test issue_1510 -- --nocapture` 통과
  - `cargo test --test issue_986 -- --nocapture` 통과
  - `cargo test --test issue_712 -- --nocapture` 통과

## 6. 남은 관찰

현재 HWP 샘플의 rhwp PDF는 한컴 2024 HWP PDF와 페이지 수/표 순서/제목 중앙 정렬 문제는
개선되지만, 양수 offset A 표와 일부 filler 문단의 세부 시각 회피는 한컴 2024와 완전히
동일하지 않다. 이번 Stage는 이슈 #1510 본문이 제안한 정렬/흐름 누적 회귀를 먼저 막는
범위로 고정하고, 완전한 TopAndBottom exclusion 모델은 별도 후속으로 분리한다.
