# PR #2723 처리 결과 보고서

## 개요

- **이슈**: #2723
- **유형**: 버그 수정
- **영역**: HML 리더 (`src/parser/hml/reader.rs`)
- **증상**: HML 직렬화(export) 시 표 셀 안 글상자 문단이 셀로 흘러들어 글상자가 비고 셀이 오염됨

## 원인 분석

HML 파서 `reader.rs`의 `finish_paragraph()`는 `</P>` 종료 태그를 만나면 완성된 문단을
현재 컨텍스트에 귀속시킨다. 귀속 우선순위는 다음과 같았다:

1. `self.cells`가 비어있지 않으면 → 셀 문단으로 간주
2. `self.rectangles`가 비어있지 않으면 → 도형(text_box) 문단으로 간주
3. 그 외 → 구역(SECTION) 문단으로 간주

**문제점**: 표 셀 안에 글상자(사각형: RECTANGLE + DRAWTEXT)가 있는 경우, `cells`와
`rectangles` 스택이 모두 활성화된다. 기존 코드는 `cells`를 먼저 검사하므로
DRAWTEXT 안쪽 문단(`<P>`)이 글상자(text_box)가 아닌 셀(`cell.paragraphs`)에
귀속되었다. 그 결과:

- 글상자의 DRAWTEXT는 빈 `<PARALIST/>`가 되어 글상자 내용이 소실됨
- 셀에 글상자 문단이 추가로 오염되어 셀 구조가 깨짐

## 수정 내용

**파일**: `src/parser/hml/reader.rs` — `finish_paragraph()` 메서드

DRAWTEXT 컨텍스트 우선 검사를 추가했다. `self.stack`(XML 요소 스택)에
`" DRAWTEXT"`가 있으면 cells/rectangles 검사보다 먼저 rectangle의 text_box에
문단을 귀속시킨다.

```rust
// [#2723] DRAWTEXT (글상자) 컨텍스트 우선
if self.stack.iter().any(|name| name == "DRAWTEXT") {
    if let Some(rectangle) = self.rectangles.last_mut() {
        rectangle.text_box.push(paragraph);
        return Ok(());
    }
}
```

이 수정은 정상적인 XML 중첩 구조를 활용한다:

```xml
<CELL>
  <PARALIST>
    <P>  ← 셀 문단 (DRAWTEXT 없음 → 기존 로직)
      <TEXT>
        <RECTANGLE>
          <DRAWINGOBJECT>
            <DRAWTEXT>
              <PARALIST>
                <P>  ← 글상자 문단 (DRAWTEXT 있음 → 우선 처리)
```

- DRAWTEXT는 항상 RECTANGLE 내부에만 존재하므로, DRAWTEXT 바깥의 `<P>`는
  이 검사를 통과하지 않아 기존 귀속 로직이 정상 동작한다.

## 영향도

| 케이스 | DRAWTEXT | cells | rectangles | 동작 |
|--------|----------|-------|------------|------|
| DRAWTEXT 없는 일반 RECTANGLE | 없음 | - | O | 기존 로직 (rectangles) |
| 셀 없는 일반 DRAWTEXT | 있음 | 없음 | O | DRAWTEXT 우선 (text_box) |
| 중첩 글상자 (글상자 안 글상자) | 있음 | - | O | DRAWTEXT 우선 (text_box) |
| 표 셀 안 글상자 | 있음 | O | O | **수정 대상 — DRAWTEXT 우선** |
| 일반 셀 문단 (글상자 없음) | 없음 | O | - | 기존 로직 (cells) |
