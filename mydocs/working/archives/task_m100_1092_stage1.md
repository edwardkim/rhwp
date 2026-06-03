# Task M100-1092 Stage 1 작업 기록

## 1. 단계 목표

Issue #1092의 첫 단계로 `samples/hwpx/aift.hwpx`의 메모 컨트롤이 HWP 저장 결과에서 어떻게
누락/오배치되는지 정답지와 비교했다.

대상 파일:

```text
source:    samples/hwpx/aift.hwpx
oracle:    samples/aift.hwp
generated: saved/111aift.hwp
```

## 2. 산출물

```text
output/poc/hwpx2hwp/task1092/stage1_memo_contract_trace/oracle_inventory.md
output/poc/hwpx2hwp/task1092/stage1_memo_contract_trace/generated_inventory.md
output/poc/hwpx2hwp/task1092/stage1_memo_contract_trace/inventory_diff.md
output/poc/hwpx2hwp/task1092/stage1_memo_contract_trace/contract_hints.md
output/poc/hwpx2hwp/task1092/stage1_memo_contract_trace/contract_analyze/
output/poc/hwpx2hwp/task1092/stage1_memo_contract_trace/memo_record_inventory.md
output/poc/hwpx2hwp/task1092/stage1_memo_contract_trace/memo_shape_id_mapping_diff.md
output/poc/hwpx2hwp/task1092/stage1_memo_contract_trace/memo_control_bundle_diff.md
output/poc/hwpx2hwp/task1092/stage1_memo_contract_trace/memo_source_trace.md
```

## 3. 확인한 사실

DocInfo 메모 count는 이미 맞다.

| 항목 | oracle | generated |
|---|---:|---:|
| `MEMO_SHAPE` count | 1 | 1 |
| `ID_MAPPINGS[15] memo_shape_count` | 1 | 1 |

하지만 `MEMO_SHAPE` payload는 1바이트가 다르다.

```text
oracle    MEMO_SHAPE: e7 3c 00 00 01 03 ...
generated MEMO_SHAPE: e7 3c 00 00 00 03 ...
```

본문 메모 field marker는 두 개 모두 존재하지만 정답지와 다르게 저장된다.

| 항목 | oracle | generated |
|---|---|---|
| ctrl_id | `%unk` | `%%me` |
| properties | `0x8001` | `0x0001` |
| command | `MEMO/65535/...` | `MEMO/65535/...` |

정답지에는 문서 말미에 `MEMO_LIST` 2개가 있으나 생성본에는 없다.

```text
oracle:
  BodyText.Section2#25924 MEMO_LIST payload=01 00 00 00
  BodyText.Section2#25929 MEMO_LIST payload=02 00 00 00

generated:
  MEMO_LIST 없음
```

## 4. HWPX source 위치

`Contents/header.xml`에는 `hh:memoProperties`가 있고, `Contents/section2.xml`의 `secPr`는
`memoShapeIDRef="1"`이다.

`Contents/section2.xml`에는 `type="MEMO"` fieldBegin 2개가 있다.

```text
paragraph_index=2693, Number=1, ID=memo1
paragraph_index=2696, Number=2, ID=memo2
```

둘 다 `Command=MEMO/65535/{number}/.../user/\;;` 형태다.

## 5. 원인 후보

Stage 1 기준으로 문제는 완전 미구현이 아니라, HWPX 메모 field를 HWP5 메모 계약으로 변환하는
단계의 불일치다.

```text
1. HWPX MEMO fieldBegin을 HWP5 정답지 field marker로 저장하지 못함
   - 현재: %%me, 0x0001
   - 정답: %unk, 0x8001

2. HWPX MEMO fieldBegin 목록에서 HWP5 MEMO_LIST record를 생성하지 못함

3. MEMO_SHAPE lineType 매핑이 정답지와 1바이트 다름
```

## 6. 다음 단계 제안

Stage 2에서는 다음 순서로 작은 후보를 만든다.

```text
1. MEMO field marker만 정답지 패턴으로 저장
2. MEMO_SHAPE lineType을 정답지와 일치
3. MEMO_LIST 2개를 materialize
4. 생성 HWP를 한컴 에디터와 rhwp-studio로 판정
```

각 후보는 `samples/aift.hwp`와 `saved/111aift.hwp`의 차이를 기준으로 판정한다.
