# Task #195 최종 보고서 — 차트/OLE 개체 렌더링 지원

> Issue: [#195](https://github.com/edwardkim/rhwp/issues/195)
> 브랜치: `local/task195` (from `local/devel`)
> 마일스톤: 미지정
> 기간: 2026-04-19

## 배경

- 재현 파일 `1.hwp` (외부, 저작권 이슈로 samples/ 포함 금지)
- 증상: 테이블 셀 안 차트 2개가 export-svg 시 완전 빈 사각형으로 출력
- 원인: rhwp의 GSO 파서가 `HWPTAG_SHAPE_COMPONENT_OLE`, `HWPTAG_CHART_DATA`를 미처리 (상수만 정의)

## 범위

- 포함:
  - `ShapeObject::Chart`, `ShapeObject::Ole` variant 추가
  - `HWPTAG_SHAPE_COMPONENT_OLE` 레코드 필드 파싱 (`parse_ole_shape`)
  - `HWPTAG_CHART_DATA` 감지 및 raw 보존(라운드트립)
  - placeholder SVG 렌더 (회색/파란 박스 + 점선)
- 제외(분리된 후속 이슈):
  - OLE 프리뷰 이미지 실제 추출
  - CHART_DATA 하위 태그(80~95) 구조화 파싱 및 차트 시리즈 렌더

## 단계별 진행

| 단계 | 커밋 | 내용 |
|------|------|------|
| 1 | `8c660f0` | 스펙 조사 + IR 설계 (문서만) |
| 2 | `80558bb` | Model 확장 (`ChartShape`/`OleShape`), 8개 매치 사이트 확장 |
| 3 | `2aa5f2f` | Parser (`parse_ole_shape`, 차트 우선 분류, 단위 테스트 3건) |
| 4 | `081df07` | Renderer (placeholder SVG 색상/테두리) |
| 5 | (본 커밋) | 검증 + 최종 보고서 + 오늘할일 갱신 |

## 주요 기술 결정

1. **차트/OLE 재분류 우선순위**: CHART_DATA 존재 시 Chart 우선, 없으면 OLE 태그로 분류
2. **라운드트립 전략**: IR 필드가 비어 있어도 `raw_chart_data` / `raw_tag_data`로 원본 바이트 보존 → 읽기·저장 시 손실 없음
3. **placeholder 스타일**: 기존 fill/stroke가 있으면 유지, 없을 때만 회색/파란 배경 오버라이드 → 사용자 의도 보존
4. **1.hwp 조사 결과**: 네이티브 HWP CHART_DATA가 아니라 MS Graph OLE 임베드 → 실무에서 OLE 경로가 더 빈번. OLE 렌더가 실질적 수혜

## 검증

- 단위 테스트: 878 passed (신규 3건 포함), 회귀 0
- tests/: 13 passed
- 1.hwp 수동 검증: 2개 OLE 차트 placeholder 정상 렌더 (`/tmp/task195_out/1_004.svg`에 `fill="#f0f0f0"` + `stroke-dasharray` 확인)
- 회귀: samples/draw-group.hwp, aift.hwp, biz_plan.hwp export-svg 크래시 없음

## 변경 파일 목록

```
src/model/shape.rs                        (+137)  enum + 신규 struct
src/renderer/layout/shape_layout.rs       ( +53)  Chart/Ole 렌더 arm
src/serializer/control.rs                 ( +68)  라운드트립 arm 3곳
src/document_core/commands/object_ops.rs  ( +10)  shape_attr 매치 확장 3곳
src/main.rs                               ( +11)  dump 출력
src/parser/control/shape.rs               ( +99)  parse_ole_shape + 분기 + 테스트
src/parser/control/tests.rs               (  +4)  테스트 헬퍼
mydocs/plans/task_195.md                  (신규)  수행계획서
mydocs/plans/task_195_impl.md             (신규)  구현계획서
mydocs/tech/hwp_chart_spec.md             (신규)  차트 스펙
mydocs/tech/hwp_ole_spec.md               (신규)  OLE 스펙
mydocs/working/task_195_stage1~5.md       (신규)  단계별 보고서
mydocs/working/task_195_report.md         (본 문서)
mydocs/orders/20260419.md                 (갱신)  오늘할일
```

## 분리 제안 후속 이슈

1. ~~OLE 프리뷰 이미지 추출 (BinData 압축 해제 + CFB 파싱 + WMF/EMF→SVG)~~ → **단계 6~14에서 완료**
2. CHART_DATA 하위 태그 파싱 + 차트 시리즈 실제 렌더 (네이티브 HWP chart data → 별도 이슈)
3. 자체 제작 `samples/chart-basic.hwp` 추가 (한컴오피스 authoring 환경 필요)
4. EMF+ (GDI+ 확장 레코드), AlphaBlend, GradientFill, Clipping Region (후속 이슈)
5. 한글 폰트 폴백 체인을 EMF 텍스트 `font-family`에 연결

---

# 스코프 확장 — 단계 6~8 (2026-04-19)

## 추가 배경

단계 1~5 완료 후, 작업지시자 요청으로 **실제 데이터 렌더링**까지 범위 확대. OLE 컨테이너에서 OOXML 차트 XML을 추출하여 네이티브 SVG로 출력하는 경로를 추가했다.

## 단계별 진행 (2차)

| 단계 | 커밋 | 내용 |
|------|------|------|
| 6 | `8675800` (통합) | BinData 해제 인프라 (`bin_data.rs`: zlib raw deflate + BinDataItem compressed 플래그) |
| 7 | `8675800` (통합) | 내부 CFB 파싱 (`ole_container.rs`: OlePres000 / OOXMLChartContents / Contents 추출) |
| 8 | `8675800` | OOXML 차트 네이티브 SVG 렌더 (`src/ooxml_chart/`: barChart / lineChart / pieChart) |

## 주요 기술 결정 (2차)

5. **차트 렌더 우선 순위**: OOXML(XML) > EMF 폴백 > placeholder. OOXML은 벡터·편집 가능한 고품질, EMF는 래스터/벡터 혼합 폴백
6. **Lazy load**: OLE 컨테이너 파싱은 렌더 시점에 수행 (import 비용 최소화)
7. **OOXML 1차 범위**: barChart / lineChart / pieChart만 완전 지원. 그 외는 Unknown → 폴백

## 검증 (2차)

- 단위 테스트: 890 passed
- 1.hwp 페이지 3/4: OLE 차트 위치에 실제 막대·선·파이 그래프 렌더 확인
- samples/ 전체: 회귀 크래시 없음

---

# 스코프 재확장 — 단계 9~14 (2026-04-19)

## 추가 배경

단계 8 완료 후, OOXMLChartContents가 없는 OLE 객체(워드/엑셀 임베딩, Visio, 수식 등)는 여전히 placeholder로 출력됨. **EMF 프리뷰 바이트를 네이티브 SVG 벡터로 변환**하는 독립 모듈을 신설하여, OOXML 부재 시에도 실제 내용을 렌더한다.

## 단계별 진행 (3차)

| 단계 | 커밋 | 내용 |
|------|------|------|
| 9 | `6971358` | EMF 스펙 조사 + IR 설계 (`emf_spec.md`, `emf_ir_design.md`) |
| 10 | `5ce00b2` | EMF 모듈 골격 + EMR_HEADER 파서 (+단위 테스트 6) |
| 11 | `6c0890a` | 객체/상태 레코드 파서 + DC 스택/ObjectTable (+단위 테스트 7) |
| 12 | `71d7bdf` | 드로잉 레코드 + SVG 컨버터 1차 (`Player`, `SvgBuilder`) (+테스트 7) |
| 13 | `407fb4a` | 텍스트(EMR_EXTTEXTOUTW) + 비트맵(EMR_STRETCHDIBITS) (+테스트 5) |
| 14 | (본 커밋) | shape_layout Ole arm EMF 폴백 연결 + 회귀 + 최종 보고서 |

## EMF 모듈 구조

```
src/emf/
├── mod.rs                         공개 API: parse_emf, convert_to_svg, Error
├── parser/
│   ├── mod.rs                     Cursor + 레코드 디스패처 (38종 분기)
│   ├── constants/record_type.rs   RecordType enum
│   ├── objects/                   Header, RECTL/POINTL/SIZEL, XForm, LogPen/Brush/FontW
│   └── records/
│       ├── header.rs              EMR_HEADER
│       ├── object.rs              CreatePen/Brush/Font + Select/Delete
│       ├── state.rs               SaveDC/RestoreDC, WorldTransform, Window/Viewport, Color/Mode
│       ├── drawing.rs             MoveTo/LineTo/Rect/Ellipse/Arc/Polyline16/...
│       ├── path.rs                BeginPath/EndPath/FillPath/...
│       ├── text.rs                EMR_EXTTEXTOUTW
│       └── bitmap.rs              EMR_STRETCHDIBITS
└── converter/
    ├── mod.rs                     Player, SvgBuilder re-export
    ├── device_context.rs          DeviceContext, DcStack, ObjectTable
    ├── svg.rs                     SvgBuilder + colorref_to_rgb + escape_xml
    └── player.rs                  레코드 순회 → SVG 노드 발행
```

## 지원 레코드 카탈로그 (총 38개 분기)

- **제어** (2): Header / Eof
- **객체** (5): CreatePen / CreateBrushIndirect / ExtCreateFontIndirectW / SelectObject / DeleteObject
- **상태** (13): SaveDC / RestoreDC / SetWorldTransform / ModifyWorldTransform / SetMapMode / SetWindowExtEx / SetWindowOrgEx / SetViewportExtEx / SetViewportOrgEx / SetBkMode / SetTextAlign / SetTextColor / SetBkColor
- **드로잉** (11): MoveToEx / LineTo / Rectangle / RoundRect / Ellipse / Arc / Chord / Pie / Polyline16 / Polygon16 / PolyBezier16
- **패스** (6): BeginPath / EndPath / CloseFigure / FillPath / StrokePath / StrokeAndFillPath
- **텍스트/비트맵** (2): ExtTextOutW / StretchDIBits
- 그 외: `Record::Unknown { record_type, payload }`로 보존

## 주요 기술 결정 (3차)

8. **WMF 모듈과 완전 분리** — EMF는 좌표 크기(32bit), RecordType, 헤더 구조가 전혀 달라 `src/emf/` 독립 모듈로 신설. 기존 `src/wmf/` 영향 없음
9. **sub-cursor 격리** — 디스패처가 각 레코드 payload를 정확한 길이로 잘라 별도 `Cursor`로 전달, 경계 침범 구조적 방지
10. **BMP 포맷 래핑** — DIB → PNG 재인코딩 대신 BMP 파일 포맷으로 래핑하여 `data:image/bmp;base64` 사용. 추가 crate 불필요, 정보 손실 없음
11. **좌표 매핑 1회** — Player는 EMF 논리 좌표 그대로 SVG 출력, `<g transform="matrix(sx 0 0 sy tx ty)">` 래퍼로 render_rect 배치. WorldTransform은 DC 저장만 (개별 도형 적용은 후속)
12. **의존성 제약** — 외부 crate 추가 없음 (base64 0.22, cfb, flate2, quick-xml 기존 재사용)

## 검증 (3차)

```
$ cargo build --release       → OK
$ cargo test --release emf::  → 25 passed; 0 failed
$ cargo test --release --lib  → 915 passed; 0 failed; 1 ignored
```

- EMF 단위 테스트 25건 (단계 10: 6, 11: 7, 12: 7, 13: 5)
- 전체 라이브러리 테스트 915 (단계 5 종료 시점 878 → +37)
- samples/ 전체 export-svg 회귀: 크래시 없음
- 1.hwp 회귀: 기존 OOXML 차트(페이지 3, 4) 렌더 유지. EMF 폴백은 OOXML 부재 시에만 활성화되므로 본 파일에서는 트리거되지 않음 (OOXML 경로 우선)
- WMF 회귀: 독립 모듈 유지로 기존 WMF 테스트 영향 없음

## 변경 파일 요약 (단계 6~14)

```
src/lib.rs                                            pub mod emf; pub mod ooxml_chart;
src/emf/                                   (신규 15)  EMF 파서 + 컨버터 + 25 단위 테스트
src/ooxml_chart/                           (신규 3)   OOXML 차트 파서 + 렌더러 (단계 8)
src/parser/bin_data.rs                     (신규)     BinData zlib raw deflate 해제 (단계 6)
src/parser/ole_container.rs                (신규)     CFB 파싱 + OlePres000/OOXMLChartContents/Contents 추출 (단계 7)
src/renderer/layout/shape_layout.rs        (수정)     Ole arm: OOXML → EMF → placeholder 3단 폴백
src/parser/mod.rs                          (수정)     bin_data / ole_container re-export
src/renderer/render_tree.rs                (수정)     RawSvgNode (단계 8)
src/renderer/svg.rs                        (수정)     RawSvg 출력 (단계 8)
mydocs/tech/emf_spec.md                    (신규)     MS-EMF 스펙 요약 (단계 9)
mydocs/tech/emf_ir_design.md               (신규)     EMF IR 설계 (단계 9)
mydocs/working/task_195_stage6~14.md       (신규)     단계별 보고서 9건
```

## 최종 후속 이슈 분리 제안

1. **EMF+ / GDI+ 레코드 지원** — AlphaBlend, GradientFill, Clipping Region (`EMR_ALPHABLEND`, `EMR_GRADIENTFILL`, `EMR_SELECTCLIPRGN`, `EMR_COMMENT_EMFPLUSRECORD` 컨테이너)
2. **EMF WorldTransform 개별 적용** — 현재는 DC에 저장만. 각 도형에 `<g transform>` 래퍼 추가 필요
3. **EMF 텍스트 회전** — `LOGFONTW.escapement` 적용
4. **한글 폰트 폴백** — `font_fallback_strategy.md` 체인을 EMF text `font-family`에 연결
5. **EMF 스톡 객체** — `0x80000000 | STOCK_*` 핸들 처리 (WHITE_BRUSH, BLACK_PEN 등 기본 객체)
6. **CHART_DATA 네이티브 파싱** — HWP 자체 차트 포맷 (현재는 raw 보존만)
7. **자체 제작 샘플 확보** — `samples/chart-basic.hwp`, `samples/ole-emf.hwp` 등

