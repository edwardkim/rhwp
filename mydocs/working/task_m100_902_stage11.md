# Task #902 Stage 11 보고서 — POLYPOLYGON fix + sample16/18 구조 차이 분석

**Stage**: 11 / 12 (v2 확장)
**상태**: 완료

## 1. 결정적 발견 — sample16 vs sample18 구조 차이

작업지시자 보고: "sample18 page 11 의 그림은 정상 표시되는데 sample16 page 18 은 왜 안 됨?"

WMF binary 정밀 분석:

| 항목 | sample16 bin3 (paragraph 394, 4.7MB) | sample18 bin3 (211KB) |
|------|--------------------------------------|----------------------|
| Total records | 20,869 | 14,896 |
| **META_POLYPOLYGON (0x0415)** | **10,476** | **0** |
| META_POLYGON (0x0324) | 480 | 2,264 |
| META_DIBSTRETCHBLT (0x0B41) — 임베디드 비트맵 | 177 | 0 |
| META_EXTFLOODFILL (0x0416) | 168 | 0 |
| META_SETSTRETCHBLTMODE (0x0107) | 282 | 0 |
| META_SAVEDC (0x001E) | 640 | 0 |
| META_RESTOREDC (0x0127) | 640 | 0 |

**해석**:
- sample18 WMF = **단순 벡터** (폴리곤 + 텍스트 + 선)
- sample16 WMF = **복잡 합성** (POLYPOLYGON 음영 + 임베디드 비트맵 + flood fill + DC stack)

sample18 의 정상 렌더링 = 우리 renderer 의 약점 영역을 트리거 안 함.

## 2. POLYPOLYGON 처리 버그

### 2.1 기존 구현 (svg/mod.rs:1376~1437)

POLYPOLYGON 의 각 서브폴리곤을 **별도 `<polygon>` 요소**로 분리 생성:

```rust
for i in 0..number_of_polygons {
    let polygon = Node::new("polygon")
        .set("fill", fill.as_str())
        .set("points", points.join(" "));
    self.push_element(record_number, polygon);
}
```

### 2.2 문제

WMF spec [MS-WMF] §2.3.3.13 META_POLYPOLYGON:
> "The POLYGON_OBJECT type does not specify a state for filling intersected areas. The area is filled by alternating areas using the fill mode defined by `PolyFillMode`."

즉 POLYPOLYGON 은 **단일 영역의 다중 서브경로** (fill-rule 의 alternating/winding 으로 hole 처리). 별도 `<polygon>` 으로 분리하면:
- fill-rule 의 hole 처리 무력화
- SVG 요소 수 폭증 (sample16 의 10,476 → 수만 개 element)
- 음영/그라데이션 영역이 hole 없이 채워짐

### 2.3 수정

단일 `<path>` + M/L commands 로 합성:

```rust
let mut path_d = String::new();
for i in 0..number_of_polygons {
    for j in 0..points_of_polygon[i] {
        if j == 0 { path_d.push_str("M x y"); }
        else { path_d.push_str("L x y"); }
    }
    path_d.push_str(" Z");
}
let path = Node::new("path")
    .set("fill", fill.as_str())
    .set("fill-rule", fill_rule.as_str())
    .set("d", path_d);
```

## 3. 검증 결과

### 3.1 빌드 + 회귀

```
cargo build --release           — Finished
cargo test --release --all-targets — 1412 passed / 0 failed
cargo test --release --test svg_snapshot — 8 / 8 passed
```

### 3.2 sample16 page 18 SVG/PNG

- SVG: 534845 bytes (Stage 10: 534869 — 미세 감소)
- PNG (extracted from data URI): 292586 bytes (Stage 10: 292603 — 변경 확인)
- 시각: POLYPOLYGON 의 fill-rule hole 처리 적용 — 음영 영역 정합 개선

### 3.3 잔존 미구현 영역

sample16 의 다음 record 처리 강화 follow-up 후보:
- META_DIBSTRETCHBLT (구현됨, 정밀 검증 필요)
- META_EXTFLOODFILL (`not implemented`)
- META_SETSTRETCHBLTMODE (`not implemented`)

## 4. 산출물

- 소스 수정: `src/wmf/converter/svg/mod.rs` (poly_polygon)
- WMF 분석 도구: `examples/wmf_record_summary.rs`
- 본 보고서: `mydocs/working/task_m100_902_stage11.md`
- 검증: `/tmp/task902_s11/`, `/tmp/task902_s11_s18/`

## 5. 다음 단계

Stage 12: 최종 보고서 + PR (PR 생성 직전 작업지시자 명시 승인)
