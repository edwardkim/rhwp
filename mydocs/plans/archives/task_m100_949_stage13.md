# Task m100 #949 Stage 13 계획: h03 contract source trace

## 1. 목적

Stage 12에서 확인한 `hwpx-h-03`의 contract 차이를 구현 소스의 책임 경로로
연결한다.

이번 단계는 HWP probe 파일을 추가로 만들지 않는다. 작업지시자 시각 판정도 요청하지
않는다. 목표는 "어떤 record를 더 graft하면 열리는가"가 아니라 "HWPX -> IR -> HWP
저장 경로에서 어떤 contract materialization이 누락되었는가"를 확정하는 것이다.

## 2. 입력

```text
HWPX source:
samples/hwpx/hwpx-h-03.hwpx

oracle HWP:
samples/hwpx/hancom-hwp/hwpx-h-03.hwp

generated HWP:
output/poc/hwpx2hwp/task949/stage10/hwpx-h-03/08_all_table_axes.hwp

Stage 12 reports:
output/poc/hwpx2hwp/task949/stage12/hwpx-h-03/

Stage 13 inventories:
output/poc/hwpx2hwp/task949/stage13/hwpx-h-03/
```

## 3. 확인할 질문

```text
1. generated HWP에서 CTRL_DATA가 빠지는 직접적인 소스 경로는 어디인가?
2. HWPX 파서가 의미 컨트롤을 만들지만 HWP5 record contract를 만들지 않는 지점은 어디인가?
3. 현재 HWPX -> HWP adapter가 table contract만 보강하고 있는지 확인한다.
4. MEMO_SHAPE/ID_MAPPINGS 차이는 즉시 구현 대상인지, 별도 contract 분석 대상인지 구분한다.
5. Stage 14에서 구현해야 할 최소 단위가 무엇인지 결정한다.
```

## 4. 완료 조건

```text
- source path별 책임 표 작성
- confirmed fact / hypothesis 분리
- Stage 14 구현 타깃 정의
- HWP probe 또는 시각 판정 요청 없음
```

## 5. 원칙

```text
- serializer는 가능한 한 generic writer로 유지한다.
- HWPX 출처에서만 필요한 보강은 src/document_core/converters/hwpx_to_hwp.rs에 둔다.
- HWP 출처 round-trip 보존 경로는 변경하지 않는다.
- raw stream graft는 구현 전략으로 사용하지 않는다.
```
