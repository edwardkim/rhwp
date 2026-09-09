# PR 초안 — #2439 HWP 반복 표 헤더 격자와 페이지 흐름 정합

## 제목

```text
fix(renderer): HWP 반복 표 헤더 격자와 페이지 흐름 정합 (#2439)
```

## 본문

compare PNG는 저장소에 커밋하지 않는다. PR 본문을 GitHub Web에서 편집하면서 각 페이지
위치에 직접 첨부하고, GitHub가 생성한 첨부 URL을 사용한다.

```markdown
## 변경 요약

HWP5 문서의 반복 표가 페이지 경계에서 잘못 분할되거나 뒤따르는 텍스트와 겹치고,
저장된 헤더 열 폭과 문단 들여쓰기가 재현되지 않던 문제를 수정합니다.

- 반복 표와 뒤따르는 서명문이 겹치지 않습니다.
- 2쪽의 `일자`/`점검 항목` 헤더와 `비고` 열 폭을 복원합니다.
- 2쪽 마지막 행이 불필요하게 3쪽으로 넘어가지 않습니다.
- 번호 줄글이 저장 `LineSeg.column_start`에 맞게 들여쓰기됩니다.
- 한컴 2024 정상 PDF와 같은 10쪽 흐름과 마지막 문장을 유지합니다.
- `LAYOUT_OVERFLOW`가 발생하지 않습니다.

## 원인

1. fresh page로 이월된 표의 placement/exclusion이 이전 페이지의 host 기준점을 유지했습니다.
2. co-anchored 표와 표 뒤 post-text가 같은 exclusion/flow 계약을 사용하지 않았습니다.
3. native HWP5 empty-host `TopAndBottom`/`RowBreak` 표의 저장 LineSeg, vertical offset,
   fragment outer margin을 typeset/full/partial 경로가 서로 다르게 소비했습니다.
4. 반복되는 정상 폭과 다른 축퇴 헤더 행이 기본 열 격자 산출에 참여해 첫 헤더 셀과
   마지막 `비고` 열을 왜곡했습니다.
5. orphan guard가 셀 padding을 제외한 높이로 판단해 실제로 들어갈 수 있는 마지막 행을
   다음 페이지로 넘겼습니다.
6. native HWP5 일반 본문 줄에서 저장 `LineSeg.column_start`를 적용하지 않았습니다.

## 변경 사항

### 반복 표 placement와 fragment flow

- fresh page로 이월된 표는 새 페이지의 현재 높이를 placement 기준점으로 사용합니다.
- 같은 visible host의 zero-offset/positive-offset 표를 순차 flow로 소비합니다.
- 실제 painted bottom과 outer bottom을 다음 문단 flow와 row fit에 반영합니다.
- orphan guard의 visible fragment 높이에 셀 top/bottom padding을 포함합니다.
- 첫 fragment에는 이전 outer-bottom, 현재 outer-top, vertical offset을 적용하고,
  연속 fragment에는 현재 outer-top만 반복합니다.
- full `PageItem::Table`과 첫 `PartialTable`이 같은 상단 좌표 계약을 사용합니다.

이 보정은 native HWP5, 비-TAC, 빈 host, Para 기준 `TopAndBottom`, `RowBreak`, 양수
offset, 단일 표, 비합성 저장 LineSeg, 뒤따르는 일반 텍스트라는 구조 증거가 모두
성립할 때만 적용합니다. #2097에서 반증된 광범위한 outer-margin 가산은 하지 않습니다.

### 헤더 열 격자

- 지배적 폭 벡터와 다른 소수 행을 `base_grid_outlier_rows`로 분리합니다.
- 행 전체 폭 합이 보존된 경우에만 `inferred_local_resize_rows`로 인정합니다.
- 축퇴 헤더 행은 기본 열 격자를 오염시키지 않고, 실제 보상 resize 행은 독립 경계를
  유지합니다.
- 추론 행의 양수 residual은 마지막 셀을 늘리지 않고 기본 열 경계로 fallback합니다.

### 저장 LineSeg 들여쓰기

- native HWP5의 비합성 full-width 일반 본문 줄에서 저장 `LineSeg.column_start`를 권위
  시작점으로 적용합니다.
- 표 셀, wrap/control, 번호 control, 합성 LineSeg, HWP3/HWPX는 제외합니다.

## 검증

- [x] `cargo test --lib`: 2,347 passed, 0 failed, 7 ignored
- [x] 관련 통합 테스트 10개 target: 36 passed, 0 failed
  - `issue_1510`, `issue_1535`, `issue_1549`, `issue_1663`
  - `issue_1772_table_outer_margin_sync`
  - `issue_1789_exclusion_probe_line_spacing`
  - `issue_2097_band_fill`
  - `issue_2322_fullpage_form_table_pair`
  - `issue_2439`, `issue_493_cell_attrs`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy -- -D warnings`
- [x] `git diff --check`
- [x] `wasm-pack build --target web --out-dir pkg`
- [x] rhwp-studio 로컬 서버 HTTP 200
- [x] 한컴 2024 정답 PDF visual sweep
  - rhwp/PDF 10/10쪽
  - compare/overlay/review 각 10장
  - 자동 이상 후보 0/10
  - `LAYOUT_OVERFLOW` 없음
  - 평균 `pixel_match_percent` 89.60574%

자동 시각 지표는 폰트 fallback의 영향을 받는 내용 픽셀 중심 보조값이며, 호환성 점수나
사람 판정을 대신하지 않습니다. 아래 이미지는 왼쪽이 rhwp 출력, 오른쪽이 한컴 2024
정상 PDF입니다. 이미지를 선택하면 원본 크기로 볼 수 있습니다.

## 페이지별 PDF 비교

### 1쪽

<!-- GitHub Web에서 compare_001.png를 여기에 첨부 -->

### 2쪽

<!-- GitHub Web에서 compare_002.png를 여기에 첨부 -->

### 3쪽

<!-- GitHub Web에서 compare_003.png를 여기에 첨부 -->

### 4쪽

<!-- GitHub Web에서 compare_004.png를 여기에 첨부 -->

### 5쪽

<!-- GitHub Web에서 compare_005.png를 여기에 첨부 -->

### 6쪽

<!-- GitHub Web에서 compare_006.png를 여기에 첨부 -->

### 7쪽

<!-- GitHub Web에서 compare_007.png를 여기에 첨부 -->

### 8쪽

<!-- GitHub Web에서 compare_008.png를 여기에 첨부 -->

### 9쪽

<!-- GitHub Web에서 compare_009.png를 여기에 첨부 -->

### 10쪽

<!-- GitHub Web에서 compare_010.png를 여기에 첨부 -->

## 관련 이슈

Closes #2439
```

## 게시 상태와 이미지 첨부

- PR: [#2512](https://github.com/edwardkim/rhwp/pull/2512)
- base: `edwardkim/rhwp:devel`
- head: `postmelee:fix/2439-split-table-flow`
- 상태: Draft
- compare 원본: `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/compare/`
- 이미지 첨부 순서:
  1. GitHub Web에서 PR 본문을 편집한다.
  2. 각 페이지 제목 아래에 `compare_001.png`부터 `compare_010.png`까지 직접 첨부한다.
  3. 미리보기에서 10개 이미지와 페이지 순서를 확인한 뒤 저장한다.

compare PNG는 저장소 경로에 추가하지 않는다.
