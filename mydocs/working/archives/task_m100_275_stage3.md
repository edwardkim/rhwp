# Task #275 단계 3 완료보고서 — RawSvg 일반 경로 (B) + Placeholder 시각 검증

**이슈**: [#275](https://github.com/edwardkim/275)
**브랜치**: `local/task275`
**계획서**: [`task_m100_275_impl.md`](../plans/task_m100_275_impl.md) §3 단계 3

## 1. 변경 내용

### 1.1 `src/renderer/svg_fragment.rs` 확장

공용 헬퍼 2건 추가:

1. `is_svg_prefix(data: &[u8]) -> bool`
   - 선행 공백 스킵 후 `<svg` 또는 `<?xml ... <svg` 시작 여부 감지
   - 256바이트 창 제한으로 잘못된 긴 XML 선언 거부
2. `wrap_svg_fragment(fragment, x, y, w, h) -> String`
   - `<svg xmlns=... width=w height=h viewBox="x y w h">{fragment}</svg>`
   - bbox 좌표계와 viewBox 일치시켜 조각 내부의 **페이지 절대좌표**가 `drawImage(img, x, y, w, h)` 호출 시 정확히 원래 위치에 렌더되도록 설계

단위 테스트 **+7건** 추가 (총 19건):
- `wrap_svg_fragment_basic` / `wrap_svg_fragment_preserves_fragment_content`
- `is_svg_prefix_direct` / `is_svg_prefix_xml_decl` / `is_svg_prefix_rejects_png` / `is_svg_prefix_rejects_html_etc` / `is_svg_prefix_xml_without_svg`

### 1.2 `src/renderer/web_canvas.rs`

#### (a) `detect_image_mime_type` 확장

`svg_fragment::is_svg_prefix` 를 호출하는 분기 추가:
```rust
} else if super::svg_fragment::is_svg_prefix(data) {
    "image/svg+xml"
```

기존 `draw_image` 가 자동으로 `data:image/svg+xml;base64,...` URL 을 만들어 `HtmlImageElement` 로 로드 → **별도 draw_svg 함수 불필요**. 코드 중복 없이 기존 IMAGE_CACHE · async 로드 · 재렌더 트리거 파이프라인을 그대로 공유.

#### (b) `RawSvg` arm 의 else (B 경로) 활성화

```rust
} else {
    let svg_doc = wrap_svg_fragment(&raw.svg, node.bbox.x, node.bbox.y,
                                    node.bbox.width, node.bbox.height);
    self.draw_image(svg_doc.as_bytes(), node.bbox.x, node.bbox.y,
                    node.bbox.width, node.bbox.height);
}
```

A 경로와 B 경로 모두 동일한 `draw_image` 를 경유. 차이는 "단일 `<image>` 를 디코드해 이미지 바이트로 넘기나 (A)" vs "조각을 `<svg>` 로 래핑해 SVG 문서 바이트로 넘기나 (B)" 뿐.

추가 라인: **+15 줄**

## 2. 검증

### 2.1 단위 테스트 (신규 7건)

`cargo test --lib svg_fragment`:
```
running 19 tests
test result: ok. 19 passed; 0 failed
```

### 2.2 전체 lib 테스트

`cargo test --lib`: **968 passed / 14 failed**
- 949 baseline + 19 (단계2 12 + 단계3 7) = **968** ✓
- 14 failed: baseline cfb_writer/wasm_api 직렬화, 변화 없음

### 2.3 WASM 빌드

- `cargo check --lib --target wasm32-unknown-unknown`: clean
- `wasm-pack build --target web`: 성공. `pkg/rhwp_bg.wasm` = **4,051,019 bytes**
  - 단계 2 대비 -4,943 bytes (릴리즈 최적화 차이)

### 2.4 시각 검증

**Placeholder 경로** (강제 재현 → 원복):
- `shape_layout.rs` 에 `FORCE_PLACEHOLDER` 가드 임시 삽입, WASM 재빌드
- 결과: bitmap.hwp 에 svg.rs 와 동등한 **회색 배경 + 점선 테두리 + "OLE 개체 (BinData #1)" 중앙 라벨** 렌더 확인
- 원복 후 diff clean

**RawSvg B 경로** (강제 복합 SVG → 원복):
- `shape_layout.rs` 의 native_image 경로에 `TEST_FORCE_BPATH` 가드 임시 삽입
- 원본 단일 `<image>` 를 `<g><rect stroke="#ff0000"/><image/><text>B-PATH</text></g>` 복합 SVG 로 교체
- 결과: bitmap.hwp 에 **빨간 사각형 테두리 + "B-PATH" 라벨 + 내부 이미지** 모두 동시 렌더 확인
  - `<svg>` 래핑 → Blob data URL 인코딩 → HtmlImageElement async 로드 → drawImage 전체 파이프라인 정상 작동
- 원복 후 diff clean

**A 경로 회귀** (원복 후):
- `bitmap.hwp` / `한셀OLE.hwp` → 단계 2 와 동일하게 이미지 렌더
- `biz_plan.hwp` / `form-002.hwpx` → 회귀 없음

## 3. 구현 결정 기록

### 3.1 draw_svg 분리 vs detect_image_mime_type 확장

**선택: MIME 감지 확장**. SVG 전용 `draw_svg_async` 를 쓰지 않고 기존 `draw_image` 에 SVG 인식을 더한 이유:
- **코드 재사용**: IMAGE_CACHE / HtmlImageElement / 재렌더 로직 중복 방지
- **동일 비동기 의미론**: OLE 이미지가 PNG 든 SVG 든 사용자 경험이 동일 (첫 프레임 미로드 → 다음 프레임 hit)
- **캐시 통합**: 한 캐시로 모든 이미지 리소스 관리. LRU(200) 크기도 공유

### 3.2 viewBox 좌표계 선택

조각 내부의 좌표는 `render_x/y/w/h` (페이지 절대좌표). `<svg>` 의 viewBox 를 **bbox 와 동일하게** 설정하고 width/height 도 bbox 크기와 동일하게 맞추면, `drawImage(img, bbox.x, bbox.y, bbox.w, bbox.h)` 호출 시:
- 이미지 내부 viewBox origin (render_x, render_y) → 캔버스 (bbox.x, bbox.y) = 같음
- 이미지 스케일 1:1 → 조각 내 좌표 = 캔버스 좌표

수식 유도 대신 **단순 동일식** 으로 투영하는 것이 디버깅에 유리하다.

### 3.3 `is_svg_prefix` 의 256바이트 제한

`<?xml ... ?>` 선언이 비정상적으로 길거나 `<svg>` 가 훨씬 뒤에 나오는 경우 (첨부 XML 등) 를 SVG 로 오인 매칭하지 않도록 검색 창 제한. SVG 표준상 xml 선언 직후 루트 요소가 즉시 나오므로 256바이트는 충분한 여유.

## 4. 커밋 예정 파일

```
src/renderer/svg_fragment.rs                 (수정: is_svg_prefix + wrap_svg_fragment + 7 테스트)
src/renderer/web_canvas.rs                   (수정: detect_image_mime_type 확장 + RawSvg else 활성)
mydocs/working/task_m100_275_stage3.md       (신규, 본 보고서)
```

임시 파일 (_bitmap.hwp, _hancell_ole.hwp, debug-load-bug.test.mjs, first-readme.txt, preview.log) 는 단계 4 에서 정리.

## 5. 다음 단계

**단계 4 — 정리 + 최종 보고서**
1. 임시 파일 정리
2. (선택) 정식 e2e 테스트 파일 편입
3. 회귀 스모크 (기존 HWP/HWPX 몇 건)
4. `mydocs/report/task_m100_275_report.md` 최종 결과보고서
5. `mydocs/orders/20260424.md` 에 Task #275 항목 추가

## 6. 승인 요청

단계 3 승인 후 커밋 + 단계 4 착수 진행.
