# 최종 보고서 — Task M100-1414

- 이슈: https://github.com/edwardkim/rhwp/issues/1414
- 제목: Document IR 구조 감사 및 정리
- 브랜치: `local/task_m100_1414`
- 작성일: 2026-06-15

## 1. 결과 요약

Document IR의 주요 구조체와 필드에 대해 생산자/소비자 추적, 후보 분류, SOLID 관점
개선 포인트 정리를 완료했다.

이번 작업은 코드 동작 변경 없는 감사 작업이다. `src/model`의 필드를 단순히 제거할 수
있는지 보는 대신, rhwp의 IR이 HWP5/HWPX/HWP3 frontend lowering 결과와 renderer/serializer
backend contract를 함께 담는 컴파일러형 중간 표현이라는 관점에서 책임 경계를 정리했다.

핵심 결론은 다음과 같다.

- 즉시 제거해도 안전하다고 판단되는 필드는 없다.
- `Table.dirty`만 제거 후보로 표시하되, 실제 제거는 후속 이슈에서 전체 검색과 회귀 검증 후
  결정해야 한다.
- `raw_ctrl_data`, `raw_header_extra`, `raw_rendering` 계열은 삭제보다 계층 분리와 source
  priority 문서화가 필요하다.
- `line_segs`와 `char_offsets`는 parser artifact가 아니라 serializer와 renderer가 공유하는
  핵심 lowering contract다.
- SOLID 관점에서는 SRP/DIP 문제가 가장 두드러지며, 특히 table backend contract helper와
  unknown control raw payload 보존이 높은 우선도의 후속 과제다.

## 2. 산출물

| 파일 | 내용 |
|---|---|
| `mydocs/plans/task_m100_1414.md` | 수행 계획서 |
| `mydocs/tech/investigations/issue-1414/document_ir_audit_1414.md` | IR 필드 인벤토리, 생산자/소비자 매트릭스, 후보 분류, SOLID 개선 포인트 |
| `mydocs/report/task_m100_1414_report.md` | 최종 보고서 |
| `mydocs/orders/20260615.md` | 2026-06-15 작업 진행 기록 |

## 3. 감사 범위

주요 검토 대상:

- `src/model/document.rs`
- `src/model/paragraph.rs`
- `src/model/control.rs`
- `src/model/shape.rs`
- `src/model/table.rs`
- `src/model/style.rs`
- `src/model/page.rs`
- `src/model/header_footer.rs`

생산자/소비자 추적 범위:

- parser/frontend: HWP5, HWPX, HWP3, ingest/generated builder
- document_core: commands, converters, queries
- renderer/layout: composer, height measurer, pagination, layout/typeset
- serializer/backend: HWP5 CFB, BodyText, Control, HWPX serializer
- diagnostics/tests: 회귀 및 보존성 진단 근거

## 4. 주요 판단

### 4.1 유지해야 하는 핵심 IR 계약

| 대상 | 판단 |
|---|---|
| `Paragraph.line_segs` | HWP `PARA_LINE_SEG` 직렬화와 renderer/pagination 조판의 핵심 입력이다. |
| `Paragraph.char_offsets` | text/control 위치, field marker, caret, line segmentation 매핑의 핵심 position map이다. |
| `Paragraph.field_ranges` | FIELD_END 재삽입과 field query/runtime range 처리에 필요하다. |
| `Paragraph.tab_extended` | inline tab/cross-run tab layout 계약이다. |
| `PageBorderFill.basis/ui_basis` | renderer 기준과 UI 기준이 다르므로 통합하면 안 된다. 문서화 대상이다. |

### 4.2 문서화가 필요한 contract

| 대상 | 판단 |
|---|---|
| `DocInfo.raw_stream`, `Section.raw_stream` | 원본 보존 캐시이자 편집 무효화 프로토콜이다. |
| `Document.extra_streams` | HWP CFB 추가 stream 보존과 form script query에 쓰인다. |
| `Document.hwpx_aux_entries` | HWPX `version.xml`, `settings.xml`, `Preview/*` 보존 전용 채널이다. |
| HWPX verbatim 필드 | `hwpx_head_tail`, `raw_para_heads`, `raw_parameters_xml`, `img_dim`, `vertical_all`은 HWPX-only 보존 contract다. |
| `PageDef.pagination_bottom_tolerance` | 파일 포맷 필드가 아니라 pagination policy hint다. |
| `Document.is_hwp3_variant` | 현재 layout policy 스위치로 필요하지만, 장기적으로 compatibility profile로 분리할 후보다. |

### 4.3 분리 후보

| 대상 | 이유 |
|---|---|
| `Table.raw_ctrl_data`와 `Table.common` | Table만 raw bytes가 serializer source-of-truth이며 `common`과 dual maintenance가 필요하다. |
| `Table.raw_table_record_attr`와 semantic flags | raw 우선순위와 semantic 재구성 규칙이 섞여 있다. |
| `Cell.raw_list_extra`와 semantic cell fields | HWP5 LIST_HEADER tail, width materialization, HWPX name semantic이 섞여 있다. |
| `Paragraph.raw_header_extra`, `raw_break_type`, `ctrl_data_records` | HWP5 backend side-channel이 paragraph semantic과 같은 구조체에 있다. |
| `CommonObjAttr.hwp5_gen_shape_attr_bit26/28` | 공통 semantic보다 HWP5 writer generation flag에 가깝다. |
| `ShapeComponentAttr.raw_rendering`와 `render_*` | serializer raw source와 renderer affine source의 우선순위 규칙이 필요하다. |

## 5. SOLID 관점

가장 중요한 개선 축은 다음과 같다.

| 원칙 | 개선 필요 지점 |
|---|---|
| SRP | `Document`, `Paragraph`, `Table`, `ShapeComponentAttr`에 semantic/model, raw preservation, backend materialization, layout hint가 혼재한다. |
| OCP | HWPX verbatim 필드와 HWP3 layout 보정이 공통 IR 필드 확장으로 계속 누적되는 경향이 있다. |
| LSP | Table은 Picture/Shape와 달리 `raw_ctrl_data`가 source-of-truth라 shape-like object 계약이 다르다. `UnknownControl`도 payload 보존 측면에서 다른 control과 치환성이 약하다. |
| ISP | renderer, HWP5 serializer, HWPX serializer, document_core commands가 큰 모델 구조체 전체에 접근한다. view/helper 계층이 필요하다. |
| DIP | document_core 명령이 serializer byte layout과 raw offset을 직접 알고 조작하는 지점이 있다. |

## 6. 후속 이슈 후보

| 우선도 | 후보 | 목적 |
|---|---|---|
| 높음 | Table backend contract helper 도입 | `raw_ctrl_data` 직접 patch와 `common` dual maintenance를 helper로 통합한다. |
| 높음 | UnknownControl raw payload 보존 | unknown control도 payload roundtrip 보존 계약을 갖게 한다. |
| 중간 | Shape transform source priority helper | `raw_rendering` clear와 `render_*` 갱신 규칙을 한 곳에 모은다. |
| 중간 | HWPX preserve wrapper | HWPX-only verbatim 필드를 format-specific payload로 묶는다. |
| 중간 | Layout compatibility profile | HWP3-origin layout 보정 플래그를 renderer policy object로 분리한다. |
| 낮음 | `CharShapeMods`/`ParaShapeMods` DTO 위치 정리 | 저장 문서 IR과 command patch DTO의 모듈 책임을 분리한다. |
| 낮음 | `Table.dirty` 제거 검토 | 실제 runtime 소비가 약한 후보로 전체 검색과 회귀 검증 후 판단한다. |

## 7. 검증

이번 작업은 코드 변경 없는 감사/문서화 작업이므로 컴파일 테스트는 수행하지 않았다.

수행한 검증:

```bash
git diff --check
```

결과: 통과.

추적 근거:

- `rg` 기반 생산자/소비자 검색 결과를 감사 문서에 파일/라인 단위로 기록했다.
- raw preservation 필드는 serializer 소비처와 document_core 무효화 경로를 확인한 뒤 분류했다.
- SOLID 개선 포인트는 실제 필드 생산/소비 경로와 연결되는 후보만 보고서에 포함했다.

## 8. 남은 사항

이번 작업은 실제 구조 변경을 하지 않는다. 후속 이슈에서는 각 후보별로 별도 계획서를 작성하고,
serializer roundtrip, renderer 시각 회귀, HWPX/HWP5 변환 경로를 개별 검증해야 한다.

특히 `Table.raw_ctrl_data`와 `UnknownControl`은 보존성/저장 안정성과 직접 연결되므로, 후속
작업 시 작은 범위의 설계 변경과 focused regression test를 먼저 작성하는 편이 안전하다.
