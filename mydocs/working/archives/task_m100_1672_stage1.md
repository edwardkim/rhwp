# task m100 1672 stage1: PDF 페이지 수 완전 정합

## 기준선

- 기준 커밋: `7b877eea7` (`task 1672: 행정업무 편람 페이지 과대 분할 완화`)
- PDF 오라클: `pdf/2025 행정업무운영 편람(최종)-2024.pdf`
- PDF 기준 페이지 수: 383쪽
- 시작 시 rhwp 페이지 수
  - HWP: 388쪽 (`+5`)
  - HWPX: 390쪽 (`+7`)
- 최종 rhwp 페이지 수
  - HWP: 383쪽 (`±0`)
  - HWPX: 383쪽 (`±0`)

## 목표

- `samples/2025 행정업무운영 편람(최종).hwp` 페이지 수를 PDF 기준 383쪽으로 맞춘다.
- `samples/2025 행정업무운영 편람(최종).hwpx` 페이지 수를 PDF 기준 383쪽으로 맞춘다.
- 기존 RowBreak/overlay 회귀 테스트를 유지한다.

## 우선 조사 지점

- Q&A 후반부: PDF 307쪽 기준이 rhwp 310쪽으로 밀리는 구간(`+3`).
- 부록 시작부: PDF 309쪽 기준이 rhwp 312쪽으로 밀리는 구간(`+3`).
- 별표4/별지 제8호서식 및 법령 말미: PDF 369~383쪽 구간이 rhwp 373~388쪽으로 밀리는 구간(`+4~+5`).
- HWPX는 HWP보다 2쪽 더 초과하므로 HWPX 전용 불일치 구간을 별도로 확인한다.

## 작업 원칙

- 페이지 수를 맞추기 위한 완화는 구조적 조건으로 제한한다.
- 표 행을 강제로 넘기지 않는 대신, 한컴 PDF와 같은 반복 헤더/짧은 잔여 행/빈 행 처리 조건을 좁혀 찾는다.
- 사용자 지시에 따라 샘플 원본과 PDF 오라클을 함께 커밋한다.

## 구현 결과

- RowBreak 분할 행 예산에서 가시 내용이 남은 셀의 padding만 예약하도록 보정했다.
- landscape RowBreak 연속 fragment에서 반복 헤더가 있는 짧은 잔여 행 흡수를 확대했다.
- HWPX 소스는 같은 샘플에서 HWP보다 2쪽 더 초과하므로 HWPX 전용 landscape/분할 행 허용치를 분리했다.
- 샘플 HWP/HWPX와 PDF 오라클을 저장소에 추가한다.

## 검증

- `cargo fmt`
- `cargo build --release`
- `cargo test --profile release-test --test issue_1156_rowbreak_fragment_fit -- --nocapture`
- `cargo test --profile release-test --test issue_1488_rowbreak_empty_overlay_pages -- --nocapture`
- `cargo test --profile release-test --test issue_rowbreak_chart_overlap -- --nocapture`
- `git diff --check`
- `pdfinfo 'pdf/2025 행정업무운영 편람(최종)-2024.pdf'`: 383쪽
- `./target/release/rhwp info 'samples/2025 행정업무운영 편람(최종).hwp'`: 383쪽
- `./target/release/rhwp info 'samples/2025 행정업무운영 편람(최종).hwpx'`: 383쪽
