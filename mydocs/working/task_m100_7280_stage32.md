# Task #7280 Stage 32 — R2ad 개별 지연 표 배치 조정 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2ac](task_m100_7280_stage31.md), 시작 head `1d8259f3a`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: 구현 완료, 고정 제품 SHA 집중 검증 준비.

## 1. 책임과 보존 범위

`place_deferred_table_control`의 본체를 `controls/deferred_placement.rs`로 옮긴다.
기존 `deferred.rs`는 후보/flush 시점의 읽기 전용 판별을 계속 맡고,
새 모듈은 선택된 표 하나의 재조회·포맷·블록 배치·각주·앵커 확정을 연결한다.
큐 순회와 인출/복원은 변경하지 않는다. 부모는 기존 시그니처의 얇은 진입점이다.

기존 엔진의 포맷·블록 배치·각주 등록을 호출하는 과도기 조정자다.
이 알고리즘 본체나 엔진 의존을 제거했다고 주장하지 않는다.
기존 인자를 그대로 전달하며 새 input 타입·trait·public API·IR·범용 rule engine은 만들지 않는다.

- 문단 조회 실패 또는 해당 컨트롤이 표가 아닌 경우의 조기 반환은 그대로다.
  이 경우 너비/높이 조회나 포맷·배치·앵커 쓰기를 새로 하지 않는다.
- 현재 단 너비 조회 → 해당 문단 구성 결과 조회 → 문단 포맷 → 지연 후보 재검증 순서다.
  `deferred_table_column_width`는 읽기만 한다. 일반 표 문단 진입용
  `prepare_table_paragraph_column`을 재사용해 float 배타 영역을 추가 소비하지 않는다.
- 재검증에 성공한 뒤에만 현재 높이 `< 1.0` 조회 → 표 포맷 → 측정 표 검색 → 현재 높이 조회를 한다.
  높이를 포맷 전에 미리 저장하거나 측정 결과를 재생성하지 않는다.
- 현재 `para_start_height`는 렌더 앵커로, 큐에 저장된 `deferred.para_start_height`는
  원 배치 시점의 분할 예산 앵커로 각각 원래 인자 위치에 전달한다. 두 값을 합치지 않는다.
- 블록 배치 → 직접 셀 각주 등록 → vpos 앵커 확정 순서와 first/last 표 플래그를 보존한다.
  각주가 쪽을 바꿀 수 있으므로 앵커 확정을 앞당기지 않는다.
- 기존 getter `flow_table_column_top`의 식은 변경하지 않고 재사용한다.
  두 신규 state getter는 단 너비와 현재 높이만 읽으며 상태를 쓰지 않는다.

기존 수치·판별·포맷/분할 알고리즘의 타당성을 변경하거나 새로 승인하지 않는다.
Rust 테스트·assertion·baseline·golden·ignore·CI 변경은 없다.

## 2. 검증 계획과 한계

`output/7280/stage32/verify-deferred-placement.mjs`는 신규 getter를 배치 조정자에 펼쳐
시작 head 본체와 대조한다. 인자/앵커 구분, 조기 반환과 등록/확정 순서,
나머지 부모/큐/기존 state·controls 불변을 확인한다.
정적 대조는 실행 증명이나 출력 시각 판정을 대체하지 않는다.

별도 review worktree에서 suite 준비 → 고정 baseline 대비 manifest/unit-tier → fmt →
native Clippy → R2ac와 동일한 313건 집중 nextest를 순차 수행한다.
기존 선택은 형제 지연 표·TAC/float 혼재·후속 문단·각주 계약을 포함한다.
잘못된 큐 항목의 모든 조기 반환이나 지연 각주 조합을 별도 fixture로 실행한 증거는 아니며,
원본 사례와 합성/변형 계약의 한계는 이전 절편 기록을 따른다.

전체 회귀·WASM/workspace lint·workspace build·Native Skia·fresh Docker WASM 및 직접 시각 대조는
구현계획 §7의 통합 게이트에 남긴다. 원격 push·PR·댓글은 이번 승인 범위가 아니다.

## 3. 고정 head 검증

검증 완료 후 제품 SHA·명령·결과·증적을 기록한다.

## 4. 후속

R2 문단/컨트롤 책임 묶음의 잔여 경계와 통합 검증 범위를 점검한다.
표 포맷·분할·이어받기 본체 및 각주 등록 알고리즘, 최종 상태 캡슐화는 후속 책임 묶음에 남는다.
