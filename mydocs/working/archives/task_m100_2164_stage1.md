# Task M100 #2164 Stage 1 완료보고서

- 이슈: #2164
- 브랜치: `codex/task_m100_2164`
- 기준: `upstream/devel` (`e966a66dff6bdd51a16ea36823bf0266cf80fdf4`)
- 재현 원본: `samples/issue2164/의견제출서(양식).hwp`
- 작성일: 2026-07-10

## 1. 실제 브라우저 재현

`wasm-pack build --target web --out-dir pkg`로 현재 Rust/WASM을 갱신한 뒤 이미 실행
중인 `http://localhost:7700`에서 원본을 열었다. `의견제출 요지` 다음의 큰 빈 셀은
본문 0번 문단의 2번 표 컨트롤, 8번 셀로 확인됐다.

셀 첫 문단에 `1212121212121212121`을 입력하고 Enter를 누른 결과는 다음과 같다.

| 시점 | 셀 문단 | 텍스트 길이 | 캐럿/줄 y(px) |
|------|---------|-------------|----------------|
| 입력 전 | 0 | 0 | 390.8 |
| 입력 전 | 1 | 0 | 418.5 |
| Enter 직전 | 0 | 19 | 390.7 |
| Enter 직후 | 0 | 19 | 390.7 |
| Enter 직후 신규 | 1 | 0 | **386.2** |
| Enter 직후 기존 후속 | 2 | 0 | 418.5 |

신규 문단 y가 앞 문단보다 4.5px 위로 역전됐다. 캔버스 표시만 늦은 것이 아니라
`getCursorRect`가 반환하는 실제 레이아웃 좌표가 잘못됐다.

## 2. 원인 계층 분리

- 19자 입력은 단일 input event의 즉시 페이지네이션 경로를 탔고 Enter 직전
  `deferredPaginationPending=false`였다. 지연 페이지네이션 flush 충돌이 직접 원인이 아니다.
- Enter는 `SplitParagraphInCellCommand`에서
  `split_paragraph_in_cell_native`까지 정상 도달했고 셀 문단 수도 2개에서 3개로 증가했다.
- 분할 뒤 기존 문단과 신규 문단을 각각 `reflow_cell_paragraph`로 재조판한다.
- `reflow_cell_paragraph`는 기존 `line_segs`를 비운 뒤 합성 `LINE_SEG`를 만들므로 두
  문단의 첫 `vertical_pos`가 모두 0이 된다.
- Top 정렬 셀의 `layout_horizontal_cell_paragraphs`는 저장된 셀 문단별
  `LINE_SEG.vertical_pos`를 절대 앵커로 우선한다. 따라서 같은 vpos를 가진 두 문단이
  같은 높이로 돌아가 겹친다.

즉 결함은 문단부호 표시나 Chrome 확장 전용 표시 문제가 아니라, **셀 문단 구조 편집
후 문단별 vpos 축을 다시 연결하지 않는 문서 코어 편집 계약**이다. 문단부호는 겹침을
더 분명하게 보이게 했을 뿐 레이아웃 좌표를 바꾸지 않는다.

## 3. 영향 범위

- 직접 영향: 표 셀 Enter 분할 뒤 신규 문단과 후속 문단 배치
- 같은 계약의 영향 후보: 셀 문단 시작 Backspace 병합 뒤 남은 후속 문단의 빈 간격
- 별도 경로: 본문 문단 Enter는 이미 `recalculate_section_vpos`를 호출하므로 동일 결함이 없다.
- 별도 경로: 중첩 셀 path 기반 분할/병합은 별도 구현을 사용하므로 focused 검증 후 같은
  좌표 계약을 적용할지 결정한다.

## 4. Stage 1 판정

Stage 1 완료. 수정 계층은 프론트 캔버스나 문단부호가 아니라
`src/document_core/commands/text_editing.rs`의 셀 문단 분할/병합 후 vpos 재계산 경로로
확정한다.
