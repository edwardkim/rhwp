# Task M100 #1379 구현계획서 — HWPX serializer subList 내부 컨트롤 보존

- 수행계획서: `mydocs/plans/task_m100_1379.md` (승인됨)
- 브랜치: `local/task1379`
- 단계: 4단계

## 0. 사전 조사 확정 사항

### 0.1 통합 지점 — `render_paragraph_parts()` (이미 존재)

`section.rs:154`의 `render_paragraph_parts(para, vert_start, ctx) -> (runs_xml, linesegs, vert_end)`가
**각주/미주(`render_note_sublist`)·머리말/꼬리말이 이미 쓰는 공유 문단 직렬화 경로**다:

- `render_runs()` 호출 — 컨트롤 슬롯 방출(`render_control_slot` 15종 디스패치) + #1378 run 분할
- lineseg: IR `line_segs` 보존 출력(#177), 없으면 fallback 생성

셀·글상자만 `render_text_runs()`(텍스트 전용)를 쓰는 비대칭이 본 이슈의 원인.
**구현 방향: 셀·글상자 문단 루프를 `render_paragraph_parts()` 호출로 전환**한다.
본문 경로 코드는 수정하지 않는다 (가시성 `pub(crate)` 조정만).

상호 재귀(셀 안 표)는 `render_control_slot → write_table → write_cell → write_sub_list →
render_paragraph_parts → render_runs` 고리로 자연 성립 — 전부 동일 crate 내 함수.

### 0.2 IR 보존 여부 (parser/hwpx/section.rs 확인 완료)

| 항목 | IR 보존 | 근거 |
|------|---------|------|
| 셀·글상자 문단 컨트롤 | **보존** | `parse_paragraph` 공유 (drawText: 3387-3391) |
| rect pt0~pt3 | 보존 | 파서 3474 → `x_coords/y_coords` |
| lineShape / fillBrush / shadow | 보존 | 3465 / 3568 / 3571 → `border_line` / `fill` / `shadow_*` |
| renderingInfo / curSz / orgSz / offset / flip / rotationInfo | 보존 | 3455-3456, 3565 → `ShapeComponentAttr` |
| drawText subList `textDirection` | **부분 보존** | 3375-3382 → `TextBox.list_attr` bit 0~2 (code 1). VERTICAL/VERTICALALL 구분은 소실 |
| drawText `name`/`editable`, rect `numberingType` | **미보존** | 파서 미적재 — 1단계에서 영향 확인 후 포함/후속 분류 |

수정은 serializer 측에 집중된다. 미보존 항목은 1단계 측정에서 한컴 열기 거부와의
관련성을 확인해 본 타스크 포함 여부를 분류한다 (승인 요청).

## 1단계 — 게이트 강화 + 전수 측정 + 미보존 항목 분류

#1378 1단계와 동일 패턴: 강화된 게이트를 먼저 세워 작업 전 실패를 가시화한다.

### 1.1 `diff_documents` 컨트롤 비교 추가 (`roundtrip.rs`)

- `DocumentDiff`에 `ParagraphControls { section, paragraph, path, expected, actual }` 추가
  — 문단별 **컨트롤 타입 시퀀스**(예: `[Table, Picture]`) 비교. #1378의
  `diff_paragraph_char_shapes` 재귀(셀·글상자 Group·각주/미주)에 동승
- Bookmark 등 슬롯 외 컨트롤 처리: `is_hwpx_inline_slot` 기준으로 비교 대상 정의
  (파서가 IR에 넣는 순서와 serializer 방출 순서의 정합을 단위 테스트로 고정)

### 1.2 전수 사전 측정

- 강화 게이트로 baseline 전수 실행 → 실패 샘플×diff 건수 측정
- `XFAIL_1378_RECURSIVE` 12건이 컨트롤 diff로도 검출되는지 교차 확인
- 신규 실패는 임시 xfail(`XFAIL_1379_CONTROLS`, 사유 명기) 등록 + still-fail 가드

### 1.3 미보존 항목 분류

- tbox-v-flow-01 원본 기준 name/editable/numberingType의 한컴 필수 여부 판단 자료 수집
  (원본 XML·OWPML 스키마 대조)
- 루트 `xmlns:hwpunitchar` 누락 등 경량 항목 목록화 → 3단계 포함 여부 분류
- 분류 결과 승인 요청

완료 조건: 게이트 강화 + 측정 보고 + 분류 승인. `cargo test --lib`/`--tests` 그린
(임시 xfail 등록 상태).

## 2단계 — 셀 경로 전환 (`table.rs`)

### 2.1 `write_sub_list` 전환

- 문단 루프 본체를 `render_paragraph_parts(para, vert_cursor, ctx)` 호출로 대체:
  - `render_hp_p_open(para, ctx.next_para_id())` + runs + linesegarray 조립
    (각주 `render_note_sublist`와 동일 패턴)
  - 기존 합성 lineseg 블록 제거 — IR `line_segs` 있으면 보존, 없으면 fallback
    (각주 경로와 동일 동작. 원본 lineseg 재현 자체는 #1380 범위로 유지)
- `render_paragraph_parts`/`render_hp_p_open` 가시성 `pub(crate)` 조정
- `hp:tc` 속성(name/header/hasMargin/textDirection/vertAlign 등) 기존 유지

### 2.2 단위 테스트 (table.rs / roundtrip.rs)

| 테스트 | 고정하는 동작 |
|--------|--------------|
| 셀 문단 Picture 방출 | 셀 내 `hp:pic` 출력 + binItemIDRef 참조 |
| 셀 안 표 재귀 | 중첩 `hp:tbl` 방출 + `next_para_id` 채번 단조 증가 |
| 셀 컨트롤+텍스트 슬롯 위치 | char_offsets 8 유닛 갭에서 컨트롤 위치 정확 |
| ta-pic-001-r roundtrip | `hp:pic` 4개 보존 (full roundtrip) |
| 셀 char_shapes 경계 | #1378 경계 시프트(8 배수) 해소 — `(0,77)` → `(8,77)` 복원 |

완료 조건: 위 테스트 + `XFAIL_1378_RECURSIVE` 중 셀(tbl) 기인분 해소 확인
(가드 실패 → 승격은 4단계에서 일괄). `cargo test --lib` 그린.

## 3단계 — 글상자 경로 전환 + rect/drawText 보존 (`shape.rs`)

### 3.1 `write_draw_text_paragraph` 전환

2.1과 동일 — `render_paragraph_parts()` 호출로 대체, 합성 lineseg 제거.

### 3.2 `write_draw_text` 속성 보존

- subList `textDirection`: `tb.list_attr & 0x07 == 1`이면 `"VERTICAL"` 출력
  (VERTICALALL 구분 소실은 1단계 분류 결과에 따라 — 필요 시 `TextBox` 보존 필드 추가)
- vertAlign: `tb.vertical_align` 반영 (현재 "TOP" 하드코딩)
- textMargin 위치를 원본 순서(subList 뒤)로 이동

### 3.3 `write_rect` 하위 요소 보존 + 요소 순서

원본/OWPML 순서에 맞춰 재구성 (1단계에서 원본 샘플 전수로 순서 확정):

1. 속성: `numberingType`·`instid`를 IR 값으로 (`DrawingObjAttr.inst_id`, 1단계 분류 반영)
2. offset / orgSz / curSz / flip / rotationInfo / renderingInfo — `ShapeComponentAttr`에서
3. lineShape — `border_line`, fillBrush — `fill`, shadow — `shadow_*`
4. drawText (3.1~3.2)
5. pt0~pt3 — `x_coords/y_coords`
6. sz / pos / outMargin / shapeComment

다른 도형(ellipse/line/arc/polygon/curve)도 공통 하위 요소(2·3번)는 같은 헬퍼로 방출
가능한 구조로 작성하되, **적용 범위는 rect 우선** — 타 도형 확대는 측정 결과에 따라
분류 (범위 폭주 방지).

### 3.4 단위 테스트 (shape.rs)

| 테스트 | 고정하는 동작 |
|--------|--------------|
| 글상자 문단 Picture 방출 | drawText 내 `hp:pic` 출력 |
| 글상자 세로쓰기 보존 | list_attr bit → `textDirection="VERTICAL"` |
| rect pt0~pt3 방출 | 좌표 4점 + 요소 순서 |
| rect lineShape/fillBrush/shadow 방출 | IR 값 반영 |
| tbox-v-flow-01 roundtrip | rt XML이 원본 필수 요소를 전부 포함 (요소 단위 대조) |

완료 조건: 위 테스트 + shape.tb 기인 xfail 1건 해소 확인. `cargo test --lib` 그린.

## 4단계 — 전수 검증 + xfail 승격 + 한컴 판정 요청

1. `XFAIL_1378_RECURSIVE` 12건(#1379 기인분) 제거 → baseline 승격
   (143E433F503322BD33 1건은 #1382로 잔존), `XFAIL_1379_CONTROLS` 해소분 동일 처리
2. `rhwp hwpx-roundtrip --batch samples/hwpx -o output/poc/task1379` 전수 — 게이트와 정합 확인
3. 대표 샘플 SVG 비교 (#1378 4단계 절차) — 컨트롤 보존으로 페이지 폭증 샘플
   (form-002, 보도자료 등) 개선 측정
4. 한컴에디터 판정 요청 (작업지시자): **tbox-v-flow-01.rt 열기 성공** +
   ta-pic-001-r.rt 셀 이미지 표시 + 대표 rt 글상자/표 표시
5. 매뉴얼 `manual/hwpx_roundtrip_baseline.md` 갱신 (게이트 비교 항목·known limitations)
6. 최종 보고서 + 오늘할일 갱신
7. CI급: `cargo test --lib` / `--tests` / fmt / clippy / wasm check

완료 조건: 전 항목 + 잔존 한계의 이슈 귀속 명기 (#1380 lineseg, #1382 autoNum).

## 위험 관리 (수행계획서 5절 보강)

| 위험 | 대응 |
|------|------|
| 게이트 강화(1단계)로 신규 실패 다수 | 측정 먼저 → 임시 xfail + 분류 승인 (#1378 검증된 패턴) |
| 셀 lineseg가 합성→IR 보존으로 바뀌며 출력 변화 | 각주 경로와 동일 동작으로의 정렬 — baseline 2-round 검사로 드리프트 검출. 시각 개선/회귀는 SVG 비교로 측정 |
| render_control_slot의 컨트롤이 셀 문맥에서 부적합한 경우 (예: Header/Footer) | 파서가 셀 문단 IR에 넣는 컨트롤만 비교·방출 대상 — 1단계 인벤토리로 실재 타입 확인 |
| 요소 순서의 한컴 기대치 불확실 | 원본 샘플 전수의 실측 순서로 확정 (스키마 추정 금지 — 권위 자료 원칙) |
| PR #1366 충돌 | `table.rs` 변경을 `write_sub_list` 내부로 한정 |
