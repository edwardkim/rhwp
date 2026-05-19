# Task M100-993 Stage 11 계획서

## 1. 목적

Stage 10 판정 결과 `03`, `06`, `07`, `09`가 개선되었다.
이 네 케이스의 공통 축은 `pi=22` 셀 `LIST_HEADER` 정답지 투영이다.

동시에 네 개선 케이스 모두에서 셀 대각선 처리가 누락되었다.
작업지시자는 이 누락이 1페이지 첫 부분의 정체 모를 선으로 표시되는 현상과 관련될 수 있다고
판단했다.

Stage 11의 목적은 다음 두 문제를 분리하는 것이다.

```text
1. pi=22 표 높이 개선 축:
   Cell LIST_HEADER width_ref/raw_list_extra contract

2. 별도 잔존 축:
   셀 대각선 slash/backSlash BorderFill contract
```

## 2. 현재 결론

Stage 10 판정으로 다음은 확정 후보로 올린다.

```text
positive common axis:
  pi=22 OracleListExtra

해석:
  TABLE tail, PARA_HEADER tail, PARA_TEXT는 단독 positive가 아니다.
  06/07/09의 개선은 OracleListExtra를 포함했기 때문으로 우선 해석한다.
```

따라서 Stage 11에서는 표 높이 문제를 더 넓히지 않고, 대각선 누락만 독립적으로 조사한다.

## 3. 기술 가설

HWP5 셀 대각선은 셀 record가 직접 선을 들고 있는 구조가 아니다.
셀은 `borderFillIDRef`로 DocInfo `BORDER_FILL`을 참조하고, 실제 대각선 정보는 `BORDER_FILL`
안에 들어간다.

확인할 HWP5 contract:

```text
1. BORDER_FILL attr
   - slash/backSlash 방향과 형태를 나타내는 비트가 포함된다.

2. BORDER_FILL diagonal payload
   - diagonal_type
   - width
   - color
```

현재 코드상 의심 지점:

```text
src/parser/hwpx/header.rs
  - hh:slash / hh:backSlash 요소를 파싱하지만,
    HWP5 BORDER_FILL.attr의 slash/backSlash 비트 materialize가 부족할 수 있다.

src/serializer/doc_info.rs
  - BorderFill.attr와 DiagonalLine payload를 그대로 HWP5로 쓴다.
  - 따라서 parser 단계에서 attr가 비어 있으면 저장 결과에서도 대각선이 누락된다.
```

## 4. 비교 대상

정답지:

```text
samples/hwpx/hancom-hwp/mel-001.hwp
```

HWPX 원본:

```text
samples/hwpx/mel-001.hwpx
```

Stage 10 positive 후보:

```text
output/poc/hwpx2hwp/task993/stage10_mel_personnel_table_contract/03_pi22_oracle_list_extra.hwp
```

## 5. 수행 절차

1. HWPX `header.xml`에서 `hh:slash`, `hh:backSlash`, `hh:diagonal`이 있는 `borderFill`을 추출한다.
2. 같은 ID의 정답 HWP `BORDER_FILL` payload를 decode한다.
3. Stage 10 positive 후보의 `BORDER_FILL` payload를 decode한다.
4. `attr`, `diagonal_type`, `width`, `color`, payload size/hash를 비교한다.
5. 대각선 contract만 graft한 probe를 생성한다.
6. 한컴 에디터에서 대각선 복구 여부와 1페이지 정체 모를 선의 변화 여부를 판정한다.

## 6. 산출물

생성 폴더:

```text
output/poc/hwpx2hwp/task993/stage11_borderfill_diagonal_contract/
```

분석 문서:

```text
output/poc/hwpx2hwp/task993/stage11_borderfill_diagonal_contract/diagonal_borderfill_diff.md
```

판정용 HWP:

| file | 목적 |
|---|---|
| `01_stage10_positive.hwp` | Stage 10 positive 기준 |
| `02_diagonal_attr_only.hwp` | BORDER_FILL attr 대각선 비트만 정답지 투영 |
| `03_diagonal_payload_only.hwp` | DiagonalLine payload만 정답지 투영 |
| `04_diagonal_attr_payload.hwp` | attr + DiagonalLine payload 동시 투영 |
| `05_diagonal_attr_payload_plus_pi22_list.hwp` | pi=22 LIST_HEADER positive + 대각선 positive 후보 |

## 7. 판정표

| file | 한컴 판정 유형 | pi=22 표 높이 | 셀 대각선 | 1페이지 첫 부분 선 | 1x1 표 배경 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `01_stage10_positive.hwp` |  |  |  |  |  |  | 기준 |
| `02_diagonal_attr_only.hwp` |  |  |  |  |  |  | attr 단독 |
| `03_diagonal_payload_only.hwp` |  |  |  |  |  |  | payload 단독 |
| `04_diagonal_attr_payload.hwp` |  |  |  |  |  |  | 대각선 후보 |
| `05_diagonal_attr_payload_plus_pi22_list.hwp` |  |  |  |  |  |  | 결합 후보 |

## 8. 성공 기준

```text
1. Stage 10에서 개선된 pi=22 표 높이가 회귀하지 않는다.
2. 누락된 셀 대각선이 정답지처럼 표시된다.
3. 1페이지 첫 부분의 정체 모를 선이 대각선 contract와 연동되는지 확인된다.
4. gradient BorderFill 1x1 표 배경이 회귀하지 않는다.
5. rhwp-studio 재로드가 정상이다.
```

## 9. 비목표

이번 단계에서는 다음을 하지 않는다.

```text
- mel-001 전체 표 배치 일반화
- 모든 BorderFill을 일괄 정답지로 교체
- 셀 높이 직접 보정
- 대각선과 무관한 DocInfo 전체 재정렬
```

## 10. 승인 요청

이 계획으로 Stage 11 `BORDER_FILL` 대각선 contract 비교와 제한 probe 생성을 진행한다.
