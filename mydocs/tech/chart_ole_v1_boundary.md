---
kind: decision
status: active
canonical: mydocs/tech/chart_ole_v1_boundary.md
last_verified: 2026-09-26
---

# 차트 OLE v1 경계 — 무엇이 들어가고 무엇이 빠지나

MEGA QUEUE 9→10 승격(M09x, #5424)에서 잠그는 **현행 v1 계약**이다.
새 파서·새 퍼즈 하네스·차트 엔진을 발명하지 않는다. 렌더러 선택(Rust SVG vs charming)은
[OLE chart renderer 선택 결정](hwp_ole_chart_renderer_architecture_decision_1251.md)이 맡는다.

## 1. 한 줄

v1 차트 OLE는 **이미 있는 세 갈래**만 쓴다. OLE는 저장 컨테이너이고 차트의
종류가 아니다. 컨테이너 안에 OOXML `c:chartSpace`가 있으면 OLE 전용 그래프를
새로 그리지 않고 일반 OOXML 차트 파서·렌더러를 쓴다.

1. HWPX/OLE 안의 **OOXML `c:chartSpace`** (`ooxml_chart`)
2. 레거시 HWP OLE **`Contents`** 의 `VtDataGrid` 최소 추출 (`ole_chart`)
3. 위가 없으면 **OlePres 미리보기** — EMF 네이티브 SVG, 그다음 WMF SVG

한컴 차트 엔진·Excel/MS Graph 바이너리·charming/ECharts 런타임은 v1 밖이다.

## 2. 배치 우선순위 (코드가 정본)

`src/renderer/layout/shape_layout.rs` 의 OLE 분기. 먼저 성공한 경로가 이기고,
`rendered = true` 가 되면 아래 후보는 실행되지 않는다.

| 순서 | 입력 | 진입점 | v1 안? |
| --- | --- | --- | --- |
| 1 | HWPX `extension == "ooxml_chart"` | `OoxmlChart::parse` → `render_svg` | 안 |
| 2 | OLE CFB `OOXMLChartContents` | 같은 `OoxmlChart::parse` | 안 |
| 3 | OLE CFB `Contents` **존재** | `parse_ole_chart_contents` | 안 |
| 3-실패 | `Contents` 는 있으나 파싱 실패 | 오류 라벨을 보관하고 EMF/WMF/이미지 미리보기를 시도한다. 모두 실패하면 placeholder | 안 (폴백 계약) |
| 4 | `Contents` 없음 + `preview_emf` | `emf::convert_to_svg` | 안 |
| 5 | EMF 없음 + `preview_wmf` (`OlePres000`) | `convert_wmf_to_svg` → data URI `<image>` | 안 |
| 6 | 네이티브 BMP/PNG/JPEG/GIF | `Ole10Native` / DIB→BMP | 안 |
| — | 그 외 | 자리표시 도형 | 안 |

컨테이너 추출은 `src/parser/ole_container.rs` 다. 읽는 스트림은
`\x02OlePres000`, `OOXMLChartContents`, `Contents`, `\x01Ole10Native` 뿐이다.

## 3. v1 안 — 잠그는 동작

### 3.1 WMF

- 진입점: `WMFConverter::new(data, SVGPlayer::new()).run()` /
  `src/renderer/svg.rs` `convert_wmf_to_svg`.
- 퍼즈: 기존 `fuzz/fuzz_targets/parse_wmf.rs` 만. **EMF 하네스는 만들지 않는다.**
- 시드: `fuzz/corpus/parse_wmf/` 의 최소 placeable + M09x 소형 도형 시드.
- 골든: `tests/cases/wmf_emf_goldens.rs` — 헤더 종류와 현행 SVG.

### 3.2 EMF

- 진입점: `emf::parse_emf`, `emf::convert_to_svg`, `emf::convert_to_standalone_svg`.
- 구현 범위는 모듈 주석의 단계 10~13 그대로다. 헤더/EOF, 객체·DC 상태,
  선/사각형/타원/호/폴리라인16/패스, `ExtTextOutW`, `StretchDIBits`.
- `SetWorldTransform` / `ModifyWorldTransform` 은 DC 에 저장만 하고 **출력 행렬에 적용하지 않는다**.
- 미지 레코드는 `Record::Unknown` 으로 건너뛴다. 전체 MS-EMF 카탈로그(200+)를 채우지 않는다.

### 3.3 레거시 OLE `Contents`

- `probe_ole_chart_contents` 로 CFB 매직·`chartSpace`·`VtChart`/`VtDataGrid` 를 본다.
- `likely_legacy_hwp_chart_contents` 일 때만 `VtDataGrid` 라벨·연속 f64·계열 축(#4098)을 IR 로 옮긴다.
- `chart_type` 은 그리드만으로는 확정하지 않고 기본 `Unknown` 일 수 있다.
- 렌더는 `render_ole_chart_svg_fragment` (Rust SVG). `charming` 은 링크하지 않는다.
- IR 스키마 `rhwp.oleChartIr` version **1**.

### 3.4 일반 차트로 처리할 수 있는 범위와 분류 절차

새 차트를 다룰 때는 개체의 외형이나 OLE 확장자보다 **편집 가능한 원본 표현**을 먼저 확인한다.

1. HWPX `Chart/chartN.xml` 또는 중첩 CFB `OOXMLChartContents`에 `c:chartSpace`가 있으면
   `OoxmlChart::parse`와 일반 OOXML SVG 렌더러를 쓴다. OLE 컨테이너에서 XML을 꺼내는
   과정은 여전히 필요하다. HWPX의 두 OOXML 사본은 계열·범주·값이 일치하는지 확인하고,
   편집 시에는 둘 다 갱신한다. [차트 CSV 명령](../manual/cli_commands.md)의
   `representationMismatch` 계약을 따른다.
2. OOXML 없이 레거시 `Contents`/`VtDataGrid`만 있으면 `parse_ole_chart_contents`로
   계열·범주·값과 확인 가능한 표현 메타데이터를 읽는다. 현재는 `OleChart` IR와 전용
   Rust SVG 경로다. 장래에 공통 그래프 IR로 통합할 수 있지만, 이를 이유로 알 수 없는
   차트 종류·축·레이블을 임의의 막대그래프로 추정하지 않는다.
3. 편집 가능한 차트 데이터가 없거나 파싱에 실패하면 EMF/WMF/이미지 미리보기 경로를
   사용한다. 미리보기는 시각 폴백이며 값·레이블의 정본이 아니다. 전부 실패하면
   오류 이유가 담긴 placeholder를 표시한다.

[#7406](https://github.com/edwardkim/rhwp/pull/7406)처럼 OOXML과 OLE 미리보기가 함께
있는 차트는 **값축·범주·계열·누적/백분율·데이터 레이블·막대 위치를 OOXML에서**
구성한다. `hncChartStyle colorIndex=-1`처럼 색상표가 XML에 확정되지 않은 경우에만
동일한 차트임을 확인한 미리보기에서 색을 보충한다. 미리보기의 낡은 막대 길이·값·레이블을
새 데이터 위에 복사하지 않는다. 값 하나를 변경해 미리보기와 어긋나게 한 대조군에서도
그래프 길이와 레이블이 편집 가능한 값에 따라 바뀌는지 검사한다. 원본 한컴 PDF와는
각 분포의 레이블·위치·백분율 및 차트 뒤 캡션/본문 위치를 같은 페이지에서 비교한다.

## 4. v1 밖 — 하지 않는 것

- 새 WMF/EMF/OLE/차트 파서, 새 `parse_emf` 퍼즈 타깃.
- ChartOBJ 전체 객체 그래프, MS Graph, Excel BIFF, 워크북 임베드 XLSX OLE.
- `Contents` 파싱 실패 뒤 미리보기로 폴백하는 현행 동작의 무근거 변경.
- 레거시 `Contents`의 전체 차트 편집·OLE 바이너리 되쓰기 및 추세선 충실도.
- OOXML 차트의 지원 범위를 레거시 `Contents` 경로에 자동 적용하는 가정.
- charming / ECharts JS 런타임을 Studio·WASM 기본 경로로 강제.
- gym / `scripts/visual_sweep.py` / 한컴 PDF 오라클 확장 (M09x 범위 아님).

이 목록을 깨려면 별도 이슈로 경계를 먼저 고친다. 골든은 엔진 변경의 **결과가 아니라
전제**다 — 의도된 변경 시에만 `UPDATE_WMF_EMF_GOLDENS=1` 로 갱신한다.

## 5. 관련

- 이슈: [#5424](https://github.com/edwardkim/rhwp/issues/5424)
- 렌더러 결정: [hwp_ole_chart_renderer_architecture_decision_1251.md](hwp_ole_chart_renderer_architecture_decision_1251.md)
- 퍼즈 운영: [fuzzing/README.md](fuzzing/README.md), [`fuzz/README.md`](../../fuzz/README.md)
- 골든 시험: `tests/cases/wmf_emf_goldens.rs`
