# Task M100 #1452 Stage 1 완료 보고서

- 이슈: #1452 `rhwp-studio: 그림 삽입/배치 속성 및 Shift+Tab 내어쓰기 개선`
- 브랜치: `local/task_m100_1452`
- 작성일: 2026-06-21

## 1. 작업 요약

사용자 피드백 중 현재 코드에서 바로 개선 가능한 세 영역을 반영했다.

1. 그림 속성의 `textWrap` 변경이 HWP 저장용 `CommonObjAttr.attr` 비트에 남도록 동기화했다.
2. 그림 삽입 파일 선택/배치 실패 경로가 조용히 사라지지 않도록 토스트 안내를 추가했다.
3. `Shift+Tab` 내어쓰기 계산 기준을 현재 줄 시작점에서 첫 줄 시작점으로 바꾸고, 계산 유틸 테스트를 추가했다.

## 2. 변경 내용

### 2.1 그림 배치 속성 attr 동기화

- `src/document_core/commands/object_ops.rs`
  - `pack_common_attr_bits`를 재사용해 `CommonObjAttr`의 known bit를 현재 IR 필드와 동기화하는 헬퍼를 추가했다.
  - 원본 보존용 미지원 비트는 유지하고, `textWrap`/위치 기준/정렬/겹침/크기 보호 등 저장 의미가 있는 bit만 다시 맞춘다.
  - 그림 속성 적용 경로와 공통 개체 속성 적용 경로에 동기화를 적용했다.
  - `textWrap=InFrontOfText/BehindText/TopAndBottom/Square` 변경 시 attr bit 21-23이 기대값으로 바뀌는 회귀 테스트를 추가했다.

### 2.2 그림 삽입 실패 안내

- `rhwp-studio/src/command/commands/insert.ts`
  - 이미지 파일 준비 중 `Image.onload/onerror`를 모두 처리한다.
  - decode 실패, 0 크기 이미지, 파일 읽기 실패를 토스트로 표시한다.
  - 파일 선택 후 배치 모드 진입 시 클릭/드래그 안내 토스트를 표시한다.
- `rhwp-studio/src/engine/input-handler-table.ts`
  - 배치 클릭 위치의 hit-test 실패 시 배치 모드를 유지하고 재시도 안내를 표시한다.
  - WASM 삽입 실패/예외를 토스트로 표시한다.

### 2.3 Shift+Tab 내어쓰기

- `rhwp-studio/src/engine/input-handler.ts`
  - `Shift+Tab` 내어쓰기 계산 기준을 현재 줄 시작점에서 첫 줄 시작점으로 변경했다.
  - 본문과 1단계 표 셀 문단에서 같은 기준을 사용한다.
- `rhwp-studio/src/engine/hanging-indent.ts`
  - 내어쓰기 px 계산을 순수 함수로 분리했다.
- `rhwp-studio/tests/hanging-indent.test.ts`
  - 첫 줄 기준 계산, 음수 보정, 비정상 좌표 보정을 테스트한다.

### 2.4 주석 정리

- `rhwp-studio/src/core/wasm-bridge.ts`
  - 본문 그림 삽입이 이제 inline이 아니라 sibling floating 기본값이라는 현재 동작에 맞게 주석을 정정했다.

## 3. 검증 결과

- `cargo test --lib issue1452_picture_text_wrap_updates_hwp_attr_bits -- --nocapture`
  - 통과
- `cargo test --lib issue1151_v9_insert_picture_body_floating_default -- --nocapture`
  - 통과
- `cargo test --lib tac_toggle_true_to_false_no_migration_this_pr -- --nocapture`
  - 통과
- `cargo fmt --check`
  - 통과
- `git diff --check`
  - 통과
- `cd rhwp-studio && npx tsc --noEmit`
  - 통과
- `cd rhwp-studio && npm test`
  - 통과, 87개 테스트

## 4. 남은 확인

- PNG 픽셀 알파가 실제 한컴/브라우저 렌더에서 깨지는 재현 파일은 아직 없다.
- 그림 개체 전체 opacity 지원은 모델/포맷 매핑 설계가 필요하므로 이번 단계에서는 포함하지 않았다.
- Shift+Tab의 글상자/각주/머리말·꼬리말 내부 정합성은 기존 미지원 범위로 남겼다.
- 전체 PR 전 검증은 `dev_environment_guide.md`의 macOS 로컬 빌드/테스트 검증 절차에 따라 별도 수행한다.
