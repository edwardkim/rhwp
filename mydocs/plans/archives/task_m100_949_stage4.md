# Task m100 #949 Stage 4 계획서: tuple-aware alignment diff

## 1. 목적

Stage 3의 `hwp5-inventory-diff`는 `stream_path + record_index` 기반 P0 diff다.
이 방식은 최초 차이를 빠르게 찾는 데는 유용하지만, 중간 record 삽입/삭제가 발생하면
이후 record가 연쇄적으로 `tag_changed`, `size_changed`, `scope_changed`로 번진다.

Stage 4의 목적은 이 shift noise를 줄이고, 사람이 판단 가능한 contract 후보 구간으로
diff 결과를 압축하는 것이다.

## 2. 문제 정의

Stage 3 `hwpx-h-01` 결과:

```text
diff_count      30106
control_changed 350
missing         7
payload_changed 7968
scope_changed   7069
size_changed    7646
tag_changed     7066
```

이 수치는 실제 구현 후보 수가 아니라 record index가 밀린 뒤 발생한 noise를 포함한다.

따라서 Stage 4에서는 다음 질문에 답해야 한다.

```text
1. 두 record sequence가 어디서 diverge하는가?
2. 다시 같은 구조로 resync되는 지점이 있는가?
3. TABLE / SHAPE / PIC 같은 control tuple 단위로 보면 어떤 tuple이 빠지거나 변형되었는가?
4. 한컴 contract 관점에서 우선 검토해야 할 구간은 어디인가?
```

## 3. Stage 4 범위

구현할 것:

```text
1. stream별 alignment mode 추가
2. LCS 기반 record sequence alignment 구현
3. alignment 결과를 Markdown/JSONL로 출력
4. diff row에 alignment_status 추가
5. divergence / resync 구간 요약 출력
6. tuple_role + control_name 기반 anchor summary 출력
```

구현하지 않을 것:

```text
1. HWPX inventory와 직접 매칭
2. HWP variant probe 자동 생성
3. HWPX -> HWP 저장기 수정
4. 한컴 시각 판정용 신규 파일 생성
5. 완전한 semantic diff 판정
```

## 4. CLI 확장

기존 CLI:

```bash
rhwp hwp5-inventory-diff <oracle.hwp> <generated.hwp> \
  [--format jsonl|md] [--section N] [--out <path>]
```

Stage 4에서 옵션 추가:

```bash
--align index|lcs
```

의미:

```text
index:
  Stage 3 방식. record_uid(stream_path + record_index) 기반 비교.

lcs:
  stream별 record signature LCS를 계산해 aligned / missing / extra / changed를 분류.
```

기본값은 `index`로 유지한다.

## 5. LCS signature

P0 signature 후보:

```text
tag_id
tuple_role
control_id
payload_hash
size
```

Stage 4에서는 너무 엄격한 signature와 너무 느슨한 signature를 분리한다.

```text
strict_signature:
  tag_id + tuple_role + control_id + size + payload_hash

structural_signature:
  tag_id + tuple_role + control_id
```

LCS anchor는 우선 `structural_signature`를 사용한다.
payload/size 차이는 aligned 이후 changed field로 분리한다.

## 6. 출력 스키마

JSONL diff item에 추가할 필드:

```text
align_mode
alignment_status
oracle_record_index
generated_record_index
oracle_record_uid
generated_record_uid
signature
changed_fields
```

`alignment_status` 후보:

```text
matched
changed
missing
extra
```

Markdown 추가 섹션:

```text
## Alignment Summary
## Divergence Blocks
## Tuple Anchor Summary
```

## 7. Stage 4 산출물

```text
output/poc/hwpx2hwp/task949/stage4/hwpx-h-01/
  inventory.diff.index.md
  inventory.diff.index.jsonl
  inventory.diff.lcs.md
  inventory.diff.lcs.jsonl
  stage4_report.md
```

## 8. 검증 명령

```bash
cargo build

./target/debug/rhwp hwp5-inventory-diff \
  samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  output/poc/hwpx2hwp/task903/web_save_repro/hwpx-h-01-web-save.hwp \
  --align lcs \
  --format md \
  --out output/poc/hwpx2hwp/task949/stage4/hwpx-h-01/inventory.diff.lcs.md

./target/debug/rhwp hwp5-inventory-diff \
  samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  output/poc/hwpx2hwp/task903/web_save_repro/hwpx-h-01-web-save.hwp \
  --align lcs \
  --format jsonl \
  --out output/poc/hwpx2hwp/task949/stage4/hwpx-h-01/inventory.diff.lcs.jsonl
```

## 9. 완료 기준

```text
1. cargo build 통과
2. --align index 기존 동작 유지
3. --align lcs 산출물 생성
4. Stage 3 대비 diff row 또는 summary noise가 줄어드는지 확인
5. divergence / resync 구간이 Markdown에 출력됨
6. Stage 4 보고서 작성
```

## 10. 리스크

LCS는 record 수가 많으면 비용이 커질 수 있다.

Stage 4에서는 stream별로 계산하고, 필요하면 다음 제한을 둔다.

```text
1. stream 단위 분리
2. structural_signature로만 LCS
3. 큰 stream은 구간 단위 alignment로 후속 분리
```

## 11. 다음 단계

Stage 5에서는 Stage 4 alignment 결과를 이용해 `contract_violation_hints.md`를 생성한다.

후보 출력:

```text
1. 최초 divergence top N
2. tuple_role별 missing/extra/payload changed
3. TABLE / PIC / SHAPE tuple 후보
4. 한컴 판정과 연결할 probe 후보 목록
```
