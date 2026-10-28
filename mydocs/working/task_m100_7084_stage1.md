# #7084 Stage 1 — 이모티콘 폰트 메트릭 우선 추적

- Issue: [#7084](https://github.com/edwardkim/rhwp/issues/7084)
- 조사일: 2026-09-13
- 제품 기준: `ad6174255e6aabfaa48131f3f5f2f6287a93e48a`
- 계획: [수행계획서](../plans/task_m100_7084.md), 로컬 계획 커밋 `53e23bfc6`.
- 상태: **중간 조사. R1 수행계획 승인 후 보존 HWP/HWPX의 브라우저 실측 완료.
  두 형식의 한컴 PDF 생성·대상 그림 확인 완료. 제품 코드·DB 수정 및 새 WASM 빌드 없음.
  실제 browser face 확정·공통 fallback 구현 계약은 아직 미확정.**

## 1. 현재 결론과 증거 경계

원본 HWPX는 `😀`를 함초롬바탕 10pt, 장평 100%, 자간 0으로 지정한다.
해당 문자 폭이 내장 DB에 없으며, 현행 측정 코드의 일반 대체 폭은 **0.5em**이다.
이는 실제 이모티콘 폰트의 폭을 조회한 값이 아니다.

DB 생성기의 문자 범위 제한과 측정 시 글리프별 대체 폰트를 찾지 않는 경로가 확인됐다.
Canvas2D에는 실제 글리프가 이 계산 폭보다 넓으면 **높이는 유지하고 너비만 줄이는** 경로도 있다.
R1 승인 후 두 입력을 실제 Studio에서 실행하여 이 경로의 발현을 확인했다(§9).
이모티콘의 자연 진행폭 18.3027px에 대해 배정 폭은 약 6.6667px이며,
그리기 시 가로 0.364245배·세로 1배였다. 브라우저가 실제 사용한 fallback face 이름은
Canvas API만으로 확정하지 않는다. 한컴의 기대 advance와 수정 후 시각 통과도 별도 판정이다.

## 2. 원본 서식과 문자

입력은 계획서에 hash를 고정한
[HWPX](../../samples/issue3587/c-form-labnote-001-stage11-filled.hwpx)다.
Python `zipfile`·`xml.etree.ElementTree`로 `Contents/section*.xml`의
직접 텍스트 `실물😀<lineBreak/>본문` run과 `Contents/header.xml`을 대조했다.
바깥 표를 포함한 상위 run의 재귀 텍스트 검색 결과와 실제 텍스트 run은 구별했다.

| 항목 | 원본 값 |
| --- | --- |
| 문자 | U+1F600; Unicode scalar 1개, UTF-16 코드 단위 2개 |
| charPrIDRef | 0 |
| height | 1000 = 10pt |
| fontRef | 모든 언어 슬롯 2 = 함초롬바탕 |
| ratio / spacing / relSz / offset | 모든 슬롯 100 / 0 / 100 / 0 |
| 첨자·굵게·기울임 | 해당 지정 없음 |

이 결과는 **HWPX 원본 확인**이다. HWP 원본의 글자모양과 두 형식의 현재 IR 및 최종
TextStyle은 다음 추적에서 별도로 확인한다. HWPX 결과를 HWP 검사 완료로 간주하지 않는다.

## 3. DB와 생성기에서 누락되는 위치

1. [metric 별칭](../../src/renderer/font_rule_projections/layout_metric.rs)의 함초롬바탕은
   `HCR Batang`으로 연결된다. [generated DB](../../src/renderer/font_metrics_generated.rs)의
   regular 항목은 em_size=1000, `FONT_0_LATIN_RANGES`를 사용한다.
2. 이 범위에는 U+1F600이 없다. **`Segoe UI Emoji` 항목도 있으나 해당 문자 폭은 없다.**
   폰트 이름이 DB에 있다는 사실과 그 폰트의 이모티콘 폭이 있다는 사실은 다르다.
3. generated 파일의 LatinRange 2,749개를 조사한 결과 최대 end는 U+FF5E,
   보조 평면 범위는 0개였다. 이 집계는 historical generated 영역이며 전체 폰트 자산 집계가 아니다.
4. [생성기](../../src/tools/font_metric_gen.rs)의 `parse_cmap`은 Format 12를 지원하고
   `u32` 문자값을 읽는다. 하지만 `extract_latin_ranges`는 아래 7개 구간만 내보낸다.
   즉 Unicode를 못 읽는 문제가 아니라 **DB 추출 범위의 제한**이다.

`0020–007E`, `00A0–00FF`, `2000–206F`, `2200–22FF`, `3000–303F`, `3130–318F`, `FF00–FF5E`.

`git show v0.8.6:src/tools/font_metric_gen.rs`와 초기 커밋 `f0f7f1a4b4`에서도 같은 7개 범위를
확인했다. 현재 저장소의 `font_metrics_generated.rs` 분리 커밋은 `8f516d8413`이다.
**DB 범위 누락 자체는 기존 제한**이며 #3587에서 새로 만든 회귀가 아니다.
화면 압축의 최초 발생 버전은 이 조사만으로 특정하지 않는다.

## 4. 없는 폭을 어떻게 계산하는가

[폭 계산](../../src/renderer/layout/text_measurement.rs)의 흐름:

`함초롬바탕 → HCR Batang → get_width(U+1F600)=None → 일반 fallback 0.5em`.

- `FontMetric::get_width`는 `char as u32`를 사용한다. 이 조회에서 surrogate 절반으로 잘리는 것은 아니다.
- `build_cluster_len`은 단독 `😀`를 길이 1로 처리한다.
- `char_width_decision`에서 CJK·전각기호·좁은 구두점 어느 분류에도 해당하지 않는다.
- glyph miss 후 다른 이모티콘 폰트의 메트릭을 찾는 단계 없이 `heuristicHalfwidth`로 내려간다.
- 원본 서식이 그대로 TextStyle에 전달된다는 전제에서 10pt=13.3333px, 계산 advance=6.6667px이다.
  이는 소스 분기 대입값이며, 아직 샘플의 실행 trace를 채집한 값은 아니다.

## 5. 실제 폰트 파일과 너비/높이의 차이

로컬 파일을 `fontTools.ttLib.TTFont`로 읽어 `name`, `head`, `cmap`, `hmtx`, `glyf`를 확인했다.
폰트 설치·복사·수정은 하지 않았다.

| 로컬 폰트 | 조사 결과 |
| --- | --- |
| `/mnt/c/Windows/Fonts/HANBatang.ttf` | family=HCR Batang, unitsPerEm=1000, U+1F600 cmap 없음 |
| `/mnt/c/Windows/Fonts/seguiemj.ttf` | family=Segoe UI Emoji, unitsPerEm=2048, glyph=u1F600, hmtx advance=2812 |

SHA-256: HANBatang=`0dbeb7b129fcec63f8d6b7f1d3b5bb991c1d599e725286518ccc598de643f014`,
seguiemj=`451df9bb5546055af625ff86aa72695df43e8e170b4c6c1316fe44e9c7a22f7c`.

Segoe UI Emoji의 해당 glyf 외곽은 (325,-359)–(2487,1803), 너비와 높이 모두 2162단위다.
10pt 환산 시 advance=18.3073px, outline 너비·높이=14.0755px다.
**outline 수치는 컬러 레이어 전체의 최종 화면 경계나 한컴 PDF 실측값이 아니다.**
또한 디스크에 파일이 있다는 사실만으로 브라우저 또는 한컴이 이 폰트를 선택했다고 판정하지 않는다.
`ttfs/hwp/HBATANG.TTF`는 name 확인 결과 Haansoft Batang이므로 HCR Batang 증거로 사용하지 않았다.

## 6. 폭 부족이 가로 압축으로 이어질 수 있는 경로

[Canvas2D](../../src/renderer/web_canvas.rs)는 `measure_text`로 실제 폭을 얻고
[canvas_cluster_fit_scale](../../src/renderer/mod.rs)에 layout advance와 비교하도록 전달한다.
실제 폭이 계산 폭보다 0.25px 이상 넓고 자간이 음수가 아니면 가로 축만 축소한다.

만약 위 Segoe UI Emoji의 advance가 실제 measureText 폭이라면:

`가로 배율 = 6.6667 / 18.3073 ≈ 0.3642`, `세로 배율 = 1`.

이는 **조건부 계산 예시**이며 현재 브라우저 측정 결과가 아니다. 원래 1:1인 위 outline은
이 경로에서 약 0.364:1로 압축될 수 있다. 계산 폭에 맞추는 paint가 이미 잘못 작은 값을
받는다면 paint의 배율만 해제해도 줄 나눔·후속 문자 위치의 오류는 남는다.

CanvasKit의 일반 `textRun` 재생은 `drawGlyphs` 경로가 있으므로 Canvas2D와 동일하다고
가정하지 않는다. 수식·원문자용 `setScaleX`를 발견했다는 이유로 일반 이모티콘 원인으로
지목하지도 않는다. 현재 활성 backend 확인이 필요하다.

## 7. 이어서 확인할 것

1. HWP/HWPX 현재 IR·TextStyle·width decision에서 원본 서식과 0.5em 경로를 실행 증적으로 연결한다.
2. 실제 Studio backend, 로드된 WASM/JS, fallback face와 자연 폭·실제 가로 배율을 기록한다.
3. 같은 입력의 한컴 출력에서 선택 폰트/글리프 비율을 대조한다. font size를 glyph 높이라고 간주하지 않는다.
4. 측정과 paint가 동일한 fallback 근거를 소비하는 수정안을 수립한다. 특정 문자 하나의 1em 보정이나
   전체 DB 재생성부터 하지 않는다. 기존 장평·자간과 비이모지 반례를 보호한다.

## 8. 다른 워드프로세서·공개 조판 엔진의 처리 조사

조사일 2026-09-13. **공개 소스·공식 API 계약 조사이며 앱 실행 비교 실험은 아니다.**
LibreOffice는 실제 제품의 공통 텍스트 엔진을 확인했다. Word·Pages의 비공개 내부 구현은
Windows·Apple의 공개 API와 구별한다. 타 제품의 동작은 설계 참고이고 한컴 호환성의 정답지를 대체하지 않는다.

### 8.1 LibreOffice Writer가 사용하는 공통 텍스트 엔진

`GenericSalLayout::SetNeedFallback`은 누락된 글리프가 있으면 grapheme 범위 전체를
fallback 대상으로 지정한다. `LayoutText`는 HarfBuzz 조형 결과의 `x_advance`,
`x_offset`, `y_offset`을 읽고 배율을 적용해 GlyphItem에 보관하고 진행 위치를 갱신한다.
즉 단순히 Unicode 문자마다 정해진 폭을 붙이는 방식이 아니라 **선택 폰트로 조형한 결과의 폭**을 쓴다.
[LibreOffice CommonSalLayout 소스](https://raw.githubusercontent.com/LibreOffice/core/master/vcl/source/gdi/CommonSalLayout.cxx)

OutputDevice는 기본 layout 후 필요한 glyph fallback을 처리하고 그 뒤 위치·정렬을 조정한다.
`GetTextBreak`도 `ImplLayout` 결과를 사용한다. 다만 명시 폭·DXArray·정렬 보정 경로가 있으므로
“LibreOffice는 항상 자연 폭 그대로이며 보정을 전혀 하지 않는다”는 의미는 아니다.
[LibreOffice text.cxx](https://raw.githubusercontent.com/LibreOffice/core/master/vcl/source/outdev/text.cxx)

실제 `testTdf153440`에는 아랍어 폰트를 기본으로 두고 🌿를 지원하는 Noto Emoji 부분 폰트를
추가하는 검사도 있다. 이모지 fallback이 일반 문자와 혼합되는 상황을 제품 테스트 대상으로 삼는다.
[LibreOffice complextext 테스트](https://raw.githubusercontent.com/LibreOffice/core/master/vcl/qa/cppunit/complextext.cxx)

### 8.2 Word와 Microsoft의 공개 조판 API — 확인 범위 구별

Word 웹판의 이모지 입력 지원은 공식 문서에서 확인되지만, 그 문서는 폭 계산 알고리즘을
공개하지 않는다. 따라서 “Word는 정확히 이 함수로 모든 이모지를 처리한다”고 주장할 수 없다.
[Word 웹판 이모지 지원](https://support.microsoft.com/en-us/word/use-emojis-in-word-for-the-web)

Microsoft DirectWrite의 `MapCharacters`는 문자열에 사용할 대체 폰트, 적용 범위,
**대체 폰트의 em 크기에 곱할 배율**을 반환한다. 이 배율은 누락된 글자에 임의 반각 폭을
배정한 뒤 가로만 눌러 넣는 것과 다르다.
[MapCharacters 공식 계약](https://learn.microsoft.com/en-us/windows/win32/api/dwrite_2/nf-dwrite_2-idwritefontfallback-mapcharacters)

`GetGlyphPlacements`는 fontFace·fontEmSize·문자/글리프 매핑 등을 입력받아 글리프별
advance와 원점 offset을 출력한다. 측정 결과에 실제 폰트와 조형 결과가 연결되는 구조다.
[GetGlyphPlacements 공식 계약](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/nf-dwrite-idwritetextanalyzer-getglyphplacements)

### 8.3 Apple의 공개 텍스트 기반 — Pages 내부와 동일시하지 않음

Core Text는 자동 폰트 대체(cascading)와 문자→글리프 변환을 제공한다.
typesetter는 문자·속성·폰트에서 glyph run을 만들고 이를 줄로 구성하며, run은 직접 그리기도
가능하다. 역시 폰트 선택·조형·배치·그리기를 연계하는 기반이다. 이 문서만으로 Pages의
이모지별 크기 보정 정책을 알 수는 없다.
[Core Text](https://developer.apple.com/documentation/CoreText),
[Core Text 구성](https://developer.apple.com/library/archive/documentation/StringsTextFonts/Conceptual/CoreText_Programming/Overview/Overview.html)

### 8.4 높이와 너비를 이해할 때 중요한 표준상 구별

컬러 이모지도 조판 메트릭이 없는 단순 그림으로 취급할 필요는 없다.
OpenType COLR v1은 **base glyph의 advance를 컬러 글리프의 advance로 사용**하도록 규정하며,
개별 색상 레이어 glyph의 advance는 무시한다. 컬러 그림 영역의 경계는 별도로 다룬다.
이는 그림 경계 너비·높이와 다음 글자까지의 진행폭을 구별해야 하는 독립 근거다.
[OpenType COLR — Glyph metrics and boundedness](https://learn.microsoft.com/en-us/typography/opentype/spec/colr#glyph-metrics-and-boundedness)

또한 Unicode 이모지에는 단독 문자뿐 아니라 VS16 표현 선택자·ZWJ 결합열 등이 있다.
“보조 평면 문자 하나당 1em”으로 확장하는 것만으로 전체 이모지의 조형을 설명할 수는 없다.
이들은 설계 반례이며 이번 단독 U+1F600 결함의 원인으로 확정한 것이 아니다.
[Unicode Emoji 표준](https://www.unicode.org/reports/tr51/)

### 8.5 rhwp에 대한 시사점 — 아직 구현 결정 아님

공개 근거로부터 도출한 원칙은 **대체할 글리프를 먼저 정하고, 그 글리프의 메트릭으로
공간을 계산하며, 동일 선택 결과로 그린다**는 것이다.

- DB의 이모지 범위를 늘리는 것만으로는 부족하다. 원본 지정 폰트가 함초롬바탕일 때
  실제 대체 글리프의 폭을 선택하는 연결도 필요하다.
- 단독 문자는 실제 폰트 메트릭, 조합열은 조형 결과의 advance를 근거로 한다.
  glyph 식별·폰트 출처·크기와 배율을 측정/그리기 사이에서 잃지 않아야 한다.
- 글리프 높이, ascent/descent, 줄간격, advance는 서로 다르다. 정사각형 강제나
  글자 크기=실제 그림 높이라는 가정은 하지 않는다. 의도된 장평·자간도 별도로 보존한다.
- 현재 사례의 0.5em 임시 폭에 맞춰 가로만 줄이는 경로가 실제로 발현되는지 먼저 계측한다.
  paint의 가로 압축만 해제하고 배치 폭을 방치하면 뒤 글자 겹침·줄 나눔 오류가 남을 수 있다.

공개 API의 기본 동작을 한컴의 저장 LineSeg·장평·자간 정책보다 우선하지 않는다.
실제 Studio 및 한컴의 동일 입력 비교가 수정안 확정의 다음 증거다. R1 승인 후 확보한
실행 관측과 한컴 출력은 §9·§10에 추가한다.

## 9. R1 승인 후 Studio 실행 관측

### 9.1 입력·환경·증적

프로젝트 E2E의 `loadApp`·`loadHwpFile`을 재사용했다. 사용자 탭·문서는 건드리지 않고
별도 진단 탭에서 두 보존 샘플을 순서대로 열었으며, 완료 후 해당 탭만 닫았다.
새 빌드·폰트 설치·문서 저장·제품 코드 수정은 하지 않았다.

- 명령: `node output/7084/runtime-trace.mjs` — exit 0.
- 원문: `output/7084/runtime-trace.json`; 재현 스크립트는 위 명령 경로.
- Windows Chrome 152.0.7977.83, Studio `http://localhost:7700`, backend `canvas2d`,
  기본 선택, backend fallback 없음, devicePixelRatio=1.
- 실제 HTTP 응답 WASM 11,015,818 bytes, SHA-256
  `c31a359626b096d5f38d44fe6b1ae9a7e58093655ed4bb6c698c969a3d3e854b`.
  디스크 `pkg/rhwp_bg.wasm`과 일치했다.
- 실제 응답 JS SHA-256 `689619531172c805b070f50bd8f98899c9c24be5c34a3383b4e43fc8e4c8c3af`.
  Vite 변환 응답이므로 디스크 JS hash와 혼동하지 않는다.
- WASM은 [#3587 Stage 25](task_m100_3587_stage25.md)의 검증 제품
  `6a8aeb9ffac5d574e7a590ce66aba5eae9f7f374` 빌드와 hash가 같다.
  **최신 HEAD를 새로 빌드했다고 주장하지 않는다.** 그 기준과 현재 HEAD 사이의 renderer 차이는
  `layout/table_layout.rs` 1개뿐이며, 이번 text measurement·fit·WebCanvas 소스는 동일하다.
- 두 입력 SHA는 수행계획서와 일치했다. 실제 로드 결과는 각각 2쪽이었다.

### 9.2 두 형식에서 동일하게 관측한 인과 연결

| 관측 항목 | HWP | HWPX |
| --- | --- | --- |
| source 위치 | section 0, paragraph 12, nestedPath `[1,5,0]` | 동일 |
| 문자 / charShapeId | U+1F600 / 0 | 동일 |
| 지정 face / language slot | 함초롬바탕 / 0 | 동일 |
| metric 조회 | HCR Batang, entry 0, character miss | 동일 |
| 폭 결정 | `heuristicHalfwidth`, 500 HWPUNIT | 동일 |
| 측정 변환 | transforms 빈 배열 | 동일 |
| paint CSS 글자 크기 | 13.333px | 동일 |
| `measureText` 진행폭 | 18.3027038574px | 동일 |
| 최종 Canvas 가로/세로 배율 | 0.3642449081 / 1 | 동일 |
| 배율 적용 후 진행폭 | 약 6.666667px | 동일 |

`getFontDecisionTrace`의 source가 대상 셀·문자를 연결하며, 페이지 1(0 기반)의
`page:1:run:11:char:2`를 확인했다. `CanvasRenderingContext2D.fillText`를 진단 탭에 한해
감싸 원래 인자를 그대로 전달하고, `😀` 호출의 font·measureText·getTransform·stack을 기록했다.
`fillText`에는 maxWidth 인자가 없었다. 높이만 그대로 두는 축소는 Canvas transform에 있다.

원인 경로는 `text_measurement.rs`의 글리프 미등록 대체 폭 → 문자 위치 →
`web_canvas.rs::draw_text`의 cluster advance / 실제 폭 비교 →
`canvas_cluster_fit_scale` → `scale(ratio * fit, 1)`이다. UI 이미지 stretch라고 볼 근거는 없다.
이 실행은 **글자가 UTF-16에서 사라진 문제나 문서 장평 36% 지정 때문이 아니라**, 조판에서
임의 반각 폭을 배정한 뒤 더 넓은 fallback glyph를 그 폭에 맞추는 불일치임을 보여준다.

### 9.3 실제 face와 그림 경계의 한계

실제 Canvas CSS chain은 `"HCR Batang", 함초롬바탕, Batang, 바탕, "Nanum Myeongjo", …, serif`다.
이는 지정 후보 목록이며 `😀`를 그린 실제 face를 직접 식별한 결과가 아니다.
로컬 Segoe UI Emoji에서 계산한 폭과 매우 비슷해도 그 face라고 확정하지 않는다(`ambiguous`).
trace의 `studioSnapshotRequired`는 query 단독 paint 정보의 부재이며, 별도로 채집한 Canvas
실행 기록과 구분한다. trace provenance의 `ledgerSourceDrift`도 원문에 그대로 보존했다.

Canvas `actualBoundingBoxLeft/Right/Ascent/Descent`는 `[-2,16,12,2]`였다.
합산 그림 경계는 변환 전 14×14px, 변환 후 약 5.10×14px이다. 이는 **브라우저의 bounding-box
반환값**이지 최종 컬러 픽셀 마스크 실측이나 한컴 그림 경계가 아니다.

### 9.4 남은 검증 경계

확인한 것은 현재 재현 환경의 압축 원인이다. CanvasKit/native 실행, 실제 browser fallback face의
확정, 최초 발생 버전의 실행 비교는 아직 미검증이다. 한컴 PDF의 진행폭과 그림은 §10에서 확인했으며
정밀 픽셀 마스크 경계 비교는 수행하지 않았다.
기존 exact font registry는 문자 서식/언어 slot에 source를 결합하며 slot 충돌을 거부한다.
따라서 해당 slot 전체를 이모지 폰트로 덮어쓰는 수정은 일반 한글까지 바꿀 위험이 있다.
글리프 단위 fallback 선택과 공통 메트릭 전달 계약을 별도 구현계획에서 설계해야 한다.

## 10. 동일 입력의 한컴 PDF — 독립적인 비교 근거

`rhwp info --json`에서 HWP의 저장 제품은 null(버전 9.0.0.562), HWPX는
`hancom-office-2020`(11.0.0.3524)이었다. MCP 매뉴얼의 선택표에 따라 두 입력 모두
명시적으로 engine `2020`으로 `start → status → download`를 실행했다.
기존 저장본은 변경하지 않았다. 서버 profile은 2020, 실제 Hancom 버전은 **12.0.0.4605**이며
이 버전 표기를 한컴오피스 제품 연도로 재해석하지 않는다.

| 입력 | 산출물 | PDF SHA-256 |
| --- | --- | --- |
| 보존 HWP | `output/7084/oracle/stage11-filled-hwp-2020.pdf` | `ba1d5fbe6af800453143d8a7a16733339a25e3943764cb3d7f1e090fa820f83a` |
| 보존 HWPX | `output/7084/oracle/stage11-filled-hwpx-2020.pdf` | `0d4dadfd9523991c722bbbdeb83641c26426820beb80f139409a2c5488125b86` |

- 양쪽 `succeeded`, 입력 전처리 `none`, 2쪽, PDF producer `Hancom PDF 1.3.0.550`.
- 서버 font_scope는 session0/verified, 등록 2·실패 0. 이는 등록 확인이며 모든 glyph가
  특정 로컬 폰트 파일과 동일하다는 보장은 아니다. 서버 원본 폰트 파일 hash는 반환되지 않았다.
- `pdffonts`에서 SegoeUIEmoji subset의 실제 PDF 포함을 확인했다.
- `mutool draw -F trace … 2`의 해당 텍스트 span은 `SegoeUIEmoji`, `adv=1.3730469`,
  `trm="83 0 0 -83"`, 바깥 transform `0.12 0 0 0.12`다. 즉 문서 출력 좌표에서
  가로·세로 글자 배율의 절댓값이 같으며 이모지 폭을 0.5em으로 압축하지 않는다.
  출력의 유효 글자 크기는 9.96pt(83×0.12), glyph advance는 약 13.6755pt다.
  rhwp의 10pt와는 PDF 좌표 양자화도 구별해 비교해야 한다.
- `pdftoppm`으로 2쪽을 각각 PNG로 내보내 직접 확인했다. `실물` 뒤에 가로로 눌리지 않은
  웃는 얼굴이 있고 다음 줄에 `본문`이 있다. 두 2쪽 PNG hash는 동일하다:
  `c4d4796f78055be09ee7132c37d53104b23709cf30e7b6ee43215ac8b9870559`.
  해시 자체를 시각 판정으로 대신하지 않고 대상 그림을 확인한 뒤 기록했다.
- PDF 텍스트 추출은 이 glyph를 U+F000으로 보고한다. 원본 U+1F600 누락으로 판정하지 않는다.
  원본 문자열, 해당 위치의 emoji font span, 실제 출력 그림을 함께 대조했다.
- MuPDF는 xref 복구·ICC 미지원 경고를 냈지만 exit 0이며 Poppler 출력에서도 해당 그림을
  확인했다. 변환 성공만으로 무경고 PDF라고 기록하지 않는다.
- 증적: `output/7084/oracle/inspection.json`, `hwp-page2.trace.xml`, `hwpx-page2.trace.xml`,
  `hwp-page2.png`, `hwpx-page2.png` 및 상위의 비공개 접속정보를 제거한 job JSON.

이 결과로 **한컴 PDF는 실제 이모지 폰트의 진행폭을 쓰고 rhwp는 0.5em을 배정한 뒤 가로를
줄인다는 해당 사례의 차이**가 확인됐다. 정확한 한컴 내부 알고리즘이나 모든 이모지의
폭이 1.373em이라는 일반 규칙을 주장하지 않는다. 이 값을 제품 상수로 쓰지 않는다.
`bug-hunter`의 독립 정답지 원칙에 따라 소스 추론·브라우저 실측·한컴 출력 관측을 구분했다.
제품 수정 뒤의 최종 시각 판정은 작업지시자에게 요청한다.

## 11. 구현계획 인계 — 2026-09-13

작업지시자가 다음 절차 진행을 승인했다. 원인 조사는 구현계획 작성에 필요한 수준으로
정리하되, §9.4의 미검증 항목을 완료로 바꾸지 않는다. 원인 분류는 **기존 메트릭 누락·
그리기 불일치**이며 최초 화면 발생 버전은 미특정이다.

코드 확인에서 설계상 다음 제약을 추가로 확인했다.

- `EmbeddedTextMeasurer`의 전체 폭·문자 위치 및 trace는 `char_width_decision`을 공유한다.
  보정은 paint에서 끝내지 않고 이 측정 소유 계층에 전달되어야 한다.
- `ExactFontSourceRegistry`는 `char_shape_id/language_index` slot당 한 source를 소유한다.
  따라서 이모지만 대체하려고 slot 전체를 Segoe UI Emoji로 등록하지 않는다.
- `local-fonts.ts::loadLocalFontBytesFor`는 이미 허가·감지된 face의 bytes를 제공한다.
  CSS 존재 감지를 SFNT bytes 확보로 승격하거나 새 권한을 자동 승인하지 않는다.
- 기존 shaping publication은 source·문자 범위·측정·replay 증명을 담는 구조가 있지만
  선택/활성화 계약도 있다. 이모지 수정을 이유로 기존 shaping을 전역 활성화하지 않는다.
- Canvas2D의 암묵적 fallback face는 비관측이다. 동일 backend/CSS descriptor/폰트 세대에서
  측정·그리기를 묶는 실행 계약과 bytes로 증명한 exact 계약을 다른 상태로 설계한다.

`git fetch upstream devel` 후 원격은 `70bf40af2a2818e72bd58b4fa66e2d4c06de2b51`이다.
착수 base 뒤에는 #7106(창 제목 표시) 1건이 추가되었다. `wasm-bridge.ts`와 `main.ts`의
문서 이름 알림 변경이며 renderer/font 변경은 아니다. 이번 절차에서는 fetch만 수행했고
작업 브랜치에 merge하지 않았다. 구현 시작 전 최신 base를 재확인·통합한다.

Stage 1 문서 보존 뒤 별도 구현계획을 작성하여 승인을 요청한다.
새 제품 코드·DB·빌드·GitHub 댓글·push·PR 변경은 없다.

## 용어

- em / unitsPerEm: 글꼴 설계 좌표의 기준 단위. 실제 그림의 높이와 같다는 뜻은 아니다.
- cmap (Character to Glyph Index Mapping): 문자 코드에서 글리프 번호로 가는 매핑.
- hmtx (Horizontal Metrics): 글리프의 가로 진행폭과 왼쪽 여백 정보.
- advance: 다음 문자가 시작할 때까지의 진행폭. 그림 자체의 너비와 구별한다.
- outline / ink bounds: 외곽선 / 실제 그려진 영역의 경계. 컬러 글리프에서는 구별하여 계측한다.
- fallback: 지정 글꼴에 없는 글자를 다른 글꼴 또는 대체 폭 정책으로 처리하는 것. 둘은 같지 않다.
- 보조 평면: U+FFFF를 넘는 문자 영역. 단독 `😀`는 문자 1개지만 UTF-16에서는 코드 단위 2개다.
- shaping(조형): 문자열과 폰트·서식에서 실제 글리프와 위치·진행폭을 구하는 과정.
- grapheme cluster: 사용자가 하나의 문자로 다루는 문자 묶음. 코드 포인트 수와 같지 않을 수 있다.
- VS16 (Variation Selector-16): 이모지 표현을 요청하는 선택자 U+FE0F.
- ZWJ (Zero Width Joiner): 지원되는 결합열에서 여러 문자를 하나의 표현으로 연결하는 U+200D.
- COLR (Color Table): OpenType의 컬러 글리프 구성 테이블. 조판 진행폭과 색상 레이어를 구별한다.
- ascent/descent: 기준선 위/아래의 폰트 메트릭. 개별 이모지의 실제 그림 높이와 같지 않을 수 있다.
