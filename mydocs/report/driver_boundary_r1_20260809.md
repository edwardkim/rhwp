# 멀티플렉서 driver 경계 실측 1차(r1) — 포맷 지식이 driver 안에서 끝나는가

- **Issue**: 이슈 1:1 대응 아님. 이 조사가 낳은 이슈 — [#4384](https://github.com/edwardkim/rhwp/issues/4384),
  [#4386](https://github.com/edwardkim/rhwp/issues/4386), [#4387](https://github.com/edwardkim/rhwp/issues/4387),
  [#4388](https://github.com/edwardkim/rhwp/issues/4388), [#4400](https://github.com/edwardkim/rhwp/issues/4400),
  [#4379](https://github.com/edwardkim/rhwp/issues/4379)(해결됨, PR #4394)
- **브랜치**: `docs/driver-boundary-survey-r1`
- **기준**: `upstream/devel` `e48fe8694`
- **작성 시각**: 2026-08-09 KST
- **방법**: driver 별 격리 worktree 에서 정적 조사(gestell). 빌드·실측이 필요한 항목은 본문에
  "미확인"으로 명시했다.

## 0. 무엇을 쟀나

rhwp 는 멀티플렉서다 — 입력 driver 4종(HWP5·HWPX·HML·HWP3)이 공통 IR `Document` 를 만들고,
출력 driver 여러 종이 그것을 소비한다. 이 조사가 잰 축은 하나다:

> **driver 의 포맷 지식이 그 driver 안에서 끝나는가.**

[`CLAUDE.md`](../../CLAUDE.md) 가 HWP3 에 대해 이 규칙을 명시한다 — *"HWP3 전용 해석은
`src/parser/hwp3/` 안에서 끝내고 렌더러·레이아웃·문서 코어에 HWP3 전용 분기를 추가하지
않는다."* 권위 문서는 [`parser_architecture.md`](../tech/parser_architecture.md) 다.

같은 축을 다시 재면 r2 가 된다. 1차의 절단선은 **정적 조사**이며, 코퍼스 실측(라운드트립 보존)은
이 문서에 포함하지 않았다.

## 1. driver 별 판정

| driver | 경계 누출 | 읽기·쓰기 비대칭 | 이슈 |
|---|---|---|---|
| HWP3 (`parser/hwp3/`) | **없음** | — | — |
| HWPX (`parser/hwpx/`, `serializer/hwpx/`) | 없음 | 3건 | #4387, #4388 |
| HML (`parser/hml/`, `serializer/hml/`) | 3건 | 2건 | #4386 |
| HWP5 (`parser/`, `serializer/` 루트) | **5건** | — | #4400 |
| 출력 렌더 (`renderer/{pdf,html,canvas,skia}`, `paint/`) | 판정 중복 3건 | — | #4379(해결) |

### 1.1 HWP3 — 누출 없음

`src/parser/hwp3/` 바깥에서 hwp3 를 아는 파일 약 75개를 전수로 분류했다. **전부**
`Document::layout_profile()` 질의를 경유하며, 포맷 재판별(`SourceFormat::Hwp3` 직접 비교)은
`model/document.rs` 의 `layout_profile()` 구현부 한 곳에만 있다. 소비부는 불투명 boolean 만 받는다.

이슈 #2403 에서 도입한 `LayoutCompatibilityProfile` 규약이 **실제로 지켜지고 있다.**
`is_hwp3_variant`/`is_hwpx_variant` shim 필드를 직접 읽는 신규 코드는 0건이었다.

경계선 사례 둘은 의도된 설계로 판정했다 — `document_core/converters/hwpx_to_hwp.rs`(문서가 명시적으로
허용한 변환 계층), `password_crypto.rs`(3포맷 공통 암호 모듈). 다만 후자를 canonical 문서가
명시적으로 예외 처리하지는 않는다.

**이 결과가 이번 조사의 기준선이다** — 규칙이 명문화된 driver 는 실제로 규칙을 지킨다.

### 1.2 HWP5 — 바이트 배치 지식이 렌더러·편집 커맨드로 샜다 (#4400)

가장 심각하다. HWP5 는 네이티브 포맷이라 IR 자체가 그 구조에서 유래했고, 그만큼 경계가 흐리다.

- **렌더러가 CTRL_HEADER 바이트를 직접 디코딩한다** — `renderer/typeset.rs:441`
  `raw_table_ctrl_height_px()` 가 `common_obj_offsets::HEIGHT` 로 `raw_ctrl_data` 를 인덱싱하고
  `u32::from_le_bytes` 로 푼다. `table.common.height` 라는 모델링된 필드가 있는데도 raw 를 우선한다.
- **렌더러가 raw 그림자 필드의 미모델링 비트를 본다** — `typeset.rs:2904` 가
  `raw_table_record_attr & 0xff00_0000` 을 검사한다. `table.attr` 에 없는 비트다.
- **편집 커맨드가 PARA_HEADER 꼬리를 손으로 조립한다** — `document_core` 의 4개 파일이
  `raw_header_extra` 의 offset 0..2 / 4..6 / 6..10 에 직접 바이트를 꽂는다. 이 필드를 소비하는 것은
  HWP5 직렬화기뿐인데, **포맷 무관 편집 경로가 상시 조립**한다.
- **도형 편집마다 HWP5 비트를 재계산한다** — `object_ops/common.rs:269` 의 `pack_common_attr_bits`.
- **역방향 의존** — `serializer/control.rs:9` 가 `document_core::converters` 를 import 한다.
  직렬화 로직이 `document_core` 에 있고 직렬화기가 거꾸로 가져다 쓴다.

**반증된 통념**: "`raw_*` 는 HWP5 전용이라 다른 driver 에선 항상 기본값"은 틀렸다. 실측 결과 다수를
HWPX 파서가 채운다 — 목적이 *"이 문서가 나중에 HWP5 로 저장될 때 바이트를 재현하기 위한 준비
데이터"* 다. 진짜 항상 기본값인 것은 레코드 계층 8종의 `raw_data` 와 `DocInfo.raw_stream`/
`Section.raw_stream` 뿐이며, 이는 [패스스루 계약](../tech/serialization_passthrough_contract.md)이
서술하는 그대로다. **계약 위반은 없다** — 게이트 3계층을 코드로 대조해 확인했다.

### 1.3 HWPX·HML — 한 driver 에만 빠진 구현

경계 누출보다 **읽기·쓰기 비대칭**이 문제였다. 셋 다 "포맷 표현력의 한계"가 아니라 배선 누락이다.

- **#4386 HML `COLDEF`** — `reader.rs` 전체에서 `COLDEF` 는 미지원 판정의 **허용 목록**(`:1334`)
  한 곳에만 등장하고 처리 분기가 없다. "지원한다"고 표시돼 경고 대상에서 빠지고 조용히 무시된다.
  `Control::ColumnDef` 는 렌더러가 실제로 소비하고(`layout.rs:4357`), **HWPX 파서는 정상적으로
  읽는다**(`hwpx/section.rs:496`).
- **#4387 HWPX `hp:colSz`** — OWPML 스키마가 정의하고(`maxOccurs="255"`) IR 에 자리가 있는데
  (`ColumnDef.widths`/`gaps`), `colSz` 문자열이 HWPX 파서·직렬화기 **양쪽 모두에 없다.**
- **#4388 HWPX 조용한 드롭** — `Control::Hyperlink` 가 `render_control_slot` 의 catch-all
  `_ => {}`(`:1565`)로 떨어진다. HWP5 직렬화기는 처리한다. `ArcShape.arc_type` 은 HWPX 파서가
  아예 안 읽어 항상 0이다.

**#4386 과 #4387 은 같은 계열이다** — HML 은 다단 정의 자체를, HWPX 는 단별 폭을 잃는다. 두 driver 가
각각 다른 층위에서 다단을 놓치고 있다.

**왜 안 드러났나**: HML fixture 3개가 전부 `<COLDEF Count="1">` 이라 드롭돼도 결과가 같다.
포맷별 fixture 가 그 포맷의 기능 범위를 대표하지 못하면 이 부류는 계속 숨는다.

### 1.4 출력 driver — 같은 판정의 중복 구현

- **`editor_only`** — `svg.rs:271` 의 bool(기본 표시)과 `paint/builder.rs:75` 의 프로필 기반
  두 벌. **#4379 로 해결**(PR #4394) — `RenderProfile::shows_editor_visuals()` 하나로 통일하고
  두 경로 대조 테스트를 붙였다.
- **#2225 그림 placeholder 억제 3벌** — `svg.rs`(bool, 기본 숨김), `skia/renderer.rs`(프로필 파생),
  `web_canvas.rs`(프로필). 같은 `SvgRenderer` 안에서 `show_missing_picture_placeholder`(기본 숨김)와
  `show_editor_only_nodes`(기본 표시)의 **극성이 반대**였다.
- **z-plane 판정 3벌** — `svg.rs::node_z_plane`, `paint/replay_order.rs`, `render_tree.rs::
  ClipReplayPlane::from_text_wrap`. `replay_order.rs:74-76` 의 주석이 병행 구현임을 자인한다.

**좋은 대조군**: 클리핑은 `layout.rs` 에서 1회 계산해 `RenderNode` 필드에 저장하고 소비자들이 읽기만
한다. 재계산이 없어 갈라질 수 없다. 위 셋과 이것의 차이가 이 축의 핵심이다.

## 2. 부수 발견 — 렌더러가 본문 문자열로 분기한다 (#4384)

driver 경계와 별개지만 같은 조사에서 나왔다. 프로덕션 렌더 경로 두 곳이 **문서 본문 내용**으로
분기한다:

- `composer.rs:794` — 문단에 `"BCP:Business Continuity Planning) 수립"` 이 있으면 줄 수를 하나 뺀다
- `layout.rs:221` — 표에 `"Filing Receipt"` 또는 `"접 수 증"` 이 있으면 도장 줄 배치를 켠다

`"접 수 증"` 은 한국 공문서에 흔한 표현이라 오탐이 실제로 가능하고, 반대로 사용자가 제목을 편집하면
처리가 사라진다. 더 나쁜 것은 **왜 그 문서가 그렇게 배치되는지가 코드에 안 남아** 일반화가
불가능하다는 점이다.

## 3. 죽은 경로

- `HtmlRenderer` / `render_page_html_native` / `renderPageHtml` — 호출자 0건. `export-html` CLI 명령
  자체가 없다.
- `render_page_canvas_legacy_native`, `render_page_canvas_native` — 호출자 0건(후자는 명령 개수만
  반환하는 테스트 하니스).
- `RHWP_RENDER_PATH=layer-svg` — WASM 에 프로세스 환경변수가 없어 브라우저에서 도달 불가. #4379 에서
  `#[cfg(not(target_arch = "wasm32"))]` 로 감싸 사실을 코드에 드러냈다.

## 4. 이 조사가 드러낸 패턴

세 부류로 수렴한다.

1. **같은 결정의 이중 구현** — #4379, #2225 placeholder, z-plane, 그리고 이 조사 밖의
   #4312(줄 스킵 판정)·#4320(캡션 산식)·#4333(높이 정의식). 전부 "한쪽만 고치면 조용히 갈라지는" 구조다.
2. **한 driver 에만 빠진 배선** — #4386, #4387, #4388. 포맷 표현력이 아니라 구현 누락이다.
3. **경계를 넘은 포맷 지식** — #4400. 규칙이 명문화된 HWP3 는 지켜졌고, 명문화되지 않은 HWP5 는
   샜다. **규칙을 적어 두는 것이 실제로 효과가 있다**는 증거이기도 하다.

## 5. 다음 회차(r2)에서 재야 할 것

- **라운드트립 보존 실측** — 착수됨. HWPX → HWP → HWPX 방향은 3,699개(`samples/` 281 +
  `hwpdocs_10k` 3,418) 스윕이 끝나 [#4395](https://github.com/edwardkim/rhwp/issues/4395)(수정 완료),
  [#4396](https://github.com/edwardkim/rhwp/issues/4396), [#4397](https://github.com/edwardkim/rhwp/issues/4397),
  [#4398](https://github.com/edwardkim/rhwp/issues/4398) 을 냈다. 반대 방향(HWP → HWPX → HWP)은 진행 중.
  별도 회차 문서로 정리한다.

  그 스윕이 이 정적 조사의 **커버리지 공백**을 드러냈다 — `diff_documents`/`IrDifference` 에
  `hp:colSz`(#4387)·`ArcShape.arc_type`(#4388) 항목이 없어 라운드트립 비교가 이 둘을 아예 보지
  않는다. 정적 조사가 찾은 결함을 실측이 확인해 주지 못하는 구조다. IR 비교기의 필드 커버리지를
  IR 정의와 대조하는 것 자체가 r2 항목이다.

- **#4386 의 미확인 후보 12종**(`PAGEBORDERFILL`, `FOOTNOTESHAPE` 등) — HML 은 위 스윕 대상이
  아니었다. 실물 fixture 가 필요하다.
- **`export-pdf` 기본 경로 검증** — 기본 backend `CompatibilitySvg` + 기본 profile `None` 조합이
  편집 전용 장식을 PDF 에 굽는지. 정적으로는 그렇게 보이나 마지막 호출 홉을 확인하지 못했다(미확인).
- **catch-all 이 삼키는 `Control` 변형 전수** — #4388 은 두 건만 확인했다.
- **바탕쪽 InFrontOfText 개체의 출력 간 순서 차이** — 코드상 gap 은 있으나 render-diff 실측 안 함.

## 6. 미확인 목록

정적 조사의 한계로 다음은 확인하지 못했다. r2 에서 실측 대상이다.

- HWP5: `Table.raw_table_record_extra`, `Cell.raw_list_extra`, `ChartShape.raw_chart_data`,
  `OleShape.raw_tag_data`, `TextBox.raw_list_header_extra`, `ColumnDef.raw_attr`,
  `Picture.raw_picture_extra`, `BinData.raw_data`, `SectionDef.raw_ctrl_extra` — grep 1회 확인,
  간접 경유 미추적
- HWPX: `GradientFill.positions` 가 HWPX 스키마상 표현 불가인지, 참조한 스키마 문서가 부분본인지
- HML: `PAGEBORDERFILL` 등 12종이 실제 한컴 생성 HML 에서 드롭되는지 (코드 경로만 확인)
- 출력: 바탕쪽 개체 순서 차이, npm 배포판 제3자 소비 여부
