# Task #7280 Stage 30 — R2ab 표 배치 뒤 각주 연결 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2aa](task_m100_7280_stage29.md), 시작 head `75b37eb89`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: 구현 완료, 고정 제품 SHA 집중 검증 준비.

## 1. 책임과 보존 범위

일반 표 문단 루프와 지연 표 배치의 중복 각주 연결을 `typeset/notes.rs`로 옮긴다.
이는 R2의 컨트롤 배치 후 연결 경계이며 R4의 각주 높이/예약/쪽 배분 알고리즘 분리가 아니다.
`notes`는 큐 소유권을 한 번 확인하고 직접 셀 각주를 순서대로 등록한다.
`state::has_fragment_queued_table_footnotes`는 기존 집합의 membership만 읽는다.
부모 엔진은 기존 등록 메서드를 callback으로 제공한다. 새로운 trait·public API·IR은 없다.

- 표 배치 완료 → 큐 소유권 확인 → cell/paragraph/control 순서의 직접 각주 등록을 보존한다.
  중첩 표 내부를 새로 재귀 탐색하거나 Endnote를 등록하지 않는다.
- guard는 진입 시 한 번만 평가한다. 각주 하나의 등록이 새 쪽을 만들 수 있으므로
  각주 목록을 미리 계획하거나 높이를 일괄 예약하지 않고 매번 즉시 기존 메서드를 호출한다.
- `FootnoteSource::TableCell`의 문단/표/셀/셀 문단/셀 컨트롤 인덱스는 원본과 같다.
- 일반 경로는 각주 등록 뒤에 `break_after_current_table`을 판단한다.
  지연 경로는 각주 등록 뒤에 원래 문단 앵커를 확정한다.
- 장식 표의 continue, stored TAC 조기 반환, host Body 각주, fragment 큐 등록,
  각주 높이 추정·쪽 배분 본체는 변경하지 않는다.

조건·수치·출력·기존 알고리즘의 타당성을 바꾸거나 새로 승인하지 않는다.
Rust 테스트·assertion·baseline·golden·ignore·CI 변경은 없다.

## 2. 검증 계획과 한계

`output/7280/stage30/verify-table-notes.mjs`에서 새 coordinator를 양쪽 호출자에 다시 펼쳐
시작 head와 비교한다. guard/순회/인덱스/등록 위치와 나머지 부모 코드 불변,
상태 query의 읽기 전용 식을 확인한다. 실행 증거나 시각 판정을 대체하지 않는다.

별도 review worktree에서 suite 준비, 고정 baseline 대비 manifest/unit-tier 검사,
fmt, native Clippy, 집중 nextest를 순차 수행한다.
R2aa 281건을 유지하고 기존 `issue_3738_rowbreak_table_footnote_fragment` 모듈 전체를 선택한다.
이 모듈의 원본 각주 쪽 소유권·분할·표/본문/각주 경계 검사와 입력을 변형한 계약을 구분한다.
통과 결과를 모든 일반/지연 각주 조합의 분기 실행 증명이나 한컴 직접 시각 일치로 격상하지 않는다.

전체 회귀·WASM/workspace lint·workspace build·Native Skia·fresh Docker WASM 및 직접 시각 대조는
구현계획 §7의 통합 게이트에 남긴다. 원격 push·PR·댓글은 이번 승인 범위가 아니다.

## 3. 고정 head 검증

검증 완료 후 제품 SHA·명령·결과·증적을 기록한다.

## 4. 후속

표 문단 루프의 잔여 진입/배치/후처리 조정 경계를 점검한다.
표 포맷/TAC·블록 배치/이어받기 본체, 각주 등록 알고리즘과 전체 통합 게이트는 남아 있다.
