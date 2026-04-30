# Task M100 #476 Stage 2 완료 보고서

## 1. 회귀 발견 + 수정

### 1-1. 발견된 회귀

`cargo test --release --lib` 실행 시 1건 실패:
- `wasm_api::tests::test_task78_rectangle_textbox_inline_images` — `samples/20250130-hongbo.hwp` 페이지 2의 `para[25]`(빈 텍스트 + 인라인 Rectangle Shape) 케이스에서 박스 안 인라인 이미지 2개가 누락됨.

원인: `paragraph_layout.rs:2087` "빈 paragraph + TAC만" 분기가 **Picture만 처리**하고 **Shape는 누락**. 기존 빌드에서는 inline_pos 미등록 + shape_layout fallback으로 우연히 동작했으나, Stage 1의 D 차단으로 fallback 경로가 막혀 박스+이미지가 누락되었다.

### 1-2. 수정 — Shape도 inline_pos 등록

`paragraph_layout.rs:2087` 빈 paragraph 분기에 Shape 처리 추가:

```rust
if let Control::Shape(shape) = ctrl {
    let common = shape.common();
    let shape_h = hwpunit_to_px(common.height as i32, self.dpi);
    let shape_y = (y + baseline - shape_h).max(y);
    tree.set_inline_shape_position(section_index, para_index, tac_ci, img_x, shape_y);
    img_x += tac_w;
    continue;
}
```

## 2. 검증 결과

### 2-1. 단위 테스트

```
cargo test --release --lib
test result: ok. 1078 passed; 0 failed; 1 ignored; 0 measured;
```

`test_task78_rectangle_textbox_inline_images` 통과.

### 2-2. 통합/문서/문자열 테스트

```
cargo test --release
모든 test result: ok. (1078 + 14 + 25 + 1 + 6 + ... 모두 통과)
```

### 2-3. clippy

```
cargo clippy --release --all-targets -- -D warnings
```

기존 baseline에 45건 에러 존재(본 변경 무관). 본 변경 파일(`shape_layout.rs`, `paragraph_layout.rs`)에 신규 clippy 에러 없음 — 변경 후 baseline과 동일.

### 2-4. 핵심 회귀 케이스 시각 검증

`samples/21_언어_기출_편집가능본.hwp` 페이지 12 (Issue #476):
- **Before D 차단**: 22번 문제 위쪽 빈 영역(y≈742)에 박스 + "배너지와 뒤플로" 텍스트가 잘못 출현
- **After D 차단**: 22번 위쪽 영역 깨끗. 사용자 보고 핵심 증상 해결.

차단 로그 확인:
```
$ RHWP_DEBUG_LAYOUT=1 ./target/release/rhwp export-svg samples/21_언어_기출_편집가능본.hwp -p 11
[#476 skip] inline Shape without inline_pos: sec=0 para=238 ci=0
```

paragraph 238의 박스(페이지 12에 잘못 라우팅된 PageItem::Shape)가 의도대로 차단되었다.

### 2-5. 페이지 11 영향

페이지 11 SVG에서는 paragraph 238의 인라인 박스가 **누락**되어 있다(이전부터 그러했음). paginator가 PageItem::Shape를 페이지 12에만 등록하기 때문. PDF 정답에는 페이지 11 단 1 본문 중간에 박스가 있어야 하므로, 본 누락은 Stage 3(A 본질 수정)에서 해결 대상이다.

## 3. 잔여 영향 / 부수 영향

### 3-1. 23번 paragraph 위치 어긋남(별개 이슈)

페이지 12 단 0의 23번 paragraph가 단 0 거의 끝(y≈1166)에 그려지고 답안 일부가 잘림. 단 0 사용량 `used=1012.4px vs hwp_used=1219.3px (diff=-206.8px)`.

이는 paragraph 238의 페이지 12 영역(lines 4..26)이 차지하는 높이가 한컴 기준보다 크게 계산되어 후속 paragraph가 밀려난 결과로 추정. 본 task의 박스 라우팅 수정과는 별개의 layout 버그로, 본 task에서는 다루지 않음(별도 이슈 권장).

### 3-2. 페이지 11 박스 누락

Stage 3(A 단계)에서 paginator가 PageItem::Shape를 박스가 속한 line의 페이지에 등록하면 자동 해결.

## 4. 다음 단계

Stage 3 — paginator의 인라인 Shape 페이지 라우팅 수정. 박스가 속한 line이 라우팅된 페이지에 PageItem::Shape를 등록하도록 변경.
