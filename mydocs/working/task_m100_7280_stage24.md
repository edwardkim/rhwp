# Task #7280 Stage 24 — R2v 표 문단 동반 개체 흐름 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2u](task_m100_7280_stage23.md), 시작 head `f88bdc150`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R2v 구조 이동 완료, 고정 제품 SHA 집중 검증 예정.

## 1. 책임 경계와 보존 계약

일반 `typeset_table_paragraph`의 Shape/Picture/Equation match arm을 대상으로 한다.
`controls/shape_flow.rs`는 기존 TAC 줄 높이·비TAC 자리차지 높이 및 이월 조건을 조회한다.
`controls::place_table_host_shape`는 이미 배치된 개체 제외 → 높이 조회 → 필요 시 단/쪽 이동 →
항목·높이 확정 → 어울림 등록 순서만 조정한다. `state.rs`는 조회용 관측값과 항목·높이 확정을 맡는다.
공개 API·IR·출력 타입은 늘리지 않으며 새 타입의 가시성은 typeset 내부로 제한한다.

### 실제 호출과 소비

- match arm의 `continue`는 coordinator의 `return`으로 대응한다. arm 뒤 loop 말미에 추가 명령이
  없으므로 동일 개체의 재배치만 건너뛰며 전체 문단 처리를 중단하지 않는다.
- 원본 컨트롤 인덱스로 앞 TAC 개체 수를 세고 해당 저장 LineSeg를 선택하는 기존 계산을 유지한다.
  이 가정의 정확성을 새로 승인하거나 재조판 알고리즘으로 대체하는 절편이 아니다.
- 두 `Option<f64>`를 하나의 높이로 합치지 않는다. TAC 높이가 있으면 먼저 사용하고, 없을 때만
  비TAC TopAndBottom/Para 조건으로 선언 높이+아래여백을 계산한다. `None`과 `Some(0)`도 보존한다.
- TAC는 현재 항목 존재, 비TAC는 `current_height < 1.0` 단 상단 여부를 선행 조건으로 사용한다.
  이를 같은 조건으로 통일하지 않는다. `available_height`는 closure로 기존 단락 평가 시점에 조회한다.
- 이월 후 새 페이지에 `PageItem::Shape`를 추가하고, 계획의 우선순위대로 현재 높이를 증가시킨다.
  다음 `register_side_wrap_picture`에는 기존 `Some(para_start_height)`를 전달한다.
  페이지 이동 뒤 문단 원점을 재계산하지 않는다.
- Equation은 기존처럼 두 높이 모두 없는 경로를 유지하지만 항목 추가·어울림 등록 호출은 생략하지 않는다.
- 실제 높이 소비 뒤에 수행되는 TAC 문단 높이 보정과 중복된 줄 높이 계산은 부모에 그대로 남긴다.
  이번 조회 결과를 그 보정에 새로 연결하거나 cap을 변경하지 않는다.
- 페이지 전환과 `register_side_wrap_picture` 구현은 기존 TypesetState에 남아 있다.
  이 절편은 모든 상태 필드 캡슐화나 모든 그림/도형 배치 경로의 통합 완료가 아니다.

조판 산식·상수·선택 순서, 테스트 source/assertion/ID, baseline/golden/ignore, CI 정책을 바꾸지 않는다.
표 컷·continuation, 줄 소속 규칙 개선과 시각 결함 수정도 범위 밖이다.

## 2. 검증 계획과 한계

`output/7280/stage24/verify-shape-flow.mjs`로 높이 식, 서로 다른 이월 guard와 지연 예산,
항목→높이→어울림 순서, coordinator 인자와 기존 부모/상태/테스트의 불변을 원본과 대조한다.
정적 비교는 실행 검증의 대체물이 아니다.

별도 review worktree에서 파생 suite 준비, 고정 baseline 대비 manifest/unit-tier,
fmt, native Clippy, 집중 nextest를 순차 실행한다. R2u 245건에 기존 계약 5건을 추가한다.

- `issue_3738_tac_sibling_shape_line_advance` 2건: 합성 저장 줄 입력의 후속 본문 18줄 보존과 SVG 쪽 밖 출력 방지.
  정상 한컴 생성본/출력 일치 증거와는 구분한다.
- `issue_1156_chart_column_flow` 2건: 실제 2단 문서의 차트 단 이동·뒤 텍스트 비겹침과 배경 투명도 대조군.
- `issue_6146_page_tail_float_band_spill` 1건: 떠나는 쪽 개체 소유와 이어지는 쪽의 중복 배치 방지.

기존 #6812 어울림 그림/TAC 표 계약도 유지한다. 수식 전용 입력이나 모든 guard 경계의 독립 실행 추적을
새로 추가하지 않는다. 실제 backend 전체 일치나 직접 시각 판독 완료를 주장하지 않는다.
전체 회귀·WASM/workspace lint·workspace build·Native Skia·fresh Docker WASM 및 직접 시각 대조는
통합 게이트에 남긴다. 원격 push·PR·댓글은 이번 승인 범위가 아니다.
