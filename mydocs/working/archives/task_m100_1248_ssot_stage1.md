# Stage 1 조사 — trailing/LineSeg SSOT 부재 지점

- **작성일**: 2026-06-03
- **관련 문서**:
  - `mydocs/tech/investigations/issue-1248/trailing_model_render_vs_pagination_1248.md`
  - `mydocs/tech/document_ir_lineseg_standard.md`
  - `mydocs/tech/hwp_save_guide.md`
- **성격**: 조사 전용. 코드 수정 없음.

## 1. 결론

현재 trailing `line_spacing`에는 SSOT가 없다.

더 정확히는 다음 3가지가 섞여 있다.

1. **저장 원본값**: HWP/HWPX `PARA_LINE_SEG.line_spacing`
2. **문단 flow advance**: 다음 문단 시작까지 이동할 높이
3. **페이지/단 fit 높이**: 페이지 끝에서 마지막 trailing을 제외해도 되는 높이

현재 코드는 이 셋을 하나의 `line_spacing` 값에서 각 레이어가 즉석으로 재해석한다. 그래서 `typeset`, `pagination`, `layout`, `height_cursor`, `table_layout`, `footnote/endnote`가 각자 trailing 포함/제외 규칙을 복제한다.

## 2. 문서/코드 의미 불일치

### 2.1 공식 스펙

한컴 공식 스펙은 `PARA_LINE_SEG`에 `줄의 높이`와 `줄간격`을 별도 필드로 둔다.

- `mydocs/tech/한글문서파일형식_5.0_revision1.3.md:1328-1338`

공식 스펙은 두 필드가 별도라는 점만 말하고, `줄의 높이`가 `줄간격`을 포함하는지까지는 명시하지 않는다.

### 2.2 저장 가이드

`hwp_save_guide.md`의 빈 문서 예시는 다음 형태다.

```text
line_height: 1000
line_spacing: 600
```

- `mydocs/tech/hwp_save_guide.md:42-50`

이는 실질적으로 `line_height=base`, `line_spacing=extra` 모델에 가깝다.

### 2.3 IR 표준 문서와 모델 주석

`document_ir_lineseg_standard.md`와 `src/model/paragraph.rs`는 `line_height`를 "line_spacing 포함"이라고 적는다.

- `mydocs/tech/document_ir_lineseg_standard.md:34`
- `src/model/paragraph.rs:143`

하지만 같은 `document_ir_lineseg_standard.md`는 문단 간 vpos 연결을 다음처럼 정의한다.

```text
next_para.first_vpos = prev.last_vpos + prev.line_height + prev.line_spacing
```

- `mydocs/tech/document_ir_lineseg_standard.md`의 "paragraph 간 vpos 연결" 절

즉 문서 내부에서도 `line_height`가 spacing을 포함한다는 설명과 `line_height + line_spacing` 연결 공식이 충돌한다.

현재 코드의 보정 함수도 `line_height=base`, `line_spacing=extra` 모델을 따른다.

- `src/renderer/mod.rs:586-610`

판정: **IR 표준 주석부터 정정 필요**. 현재 실사용 모델은 `line_height`가 줄 본체 높이, `line_spacing`이 다음 줄/문단으로 가는 추가 간격이다.

## 3. 코드상 SSOT 분산 지점

### 3.1 typeset: 두 높이를 만든다

`FormattedParagraph`는 `total_height`와 `height_for_fit`을 모두 가진다.

- `src/renderer/typeset.rs:814-828`

생성 시:

```rust
total_height = spacing_before + sum(line_height + line_spacing) + spacing_after
height_for_fit = total_height - last_line_spacing
```

- `src/renderer/typeset.rs:2993-3002`

배치 시 fit 판정은 `height_for_fit`, 누적은 단단/다단에 따라 다르다.

- `src/renderer/typeset.rs:3213-3240`

여기서 이미 "문단 높이"가 두 개다.

| 이름 | 의미 |
|---|---|
| `total_height` | flow advance 후보 |
| `height_for_fit` | 페이지 끝에서 trailing을 제외한 fit 후보 |

문제는 이 둘의 의미가 타입으로 분리되어 있지 않아, 이후 레이어가 다시 추측한다는 점이다.

### 3.2 pagination: 같은 trailing 제외 규칙을 다시 만든다

pagination은 일반 문단 fit에서 마지막 `line_spacing`을 다시 직접 뺀다.

- `src/renderer/pagination/engine.rs:1148-1168`

TAC 표 flush에서도 직접 뺀다.

- `src/renderer/pagination/engine.rs:512-523`

빈 마지막 문단 guard에서도 직접 뺀다.

- `src/renderer/pagination/engine.rs:536-554`

표 직후 overflow threshold에서도 직접 뺀다.

- `src/renderer/pagination/engine.rs:1242-1257`

TAC 표 높이에서도 직접 `without_trail`을 만든다.

- `src/renderer/pagination/engine.rs:1856-1868`

판정: pagination은 `FormattedParagraph.height_for_fit` 같은 공통 산출물을 신뢰하지 않고, 상황별로 trailing policy를 재구성한다.

### 3.3 render layout: flow advance와 content bottom을 분리한다

문단 렌더는 셀 밖 일반 문단에서 대부분 `line_height + line_spacing`으로 advance한다.

- `src/renderer/layout/paragraph_layout.rs:4375-4491`

하지만 overflow 검출용 content bottom은 trailing/spacing_after를 제외한다.

- `src/renderer/layout.rs:503-508`
- `src/renderer/layout/paragraph_layout.rs:4457-4463`

zone leaving에서도 마지막 문단 trailing을 별도로 빼는 특례가 있다.

- `src/renderer/layout.rs:2442-2491`

판정: render는 실제 y 이동, 콘텐츠 하단, zone 하단을 서로 다른 값으로 관리한다. 개념 자체는 필요하지만, 공통 타입이 없어 주석과 특례가 기준점 역할을 한다.

### 3.4 height_cursor: 저장 vpos와 렌더 y 사이에서 trailing을 추측한다

`HeightCursor::vpos_adjust`는 직전 문단의 vpos 끝을 다음처럼 계산한다.

```rust
prev_vpos_end = seg.vertical_pos + seg.line_height + seg.line_spacing
```

- `src/renderer/height_cursor.rs:139`

그러나 lazy base 산출 시 trailing bridge를 조건부로 0 또는 `last.line_spacing`으로 선택한다.

- `src/renderer/height_cursor.rs:150-178`

미주 compact flow에서는 `line_spacing`이 다음 중 하나로 쓰인다.

- 이미 y_offset에 포함된 값
- 아직 y_offset에 누락되어 더해야 할 값
- 한컴 저장 vpos가 의도한 gap
- between-notes와 중복되므로 빼야 할 값

관련 분기:

- `src/renderer/height_cursor.rs:274-322`
- `src/renderer/height_cursor.rs:420-467`

판정: `height_cursor`는 현재 구조에서 "SSOT 부재의 보정기" 역할을 한다. 원천 모델이 아니라 레이어 간 불일치를 조정하는 추측 레이어다.

### 3.5 footnote/endnote: note spacing과 trailing의 책임이 겹친다

각주 마지막 paragraph는 trailing을 빼고 note spacing이 간격을 책임진다.

- `src/renderer/layout/picture_footnote.rs:735-785`
- `src/renderer/layout/picture_footnote.rs:907-917`

미주는 `typeset`에서 between-notes를 직전 paragraph 마지막 `line_spacing`에 주입하고, 별도로 `endnote_between_notes_hu`도 저장한다.

- `src/renderer/typeset.rs:2204-2228`
- `src/renderer/height_cursor.rs:58-61`

판정: footnote/endnote 영역은 `line_spacing`과 note spacing이 같은 시각 간격을 서로 책임질 수 있어, 중복 방지 특례가 필수로 늘어난다.

### 3.6 표 셀: HeightMeasurer와 table_layout이 정책을 복제한다

셀 내부 줄 높이는 `HeightMeasurer`와 `table_layout` 모두에서 `include_trailing_ls` 정책을 직접 가진다.

- `src/renderer/height_measurer.rs:762-781`
- `src/renderer/layout/table_layout.rs:3705-3727`
- `src/renderer/layout/table_layout.rs:3741-3760`

판정: 현재는 두 구현이 같은 정책을 주석으로 맞추고 있다. 공통 helper가 없으면 한쪽만 수정되는 회귀가 계속 날 수 있다.

## 4. SSOT 부재의 실제 의미

SSOT가 없다는 말은 단순히 "공통 상수가 없다"가 아니다.

현재 없는 것은 다음 4개 계약이다.

| 계약 | 현재 상태 | 영향 |
|---|---|---|
| LineSeg 의미 계약 | 문서/주석이 `line_height` 의미를 혼동 | 저장/렌더/측정 해석이 흔들림 |
| 문단 flow advance 계약 | `total_height`, `height_for_fit`, `line_advances_sum`이 흩어짐 | 페이지네이션/렌더 누적 drift |
| fit height 계약 | 마지막 trailing 제외 여부를 호출부가 직접 결정 | 페이지 끝/다단/표 직후 회귀 |
| note gap 계약 | `line_spacing`, `between-notes`, note spacing이 중복 책임 | 미주/각주 특례 증가 |

## 5. 권장 보완 방향

### 5.1 1단계: 문서 정정

가장 먼저 `document_ir_lineseg_standard.md`와 `src/model/paragraph.rs` 주석을 정정한다.

권장 문구:

```text
line_height: 줄 본체 높이. line_spacing을 포함하지 않는다.
line_spacing: 이 줄 뒤에 이어지는 추가 간격. 다음 줄/문단 시작까지의 advance는
              line_height + line_spacing이다. 단, 페이지/단/셀/각주/미주 경계에서는
              consumer의 trailing policy에 따라 제외될 수 있다.
```

### 5.2 2단계: 공통 metric helper 도입

새 helper 또는 타입을 도입한다.

예시:

```rust
struct LineAdvance {
    content_h: f64,
    trailing_h: f64,
}

impl LineAdvance {
    fn flow_advance(&self) -> f64 { content_h + trailing_h }
    fn content_advance(&self) -> f64 { content_h }
}

enum TrailingContext {
    NormalFlow,
    PageFitLastItem,
    CellLastLine { row_break: bool, multi_para: bool },
    FootnoteLastPara,
    EndnoteSameNote,
    EndnoteBetweenNotesBoundary,
    ZoneLeaving,
}
```

핵심은 "마지막 trailing을 뺄 것인가"를 호출부가 직접 `- trailing_ls`로 쓰지 않고, context별 정책으로 통과시키는 것이다.

### 5.3 3단계: 중복 제거 우선순위

권장 순서:

1. 문서/주석 정정
2. 셀 내부 `include_trailing_ls` 공통 helper화 (`height_measurer`와 `table_layout` 동시 적용)
3. `FormattedParagraph`에 `fit_height`, `flow_advance`, `content_bottom_delta`를 명시하고 호출부 `- trailing_ls` 직접 계산 축소
4. 미주 A 정규화는 별도 이슈로 유지. D/E 게이트는 건드리지 않는다.

## 6. 후속 이슈 필요 여부

필요하다.

권장 이슈 제목:

```text
LineSeg trailing/flow metric SSOT 정리 — 문서 의미 정정 및 공통 trailing policy helper 도입
```

권장 범위:

- `document_ir_lineseg_standard.md`, `src/model/paragraph.rs` 주석 정정
- `line_height=base`, `line_spacing=extra` 모델 명문화
- 셀/표/본문의 trailing include/exclude 공통 helper 도입
- 코드 동작 변경은 최소화하고, 우선 중복 정책을 같은 helper로 묶기

비범위:

- 미주 `height_cursor` D/E 게이트 제거
- #1248 A 정규화 전체 구현
- 페이지네이션 결과 변경을 동반하는 대규모 리라이트

## 7. 현재 판정

SSOT 부재는 실제다. 특히 문서상 `line_height` 의미가 코드와 다르게 적혀 있어, 이후 구현자가 `line_height`를 spacing 포함값으로 오해할 위험이 크다.

다만 한 번에 render/pagination/typeset 전체를 통일하는 것은 위험하다. 먼저 문서 의미를 정정하고, 셀/표처럼 동일 정책이 복제된 부분부터 helper화하는 것이 가장 안전하다.
