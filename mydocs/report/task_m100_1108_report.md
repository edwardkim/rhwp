# Task M100-1108 완료 보고서

## 1. 이슈

GitHub Issue #1108:

```text
HWPX exam_social 문단 시작 문단번호 출력 누락
```

대상 파일:

```text
samples/hwpx/exam_social.hwpx
samples/exam_social.hwp
```

## 2. 문제

`samples/hwpx/exam_social.hwpx`를 rhwp-studio에서 렌더링하면 문단 시작의 문단번호가 출력되지
않았다. 같은 문서의 HWP 정답지 `samples/exam_social.hwp`는 문단 시작에서 문단번호가 정상 출력된다.

## 3. 원인

HWPX 번호 정의의 `hh:paraHead`는 번호 형식을 본문 텍스트로 저장한다.

```xml
<hh:paraHead ...>^1.</hh:paraHead>
```

기존 HWPX parser는 `hh:paraHead@text` 속성만 읽고 본문 텍스트를 읽지 않았다. 그 결과
`Numbering.level_formats`가 비어 있었고, 문단 자체는 `head=Number`로 인식되지만 렌더링할 번호
형식이 없어 문단번호가 출력되지 않았다.

추가로 `paraPr@condense` 값도 HWP5 `ParaShape.attr1`의 공백 최소값 bits 9..15에 반영되지 않아,
HWP 정답지와 문단 모양 raw 속성이 달라졌다.

## 4. 수정 내용

`src/parser/hwpx/header.rs`를 수정했다.

```text
1. hh:paraHead Empty/Start 이벤트를 모두 처리
2. hh:paraHead 본문 텍스트와 text 속성을 모두 번호 형식으로 수집
3. level/start/numFormat 속성 순서에 의존하지 않도록 수집 후 적용
4. HWPX numFormat 문자열을 HWP5 numbering format code로 매핑
5. paraPr@condense를 HWP5 ParaShape.attr1 bits 9..15에 반영
6. paraPr@fontLineHeight를 HWP5 ParaShape.attr1 bit 22에 반영
```

추가 테스트:

```text
test_parse_hwpx_numbering_para_head_text_body
test_parse_hwpx_numbering_para_head_empty_text_attr
test_parse_hwpx_para_shape_condense_attr_bits
```

## 5. 산출물

```text
output/poc/hwpx/task1108/stage1_para_number_trace/
mydocs/working/task_m100_1108_stage1.md
```

판정용 SVG:

```text
output/poc/hwpx/task1108/stage1_para_number_trace/hwp_page1_after/exam_social_001.svg
output/poc/hwpx/task1108/stage1_para_number_trace/hwpx_page1_after/exam_social_001.svg
```

## 6. 검증

실행한 검증:

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

작업지시자 시각 판정:

```text
SVG 시각 판정 통과
웹 캔바스 시각 판정 통과
```

## 7. 결론

HWPX `exam_social.hwpx`의 문단 시작 문단번호 누락 문제를 해결했다. HWPX 번호 정의를 올바르게
IR에 반영하여 HWP 정답지와 동일하게 문단번호가 출력된다.

이번 이슈는 완료 처리 가능하다.

## 8. 후속 이슈 연결 파일

다음 이슈에서 다룰 HWPX to HWP 저장 경로의 파일손상 재현 파일을 함께 보존한다.

```text
saved/111exam_social.hwp
```

작업지시자 판정:

```text
한컴 에디터 파일손상 판정
```
