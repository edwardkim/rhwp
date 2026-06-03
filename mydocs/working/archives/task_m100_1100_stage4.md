# Task #1100 Stage 4 — HWPX 바탕쪽 PAGE autoNum 치환 범위 정정

## 1. 배경

Stage 3에서 HWPX 바탕쪽 세로선 렌더링은 정상화되었지만, 바탕쪽이 보이기 시작하면서
바탕쪽 내부 표에 쪽번호가 출력되는 문제가 드러났다.

문제 샘플:

```text
samples/hwpx/exam_social.hwpx
```

관찰:

```text
masterPage@pageNumber="0"
masterPage/hp:subList@hasNumRef="0"
바탕쪽 내부 표 셀에 hp:autoNum numType="PAGE" 존재
본문 첫 문단에는 hp:newNum num="1" numType="PAGE" 존재
```

## 2. 원인

`hp:newNum`은 쪽번호 시작값을 지정하는 컨트롤이다. 쪽번호 표시 위치나 표시 여부를 의미하지 않는다.

현재 렌더러는 표/글상자 내부의 `AutoNumber(Page)`를 현재 페이지 번호로 무조건 치환했다.
그 결과 HWPX 바탕쪽에서 `pageNumber="0"`과 `hasNumRef="0"`로 번호 참조가 비활성화된 경우에도
바탕쪽 내부 `PAGE autoNum`이 실제 페이지 번호로 출력되었다.

## 3. 수정

수정 파일:

```text
src/model/header_footer.rs
src/parser/hwpx/section.rs
src/parser/body_text.rs
src/renderer/layout.rs
```

구현:

```text
1. HWPX masterPage@pageNumber를 MasterPage.hwpx_page_number로 보존한다.
2. HWP5에서 읽은 바탕쪽은 해당 XML 속성이 없으므로 None으로 둔다.
3. HWPX 바탕쪽이 pageNumber=0이고 hasNumRef=0이면,
   바탕쪽 렌더링 중 current_page_number를 0으로 두어 PAGE autoNum 치환을 억제한다.
4. newNum은 계속 쪽번호 시작값으로만 처리하고, 위치/표시 여부 판단에 사용하지 않는다.
```

## 4. 생성 파일

```text
output/poc/hwpx/task1100/stage5_masterpage_page_number_guard/exam_social_001.svg
```

Stage 3 SVG에서는 바탕쪽 하단 표에 자동 치환된 `1`과 고정 텍스트 `32`가 함께 출력되었다.

```text
stage4_hwpx_line_points_svg/exam_social_001.svg:
  bottom master-page table text = 1, 3, 2
```

Stage 4 수정 후에는 자동 치환된 `1`이 제거되고 고정 텍스트 `32`만 남는다.

```text
stage5_masterpage_page_number_guard/exam_social_001.svg:
  bottom master-page table text = 3, 2
```

## 5. 검증

```text
cargo fmt --check
cargo test parser::hwpx::section::tests
cargo check
target/debug/rhwp export-svg samples/hwpx/exam_social.hwpx \
  -o output/poc/hwpx/task1100/stage5_masterpage_page_number_guard -p 0
```

결과:

```text
성공
```

