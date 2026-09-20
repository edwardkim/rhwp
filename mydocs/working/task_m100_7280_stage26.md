# Task #7280 Stage 26 — R2x 데코레이션 표 host 텍스트 배치 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2w](task_m100_7280_stage25.md), 시작 head `1d66e82fa`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R2x 구현, 고정 제품 SHA 집중 검증 예정. R2 전체/PR 준비 완료가 아니다.

## 1. 책임과 보존 범위

일반 `typeset_table_paragraph`의 컨트롤 루프 뒤 decoration-only host 텍스트 배치를 분리한다.
`controls/decoration_host.rs`는 기존 소유 조건·줄 범위·전진량을 조회하여 기존 공용 타입
`ParagraphFragment`를 반환한다. `controls::place_decoration_host_text`는 결과가 있을 때만
`state::commit_decoration_host_text`를 호출한다. state는 항목 추가 → 높이 전진만 수행한다.
새 범용 rule engine이나 public API, IR·PageItem 스키마는 만들지 않는다.

### 실제 호출과 소비

- 부모 컨트롤 루프의 `decoration_host_text_pending`/`flow_table_owns_host_text` 생산은 그대로 둔다.
  모든 컨트롤의 앵커·continuation bounds가 확정된 뒤 한 번만 호출하고, 그 뒤 R2w TAC 정산이 실행된다.
  일반/TAC 표가 host 텍스트를 소유하면 중복 항목을 추가하지 않는다.
- 가시 비공백 텍스트가 있으면 기존 `fmt.line_heights.len()`을 줄 범위로 사용한다.
  양수 줄 수에서만 `PartialParagraph(0..total_lines)`를 만들며 높이는 기존
  `fmt.line_advances_sum(0..total_lines)`이다. 줄 수 0에서 빈 host 경로로 넘어가지 않는다.
- 글자가 없는 경로는 기존 `stored_decoration_host_line_advance_hu`를 본문 그대로 이동한다.
  원본 저장 줄만 인정하는 조건, 양수 델타, 현재 뒤간격·다음 앞간격, HWPUNIT 변환과 ±1 허용치를 유지한다.
  비공백 텍스트 검사와 helper 안의 visible-text 검사는 서로 다르다. 공백 문자열을 새롭게 동일시하지 않는다.
- HWP5/HWPX profile 조회는 closure로 전달하여 기존 빈 host 분기에서만 평가한다.
  engine profile을 state profile로 대체하거나 문단 진입 시 미리 읽지 않는다.
- 기존 `FullParagraph`와 `PartialParagraph` 선택을 유지한다. 일반 split 상태 명령은 trim 값을
  초기화하므로 재사용하지 않는다. 전용 명령은 기존처럼 trim/vpos/페이지 전이를 바꾸지 않는다.
- 높이 계산은 상태 쓰기 없는 계획 단계로 이동한다. 가시 경로의 합산은 동일 메트릭 벡터의
  안전한 범위 합계이며, 빈 host의 px 변환도 상태·진단에 의존하지 않는다. 기존에는 항목 push 뒤
  계산했으나 이제 계산 후 push한다. 상태 효과는 여전히 push → `current_height += height`다.
- 확정 높이는 이후 TAC cap의 입력으로 그대로 남는다. cap을 건너뛰거나 새 clamp를 추가하지 않는다.
  측정/paint 또는 저장 정보 수용 규칙을 바꾸는 절편이 아니며 새로운 피델리티 개선을 주장하지 않는다.

조판 계산·상수·분기 선택, 테스트 source/assertion/ID, baseline/golden/ignore 및 CI는 변경하지 않는다.
공통 visible-text helper는 다른 경로도 사용하므로 부모에 남겼고 명시적으로 import한다.
이는 최종 의존 방향 정리의 이행 항목이며 모든 상위 의존 제거 완료는 아니다.

## 2. 검증 계획과 한계

`output/7280/stage26/verify-decoration-host.mjs`로 저장 줄 helper, 계획에서 복원한 기존 분기/항목/높이,
지연 profile 조회, 조정자 인자, 상태 push/advance와 부모의 다른 경로/테스트 불변을 대조한다.
순수 높이 계산이 push 앞으로 이동한 사실은 별도로 표시한다. 정적 대조는 실행 검증의 대체물이 아니다.

별도 review worktree에서 파생 suite 준비 → 고정 baseline 대비 manifest/unit-tier → fmt →
native Clippy → 집중 nextest를 순차 수행한다. R2w 255건에 다음 기존 10건을 더한다.

- `issue_1755_host_heading_pre_emit` 2건: 일반 지연 표의 제목 소유 대조군과 합성 decoration host 제목의
  실제 render-tree TextRun 존재. 후자는 텍스트 존재 검사이며 형제 표 전체 기하 동일성 증거가 아니다.
- `issue_7047_textless_host_ladder_spacing` 5건: 실제 임대차 서식의 글상자·제목 표 상단,
  남은 상단 편차 범위, 제목/글상자 비겹침과 로고 정렬. 기존 테스트가 참조하는 한컴 PDF 좌표와
  이번 assertion 실행을 구분하며 이번 절편에서 PDF/화면을 직접 대조한 것으로 보고하지 않는다.
- `issue_703` 3건: 장식 표의 흐름 비소비 및 과거 함께 분류된 표/각주 문서 대조군.
  페이지 수만 검사하므로 세부 기하·텍스트 소유권 검증으로 격상하지 않는다.

모든 공백/제어문자 조합, 줄 수 0, 여러 decoration 표와 일반 표 혼재를 독립 fixture로 추가하지 않는다.
해당 세부 조합은 정적 보존 대조와 기존 집중 범위 이상으로 실행 입증했다고 주장하지 않는다.
전체 회귀·WASM/workspace lint·workspace build·Native Skia·fresh Docker WASM 및 직접 시각 대조는
구현계획 §7의 통합 게이트에 남긴다. 원격 push·PR·댓글은 이번 승인 범위가 아니다.

## 3. 고정 head 검증

제품 커밋 후 실제 결과를 기록한다. 예정 265건을 실행 결과로 간주하지 않는다.

## 4. 후속

일반 표 문단의 장식 표 배치·일반/TAC 표 배치 선택 등 잔여 조정 책임을 이어서 분리한다.
표 컷/continuation, 각주, 최종 상태 캡슐화와 전체 통합 검증은 남아 있다.
