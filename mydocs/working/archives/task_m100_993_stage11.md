# Task M100-993 Stage 11 완료 보고서

## 1. 목적

Stage 10에서 `03`, `06`, `07`, `09`가 개선되었고, 공통 개선 축은 `pi=22` 셀
`LIST_HEADER` 정답지 투영이었다.

동시에 개선된 네 케이스 모두에서 셀 대각선 처리가 빠졌다. 작업지시자는 이 누락이
1페이지 첫 부분에 보이는 정체 모를 선과 연결될 가능성을 제기했다.

Stage 11의 목적은 다음 두 축을 분리하는 것이다.

```text
1. 이미 개선된 pi=22 표 높이 축:
   cell LIST_HEADER width_ref/raw_list_extra contract

2. 새로 분리할 잔존 축:
   DocInfo BORDER_FILL slash/backSlash diagonal contract
```

## 2. 입력

정답지:

```text
samples/hwpx/hancom-hwp/mel-001.hwp
```

Stage 10 positive 기준 파일:

```text
output/poc/hwpx2hwp/task993/stage10_mel_personnel_table_contract/03_pi22_oracle_list_extra.hwp
```

생성 폴더:

```text
output/poc/hwpx2hwp/task993/stage11_borderfill_diagonal_contract/
```

## 3. 핵심 발견

정답지와 Stage 10 positive 기준 파일의 `DocInfo/BORDER_FILL`을 비교했다.
대각선 contract가 실제로 다른 항목은 하나뿐이다.

```text
BORDER_FILL #4

oracle:
  attr = 0x0040
  slash_bits = 0
  backslash_bits = 2
  diagonal_type = 1
  diagonal_width = 1
  diagonal_color = 0x00000000

generated:
  attr = 0x0000
  slash_bits = 0
  backslash_bits = 0
  diagonal_type = 0
  diagonal_width = 0
  diagonal_color = 0x00000000
```

해석:

```text
1. 셀 대각선은 셀 record에 직접 저장되는 값이 아니다.
2. 셀은 borderFillIDRef로 DocInfo BORDER_FILL을 참조한다.
3. 정답지는 BORDER_FILL #4에 backSlash 비트와 DiagonalLine payload를 가진다.
4. generated는 해당 attr/payload가 모두 비어 있어 대각선이 누락된다.
```

## 4. 원인 후보

현재 코드상 의심 지점은 HWPX header 파서다.

```text
src/parser/hwpx/header.rs
```

관찰:

```text
1. hh:slash 요소는 파싱하지만 HWP5 BORDER_FILL.attr의 slash 비트를 materialize하지 않는다.
2. hh:backSlash 요소 처리도 HWP5 attr 비트로 충분히 연결되어 있지 않다.
3. diagonal type은 HWPX 문자열 값(SOLID 등)을 HWP5 line type 값으로 변환해야 하는데,
   현재 parse_u8 방식이면 문자열 type이 0으로 떨어질 수 있다.
```

즉 이번 축은 다음 contract로 정리한다.

```text
HWPX hh:slash / hh:backSlash / hh:diagonal
  -> HWP5 DocInfo BORDER_FILL.attr slash/backSlash bits
  -> HWP5 DocInfo BORDER_FILL DiagonalLine(type, width, color)
```

## 5. 생성한 probe

생성 명령:

```text
cargo run --bin rhwp -- hwp5-borderfill-diagonal-probe \
  samples/hwpx/hancom-hwp/mel-001.hwp \
  output/poc/hwpx2hwp/task993/stage10_mel_personnel_table_contract/03_pi22_oracle_list_extra.hwp \
  --out-dir output/poc/hwpx2hwp/task993/stage11_borderfill_diagonal_contract
```

생성 파일:

| file | 목적 |
|---|---|
| `01_stage10_positive.hwp` | Stage 10 positive 기준 |
| `02_diagonal_attr_only.hwp` | BORDER_FILL attr 대각선 비트만 정답지 투영 |
| `03_diagonal_payload_only.hwp` | DiagonalLine payload만 정답지 투영 |
| `04_diagonal_attr_payload.hwp` | attr + DiagonalLine payload 동시 투영 |
| `05_diagonal_attr_payload_plus_pi22_list.hwp` | Stage 10 positive + 대각선 후보 |

주의:

```text
입력 기준 파일이 이미 Stage 10 pi22 LIST_HEADER positive이므로,
04와 05는 바이트 단위로 동일하다.
05는 판정표에서 "pi22 positive와 결합된 대각선 후보"라는 의미를 명확히 하기 위한 이름이다.
```

생성 로그와 diff:

```text
output/poc/hwpx2hwp/task993/stage11_borderfill_diagonal_contract/diagonal_borderfill_diff.md
```

## 6. 생성 결과

| file | bytes | rhwp reload | hash |
|---|---:|---|---|
| `01_stage10_positive.hwp` | 162304 | ok, pages=20 | `8d32943cc949163d` |
| `02_diagonal_attr_only.hwp` | 162304 | ok, pages=20 | `551d639da17e8a5a` |
| `03_diagonal_payload_only.hwp` | 162304 | ok, pages=20 | `5f5e8f2d8f5426de` |
| `04_diagonal_attr_payload.hwp` | 162304 | ok, pages=20 | `f21e541f1b837621` |
| `05_diagonal_attr_payload_plus_pi22_list.hwp` | 162304 | ok, pages=20 | `f21e541f1b837621` |

## 7. 작업지시자 판정표

| variant | 한컴 판정 유형 | 2페이지 인원현황 표 높이 | 셀 대각선 | 1페이지 첫 부분 선 | 1x1 표 배경 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `01_stage10_positive` |  |  |  |  |  |  | 기준 |
| `02_diagonal_attr_only` |  |  |  |  |  |  | attr 단독 |
| `03_diagonal_payload_only` |  |  |  |  |  |  | payload 단독 |
| `04_diagonal_attr_payload` | 성공 | 개선 유지 | 구현됨 | 남아 있음 | 성공 | 성공 | 대각선 후보 |
| `05_diagonal_attr_payload_plus_pi22_list` | 성공 | 개선 유지 | 구현됨 | 남아 있음 | 성공 | 성공 | 04와 동일 hash |

## 8. 검증

```text
cargo check
```

결과:

```text
success
```

## 9. 다음 판정 포인트

한컴 에디터 판정은 다음 순서가 가장 효율적이다.

```text
1. 02_diagonal_attr_only
   - attr 비트만으로 대각선/정체 모를 선이 변하는지 확인

2. 03_diagonal_payload_only
   - payload만으로는 변화가 없는지 확인

3. 04_diagonal_attr_payload
   - attr + payload가 함께 있어야 정답 대각선이 복구되는지 확인
```

예상:

```text
HWP5 대각선 렌더링은 attr 방향 비트와 DiagonalLine payload가 모두 필요하다.
따라서 최종 positive는 04/05가 될 가능성이 높다.
```

## 9.1 작업지시자 판정 해석

작업지시자 판정 결과 `04_diagonal_attr_payload`와
`05_diagonal_attr_payload_plus_pi22_list`에서 한컴 에디터와 rhwp-studio 모두 셀 대각선 처리가
구현되었다.

따라서 다음은 확정한다.

```text
셀 대각선 누락 원인:
  DocInfo BORDER_FILL의 attr slash/backSlash 비트와 DiagonalLine payload 누락

필요 contract:
  HWPX hh:slash / hh:backSlash / hh:diagonal
    -> HWP5 BORDER_FILL.attr diagonal bits
    -> HWP5 DiagonalLine(type, width, color)
```

반면 1페이지 시작 부분의 정체 모를 선은 `04/05`에서도 한컴 에디터에 남아 있다.
따라서 이 현상은 셀 대각선 누락의 부작용이 아니다.

다음 단계에서는 이 선을 별도 축으로 분리한다.

```text
잔존 축:
  1페이지 시작 부분에 나타나는 선

해석:
  BorderFill diagonal contract와 독립적이다.
  shape/line control, page border, paragraph border, 혹은 잘못 materialize된 drawing record 후보로
  다시 추적해야 한다.
```

## 9.2 1페이지 고스트 선 관찰

작업지시자가 한컴 에디터의 조판부호 표시 상태에서 다음 차이를 확인했다.

원본 `mel-001.hwpx`를 한컴 에디터에서 열었을 때 1페이지 첫 부분의 문단부호 순서:

```text
[쪽 번호 위치][감추기][탭][머리말 양쪽]
```

rhwp가 HWP로 저장한 파일을 한컴 에디터에서 열었을 때의 순서:

```text
[탭]{고스트 선}[쪽 번호 위치][감추기][머리말 양쪽]
```

이 관찰로 인해 고스트 선의 후보를 다시 좁힌다.

```text
1. 고스트 선은 셀 대각선 누락과 독립적이다.
2. 고스트 선은 1페이지 첫 문단의 control sequence 재구성 문제일 가능성이 높다.
3. 특히 [탭]이 앞쪽으로 이동하고, 그 뒤에 선처럼 보이는 drawing artifact가 나타난다.
4. 따라서 다음 단계는 표/셀 쪽이 아니라 첫 문단의 control order, control payload,
   PARA_TEXT/control char stream, CTRL_HEADER 순서를 정답지와 비교해야 한다.
```

## 10. 구현 변경

Stage 11에서는 판정용 diagnostic command를 추가했다.

```text
src/diagnostics/hwp5_borderfill_diagonal_probe.rs
src/diagnostics/mod.rs
src/main.rs
```

추가 명령:

```text
rhwp hwp5-borderfill-diagonal-probe <oracle.hwp> <generated.hwp> --out-dir <dir>
```

이 명령은 HWP 전체를 재직렬화하지 않고 `/DocInfo`의 `BORDER_FILL` 레코드에서
대각선 관련 `attr` 2바이트와 `DiagonalLine` 6바이트만 제한적으로 graft한다.
