# Task M100-1099 Stage 3 계획서

## 1. 목적

Stage 2에서 Header/Footer 내부 Table materialization 결함은 제거했지만, 1페이지 축소 샘플은
여전히 한컴 에디터에서 파일손상 판정을 받았다.

이번 단계는 다음 후보만 분리한다.

```text
SectionDef 아래 바탕쪽(master page) LIST_HEADER envelope / 순서 / 확장 플래그 contract
```

## 2. 근거

Stage 2 생성 HWP는 rhwp에서 reload 가능하지만, 정답 HWP와 바탕쪽 해석이 다르다.

```text
정답 HWP:
  [0] Both
  [1] Odd
  [2] Both, is_ext=true, overlap=true, ext_flags=0x0003

생성 HWP:
  [0] Both
  [1] Odd
  [2] Even, is_ext=true, overlap=true, ext_flags=0x0003
```

HWPX 원본은 다음 순서로 들어온다.

```text
masterpage0.xml: EVEN
masterpage1.xml: ODD
masterpage2.xml: LAST_PAGE, pageDuplicate=0
```

현재 HWP5 writer는 HWPX 출처 master page를 그대로 순서대로 materialize한다.
그러나 HWP5 raw parser는 바탕쪽 LIST_HEADER 순서를 `[Both, Odd, Even]`으로 해석한다.
따라서 HWPX의 `LAST_PAGE` 확장 바탕쪽을 HWP5로 저장할 때 한컴 정답지와 같은 envelope가
필요한지 확인해야 한다.

## 3. 작업 범위

대상 파일은 1페이지 축소 샘플로 고정한다.

```text
source: samples/hwpx/exam-kor-1p.hwpx
oracle: samples/exam-kor-1p.hwp
baseline: output/poc/hwpx2hwp/task1099/stage2_header_footer_table_contract/exam-kor-1p-stage2.hwp
```

출력 위치:

```text
output/poc/hwpx2hwp/task1099/stage3_masterpage_envelope_contract/
```

## 4. 구현 후보

다음 후보를 생성한다.

```text
01_stage2_baseline.hwp
  Stage 2 기준 파일 복사본

02_masterpage_apply_order_normalized.hwp
  HWPX master page를 HWP5 저장 전에 Both/Odd/Even 계열 순서로 정렬한다.
  LAST_PAGE 확장 바탕쪽은 Both extension으로 유지한다.

03_masterpage_extension_padding.hwp
  정답 HWP의 raw LIST_HEADER 위치/level을 참고해 LAST_PAGE extension 앞 envelope를 보강한다.
  목적은 HWP5 parser와 한컴 에디터가 세 번째 바탕쪽을 일반 Even이 아니라 확장 Both로 해석하게 하는 것이다.
```

`03`은 `02` 결과가 실패할 경우에만 생성한다.

## 5. 판정 기준

작업지시자 판정 대상:

| file | 한컴 판정 유형 | 바탕쪽 출력 | 지문 박스 출력 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1099/stage3_masterpage_envelope_contract/01_stage2_baseline.hwp` |  |  |  |  |  | baseline |
| `output/poc/hwpx2hwp/task1099/stage3_masterpage_envelope_contract/02_masterpage_apply_order_normalized.hwp` |  |  |  |  |  | 1차 후보 |
| `output/poc/hwpx2hwp/task1099/stage3_masterpage_envelope_contract/03_masterpage_extension_padding.hwp` |  |  |  |  |  | 2차 후보 |

성공 조건:

```text
1. 한컴 에디터에서 파일손상 없이 열린다.
2. 1페이지 바탕쪽, 지문 박스, 본문 배치가 Stage 2보다 나빠지지 않는다.
3. rhwp reload에서 master page 해석이 정답 HWP와 같은 [Both, Odd, Both extension]이 된다.
```

## 6. 검증

필수 실행:

```text
cargo fmt --check
cargo test document_core::converters::hwpx_to_hwp::tests::header_footer_nested_tables_are_materialized
cargo build --bin rhwp
```

## 7. 승인 요청

위 계획대로 Stage 3 master page envelope 후보를 생성하고, 1페이지 한컴 판정 파일을 만들겠다.
