# Task m100 #949 Stage 11 작업 보고서: h03 contract 후보 분리 검증

## 1. 목적

Stage 10에서 TABLE 축이 `hwpx-h-03` 파일손상 직접 원인이 아니라는 점을 확인했다.
이번 단계는 남은 후보를 다음 두 축으로 분리한다.

```text
1. DocInfo ID_MAPPINGS / MEMO_SHAPE
2. BodyText 도형 묶음 내부 CTRL_DATA
```

## 2. 입력

```text
oracle:
samples/hwpx/hancom-hwp/hwpx-h-03.hwp

generated baseline:
output/poc/hwpx2hwp/task949/stage10/hwpx-h-03/08_all_table_axes.hwp
```

## 3. 구현

새 진단 명령을 추가했다.

```text
rhwp hwp5-contract-probe <oracle.hwp> <generated.hwp> --out-dir <dir>
```

소스:

```text
src/diagnostics/hwp5_contract_probe.rs
src/diagnostics/mod.rs
src/main.rs
```

명령 역할:

```text
- generated DocInfo의 ID_MAPPINGS를 oracle ID_MAPPINGS로 교체
- generated DocInfo에 oracle MEMO_SHAPE record를 삽입
- generated BodyText에서 oracle에는 있으나 generated에는 없는 CTRL_DATA를 직전 대응 record 뒤에 삽입
```

## 4. 생성 명령

```bash
cargo run --quiet --bin rhwp -- hwp5-contract-probe \
  samples/hwpx/hancom-hwp/hwpx-h-03.hwp \
  output/poc/hwpx2hwp/task949/stage10/hwpx-h-03/08_all_table_axes.hwp \
  --out-dir output/poc/hwpx2hwp/task949/stage11/hwpx-h-03_contract_probe
```

## 5. 생성 산출물

```text
output/poc/hwpx2hwp/task949/stage11/hwpx-h-03_contract_probe/
```

| file | bytes | hash | rhwp reload | ID_MAPPINGS | MEMO_SHAPE | CTRL_DATA |
|---|---:|---|---|---:|---:|---:|
| `01_id_mappings_only.hwp` | 38400 | `8c1c1550f778cadc` | ok, pages=9 | 1 | 0 | 0 |
| `02_memo_shape_only.hwp` | 38400 | `f7e473dd0e78690a` | ok, pages=9 | 0 | 1 | 0 |
| `03_id_mappings_memo_shape.hwp` | 38400 | `adf4baa3bd112631` | ok, pages=9 | 1 | 1 | 0 |
| `04_shape_ctrl_data_only.hwp` | 38400 | `9e77f98b46131f7d` | ok, pages=9 | 0 | 0 | 1 |
| `05_id_mappings_ctrl_data.hwp` | 38400 | `a0ca9c7c8d338075` | ok, pages=9 | 1 | 0 | 1 |
| `06_memo_shape_ctrl_data.hwp` | 38400 | `68a76246053a072f` | ok, pages=9 | 0 | 1 | 1 |
| `07_id_mappings_memo_shape_ctrl_data.hwp` | 38400 | `0e56669c4dfd8f74` | ok, pages=9 | 1 | 1 | 1 |

생성 보고서:

```text
output/poc/hwpx2hwp/task949/stage11/hwpx-h-03_contract_probe/stage11_generation.md
```

보조 diff:

```text
output/poc/hwpx2hwp/task949/stage11/hwpx-h-03_contract_probe/07_hints.md
```

## 6. 작업지시자 판정 요청 철회

이 단계에서 생성한 HWP 파일들은 작업지시자 판정 대상으로 사용하지 않는다.

이유는 다음과 같다.

```text
1. 정답 HWP record를 generated HWP에 graft한 파일은 성공해도 구현 방향을 정확히
   알려주지 않는다.
2. ID_MAPPINGS, MEMO_SHAPE, CTRL_DATA는 독립 record가 아니라 count, reference,
   parent/child control graph와 함께 해석된다.
3. 일부 record만 끼워 넣은 파일은 그 자체로 비정상 조합이 될 수 있으므로, 실패해도
   해당 축이 원인이 아니라고 결론낼 수 없다.
4. 이 판정은 작업지시자 시각 확인 비용을 사용하지만, 다음 구현 행동을 충분히 바꾸지
   못한다.
```

따라서 Stage 11의 산출물은 "판정 후보"가 아니라, 기존의 실패 패턴을 재사용한 사례로
기록한다.

## 7. 현재 해석

이번 파일은 구현 결과물이 아니라 정답 HWP의 contract record를 graft한 파일이다.
그러나 이 접근은 현재 과제의 핵심을 돌파하지 못한다.

기존 접근의 문제:

```text
- "파일이 열리는지"를 계속 묻는 방식은 한컴 에디터 contract 위반 위치를 설명하지 못한다.
- 정답지 record graft는 HWPX -> IR -> HWP 저장기가 어떤 값을 생성해야 하는지 알려주지
  못한다.
- rhwp-studio가 열 수 있지만 한컴 에디터가 거부하는 조건을 구조적으로 설명해야 한다.
```

다음 단계는 HWP 파일을 더 만들어 판정받는 것이 아니라, 정답 HWP와 생성 HWP의
record/control contract 차이를 기계적으로 설명하는 분석으로 전환한다.

## 8. 검증

```text
cargo check: 통과
hwp5-contract-probe: 7개 HWP 생성 통과
rhwp reload: 7개 모두 pages=9
```

## 9. Stage 11 결론

Stage 11은 중단한다.

확정된 결론은 다음이다.

```text
- Stage 10에서 TABLE 축 단독 원인이 아님은 확인되었다.
- Stage 11의 graft 판정 방식은 작업지시자 판정을 요구할 만큼 유효하지 않다.
- 다음 단계는 "정답 HWP와 generated HWP의 contract graph 차이"를 분석하고,
  누락된 control/record가 HWPX IR에서 어디서 사라지는지 추적해야 한다.
```

새 전략의 기준:

```text
1. 작업지시자 판정은 구현 후보가 명확하고 결과가 다음 행동을 바꿀 때만 요청한다.
2. probe 파일을 먼저 만들지 않는다.
3. 먼저 oracle/generated record graph를 비교한다.
4. 비교 결과를 HWPX source, IR, HWP serializer 경로에 연결한다.
5. 구현은 "정확한 contract 재구성" 단위로만 진행한다.
```
