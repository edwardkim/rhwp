# Stage 3 완료보고서 — Task #1171

- **이슈**: [#1171](https://github.com/edwardkim/rhwp/issues/1171)
- **브랜치**: `local/task1171`
- **단계 목표**: 프런트엔드 picture 우선 hit-test — 글상자 내부 클릭이 텍스트 편집으로
  단락되기 전에 글상자 안 picture 를 선제 선택.
- **작성일**: 2026-06-02

## 변경 내용 (`rhwp-studio/src/engine/input-handler-mouse.ts`)

글상자 경계선 검사(705-720) 직후, 텍스트 편집 진입(`if (hit.isTextBox)` 캐럿 배치) 직전에
선제 hit-test 분기 추가:
- `hit.isTextBox` 일 때 `findPictureAtClick(pageIdx, pageX, pageY)` 선제 호출.
- 반환 picHit 이 `type==='image'|'equation'` 이고 `cellPath`(글상자/셀 sentinel) 동반이면
  → 기존 picture dispatch(`enterPictureObjectSelectionDirect`, 842-848 호출과 동일 인자)로
  객체선택 진입 + `return`.
- picHit 없거나 picture 아니거나 cellPath 없으면 → 아래 기존 텍스트 편집으로 fall-through
  (picture 없는 글상자 영역 클릭은 기존대로 텍스트 편집).

설계 근거: 표 셀 picture 는 `hit_test_native` 가 `isTextBox=false` 라 기존 picture 처리
(762행)까지 fall-through 되지만, Shape text_box 는 `isTextBox=true` 라 744행 텍스트 편집에서
단락되어 762행에 도달하지 못했다. 본 선제 분기가 그 단락 이전에 글상자 picture 를 가로챈다.

## 검증

- `npx tsc --noEmit`: **변경 파일(input-handler-mouse.ts) 에러 0**. (전체 에러는
  canvaskit-renderer.ts 의 `canvaskit-wasm` 모듈 미설치 — 본 변경과 무관한 기존 문제.)
- 디스패치 인자는 기존 picture 선택 경로(842-848)와 동일하므로 select 상태/렌더 정합.
- **행위 검증(클릭→객체선택)은 Stage 5 통합 검증에서 수행** — findPictureAtClick 이
  Stage 1-2 의 Rust 변경(cellPath 노출)을 포함한 새 WASM 빌드를 필요로 한다(현재 pkg/ 는
  Stage 1-2 미반영). Stage 5 에서 WASM 재빌드 후 E2E/수동 검증.

## Stage 3 보강 — findPictureAtClick 우선순위 (★ 행위 검증 중 추가)

WASM 재빌드(Stage 1-2 반영) 후 E2E 검증에서 **findPictureAtClick 이 글상자 picture bbox
중심 클릭에서 'shape' 를 반환**(image 아님)함을 확인했다. 원인: 사각형(Shape, InFrontOfText)
bbox 가 내부 picture 를 덮고, collect_controls 가 Shape 를 자식 picture 보다 먼저 방출하므로
findPictureAtClick 1차 패스가 Shape 를 먼저 hit 한다(이슈 작업범위 #1 "findPictureAtClick
hit-test 우선순위 조정"이 가리킨 결함). 위 mouse handler 선제 분기는 findPictureAtClick 이
image 를 반환해야 작동하므로, 이 우선순위 수정이 필수다.

**수정** (`rhwp-studio/src/engine/input-handler-picture.ts` `findPictureAtClick` 선두):
- 1차 패스 이전에 우선 패스 추가 — 클릭이 **컨테이너 Shape 와 cellPath 동반 nested
  image/equation 둘 다**에 들어가면 picture 를 우선 반환. `shapeHit && nestedPic` 일 때만
  동작하므로 겹치는 Shape 가 없는 표 셀 picture 는 영향 없음. BehindText 제외(기존 정책 유지).

## Stage 4 결론 — insert.ts 코드 변경 불필요 (검증으로 확정)

`insert.ts:272-283` 의 cellPath 재구성은 스칼라(`outerTableControlIdx`/`cellIdx`/`cellParaIdx`)
로 depth-1 경로 `[{controlIdx, cellIdx, cellParaIdx}]` 를 만들며 키가 백엔드
`parse_cell_path_json` 규약과 일치한다. 글상자 picture 는 depth-1 이고 Stage 1 sentinel 로 세
스칼라가 (0,0,0) 으로 채워지므로 기존 재구성이 그대로 올바른 cellPath 를 생성한다. E2E 에서
`getCellPicturePropertiesByPath/setCellPicturePropertiesByPath` 를 동일 cellPath 형식으로 호출한
round-trip(width 15040→20040)이 성공하여 확정. **Stage 4 는 무변경**(다단계 중첩 견고화는
수행계획서 §8 후속).

## 검증 (E2E, headless Chrome + WASM 재빌드)

신규 E2E `rhwp-studio/e2e/textbox-picture-1171.test.mjs` (tac-img-02.hwp 로드):
- 글상자 picture(섹션0 문단25, page index 5)가 controls 에 cellPath sentinel
  `[{controlIndex:0,cellIndex:0,cellParaIndex:0}]` 로 노출 (Stage 1).
- **findPictureAtClick(bbox 중심) → type='image' + cellPath 동반** (Stage 3 우선순위 수정 후).
- by_path round-trip: `getCellPicturePropertiesByPath`(width=15040) → `set`(20040) → 재조회 20040
  (Stage 2 + bridge + insert.ts cellPath 형식, Stage 4).
- 기존 hit-test E2E(shape-inline, body-outside-click-fallback) 예외 없이 완료(회귀 없음 —
  우선 패스는 shape+nested-picture 동시 hit 시에만 동작).
- `npx tsc --noEmit`: 변경 파일(input-handler-mouse.ts, input-handler-picture.ts) 에러 0.

## 다음 단계 (Stage 5)

통합 검증: 전체 cargo test/clippy, studio tsc/build, tac-img-02.hwp p6/p7 수동 클릭
(객체선택/속성 dialog/외곽 Shape 선택/picture 없는 영역 텍스트 편집).
