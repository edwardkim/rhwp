# Task M100-949 Stage 16 Working Notes

## 1. 목적

Stage 15 구현 결과를 실제 샘플 세트에 적용해 저장 산출물을 만든다.

대상:

```text
samples/hwpx/hwpx-h-01.hwpx
samples/hwpx/hwpx-h-02.hwpx
samples/hwpx/hwpx-h-03.hwpx
```

## 2. 산출 경로

```text
output/poc/hwpx2hwp/task949/stage16_adapter_regression/
```

## 3. 작업지시자 시각 판정표

| file | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task949/stage16_adapter_regression/hwpx-h-01.hwp` | 열림 | 정상 | 실패 | 성공 | 성공 | 비정상 | 2페이지 이미지 개체 묶기 성공 |
| `output/poc/hwpx2hwp/task949/stage16_adapter_regression/hwpx-h-02.hwp` | 열림 | 정상 | 실패 | 성공 | 성공 | 비정상 | 2페이지 이미지 개체 묶기 실패 |
| `output/poc/hwpx2hwp/task949/stage16_adapter_regression/hwpx-h-03.hwp` | 파일손상 | 정상 | 실패 | - | 2페이지에서 중단 | 비정상 | 2페이지 이미지 개체 묶기 실패 |

## 4. 생성 결과

```text
output/poc/hwpx2hwp/task949/stage16_adapter_regression/hwpx-h-01.hwp
output/poc/hwpx2hwp/task949/stage16_adapter_regression/hwpx-h-02.hwp
output/poc/hwpx2hwp/task949/stage16_adapter_regression/hwpx-h-03.hwp
```

파일 크기와 rhwp 재로드 정보:

| file | size | rhwp reload |
|---|---:|---|
| `hwpx-h-01.hwp` | 374,784 bytes | ok, sections=2, pages=9 |
| `hwpx-h-02.hwp` | 32,256 bytes | ok, sections=2, pages=9 |
| `hwpx-h-03.hwp` | 38,400 bytes | ok, sections=2, pages=9 |

생성 로그:

```text
output/poc/hwpx2hwp/task949/stage16_adapter_regression/generation.md
```

## 5. `hwpx-h-03` CTRL_DATA 계약 확인

```text
output/poc/hwpx2hwp/task949/stage16_adapter_regression/ctrl_data_trace_hwpx-h-03.md
```

요약:

```text
oracle    CTRL_DATA records = 1, total bytes = 76, hash = 024e873ad9c2bd92
generated CTRL_DATA records = 1, total bytes = 76, hash = 024e873ad9c2bd92
record_index = 833, level = 6, parent = SHAPE_COMPONENT#832@lv5
```

Stage 15에서 추가한 `hp:pic@href -> CTRL_DATA` 계약은 `hwpx-h-03` 정답지와 위치/레벨/payload가
일치한다.

## 6. 시각 판정 해석

작업지시자 판정 결과:

```text
hwpx-h-01: 한컴 열림, 이미지 정상, 표/셀 배치 실패, 마지막 페이지 출력 성공
hwpx-h-02: 한컴 열림, 이미지 정상, 표/셀 배치 실패, 마지막 페이지 출력 성공
hwpx-h-03: 한컴 파일손상, 이미지 정상, 표/셀 배치 실패, 2페이지에서 중단
```

## 6.1 정정: Stage 16은 성공 후보 검증이 아니다

Stage 16 산출물은 성공 후보를 검증한 것이 아니다.

이 단계에서 사용한 것은 "현재 adapter 저장 결과"였고, `hwpx-h-01`에서 이미 실패하는 표 배치
계약을 포함하고 있었다. 따라서 이 결과를 기반으로 `hwpx-h-02`, `hwpx-h-03`에 같은 방식이
통할 것이라고 해석하면 안 된다.

잘못된 판단:

```text
Stage 15의 hp:pic@href -> CTRL_DATA 계약을 반영했으니,
현재 adapter 결과를 세 샘플에 적용해 regression 검증을 해도 된다.
```

올바른 판단:

```text
1. hwpx-h-01에서 이미 표 배치가 실패하는 current adapter는 성공 후보가 아니다.
2. 이미 성공 판정을 받은 Stage 9의 table-axis 계약을 current adapter에 정확히 반영하지 않은 상태에서
   세 샘플 regression을 수행하면, 실패가 재현되는 것이 정상이다.
3. 따라서 Stage 16은 "성공 후보 판정"이 아니라 "현재 adapter baseline failure snapshot"으로만 취급한다.
```

이 단계의 판정으로 말할 수 있는 것은 다음뿐이다.

```text
- hp:pic@href -> CTRL_DATA payload/record 위치는 hwpx-h-03 정답지와 일치한다.
- 그러나 current adapter는 여전히 hwpx-h-01의 표 배치 성공 조건을 포함하지 않는다.
- 따라서 hwpx-h-01 표 배치 실패는 Stage 15 변경의 문제가 아니라, 아직 반영되지 않은 table-axis
  contract 문제다.
```

## 6.2 남은 문제 분리

중요한 분리:

```text
1. Stage 15의 hp:pic@href -> CTRL_DATA 계약은 hwpx-h-03 정답지와 일치한다.
2. 그런데 hwpx-h-03은 여전히 파일손상이다.
3. 단, Stage 16 산출물은 table-axis 성공 계약을 포함하지 않은 baseline이므로,
   hwpx-h-03 파일손상 원인을 여기서 곧바로 shape/group contract로 단정하면 안 된다.
```

다음 단계의 선행 조건은 `hwpx-h-01`에서 이미 성공했던 Stage 9 table-axis 계약을 실제 adapter
구현 후보로 정확히 반영하고, `hwpx-h-01`을 sentinel로 다시 성공시키는 것이다.

그 후에야 `hwpx-h-02`, `hwpx-h-03`의 2페이지 이미지 개체 묶기 실패가 table-axis 미반영의
부작용인지, 별도의 shape/group contract인지 분리할 수 있다.

## 7. 실행한 검증

```bash
cargo check --quiet
cargo run --quiet --bin rhwp -- hwp5-inventory ... --format md --out ...
cargo run --quiet --bin rhwp -- hwp5-ctrl-data-trace ... --section 0 --out ...
cargo run --quiet --bin rhwp -- info output/poc/hwpx2hwp/task949/stage16_adapter_regression/hwpx-h-01.hwp
cargo run --quiet --bin rhwp -- info output/poc/hwpx2hwp/task949/stage16_adapter_regression/hwpx-h-02.hwp
cargo run --quiet --bin rhwp -- info output/poc/hwpx2hwp/task949/stage16_adapter_regression/hwpx-h-03.hwp
```
