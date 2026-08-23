# Task M100 — #5769 후속 1: Shape z 순서 역연산화

- 일자: 2026-08-23
- 브랜치: `feat/5769-zorder-inverse` (#5915 스택)
- 이슈: edwardkim/rhwp#5769 "다음 작업 대상 1. z순서 4곳"

## 무엇을 했나

z 순서 변경(정렬 메뉴 front/forward/backward/back 4모드 + 개체 선택 시 자동
맨앞)을 스냅샷(슬롯 1)에서 **속성쌍 역연산 커맨드**(슬롯 0)로 옮겼다.

### 근거 — 참인 역연산 판정(#5769 선결 규약)

`change_shape_z_order_native` 의 변경 실체는 대상(+교환 이웃)의
`common.z_order` **스칼라 대입 1~2개**뿐이다(`object_ops/shape.rs`). 대입 외
부작용이 없으므로 속성쌍 왕복은 참인 역연산이다 — 그림 회전(`rotate_image`
되돌리기 부재)과 달리 선결 규약을 통과한다.

### 구현

**Rust**
- `change_shape_z_order_native` 응답 확장: 실제 변경 시
  `"moves":[{ppi,ci,before,after},…]`(대상+교환 이웃)를 함께 돌려준다. 무변경
  경계("이미 맨 앞/뒤")는 종전대로 moves 없이 조기 반환하며 raw 도 건드리지
  않는다. 기존 소비자는 `zOrder` 키만 읽으므로 추가 필드는 안전하다.
- `apply_shape_z_order_pairs_native(section_idx, pairs_json)` 신설: 절대 대입.
  전수 검증 후 적용(부분 적용 금지), 빈 pairs 는 무효화도 없는 깨끗한 no-op.
  wasm 바인딩 `applyShapeZOrderPairs`.

**TS**
- `SetZOrderCommand`(command.ts): 최초 execute 는 기존 상대 연산을 실행해
  moves 를 받고(단일 출처), redo 는 저장된 after 쌍 절대 대입 — 상대 연산은
  멱등하지 않으므로(front=항상 max+1) 재실행하지 않는다. undo 는 before 쌍
  대입 뒤 `restoreSectionRaw`. 생명주기는 SetSectionPropsCommand 와 동일
  (capture→적용 / old 재적용→복원). `snapshotResourceCount()=0`, 빈 moves =
  isNoOp(phantom 엔트리·캡처 낭비 없음).
- 배선: insert.ts 정렬 메뉴 퍼널 4곳 + input-handler-mouse bringShapeToFront
  를 kind:'command' 로 재배선. 양식 모드 게이트는 command 라우터가 execute
  보다 먼저 도는 기존 계약 유지(#3230).
- 레지스트리: `applyShapeZOrderPairs` 를 MUTATING_METHODS 에 분류(#2327).

### 검증

- **Rust 게이트** `tests/cases/issue_5769_zorder_inverse_byte_identity.rs` 6종:
  forward 교환 undo 바이트 수렴 / front 단일 inverse 수렴 / redo 재현 /
  무변경 경계(moves 없음·바이트 무흔적) / 오염 pairs 거절(부분 적용 없음) /
  빈 pairs no-op. — **6/6 통과**
- **#2724 가드**: 새 네이티브는 위임 대상이 직접 무효화(raw_stream=None)라
  EXEMPT 불필요 — 가드 5/5 통과로 확인.
- clippy `-D warnings` clean, `cargo fmt --check` 통과.
- **npm test 1072 pass / 0 fail**. 구형 가드 3건(스냅샷 구현 고정분)을 새
  계약으로 의식 갱신: undo-drag-click-routing(mouse 경로),
  undo-menu-object-ops(changeShapeZOrder 목록 제외+전용 가드 추가),
  undo-noop-skip(무변경 판정 주체가 TS 값 비교→Rust moves 로 이동).
  신규 소스 가드 `issue-5769-zorder-inverse.test.ts` 4종 추가.
- tsc: 내 파일 신규 에러 없음(stale d.ts 3건은 Stage 4 선행 노이즈).

### 슬롯 효과

changeZOrder 엔트리 스냅샷 1슬롯 → **0**. 개체 선택 클릭마다 자동 front 가
붙는 실사용 패턴에서 특히 체감이 크다(클릭 1회당 엔트리 1개).

### 남은 것 / 메모

- manifest 등록: `--adopt-new` 로 regression_suite_004 에 흡수됨
  (tests/suites/manifest.json diff). 로컬 `--check` 드리프트(suite-policy.json
  대비 generated 잔재)는 본 변경 이전 상태 — CI prepare 가 자체 재생성.
- 다구역 z 교차(서로 다른 구역 shape 간 정렬)는 현재 네이티브가 구역 내부만
  지원 — 필요 시 별도 과제.
