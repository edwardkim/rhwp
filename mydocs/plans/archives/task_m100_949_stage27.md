# Task M100 #949 Stage 27 계획서 - TextBox sequence contract 진단

## 1. 배경

Stage 26에서 GenShape `CTRL_HEADER`의 공통 attr 후보를 반영했지만, 한컴 판정은 Stage 23과
동일했다.

```text
hwpx-h-01: 성공
hwpx-h-02: 성공
hwpx-h-03: 파일손상, 2페이지 글상자 전까지만 출력
hy-001   : 파일손상, 2페이지 마지막 표 전까지만 출력
```

따라서 다음 단계는 `CTRL_HEADER` attr bit를 더 좁히는 방식이 아니라, 정답 HWP와 generated HWP의
글상자/그림 주변 HWP5 record sequence와 payload contract를 직접 비교한다.

## 2. 목표

이번 단계의 목표는 구현 후보를 만들기 전에 다음을 분리하는 것이다.

```text
1. 문제 지점의 record가 통째로 빠졌는지
2. record 순서/level이 틀렸는지
3. record는 있으나 payload/envelope가 다른지
4. h03과 hy-001이 같은 계열의 실패인지
```

## 3. 비교 대상

정답지와 Stage 26 생성본을 비교한다.

```text
samples/hwpx/hancom-hwp/hwpx-h-03.hwp
output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/hwpx-h-03.hwp

samples/hwpx/hancom-hwp/hy-001.hwp
output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/hy-001.hwp
```

## 4. 산출물

```text
output/poc/hwpx2hwp/task949/stage27_textbox_sequence_contract/h03_oracle_section0.jsonl
output/poc/hwpx2hwp/task949/stage27_textbox_sequence_contract/h03_stage26_section0.jsonl
output/poc/hwpx2hwp/task949/stage27_textbox_sequence_contract/hy_oracle_section0.jsonl
output/poc/hwpx2hwp/task949/stage27_textbox_sequence_contract/hy_stage26_section0.jsonl
output/poc/hwpx2hwp/task949/stage27_textbox_sequence_contract/h03_shape_bundles_w12.md
output/poc/hwpx2hwp/task949/stage27_textbox_sequence_contract/hy_shape_bundles_w12.md
output/poc/hwpx2hwp/task949/stage27_textbox_sequence_contract/sequence_findings.md
```

## 5. 판정 기준

```text
A. record sequence가 누락됨: parser/IR control 생성 문제
B. sequence는 있으나 level/order가 틀림: HWP5 serializer nesting 문제
C. sequence와 level은 있으나 size/payload가 다름: HWP5 record envelope materialization 문제
D. h03/hy-001이 서로 다른 실패: 샘플별 별도 contract로 분리
```

## 6. 다음 단계 연결

Stage 27은 구현 단계가 아니다. 결과가 `C`이면 Stage 28에서는 `SHAPE_COMPONENT` payload 구조를
정답지 기준으로 디코드한 뒤, source XML을 내부 구조체로 materialize하는 계획서를 별도로 작성한다.
