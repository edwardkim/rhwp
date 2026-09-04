# Task M100 #1470 stage1 완료 보고서

- 이슈: #1470 `스타일 적용/편집 불일치: 왼쪽 여백 배율, 줄간격 미반영, 표 캡션/생성 위치 문제`
- 수행계획서: `mydocs/plans/task_m100_1470.md`
- 구현계획서: `mydocs/plans/task_m100_1470_impl.md`
- 작업 브랜치: `task_m100_1470`
- 작성일: 2026-06-22

## 1. 절차 메모

초기 진행에서 Hyper-Waterfall 순서를 어기고 구현 draft를 먼저 작성한 오류가 있었다.
이후 수행계획서와 구현계획서를 작성하고 작업지시자 승인을 받은 뒤, 기존 draft를 승인된 구현 범위에 맞춰 검토하고 검증했다.

## 2. 구현 요약

### 스타일 적용/편집 재흐름

- 스타일 적용 후 본문 문단의 `LineSeg`를 현재 CharShape/ParaShape 기준으로 다시 계산한다.
- 셀 문단 스타일 적용 후 셀 문단 reflow와 표 dirty 마킹을 수행한다.
- 스타일 편집 API는 해당 스타일을 사용하는 본문 문단과 표 셀 문단을 찾아 shape ID 갱신 후 reflow한다.
- 일반 스타일 적용 시 기존 참조 문단의 실효 ParaShape보다 스타일 정의의 ParaShape를 우선 사용한다.
- 번호/개요 문단의 번호 문맥 보존 경로는 유지했다.

### Studio 스타일 UI/API

- `createStyle` JSON에 optional `baseParaShapeId`, `baseCharShapeId`를 추가했다.
- 스타일 추가 대화상자는 현재 커서의 문단/글자 속성을 초기값으로 사용한다.
- 블록 선택 상태에서 스타일 적용 시 기존 문단 서식 target 수집 로직을 재사용해 여러 문단에 적용한다.
- 다중 스타일 적용은 snapshot command로 묶어 Undo/Redo 안정성을 우선했다.

### 표 생성 옵션

- 표 만들기 상세 대화상자의 `글자처럼 취급`, 직접 지정 너비, 직접 지정 높이를 옵션으로 전달한다.
- `WasmBridge.createTableEx` 래퍼를 추가했다.
- Rust `createTableEx`는 `colWidths`, `rowHeights` 배열을 처리한다.
- native 표 생성은 optional row heights를 반영해 표 높이와 셀 높이를 설정한다.
- 단순 그리드 피커는 기존 `createTable` 경로를 유지했다.

### 표 캡션

- 표 캡션 생성 시 AutoNumber 컨트롤을 literal `"표 N "` 텍스트로 치환하지 않도록 변경했다.
- 캡션 문단은 AutoNumber inline 컨트롤 모델을 유지한다.
- `Caption` 스타일이 있으면 캡션 문단 style/shape ID로 사용한다.
- `hasCaption=false`가 기존 캡션을 삭제하고 attr bit 29를 해제하도록 보정했다.

## 3. 추가 테스트

`src/wasm_api/tests.rs`에 #1470 focused 테스트를 추가했다.

- `issue_1470_style_update_reflows_and_keeps_margin_unit`
- `issue_1470_create_table_ex_applies_size_options`
- `issue_1470_table_caption_keeps_autonumber_and_can_be_removed`

## 4. 검증 결과

통과:

- `git diff --check`
- `cargo fmt --check`
- `cargo test --release issue_1470 --lib`
- `cargo test --release --test issue_1172_para_margin_roundtrip`
- `cargo test --release test_paste_picture_into --lib`
- `wasm-pack build --target web --out-dir pkg`
- `cd rhwp-studio && npm run build`

참고:

- `wasm-pack build` 중 현재 플랫폼용 prebuilt `wasm-bindgen`을 받지 못해 `cargo install` fallback을 사용했지만, 빌드는 정상 완료됐다.
- `rhwp-studio` 빌드는 기존과 같은 Vite chunk size warning을 출력했지만 실패하지 않았다.
- `cargo clippy --all-targets -- -D warnings`는 사용자의 중지 지시에 따라 실행을 중단했으며, 이번 stage 검증 완료 항목에 포함하지 않는다.

## 5. 남은 위험

- 스타일 적용의 `ParaShape` 결정 기준 변경은 번호/개요 문단과 일반 문단의 경계에서 추가 표본 검증이 필요할 수 있다.
- 표 캡션 AutoNumber 유지 변경은 HWP/HWPX 직렬화 라운드트립 전체 검증 전에는 잔여 위험이 있다.
- 다중 스타일 적용은 snapshot command를 사용하므로 정확성은 우선 확보했지만, 대규모 선택 범위에서 메모리 사용량 검토가 남아 있다.

## 6. 다음 단계 제안

작업지시자 검토 후 다음 중 하나로 진행한다.

1. stage1 변경 범위를 유지하고 PR 준비 검증으로 이동
2. 특정 항목을 분리해 stage2로 좁혀 추가 보정
3. 현재 구현 draft 중 일부를 폐기하고 문서 계획을 재조정
