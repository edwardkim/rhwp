# Task m100 #1100 Stage 8 - 바탕쪽 하단 쪽번호 복원

## 1. 배경

Stage 7에서 짝수쪽 머리말의 `AutoNumber(Page)` 중복 출력은 해결되었다.
하지만 이후 웹 캔바스 시각 판정에서 꼬리말 쪽 하단 쪽번호가 사라진 문제가 확인되었다.

## 2. 원인

`exam_social.hwpx`의 footer 자체는 빈 문단이다.
하단 쪽번호는 footer 텍스트가 아니라 `masterpage*.xml` 안의 하단 표에 들어 있는
`hp:autoNum numType="PAGE"` 컨트롤로 표현된다.

그런데 Stage 5 가드에서 다음 조건이면 바탕쪽 전체의 현재 쪽번호를 `0`으로 바꾸고 있었다.

```text
masterPage@pageNumber = 0
hp:subList@hasNumRef = 0
```

이 조건은 HWPX 바탕쪽의 `AutoNumber(Page)` 표시 여부를 판단하는 신뢰 가능한 신호가 아니다.
`exam_social.hwpx`도 위 값은 모두 0이지만, 한컴 기준으로 하단 쪽번호는 표시되어야 한다.

## 3. 수정

바탕쪽 렌더링 시 `current_page_number`를 0으로 강제하지 않고 실제 페이지 번호를 유지하도록 했다.

머리말 쪽번호 중복은 Stage 7의 placeholder 단위 치환으로 해결하므로, 바탕쪽 전체 번호를 끄는
방식은 사용하지 않는다.

수정 파일:

```text
src/renderer/layout.rs
tests/issue_1100_exam_social_hwpx_header.rs
```

## 4. 생성한 시각 판정 파일

```text
output/poc/hwpx/task1100/stage10_footer_page_number_restore/exam_social_002.svg
```

확인 결과:

```text
하단 바탕쪽 쪽번호:
  x=486.8, y=1406.76 -> "2"

짝수쪽 머리말:
  첫 placeholder -> "2"
  다음 fwSpace   -> "\u{2007}"
```

HWP 정답지 SVG와 동일한 하단 쪽번호 위치에 `2`가 복원되었다.

## 5. 회귀 테스트

추가 테스트:

```text
issue_1100_hwpx_master_page_footer_page_number_is_preserved
```

기존 테스트:

```text
issue_1100_hwpx_header_negative_para_offset_clamped_to_header_origin
issue_1100_hwpx_even_header_page_auto_number_replaces_one_placeholder_only
```

## 6. 실행한 검증

```text
cargo fmt --check
cargo check
cargo test --test issue_1100_exam_social_hwpx_header
git diff --check
cargo run --bin rhwp -- export-svg samples/hwpx/exam_social.hwpx \
  --output output/poc/hwpx/task1100/stage10_footer_page_number_restore \
  --page 1
docker compose --env-file .env.docker run --rm wasm
cp pkg/rhwp.js pkg/rhwp_bg.wasm pkg/rhwp.d.ts pkg/rhwp_bg.wasm.d.ts web/
```

결과:

```text
성공
```
