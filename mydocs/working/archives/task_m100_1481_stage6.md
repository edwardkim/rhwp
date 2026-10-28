# Task M100 #1481 Stage 6

- 이슈: #1481 자리차지 표 생성/Enter 동작 한컴 정합 보정
- 브랜치: `task_m100_1481`
- 작성일: 2026-06-23
- 상태: 구현/검증 완료

## 증상

Stage 5에서 첫 조판부호가 표 상단과 겹쳐 보이도록 렌더러와 생성 경로를 보정했지만, 자리차지 표의 표 앞 조판부호 위치에서 `Enter`를 누르는 동작이 아직 한컴과 다르다.

현재 rhwp에서는 표 앞/상단 조판부호 위치에서 `Enter`를 누를 때 빈 문단이 표 위에 쌓이거나, 표가 아래로 밀린 것처럼 보이는 경로가 남아 있다.

한컴 기준 동작은 다음과 같다.

- 표 생성 직후 첫 조판부호는 표 상단 좌측과 겹쳐 보인다.
- 빈 문단 끝 조판부호 위치(`charOffset=1`)에서 표를 만들어도 표 위에 생성 경로의 빈 줄이 남으면 안 된다.
- 표는 `자리 차지` 객체이므로 표 앞 조판부호 위치에서 `Enter`를 누르면 새 조판부호가 표 아래에 생긴다.
- 반복 `Enter`도 표 위쪽에 빈 줄을 쌓지 않고 표 아래쪽 문단 흐름을 늘린다.
- 표 생성 직후 아래쪽 탈출용 빈 문단 조판부호도 유지되어야 한다.
- 마지막 셀에서 `Tab`을 누르면 표 밖으로 빠지는 대신 한컴처럼 아래쪽에 새 줄이 자동 추가되고 새 줄 첫 셀로 이동해야 한다.

## 검토 대상

- `SplitParagraphCommand` / `split_paragraph_native()`에서 표 host 문단의 `charOffset=0` 분할 처리
- 비-TAC `TopAndBottom + VertRelTo::Para` 표가 있는 빈 host 문단의 Enter 특수 처리
- 표 생성 직후 커서 위치와 `Enter` 처리 위치가 한컴의 표 앞 조판부호 위치와 일치하는지 확인
- 표 뒤 빈 문단 유지 정책과 반복 Enter 시 문단 삽입 위치
- 렌더러의 host 문단부호 직접 렌더링과 문단 분할 결과의 중복/순서 정합
- 상세 표 만들기 경로의 빈 문단 끝 offset이 기존 빈 문단을 남기고 표를 다음 문단에 삽입하는지 여부
- Studio 새 문서가 사용하는 `blank2010.hwp` 템플릿의 첫 문단이 SectionDef/ColumnDef 구조 컨트롤을 포함해 일반 `Paragraph::new_empty()`와 다르게 판정되는지 여부
- 셀 내부 `Tab` 처리에서 마지막 셀 여부 판정과 자동 줄 추가 후 커서 이동

## 구현 방향

- 자리차지 표 host 문단 시작 위치에서 Enter가 들어오면 일반 문단 분할 대신 표 뒤 문단 삽입으로 라우팅한다.
- 표 위에는 새 빈 문단이 생기지 않게 하고, 표 아래 빈 문단 또는 새 문단으로 커서를 이동한다.
- 표 생성 직후 렌더링은 한컴처럼 표 상단과 첫 조판부호가 겹치고, 표 아래 조판부호가 유지되도록 한다.
- 기존 TAC 표, 일반 텍스트 문단, 글상자/셀 내부 Enter 동작은 영향을 주지 않는다.

## 구현 내용

- `split_paragraph_native()`에서 `charOffset=0`이고 문단이 비-TAC `TopAndBottom + VertRelTo::Para` 표 host인 경우를 감지한다.
- 해당 케이스는 `Paragraph::split_at(0)`으로 표를 새 문단으로 이동시키지 않고, 표 문단 바로 뒤에 빈 문단을 삽입한다.
- 새 빈 문단은 표 host 문단의 문단/글자 모양과 줄 폭을 상속한다.
- 반환 JSON은 기존 Enter 명령과 동일하게 `paraIdx=<표 뒤 새 문단>, charOffset=0`을 유지한다.
- 회귀 테스트에서 표 앞 조판부호 위치 Enter 후에도 표가 기존 host 문단에 남고, 새 빈 문단이 표 아래에 생기는지 검증한다.
- `create_table_native()`는 빈 문단이면 UI에서 넘어온 `charOffset` 값과 무관하게 현재 문단을 표 host로 교체한다.
- `blank2010.hwp` 첫 문단처럼 텍스트 없이 SectionDef/ColumnDef 구조 컨트롤만 가진 문단은 구조 컨트롤을 보존한 채 같은 문단에 표 컨트롤을 병합한다.
- 이 경우 표 host line segment는 표 문단 기준으로 교체하고, 반환 `controlIdx`는 구조 컨트롤 뒤의 실제 표 컨트롤 인덱스로 반환한다.
- 사용자가 별도로 만든 표 위 빈 문단은 보존하고, 현재 삽입 대상 빈 문단만 표 host로 교체하는 회귀 테스트를 추가한다.
- 실제 프론트 출력에 가까운 PageLayerTree `textControlMark` 좌표도 함께 검사해 표 생성 경로의 빈 줄 잔존을 방지한다.
- 빈 문단의 stale/초과 `charOffset`에서도 표 위 생성 경로 빈 줄이 남지 않도록 회귀 테스트를 추가한다.
- Studio 새 문서 템플릿 기반 생성 경로에서도 표 위 생성 경로 빈 줄이 남지 않고 SectionDef/ColumnDef 뒤에 표가 보존되는지 회귀 테스트를 추가한다.
- 프론트 키보드 핸들러에서 일반 셀 `Tab`은 기존 셀 이동을 유지하되, 마지막 셀 `Tab`만 `insertTableRow` snapshot 편집으로 라우팅한다.
- 마지막 셀 `Tab`으로 추가한 줄은 새 줄의 첫 셀 시작 위치로 커서를 이동시킨다.

## 검증 계획

```bash
cargo fmt
cargo test --release issue_1481 --lib
cargo test --release document_core::commands::object_ops::issue_1151_v2_tac_toggle_tests::v6_resize_cell_then_tac_toggle_picture_below_table --lib
cd rhwp-studio && node --test tests/table-keyboard-navigation.test.ts tests/table-create-dialog.test.ts
cd rhwp-studio && npm run build
cd rhwp-studio && npm test
wasm-pack build --target web --out-dir pkg
cargo test --release --lib
git diff --check
```

## 검증 결과

- `cargo fmt`: 통과
- `cargo test --release issue_1481 --lib`: 통과 (9 passed)
- `cargo test --release document_core::commands::object_ops::issue_1151_v2_tac_toggle_tests::v6_resize_cell_then_tac_toggle_picture_below_table --lib`: 통과
- `cd rhwp-studio && node --test tests/table-keyboard-navigation.test.ts tests/table-create-dialog.test.ts`: 통과 (4 passed)
- `cd rhwp-studio && npm run build`: 통과
- `cd rhwp-studio && npm test`: 통과 (124 passed)
- `wasm-pack build --target web --out-dir pkg`: 통과
- `cargo test --release --lib`: 통과 (1923 passed, 6 ignored)
