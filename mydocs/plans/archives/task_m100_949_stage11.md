# Task m100 #949 Stage 11 계획: h03 contract 후보 분리 검증

## 1. 배경

Stage 10에서 `hwpx-h-03`에 `hwpx-h-01` 성공 축인 TABLE/CTRL_HEADER(Table) 보정을
적용했지만 한컴 에디터 파일손상 판정은 사라지지 않았다.

따라서 다음 후보를 분리한다.

```text
1. DocInfo ID_MAPPINGS / MEMO_SHAPE 계약 불일치
2. 2페이지 도형 묶음 내부 CTRL_DATA 누락
```

## 2. 입력

```text
oracle:
samples/hwpx/hancom-hwp/hwpx-h-03.hwp

generated baseline:
output/poc/hwpx2hwp/task949/stage10/hwpx-h-03/08_all_table_axes.hwp
```

이번 baseline은 이미 Stage 10 TABLE 축이 모두 graft된 파일이다. 따라서 Stage 11은 TABLE 축을
다시 확대하지 않고, DocInfo memo/count 축과 BodyText CTRL_DATA 축만 판정한다.

## 3. 작업

새 진단 명령 `hwp5-contract-probe`를 사용한다.

```bash
cargo run --quiet --bin rhwp -- hwp5-contract-probe \
  samples/hwpx/hancom-hwp/hwpx-h-03.hwp \
  output/poc/hwpx2hwp/task949/stage10/hwpx-h-03/08_all_table_axes.hwp \
  --out-dir output/poc/hwpx2hwp/task949/stage11/hwpx-h-03_contract_probe
```

생성 축:

```text
01: ID_MAPPINGS only
02: MEMO_SHAPE only
03: ID_MAPPINGS + MEMO_SHAPE
04: CTRL_DATA only
05: ID_MAPPINGS + CTRL_DATA
06: MEMO_SHAPE + CTRL_DATA
07: ID_MAPPINGS + MEMO_SHAPE + CTRL_DATA
```

## 4. 판정 포인트

```text
- 한컴 에디터 파일손상 판정이 사라지는지
- 파일손상이 사라질 경우 어떤 축에서 사라지는지
- rhwp-studio reload/render가 유지되는지
- 이미지 출력과 표/셀 배치가 Stage 10 baseline보다 나빠지지 않는지
```

## 5. 해석 기준

```text
- 03 성공, 01/02 실패: ID_MAPPINGS와 MEMO_SHAPE는 결합 contract로 처리해야 한다.
- 04 성공: 2페이지 도형 묶음 내부 CTRL_DATA 누락이 직접 원인이다.
- 07만 성공: DocInfo memo/count 축과 CTRL_DATA 축이 함께 필요하다.
- 07도 실패: 아직 남은 DocInfo tail 또는 shape payload 축을 별도 분리한다.
```

## 6. 사후 판정

이 계획은 실행 후 폐기한다.

폐기 사유:

```text
1. 정답 HWP record graft는 한컴 에디터 contract 위반 위치를 설명하지 못한다.
2. 일부 record만 교체한 파일은 성공/실패 어느 쪽도 구현 단위를 충분히 특정하지 못한다.
3. 작업지시자 시각 판정 비용을 쓰지만 다음 구현 행동을 충분히 바꾸지 못한다.
4. 이 방식은 이전 단계에서 반복적으로 실패한 "probe 파일 생성 -> 시각 판정" 패턴을
   재사용한 것이다.
```

Stage 11은 더 진행하지 않고, Stage 12에서 정답 HWP와 generated HWP의 record/control
contract graph를 먼저 설명하는 방식으로 전환한다.
