# Task M100 #1139 Stage 21

## 목적

Stage20 중간 반영 커밋 이후 남은 `3-09월_교육_통합_2022.hwp` 미주/수식 상호작용 문제를 새 기준선에서 이어서 해결한다.

## 시작 기준

- Stage20 변경은 `54f47e00` 커밋으로 분리했다.
- Stage20 문서에 현재까지 수정 내용과 남은 항목을 정리했다.
- Stage21에서는 별도 선행 분석을 새로 반복하지 않고, Stage20의 남은 항목을 기준으로 진행한다.

## 남은 문제

- 미주 내부 수식이 본문 수식처럼 개별 선택되고 `개체 속성`을 열 수 있어야 한다.
- 미주 내부 텍스트/수식 혼합 문단의 드래그 선택과 `문단 모양` 경로가 본문과 동일하게 동작해야 한다.
- 15쪽 이후 overflow, 17~18쪽 헤더 오버랩, 9쪽/12쪽 회귀 여부를 다시 확인해야 한다.
- 렌더링 결과가 한컴 정답지와 미세하게 다른 페이지는 PNG 기준으로 위치 차이를 좁힌다.

## 원인 정리

- 본문/표 셀 수식은 `getPageControlLayout`의 `equation` 항목에 `secIdx`, `paraIdx`, `controlIdx`, 필요 시 `cellIdx`, `cellParaIdx`가 들어가고, rhwp-studio가 이 값을 `getEquationProperties`로 전달한다.
- 미주는 페이지네이션 단계에서 본문 뒤의 가상 문단(`paragraphs.len() + endnote_paragraphs index`)으로 합쳐져 렌더된다. 이 가상 문단 인덱스는 화면 배치에는 유효하지만, 원본 문서 모델에서 `Endnote` 내부 문단과 컨트롤을 다시 찾기에는 부족하다.
- `EquationNode`에는 현재 본문/표 셀용 경로만 있고, `Endnote` 원본 경로(`section`, 바깥 본문 문단, 바깥 `Endnote` 컨트롤, 미주 내부 문단, 내부 수식 컨트롤)를 담을 필드가 없다.
- rhwp-studio의 `insert:picture-props`와 `format:object-properties` 커맨드는 `ref.type === 'equation'`이면 즉시 반환하므로, 수식 개체가 선택되더라도 `개체 속성` 메뉴가 열리지 않는다.

## 수정 방향

1. 렌더 트리의 수식 노드에 note 내부 원본 경로를 추가한다.
   - 최소 형태: `noteRef: { kind: "endnote" | "footnote", sectionIdx, paraIdx, controlIdx, noteParaIdx }`
   - Stage21의 직접 대상은 미주이므로 먼저 `endnote` 경로를 연결한다.
2. `getPageControlLayout`의 `equation` JSON에 note 경로를 노출한다.
   - 본문/표 셀 수식의 기존 JSON은 유지한다.
   - 미주 수식은 가상 문단 인덱스만 쓰지 않고 원본 `Endnote` 내부 문단/컨트롤을 함께 반환한다.
3. WASM에 미주/각주 내부 수식 속성 조회/적용 API를 추가한다.
   - 기존 `getEquationProperties`는 본문/표 셀 경로를 유지한다.
   - 신규 API는 `sectionIdx`, `paraIdx`, `controlIdx`, `noteParaIdx`, `innerControlIdx`로 `Control::Endnote` 또는 `Control::Footnote` 내부 `Control::Equation`을 찾는다.
4. rhwp-studio 개체 선택 참조 타입에 note 경로를 추가하고, 수식 선택/렌더링/속성 조회에서 전달한다.
5. `개체 속성` 커맨드의 수식 거부 분기를 제거하거나, 수식이면 `EquationEditorDialog`로 연결한다.
   - 현재 UI에 별도 수식 개체 속성 대화상자가 없으므로 Stage21에서는 기존 `EquationEditorDialog`를 수식 속성 경로로 재사용하는 방향이 가장 작다.
6. 구현 후 `cargo fmt`, `cargo build`, 필요 시 `wasm-pack build --target web --out-dir pkg`, 9/12/15/16/17/18쪽 PNG 비교를 수행한다.

## 진행 원칙

- 소스 수정 전에는 원인과 수정 방향을 먼저 정리하고 작업지시자 승인을 받는다.
- SVG/PNG 산출물이 생기면 PNG 경로를 우선 보고한다.
- WASM 확인이 필요하면 `wasm-pack build --target web --out-dir pkg`로 갱신한다.

## 구현 결과

- 페이지네이션 결과에 미주 가상 문단의 원본 경로(`EndnoteParaSource`)를 보존하도록 했다.
- `EquationNode`와 `getPageControlLayout`의 `equation` 항목에 미주 내부 수식의 `noteRef`를 노출했다.
- 본문/표 셀 경로와 별도로 미주 내부 수식 속성 조회/적용 WASM API를 추가했다.
  - `getNoteEquationProperties`
  - `setNoteEquationProperties`
- rhwp-studio의 선택 참조, hit-test, 커맨드, `EquationEditorDialog`가 `noteRef`를 전달하도록 연결했다.
- `개체 속성` 및 `insert:equation-edit` 경로에서 미주 내부 수식도 기존 수식 편집/속성 대화상자를 열 수 있게 했다.
- `tests/issue_1139_inline_picture_duplicate.rs`에 미주 내부 수식의 `noteRef` 노출과 속성 조회/수정 회귀 테스트를 추가했다.
- compact 미주 흐름의 큰 역방향 VPOS 보정 후보를 `height_cursor.rs`에 반영했다.

## 검증 결과

- `cargo fmt` 완료
- `cargo build` 통과
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 통과
  - 9개 테스트 통과
  - 기존에 남아 있던 12쪽 `LAYOUT_OVERFLOW_DRAW` 약 0.9px 경고는 유지
- `wasm-pack build --target web --out-dir pkg` 통과
- `npm run build` 통과

## 남은 항목

- `height_cursor.rs`의 compact 미주 역방향 VPOS 보정은 #1139 회귀 테스트를 통과했지만, SVG 재출력 후 15/16/17/18쪽 overflow 해소 여부 확인은 중간에 중단되어 최종 확인이 남았다.
- 중단 전 이전 SVG 기준으로는 다음 overflow가 있었다.
  - 12쪽: 약 0.9px draw overflow
  - 16쪽: 약 17.6px overflow
  - 18쪽: 최대 약 349.6px overflow
- Stage22에서는 새 기준으로 SVG/PNG를 다시 출력해 15/16/17/18쪽 overflow와 한컴 정답지 시각 정합을 먼저 확인한다.

## 커밋 판단

- Stage21의 주목표였던 미주 내부 수식 선택/개체 속성 경로는 자동 검증까지 완료했다.
- 시각 정합과 overflow 잔여 확인은 별도 Stage22로 분리해 계속 진행한다.
