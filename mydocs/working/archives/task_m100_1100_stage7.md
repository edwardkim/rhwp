# Task M100-1100 Stage 7 작업 기록

## 1. 목적

Stage 6 이후 작업지시자 시각 판정에서 확인된 잔여 문제를 처리한다.

현상:

```text
exam_social.hwpx page 2 짝수쪽 머리말에서 쪽번호가 2번 출력된다.
```

## 2. 원인

짝수쪽 머리말의 제목 글상자는 HWPX에서 다음 구조로 저장된다.

```text
AutoNumber(Page) + fwSpace + "(사회·문화)"
```

기존 렌더링은 `AutoNumber(Page)`가 들어 있는 문단을 처리할 때 공백 run 전체를 현재 쪽번호로
치환했다.

그 결과 실제 쪽번호 placeholder뿐 아니라 뒤따르는 `fwSpace`까지 쪽번호로 바뀌어 `22(사회·문화)`
처럼 보였다.

## 3. 수정 내용

수정 파일:

```text
src/renderer/layout.rs
src/renderer/layout/shape_layout.rs
src/renderer/layout/table_layout.rs
tests/issue_1100_exam_social_hwpx_header.rs
```

구현:

```text
1. AutoNumber(Page) 문단에서 공백 run 전체를 치환하지 않는다.
2. AutoNumber(Page) placeholder 문자 위치만 현재 쪽번호로 치환한다.
3. 뒤따르는 fwSpace는 공백으로 그대로 보존한다.
4. 동일한 문제가 표 셀 안 쪽번호에서도 재발하지 않도록 공통 helper를 사용한다.
```

## 4. 생성한 시각 판정 파일

```text
output/poc/hwpx/task1100/stage9_even_header_page_auto_once/exam_social_002.svg
```

확인 결과:

```text
page 2 header title:
  첫 번째 placeholder -> "2"
  두 번째 fwSpace     -> "\u{2007}"
```

## 5. 회귀 테스트

추가 테스트:

```text
issue_1100_hwpx_even_header_page_auto_number_replaces_one_placeholder_only
```

검증 내용:

```text
1. page 2 머리말 제목의 첫 placeholder는 "2"로 출력된다.
2. 그 다음 위치의 fwSpace는 "\u{2007}"로 유지된다.
3. fwSpace 위치가 두 번째 "2"로 바뀌지 않는다.
```

## 6. 실행한 검증

```text
cargo fmt --check
cargo check
cargo test --test issue_1100_exam_social_hwpx_header
cargo run --bin rhwp -- export-svg samples/hwpx/exam_social.hwpx \
  --output output/poc/hwpx/task1100/stage9_even_header_page_auto_once \
  --page 1
```

결과:

```text
성공
```

## 7. 다음 단계

wasm 빌드를 수행하고 `pkg` 산출물을 `web`에 반영했다.

```text
docker compose --env-file .env.docker run --rm wasm
cp pkg/rhwp.js pkg/rhwp_bg.wasm pkg/rhwp.d.ts pkg/rhwp_bg.wasm.d.ts web/
```

작업지시자 웹 캔바스 시각 판정을 요청한다.
