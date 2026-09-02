---
kind: guide
status: active
canonical: mydocs/manual/render_backend_contract_catalog.md
last_verified: 2026-08-18
---

# RenderBackend 계약 카탈로그 (M06-f)

`src/render_backend/` 가 지키는 **출력 백엔드 최소 계약** 을 종류·능력·생명주기·
정직성·픽스처로 펼친 작성 가이드다. 어댑터 본체는 M06-1/2, 광고 정직성은 M06-3,
상호 diff 하네스는 M06-4, 네 번째 어댑터 작성 가이드는 M06-5 가 맡는다.
이 문서는 그 위에 **시험 가능한 표** 를 얹는다.

`src/renderer/**` 는 고치지 않는다. 직렬화기(`src/serializer/**`) 도 고치지 않는다.

## 1. 생명주기

호출 순서는 다음 정규식과 같아야 한다.

```
( begin_page  draw*  end_page )*  finish
```

어기면 백엔드는 오류를 내야 하고, 조용히 넘어가면 안 된다. 판정은
`PageState` 한 곳이 맡는다.

| 위반 | 오류 |
| --- | --- |
| `begin_page` 없이 `draw` | `NoOpenPage { call: "draw" }` |
| `begin_page` 없이 `end_page` | `NoOpenPage { call: "end_page" }` |
| 열린 페이지에 `begin_page` | `PageAlreadyOpen` |
| 열린 페이지에 `finish` | `UnclosedPage` |
| 폭/높이가 양수 유한값이 아님 | `InvalidPageSize` |
| `multi_page: false` 인데 두 번째 페이지 | `MultiplePagesUnsupported` |

`finish(self)` 는 산출물 소유권을 넘긴다. trait object 는 `finish_boxed` 를 쓴다.

## 2. 좌표·단위

- 단위는 **px**. HWPUNIT 환산은 이 계층 앞에서 끝난다.
- 원점은 페이지 왼쪽 위, y 는 아래로 증가.
- 좌표는 페이지 절대 좌표. `PaintOp` 는 평탄화된 leaf 다.
- 형식 고유 단위(pt, device px) 환산은 백엔드 안에서만 한다.

## 3. PaintOp 종류 표

문자열은 LayerTree JSON `"type"` 과 글자 그대로 같다.

| kind | 기본 plane | 필요 capability | 설명 |
| --- | --- | --- | --- |
| `pageBackground` | `background` | `none` | 페이지 배경. 재생은 항상 첫 plane. |
| `textRun` | `flow` | `vectorText` | 선택·검색 가능한 텍스트 런. |
| `glyphRun` | `flow` | `vectorText` | 셰이핑된 글리프 런. |
| `glyphOutline` | `flow` | `vectorText` | 글리프 외곽선. 일반 Path 가 아니다. |
| `charOverlap` | `flow` | `vectorText` | 글자겹침 명시 visual op. |
| `textControlMark` | `flow` | `none` | 문단 끝·줄바꿈·필드 마커. |
| `tabLeader` | `flow` | `none` | 탭 리더 geometry. |
| `textDecoration` | `flow` | `vectorText` | 밑줄·취소선·강조점. |
| `footnoteMarker` | `flow` | `vectorText` | 각주·미주 위첨자 마커. |
| `line` | `flow` | `none` | 직선. |
| `rectangle` | `flow` | `none` | 사각형. |
| `ellipse` | `flow` | `none` | 타원. |
| `path` | `flow` | `none` | 임의 패스. |
| `image` | `flow` | `images` | 래스터 이미지. |
| `equation` | `flow` | `none` | 수식 SVG 조각. |
| `formObject` | `flow` | `none` | 양식 컨트롤. |
| `placeholder` | `flow` | `none` | 자리표시. |
| `rawSvg` | `flow` | `none` | 미리 렌더된 SVG 조각. |

`glyphRun` / `glyphOutline` 는 셰이핑 입력이 필요해 합성 장면 빌더가 만들지 않는다.
카탈로그 행과 `paint_op_kind` match 는 존재한다.

## 4. 능력 정직성

`BackendCapabilities` 필드는 **최종 산출물이 그 성질을 보존하는가** 이다.

| 백엔드 | raster | vectorText | fonts | gradients | clip | images | multiPage | deterministic |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| svg | no | yes | no | yes | no | yes | no | yes |
| null | no | no | no | no | no | no | yes | yes |
| trace | no | no | no | no | no | no | yes | yes |
| png | yes | no | no | live | no | live | no | no |
| skia | yes | no | no | live | no | live | no | no |

`live` 는 `native-skia` 네이티브 빌드에서만 켜진다. 꺼져 있으면 `finish` 는 빈 산출물이다.

래스터 전용인데 `vector_text: true` 이면 `is_consistent()` 가 거짓이다.

## 5. 재생 순서

`replay_page` 는 `PaintReplayPlane::ORDERED` (배경 → 글 뒤 → 본문 → 글 앞) 를
바깥 루프로 돈다. 트리에 배경을 마지막에 넣어도 추적 로그의 첫 op 는
`pageBackground` 다. 픽스처 `s004-reorder` 가 이 불변식을 닫는다.

## 6. 픽스처

합성 장면은 `tests/fixtures/render_backend/scenes/*.json` 이다. 생성기는
`tools/render_backend/gen_m06f.py`. 각 장면은 id·치수·op·기대 kind 순서·
기대 TraceBackend 로그를 담는다. 장면 수는 196 이다.

통합 시험은 `tests/cases/render_backend_m06f_*.rs` 다. source-side
`#[cfg(test)]` 모듈은 늘리지 않는다.

## 7. 새 어댑터 체크리스트

1. `src/renderer/**` 를 고치지 않고 기존 공개 API 만 호출한다.
2. `PageState` 로 생명주기를 판정한다.
3. `BackendCapabilities` 광고 = 실지원. 얇게 평탄화하면 `clipping` 을 켜지 않는다.
4. 선택 피처가 꺼져도 타입은 컴파일되고 생명주기는 지키며, 광고가 빈 산출물을 숨기지 않는다.
5. 정직성 대조는 `honesty_table_holds` / M06-3 단위 시험에 접는다.
6. 상호 비교는 같은 `OutputFamily` 끼리만 바이트를 맞댄다.
7. 카탈로그에 없는 kind 를 새로 만들면 `PAINT_OP_KIND_SPECS` 와 `paint_op_kind` 를 같이 고친다.

## 8. 하지 않는 것

- gym / `scripts/visual_sweep.py` 수정
- serializer 수정
- `src/renderer/canvaskit_policy.rs` · `src/renderer/pdf.rs` 수정
- source-side `#[test]` 증가
