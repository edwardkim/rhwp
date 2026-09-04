# Task M100 #1451 1단계 완료보고서 — shapeComment 방출 구현

- 이슈: #1451
- 브랜치: `local/task1451`
- 작성일: 2026-06-21
- 단계: 1/3 (shapeComment 방출 구현)

## 변경 내용

`src/serializer/hwpx/section.rs` `render_common_shape_xml`:
caption 방출 직후, `</hp:{tag}>` 닫기 직전에 shapeComment 방출 추가.

```rust
// 설명 (#1451) — caption 직후 (OWPML AbstractShapeObjectType: outMargin→caption→shapeComment).
// picture.rs:104 선례와 동일 순서. legacy 경로(ellipse/arc/polygon/curve/chart/ole) 보존.
// 빈 description 미방출은 write_shape_comment 내부 가드로 보장된다.
match writer_to_string(|w| super::shape::write_shape_comment(w, c)) {
    Ok(xml) => out.push_str(&xml),
    Err(e) => eprintln!("[hwpx] Shape({tag}) shapeComment 직렬화 실패: {e}"),
}
```

- 기존 `shape.rs:718 write_shape_comment` 재사용 (빈 description 가드 일원화).
- `writer_to_string` (section.rs:1066, 동일 모듈) 으로 `Writer<W>` → String 어댑팅.

## 검증 (table-vpos-01.hwpx)

| 항목 | 수정 전 | 수정 후 |
|---|---|---|
| round-trip diff (`hwpx-roundtrip`) | 2 (polygon 2건) | **0 (PASS)** |
| 재직렬화 출력 shapeComment | 3 | **5 (다각형 2건 보존)** |
| XML 구조 | (polygon에 shapeComment 없음) | `<hp:outMargin/><hp:shapeComment>다각형입니다.</hp:shapeComment></hp:polygon>` |

OWPML 순서(outMargin → shapeComment) 정확. 알고리즘/스키마 변경 없음, 누락 요소 방출만 추가.

## 다음 단계

2단계: 게이트(task1392_*) + 보존 fixture + baseline 회귀 검증 + Polygon 보존 가드 테스트 추가.
