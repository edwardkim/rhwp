# Task m100 #949 Stage 14 계획: CTRL_DATA ParameterSet trace

## 1. 목적

Stage 13에서 확인한 oracle-only `CTRL_DATA#833`를 바이트 덩어리가 아니라 HWP5
ParameterSet 구조로 해석한다.

이번 단계도 HWP probe 파일을 만들지 않는다. 작업지시자 시각 판정 요청도 없다.

## 2. 입력

```text
oracle HWP:
samples/hwpx/hancom-hwp/hwpx-h-03.hwp

generated HWP:
output/poc/hwpx2hwp/task949/stage10/hwpx-h-03/08_all_table_axes.hwp

HWPX source:
samples/hwpx/hwpx-h-03.hwpx
```

## 3. 구현 범위

```text
1. CTRL_DATA payload를 ParameterSet으로 재귀 decode하는 진단 명령 추가
2. oracle/generated CTRL_DATA 개수와 payload 구조 비교
3. HWPX 원본에서 대응되는 의미 정보가 존재하는지 확인
```

## 4. 완료 조건

```text
- oracle CTRL_DATA#833의 ps_id/item_id/type/string 구조 확인
- generated에는 대응 CTRL_DATA가 없음을 문서화
- HWPX 원본의 대응 속성 위치 확인
- 다음 구현 단위가 "HWPX href -> HWP CTRL_DATA"인지 판단
```

## 5. 비목표

```text
- HWP 파일 mutation/graft
- 대량 probe 생성
- 작업지시자 시각 판정 요청
- 아직 해석하지 않은 CTRL_DATA payload를 임의로 복사하는 구현
```
