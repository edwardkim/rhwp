# Task M100-1108 Stage 1 작업 기록

## 1. 목표

`samples/hwpx/exam_social.hwpx`에서 문단 시작 문단번호가 출력되지 않는 원인을
`samples/exam_social.hwp` 정답지와 비교해 찾고, HWPX 렌더링 경로에서 보정한다.

## 2. 산출물

```text
output/poc/hwpx/task1108/stage1_para_number_trace/
```

주요 파일:

```text
hwp_page1_after/exam_social_001.svg
hwpx_page1_after/exam_social_001.svg
hwp_page1_after_text_nodes.txt
hwpx_page1_after_text_nodes.txt
hwp_dump_section0_after.txt
hwpx_dump_section0_after.txt
hwpx_diag_after.txt
hwpx_dump_page1_after.txt
ir_diff_section0_after.txt
```

## 3. 원인

HWPX 번호 정의는 다음처럼 `hh:paraHead`의 본문 텍스트로 번호 형식을 가진다.

```xml
<hh:paraHead ...>^1.</hh:paraHead>
```

기존 HWPX parser는 `hh:paraHead@text` 속성만 읽고 본문 텍스트를 읽지 않았다. 그 결과
`Numbering.level_formats`가 비어 있었고, 문단은 `head=Number`로 인식되지만 실제 렌더링할
번호 형식이 없어 문단번호가 출력되지 않았다.

또한 `start` 속성은 `level`보다 먼저 나오는 경우가 있는데, 기존 구현은 level이 확정된 뒤의
속성만 반영하는 형태여서 시작 번호 보존도 불안정했다.

## 4. 수정

`src/parser/hwpx/header.rs`에서 HWPX numbering 파싱을 보강했다.

```text
1. hh:paraHead를 Empty/Start 이벤트 모두 처리한다.
2. hh:paraHead 본문 텍스트와 text 속성을 모두 번호 형식 후보로 읽는다.
3. level, start, numFormat의 속성 순서에 의존하지 않도록 먼저 수집 후 적용한다.
4. HWPX numFormat 문자열을 HWP5 numbering format code로 매핑한다.
5. paraPr@condense 값을 HWP5 ParaShape attr1 bits 9..15에 반영한다.
6. paraPr@fontLineHeight 값을 HWP5 ParaShape attr1 bit 22에 반영한다.
```

추가 테스트:

```text
test_parse_hwpx_numbering_para_head_text_body
test_parse_hwpx_numbering_para_head_empty_text_attr
test_parse_hwpx_para_shape_condense_attr_bits
```

## 5. 확인 결과

수정 후 `diag`에서 HWPX 번호 형식이 채워진다.

```text
Numbering: 7개
[0] start=0, formats: L1="^1.", L2="^2.", L3="^3)", L4="^4)", L5="(^5)", L6="(^6)", L7="^7"
```

첫 번째 문제 문단도 번호 문단으로 추적된다.

```text
구역0:문단0 head=Number level=0 num_id=3 text="밑줄 친 ㉠∼㉤과 같은 현상의 일반적 특징에 대한 설명으로 옳은 것은?"
```

HWPX SVG text node에도 문단번호가 출력된다.

```text
>1</text>
>.</text>
>[</text>
>새</text>
>번</text>
>호</text>
>]</text>
```

`paraPr@condense="30"`도 HWPX IR에 보존된다.

```text
HWPX 문단0: head=Number level=0 num_id=3 attr1=0x00003C00
HWP  문단0: head=Number level=0 num_id=3 attr1=0x01003C00
```

HWP와 HWPX의 첫 단 페이지 사용량은 같아졌다.

```text
HWPX page1 단0 used=1106.5px
```

## 6. 남은 차이

`ir_diff_section0_after.txt` 기준으로 남은 차이는 다음 축이다.

```text
문단 0.0:
  cc: HWPX=96 vs HWP=104
  char_offsets[0]: HWPX=56 vs HWP=64
  TabDef[2] 일부 pos 차이
```

문단번호 누락은 해결되었지만, HWP 정답지에는 같은 문단에 `구역나누기` control과 추가 raw
paragraph metadata가 존재한다. 현재 Stage 1에서는 HWPX 렌더링의 문단번호 복원을 완료 범위로
보고, 문단 시작 위치의 잔여 차이는 작업지시자 시각 판정 결과에 따라 후속 stage로 분리한다.

## 7. 실행한 검증

```text
cargo fmt --check
cargo check
cargo build --bin rhwp
cargo test parse_hwpx
docker compose --env-file .env.docker run --rm wasm
```

결과:

```text
success
```

## 8. 시각 판정

작업지시자 판정:

```text
SVG 시각 판정 통과
웹 캔바스 시각 판정 통과
```

WASM 빌드 후 다음 파일을 갱신했다.

```text
pkg/rhwp.js
pkg/rhwp_bg.wasm
rhwp-studio/public/rhwp.js
rhwp-studio/public/rhwp_bg.wasm
```
