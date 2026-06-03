# Task m100 #949 Stage 3 작업 보고서: hwp5-inventory-diff P0

## 1. 목적

Stage 3에서는 `hwp5-inventory` 결과를 이용해 두 HWP 파일의 record 차이를 자동 분류하는
`hwp5-inventory-diff` 명령을 구현했다.

이 단계의 목적은 한컴 HWP oracle과 rhwp generated HWP 사이에서 HWP5 record contract 차이가
어느 stream / record 위치에서 나타나는지 빠르게 확인하는 것이다.

## 2. 구현 내용

추가/수정한 파일:

```text
src/diagnostics/hwp5_inventory_diff.rs
src/diagnostics/mod.rs
src/main.rs
```

추가 CLI:

```bash
rhwp hwp5-inventory-diff <oracle.hwp> <generated.hwp> \
  [--format jsonl|md] [--section N] [--out <path>]
```

비교 대상은 HWP 파일 두 개다.
명령 내부에서 각각 `hwp5-inventory`를 생성한 뒤 비교한다.

## 3. Diff 분류

Stage 3 P0에서 지원하는 `diff_kind`:

```text
missing
extra
tag_changed
size_changed
payload_changed
scope_changed
control_changed
```

비교 키:

```text
record_uid
```

현재 `record_uid`는 다음 값에서 만들어진다.

```text
stream_path + record_index
```

## 4. 생성 산출물

```text
output/poc/hwpx2hwp/task949/stage3/hwpx-h-01/inventory.diff.jsonl
output/poc/hwpx2hwp/task949/stage3/hwpx-h-01/inventory.diff.md
```

입력 파일:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
output/poc/hwpx2hwp/task903/web_save_repro/hwpx-h-01-web-save.hwp
```

## 5. 검증 결과

빌드:

```text
cargo build 성공
```

CLI help:

```text
rhwp hwp5-inventory-diff <oracle.hwp> <generated.hwp> [--format jsonl|md] [--section N] [--out <path>]
```

산출물 line count:

```text
inventory.diff.jsonl 30106
inventory.diff.md    30127
```

Markdown summary:

```text
control_changed 350
missing         7
payload_changed 7968
scope_changed   7069
size_changed    7646
tag_changed     7066
```

초기 diff 예:

```text
BodyText.Section0#0
  size_changed    PARA_HEADER 24 -> 22
  payload_changed PARA_HEADER payload hash 변경

BodyText.Section0#1000
  tag_changed     PARA_HEADER -> PARA_CHAR_SHAPE
  size_changed    24 -> 8
  payload_changed
  scope_changed
```

## 6. 해석

Stage 3 도구는 HWP5 record 차이를 자동으로 잡아낸다.
다만 현재 비교 키가 `stream_path + record_index`라, 중간 record 누락/추가가 발생하면 이후
record들이 연쇄적으로 `tag_changed`, `size_changed`, `scope_changed`로 보인다.

따라서 Stage 3 결과는 다음 용도로 적합하다.

```text
1. 차이가 발생하는 최초 record 부근 찾기
2. stream/section 단위 record count 차이 확인
3. payload hash 변경 규모 파악
4. 다음 Stage에서 tuple-aware alignment가 필요한 구간 식별
```

Stage 3 결과만으로 다음을 단정하면 안 된다.

```text
1. 모든 tag_changed가 독립 원인이라는 결론
2. 모든 payload_changed가 직접 구현 대상이라는 결론
3. record_index가 밀린 뒤의 후속 diff를 실제 contract violation으로 확정
```

## 7. 현재 한계

현재 P0 diff는 sequence alignment가 없다.

한계:

```text
1. record 삽입/삭제 후 shift noise가 크다.
2. TABLE / PIC / SHAPE tuple 묶음 단위 비교가 아니다.
3. HWPX control과 직접 매칭하지 않는다.
4. diff 결과를 contract_violation_hints.md로 압축하지 않는다.
```

## 8. 다음 단계

Stage 4에서는 tuple-aware alignment를 설계한다.

우선순위:

```text
1. stream별 record sequence LCS
2. tuple_role + control_id 기반 anchor matching
3. TABLE / SHAPE_COMPONENT / PIC tuple 묶음 비교
4. 최초 divergence 지점과 재동기화 지점을 report로 출력
```

Stage 4 목표는 Stage 3의 30,106개 diff row를 사람이 판단 가능한 contract 후보 목록으로
압축하는 것이다.
