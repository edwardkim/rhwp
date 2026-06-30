# 최종 보고서 — #1707 행/열 바꿈 복사/붙여넣기 명칭 정리

- 이슈: edwardkim/rhwp#1707
- 브랜치: `local/task_m100_1707-rename-transpose-copy`
- 기준: `upstream/devel`
- 성격: #1634 후속 UI/문서 명칭 정리

## 1. 변경 내용

#1634에서 구현된 표 셀 transpose 기능의 사용자 노출 명칭을 다음 기준으로 정리했다.

- `셀 전치 복사` → `행/열 바꿈 복사`
- `셀 전치 붙여넣기` → `행/열 바꿈 붙여넣기`

내부 API 식별자(`transpose`, `copyTableCellsTransposed` 등)는 기존 WASM/Studio 계약을 유지하기 위해 변경하지 않았다.

## 2. 반영 범위

- Studio 메뉴 HTML과 command label/error context
- Studio 정적 테스트의 메뉴 라벨 기대값
- `DocumentCore`, WASM API, table model의 사용자 오류 메시지와 공개 주석
- 활성 구현 계획서 `mydocs/plans/task_m100_1634_impl.md`

역사 보관 문서와 내부 함수명은 변경하지 않았다.

## 3. 검증

- `git diff --check`
- `CARGO_INCREMENTAL=0 cargo fmt --check`
- `npm test -- tests/menu-shortcut-labels.test.ts`
- `CARGO_INCREMENTAL=0 cargo test transpose -- --nocapture`
- 대상 범위 검색: `셀 전치`, `전치 복사`, `전치 붙여넣기`, `표 전치`, `전치된`, `전치할` 잔여 없음

## 4. 결론

기능 동작은 유지하고, 사용자에게 노출되는 명칭을 #1707 기준으로 정리했다.

## 5. PR 및 후속 처리

- PR: edwardkim/rhwp#1709
- merge commit: `8461b0ad2b257163266eac9ece3f30dc70f5073c`
- merge 시각: 2026-06-30 23:38 KST
- 최신 head `61cd8adc29a8266e1fc3f5aa1ba76475313ea958` 기준 GitHub Actions 통과
  - Build & Test
  - CodeQL
  - Canvas visual diff / Render Diff preflight
- #1707은 `Closes #1707` 자동 close가 동작하지 않아 수동 close 처리했다.
