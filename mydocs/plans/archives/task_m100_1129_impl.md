# 구현계획서 — Task #1129: rhwp-studio 한컴오피스식 격자 보기 및 격자 설정 활성화

- 이슈: [edwardkim/rhwp#1129](https://github.com/edwardkim/rhwp/issues/1129)
- 수행계획서: `mydocs/plans/task_m100_1129.md`
- 브랜치: `local/task_m100_1129`
- 작성일: 2026-05-26
- 상태: 작업지시자 승인 대기

## 1. 구현 전제

이번 작업의 1차 목표는 rhwp-studio 편집 화면에 한컴오피스식 격자 보기 기능을 제공하는 것이다. 저장 포맷의 줄맞춤 의미와 화면 보기 오버레이는 분리한다.

기본값 후보:

| 항목 | 기본값 |
|------|--------|
| 격자 보기 | 꺼짐 |
| 격자 유형 | 점 |
| 격자 위치 | 글 뒤 |
| 격자 방식 | 격자와 상관 없이 표시 |
| 가로 간격 | 3.00 mm |
| 세로 간격 | 3.00 mm |
| 기준 위치 | 쪽 |
| 가로/세로 오프셋 | 0.00 mm |

단위 변환:

```text
1mm = 96 / 25.4 px
gridPx = mm * 96 / 25.4 * zoom
```

## 2. Stage 1 — 격자 보기 토글과 오버레이

### 2.1 상태 모델

새 타입 후보:

```text
rhwp-studio/src/view/grid-overlay.ts
rhwp-studio/src/view/grid-settings.ts
```

상태 필드:

```ts
type GridPattern = 'dots' | 'horizontal' | 'vertical' | 'both';
type GridLayer = 'behindText' | 'inFrontOfText';
type GridSnapMode = 'free' | 'magnetic' | 'gridOnly';
type GridOrigin = 'page' | 'paper';

interface GridViewSettings {
  visible: boolean;
  pattern: GridPattern;
  layer: GridLayer;
  snapMode: GridSnapMode;
  horizontalMm: number;
  verticalMm: number;
  origin: GridOrigin;
  offsetXmm: number;
  offsetYmm: number;
}
```

### 2.2 오버레이 방식

`CanvasView.renderPage()`가 페이지 canvas를 `#scroll-content`에 추가할 때 같은 page index를 가진 오버레이 element도 추가한다.

후보 구조:

```html
<canvas data-page-index="0">
<div class="page-grid-overlay" data-page-index="0"></div>
```

오버레이는 canvas와 같은 `top`, `left`, `transform`, `width`, `height`를 가진다. 줌/리사이즈/페이지 재렌더링 시 `refreshPages()` 흐름에서 함께 제거/재생성한다.

CSS는 `background-image`를 사용한다.

1. 점: `radial-gradient(...)`
2. 가로선: `linear-gradient(to bottom, ...)`
3. 세로선: `linear-gradient(to right, ...)`
4. 가로/세로선: 두 `linear-gradient` 조합

`pointer-events: none`으로 입력 이벤트를 막지 않는다.

### 2.3 명령 연결

`viewCommands`에 `view:toggle-grid`를 추가한다.

동작:

1. `GridViewSettings.visible` 토글
2. `[data-cmd="view:toggle-grid"]` 활성 상태 갱신
3. `document-view-changed` 또는 별도 `grid-view-changed` 이벤트 발행

`rhwp-studio/index.html`의 기존 `격자 보기` 버튼에 `data-cmd="view:toggle-grid"`를 연결한다. 보기 메뉴에도 `격자 보기` 항목이 필요하면 `격자 설정` 위에 추가한다.

## 3. Stage 2 — 격자 설정 대화상자 확장

현재 `GridSettingsDialog`는 `이동 간격` 하나만 설정한다. 한컴오피스식 설정을 담기 위해 다음 중 하나를 선택한다.

권장안:

1. `GridSettingsDialog`를 보기 격자 설정 대화상자로 확장한다.
2. 기존 표/개체 이동 간격은 `snapStepMm` 또는 `objectMoveStepMm` 이름으로 `InputHandler`에 남긴다.
3. 대화상자에서 보기 격자 간격과 자석 간격을 함께 설정하되, 라벨을 명확히 분리한다.

대화상자 구성:

| 그룹 | 컨트롤 |
|------|--------|
| 격자 보기 | 체크박스, 점/가로선/세로선/가로세로선 segmented control |
| 격자 위치 | 글 뒤/글 앞 radio |
| 격자 방식 | 상관 없이/자석 효과/격자에만 붙이기 radio |
| 격자 간격 | 가로 mm, 세로 mm number input |
| 기준 위치 | 쪽/종이 radio |
| 오프셋 | 가로 mm, 세로 mm number input |

설정 적용:

1. 확인 버튼: 상태 저장 후 오버레이 재렌더.
2. 취소 버튼: 기존 상태 유지.
3. 입력 범위: 0.5mm 이상 50mm 이하, 0.5mm step.

## 4. Stage 3 — 스펙 필드 반영

보기 기능이 동작한 뒤, 문서 격자 필드 보존을 최소 범위로 추가한다.

### 4.1 HWP5 SectionDef

현재 `parse_section_def()`는 세로/가로 줄맞춤 값을 읽고 버린다.

변경 후보:

```rust
pub struct SectionDef {
    pub line_grid: HwpUnit16,
    pub char_grid: HwpUnit16,
}
```

`src/parser/body_text.rs::parse_section_def()`에서 `_vertical_align`, `_horizontal_align` 대신 필드에 저장한다.

### 4.2 HWPX secPr grid

`src/parser/hwpx/section.rs`에서 `<hp:grid lineGrid="..." charGrid="..." wonggojiFormat="..."/>`를 파싱해 `SectionDef`에 반영한다.

### 4.3 ParaShape snapToGrid

`src/parser/hwpx/header.rs::parse_para_shape()`에서 `snapToGrid`를 읽어 `ParaShape.attr1 bit 8`에 반영한다.

주의:

- OWPML 기본값은 `snapToGrid=true`다.
- 기존 serializer는 canonical default와 `snapToGrid` 기본값을 이미 다루는 흔적이 있다.
- 기본값 처리 변경으로 HWPX round-trip이 흔들리지 않도록 단위 테스트를 추가한다.

### 4.4 HWP3

HWP3 스펙에는 대응 필드가 확인되지 않았으므로 별도 저장 필드는 추가하지 않는다. 필요하면 기본 0/off로 둔다.

## 5. 테스트 계획

### 5.1 Rust 단위 테스트

후보:

```text
src/parser/doc_info.rs
src/parser/body_text.rs
src/parser/hwpx/header.rs
src/parser/hwpx/section.rs
```

검증 항목:

1. HWP5 section def의 line/char grid 값이 `SectionDef`에 저장된다.
2. HWPX `<hp:grid lineGrid="..." charGrid="...">`가 `SectionDef`에 저장된다.
3. HWPX `paraPr snapToGrid="0"`이면 `ParaShape.attr1 bit 8`이 꺼진다.
4. HWPX `paraPr snapToGrid="1"`이면 `ParaShape.attr1 bit 8`이 켜진다.

### 5.2 rhwp-studio 빌드

```bash
cd rhwp-studio
npm run build
```

### 5.3 브라우저 시각 검증

개발 서버:

```bash
cd rhwp-studio
npm run dev -- --host 0.0.0.0 --port 7700
```

검증 시나리오:

1. 문서 로드.
2. `격자 보기` 클릭.
3. 격자 오버레이가 페이지 안쪽 좌표와 맞는지 확인.
4. 150%, 100%, 50% 줌에서 위치가 유지되는지 확인.
5. 다중 페이지 그리드 모드에서 각 페이지의 오버레이가 해당 페이지에 붙는지 확인.
6. 설정에서 3mm/5mm 변경 후 간격이 바뀌는지 확인.

## 6. 위험과 대응

| 위험 | 대응 |
|------|------|
| 격자 오버레이가 클릭/선택 이벤트를 가로챔 | `pointer-events: none` 고정 |
| 줌/다중 페이지 모드에서 좌표 어긋남 | canvas와 동일한 `VirtualScroll` 좌표를 사용 |
| 글 앞/글 뒤 구현이 canvas 단일 레이어와 충돌 | 우선 오버레이를 canvas 위/아래 DOM layer로 분리하고, 글 뒤는 canvas 배경 위에 패턴을 재렌더하는 대안을 검토 |
| 기존 표/개체 이동 간격 의미가 사라짐 | 이름을 분리하고 기존 `InputHandler.gridStepMm` 동작 유지 |
| HWPX 기본값 처리 회귀 | `snapToGrid` true 기본값 테스트 추가 |

## 7. 승인 후 첫 작업 순서

1. `GridViewSettings`와 오버레이 헬퍼를 추가한다.
2. `CanvasView`에서 페이지 렌더/해제 시 오버레이를 함께 관리한다.
3. `view:toggle-grid` 명령과 툴바/메뉴 연결을 추가한다.
4. `GridSettingsDialog`를 확장한다.
5. HWP5/HWPX 스펙 필드를 최소 반영한다.
6. Rust 테스트, rhwp-studio 빌드, 브라우저 검증을 수행한다.
7. 단계별 보고서와 최종 보고서를 작성한다.

## 8. 승인 게이트

작업지시자가 이 구현계획서를 승인하기 전까지 소스 파일은 수정하지 않는다.
