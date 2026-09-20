# Task #7280 Stage 25 — R2w TAC 문단 배치 후 높이 정산 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2v](task_m100_7280_stage24.md), 시작 head `c42e55643`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R2w 구현, 고정 제품 SHA 집중 검증 예정. R2 전체 완료나 PR 준비 완료가 아니다.

## 1. 책임 경계와 보존 계약

`typeset_table_paragraph` 끝의 TAC 높이 정산만 분리한다.
`controls/tac_reconcile.rs`는 높이·상한·저장 줄간격 누락 여부와 앵커·최종 하단을 조회한다.
`controls::reconcile_tac_height`는 조회 → 누락 표시/진단/저장 좌표 무효화 → 앵커 및 상한
조회 → 진단 → 최종 하단 조회 → 높이 확정 순서를 조정한다.
`state.rs`는 필요한 관측값과 두 상태 명령을 제공한다. 공개 API·IR·출력 타입은 변경하지 않는다.

- TAC 존재, 양수 문단 높이, 저장 LineSeg 존재, 배치 전후 페이지 수 동일이라는 바깥 guard는
  부모에 남긴다. 앞선 decoration-only host 텍스트 배치도 그대로 둔다.
- 저장 줄 높이와 실측 표 높이 선택, 줄간격 절반 가산, TAC 그림/도형의 선행 TAC 개수와 저장 줄
  대응을 유지한다. 기존 줄 소속 가정의 정확성을 새로 승인하는 변경이 아니다.
- owned rowbreak frame, 문단 앞 간격, 표 위 바깥여백, 저장 스텝·문단 간격 누락 서명과
  1.0/0.5/2.0 임계값을 그대로 둔다. 앵커의 전방 24px 조건도 바꾸지 않는다.
- `TacHeightCap`의 `tac_seg_total`, `cap`, `stored_step_px`, `ladder_total`,
  `ladder_omits_spacing`을 명시적으로 반환한다. 원래 조건과 합산 순서를 유지한다.
- effective TAC 판별에는 engine의 기존 `TacFlowQuery`를 사용하고, HWPX 저장 레이아웃
  판별에는 기존 state profile을 사용한다. 서로 같은 값이라고 가정하여 합치지 않는다.
- `RHWP_DIAG_TACSIB`는 해당 높이 가산 직후 callback으로 실행한다.
  줄간격 누락 명령은 `stored_ladder_spacing_omitted=true` → 진단 callback →
  `vpos_ladder_dirty=true` 순서를 보존한다. callback은 정적 디스패치이며 새 등록 체계가 아니다.
  조회에는 문서/조판 상태 쓰기 권한이 없지만 진단 callback의 외부 효과까지 없는 순수 함수는 아니다.
- 앵커 계산 후 줄간격 누락 상한을 선택하고, 편집 세션 성장량의 `max`를 적용한다.
  TACCAP 진단 뒤 해당 문단의 clearance를 합산하고, 전체 inline placement 유무에 따른
  flow bottom 보정을 유지한다. 현재 높이가 최종 하단보다 클 때만 낮추는 기존 조건을 보존한다.
- 이번 분리는 기존 cap/되감김의 타당성을 재판정하거나 조판 오류를 수정하는 작업이 아니다.
  표 컷·continuation과 나머지 표 문단 조정, 최종 상태 캡슐화는 후속에 남긴다.

조판 산식·상수·선택 순서, 테스트 source/assertion/ID, baseline/golden/ignore, CI 정책은 변경하지 않는다.

## 2. 검증 계획과 한계

`output/7280/stage25/verify-tac-reconcile.mjs`로 새 조회/상태 명령/진단을 기존 순서로 재구성하여
원본과 비교한다. 공백·주석·후행 쉼표와 rustfmt의 단일 표현식 closure 중괄호만 정규화한다.
coordinator 인자·호출 순서, 관측값 필드, 부모 guard/다른 경로/테스트와 나머지 명령의 불변도 검사한다.
정적 비교는 실행 검증이나 모든 진단 환경변수 조합의 실행 증거를 대신하지 않는다.

별도 review worktree에서 파생 suite 준비, 고정 baseline 대비 manifest/unit-tier, fmt,
native Clippy와 집중 nextest를 순차 실행한다. R2v 250건에 아래 기존 5건을 추가한다.

- `issue_2319_no_lineseg_tac_table_height` 2건: LineSeg가 없는 TAC 표의 높이를 잘못 되감지 않는 경로.
- `issue_1835_tac_stale_height` 2건: 저장 높이가 낡은 HWPX 표의 실측 높이와 뒤 문단 비겹침.
  실제 편집 세션에서 성장한 경로를 실행한 증거와는 구분한다.
- `issue_2220_tac_host_line_outer_margin` 1건: TAC host 바깥여백 이중 가산과 오른쪽 단 말미 위치.

기존 #6812 어울림/owned frame, #3738 TAC 동반 개체, #7049/#7062 host 줄 계약도 유지한다.
줄간격 누락 서명의 모든 경계값과 편집 세션 성장 분기의 독립 실행 추적은 이번 절편에 추가하지 않는다.
전체 회귀·WASM/workspace lint·workspace build·Native Skia·fresh Docker WASM과 직접 시각 대조는
통합 게이트에 남긴다. 원격 push·PR·댓글은 이번 승인 범위가 아니다.

## 3. 고정 head 검증

제품 커밋 후 결과를 기록한다. 계획한 255건과 실제 실행 결과는 구분한다.

## 4. 후속

표 문단의 decoration-only 텍스트 배치 등 잔여 흐름 조정 책임을 분리한다.
표 컷/continuation, 각주, 최종 상태 캡슐화와 전체 통합 게이트는 남아 있다.
