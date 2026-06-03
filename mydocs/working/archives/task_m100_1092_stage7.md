# Task M100-1092 Stage 7 작업 기록

## 1. 단계 목표

Stage 6 후보는 첫 번째 메모의 한컴 표시 스타일을 정답지와 맞췄지만, 두 번째 메모는 여전히 기존처럼
스타일이 적용되지 않았다. Stage 7에서는 첫 번째/두 번째 메모 차이를 raw record로 비교해,
memo `FIELD_END` marker가 메모별 index를 보존하도록 수정한다.

## 2. 핵심 발견

두 번째 메모 anchor:

```text
section = 2
needle = 공동기관2
anchor text = "    - 공동기관2 : 직스테크놀로지"
```

정답지 `samples/aift.hwp`의 두 번째 메모 anchor `PARA_TEXT`는 다음 marker를 가진다.

```text
FIELD_BEGIN:
  03 00 65 6d 25 25 00 00 00 00 00 00 00 00 03 00

FIELD_END:
  04 00 65 6d 25 00 01 ff ff 00 02 00 00 00 04 00
```

Stage 6 생성본은 첫 번째 메모와 같은 index `1`을 계속 사용했다.

```text
Stage 6 FIELD_END:
  04 00 65 6d 25 00 01 ff ff 00 01 00 00 00 04 00
```

따라서 첫 번째 메모는 우연히 맞았고, 두 번째 메모는 `CTRL_HEADER`의 memo index가 `2`인데
`PARA_TEXT` 내부 `FIELD_END` marker는 index `1`로 남아 한컴 표시 스타일이 풀렸다.

## 3. 구현 내용

수정 소스:

```text
src/serializer/body_text.rs
```

적용 내용:

```text
1. FIELD_END 삽입 정보를 단순 ctrl_id가 아니라 FieldEndMarker { ctrl_id, memo_index }로 운반한다.
2. Memo field는 Field.memo_index를 FIELD_END marker의 6번째 code unit에 기록한다.
3. Field.memo_index가 0이면 command의 `MEMO/65535/<index>/...`에서 fallback으로 추출한다.
4. 기존 일반 FIELD_END 직렬화는 유지한다.
5. 단위 테스트를 memo index 2 케이스로 갱신했다.
```

## 4. 생성 후보

출력 파일:

```text
output/poc/hwpx2hwp/task1092/stage7_memo_field_end_index/aift-memo-field-end-index.hwp
```

파일 정보:

```text
size = 4,605,952 bytes
rhwp info = ok, sections=3, pages=76
```

## 5. Raw record 검증

정답지와 Stage 7 후보의 두 번째 메모 anchor 비교:

| file | PARA_TEXT hash | CTRL_HEADER hash | memo index |
|---|---|---|---:|
| `samples/aift.hwp` | `5cffb518e5814aca` | `46a646257c075cf6` | 2 |
| `stage7/aift-memo-field-end-index.hwp` | `5cffb518e5814aca` | `46a646257c075cf6` | 2 |

Stage 7 후보의 두 번째 메모 anchor `PARA_TEXT`:

```text
03 00 65 6d 25 25 00 00 00 00 00 00 00 00 03 00
20 00 20 00 20 00 20 00 2d 00 20 00 f5 ac d9 b3
30 ae 00 ad 32 00 20 00 3a 00 20 00 c1 c9 a4 c2
4c d1 6c d0 80 b1 5c b8 c0 c9
04 00 65 6d 25 00 01 ff ff 00 02 00 00 00 04 00
0d 00
```

## 6. 실행한 검증

```text
cargo fmt
cargo test -q test_memo_field_end_uses_hancom_marker_tail
cargo fmt --check
cargo check
cargo run --quiet --bin rhwp -- info output/poc/hwpx2hwp/task1092/stage7_memo_field_end_index/aift-memo-field-end-index.hwp
cargo run --quiet --bin rhwp -- hwp5-anchor-trace output/poc/hwpx2hwp/task1092/stage7_memo_field_end_index/aift-memo-field-end-index.hwp --section 2 --needle 공동기관2 --window 4 --out /tmp/aift_stage7_anchor2_trace.md
```

결과:

```text
success
```

## 7. 한컴 판정 결과

작업지시자 판정 결과, Stage 7 후보는 성공이다.

| file | 한컴 판정 유형 | 첫 번째 메모 스타일 | 두 번째 메모 스타일 | 표/페이지 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1092/stage7_memo_field_end_index/aift-memo-field-end-index.hwp` | 성공 | 성공 | 성공 | 별도 이슈로 분리 | 성공 | memo FIELD_END index 정합 후보 |

이번 단계에서 확정한 계약:

```text
1. HWPX memo anchor를 HWP5로 저장할 때 FIELD_BEGIN은 기존 `%%me` marker를 사용한다.
2. FIELD_END는 일반 `%%me` marker가 아니라 한컴 전용 memo end marker를 사용한다.
3. FIELD_END marker의 6번째 code unit에는 해당 memo index를 기록해야 한다.
4. memo index가 1로 고정되면 첫 번째 메모만 정상 표시되고, 두 번째 이후 메모 스타일은 풀린다.
```
