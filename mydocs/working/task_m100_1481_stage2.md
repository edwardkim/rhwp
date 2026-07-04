# Task M100 #1481 Stage 2

- 이슈: #1481 표 줄/칸 편집 회귀 및 한컴식 줄/칸 메뉴 정합
- 브랜치: `task_m100_1481`
- 작성일: 2026-06-22
- 상태: 구현 및 검증 완료. 커밋 전 확인 대기.

## 목표

Stage 1에서 추가한 `줄/칸 추가하기`, `줄/칸 지우기` 대표 대화상자 기능을 한컴 메뉴 표기와
단축키 표시 기준에 맞추고, Stage 1에서 빠진 행 추가/삭제 높이 회귀까지 보정한다.

## 범위

- 상단 표 메뉴에서 개별 빠른 실행 항목 대신 한컴식 대표 항목을 표시한다.
  - `줄/칸 추가하기(I)...`
  - `줄/칸 지우기(E)...`
- 메뉴 단축키 표시는 #1477 기준에 맞춰 플랫폼별로 표시한다.
  - macOS: `⌥Insert`, `⌥Delete`
  - Windows/Linux: `Alt+Insert`, `Alt+Delete`
- 우클릭 컨텍스트 메뉴의 줄/칸 구조 편집 항목도 대표 항목으로 표시한다.
- macOS 브라우저가 `Option+Insert`를 `Help`/`code=Insert`로 전달하는 경우도 처리한다.
- 일반 표 행 추가/삭제 후 외곽 높이가 셀 저장 height 합으로 붕괴하지 않게 보정한다.
- Stage 1에서 구현한 실제 대표 대화상자 기능과 단축키 매핑은 유지한다.

## 검증 계획

```bash
cd rhwp-studio && npx tsc --noEmit
cd rhwp-studio && node --test tests/navigation-keymap.test.ts tests/menu-shortcut-labels.test.ts tests/shortcut-map.test.ts
cd rhwp-studio && npm test
cargo fmt --check
cargo test --release issue_1481 --lib
wasm-pack build --target web --out-dir pkg
git diff --check
```

## 구현 결과

- `rhwp-studio/index.html`
  - 상단 표 메뉴의 줄/칸 구조 편집 항목을 한컴식 대표 메뉴 2개로 정리했다.
  - `줄/칸 추가하기(I)... Alt+Insert`
  - `줄/칸 지우기(E)... Alt+Delete`
- `rhwp-studio/src/command/commands/table.ts`
  - 대표 명령 label을 메뉴 표기와 맞췄다.
- `rhwp-studio/src/engine/navigation-keymap.ts`
  - 표 줄/칸 대표 단축키를 #1477 플랫폼별 표시 규칙에 맞춰 macOS에서는 `⌥Insert`, `⌥Delete`로 표시한다.
- `rhwp-studio/src/engine/input-handler.ts`
  - 표 셀 우클릭 컨텍스트 메뉴의 개별 줄/칸 추가·삭제 항목을 대표 메뉴 2개로 정리했다.
- `rhwp-studio/src/command/shortcut-map.ts`
  - macOS `Option+Insert`가 `Help`/`code=Insert`로 들어오는 브라우저 이벤트 변형도 대표 추가 명령으로 매핑했다.
- `src/model/table.rs`
  - 일반 표의 셀 저장 height 합보다 큰 외곽 표시 height를 행 추가/삭제 후에도 보존·증감하도록 보정했다.
- `src/wasm_api/tests.rs`
  - 행 추가/삭제 후 일반 표 외곽 height가 납작해지지 않는 회귀 테스트를 추가했다.
- `rhwp-studio/tests/navigation-keymap.test.ts`
  - 한컴 표 줄/칸 단축키 플랫폼별 표시 테스트 추가.
- `rhwp-studio/tests/menu-shortcut-labels.test.ts`
  - 상단 표 메뉴와 컨텍스트 메뉴 대표 항목/단축키 표시, 개별 항목 제거를 고정.
- `rhwp-studio/tests/shortcut-map.test.ts`
  - macOS `Help`/`code=Insert` 이벤트 변형 매핑을 고정.

## 검증 결과

```bash
cd rhwp-studio && npx tsc --noEmit

cd rhwp-studio && node --test tests/navigation-keymap.test.ts tests/menu-shortcut-labels.test.ts tests/shortcut-map.test.ts
# 30 passed

cd rhwp-studio && npm test
# 120 passed

cargo fmt --check

cargo test --release issue_1481 --lib
# 4 passed

wasm-pack build --target web --out-dir pkg

IAB http://localhost:7700/
# 상단 표 메뉴/표 셀 우클릭 메뉴: `줄/칸 추가하기(I)...`, `줄/칸 지우기(E)...`
# macOS 단축키 표시: `⌥Insert`, `⌥Delete`
# Windows/Linux 표시 원본: `Alt+Insert`, `Alt+Delete` 테스트 고정
# 일반 표 `위쪽에 줄 추가하기` 후 외곽 height 증가 회귀 테스트 통과

git diff --check
```
