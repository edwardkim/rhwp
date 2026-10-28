# Stage 233: Rowspan block의 source band 소유권

## 목적

전체 integration 회귀에서 발견된
`samples/task2097/1741000_project_application.hwp`의 3쪽 과다 조판을 한글 COM
기준 2쪽으로 복원한다. 고정 px squeeze 허용치 없이 저장 line과 실제 content cut으로
RowBreak rowspan block의 마지막 body band 소유권을 결정한다.

## 분리 결과

- Stage 232 뒤 전체 `cargo test --profile release-test --tests`는 #2097 squeeze에서
  처음 멈췄고, 1741000 샘플은 `3`쪽으로 실패했다. 최신 `upstream/devel`은 같은
  `issue_2097_squeeze`의 세 pin을 통과하므로 독립된 누적 회귀다.
- 첫 조각의 block `rows 10..12`는 선언 높이 `98.1px`가 남은 body band
  `87.8px`를 넘는다. 그러나 `advance_row_block_cut`의 저장 line cut은 hard break
  없이 완결되고, block content height는 `64.1px`라 같은 band에 완전히 들어간다.
- 이 block은 rowspan label과 두 일반 행으로 구성되며 nested table이나 control이
  없다. 각 visible paragraph에는 비합성 저장 LineSeg가 있다. 즉 이월되는 `10.3px`
  는 source content가 아니라 선언된 아래 blank다.
- upstream 구현은 `13px` 초과, `100px` 잔여, `12px` headroom 상수로 이를
  허용한다. 같은 수치의 다른 문서에는 반례가 있으므로 이 Stage에서는 그 상수를
  도입하지 않는다.

## 구현

- continuation 중간의 RowBreak rowspan block만 후보로 삼는다.
- block 전체의 source content가 남은 band에서 `fully_consumed`이고 hard break가
  없으며, nested/control·저장 LineSeg 누락이 없을 때만 수용한다.
- content가 실제로 `remaining_band` 안에 완결되는지를 직접 확인한다. 초과 허용치나
  최소 blank/headroom은 사용하지 않는다.
- fragment의 선행 행과 cell spacing은 보존하고, 마지막 행만
  `remaining_band - preceding_rows`로 override한다. 따라서 scanner가 소비한 높이와
  renderer가 그린 physical band가 정확히 같으며, 선언 blank만 별도 tail page를
  만들지 않는다.

## 검증 범위

- `issue_2097_squeeze`의 1741000, 21298295, 21761835 한글 COM page-count pin.
- Stage 232의 `issue_2097_band_fill`과 #3820 집중 gate.
- 전체 `cargo test --profile release-test --lib` 및
  `cargo test --profile release-test --tests`.
