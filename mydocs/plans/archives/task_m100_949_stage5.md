# Task m100 #949 Stage 5 계획서: contract_violation_hints 생성

## 1. 목적

Stage 4의 `--align lcs` 결과는 diff row를 `30106 -> 3890`으로 줄였다.
하지만 아직 사람이 바로 구현 후보를 판단하기에는 row 수가 많다.

Stage 5에서는 `hwp5-inventory-diff`에 사람이 읽는 후보 보고서 출력을 추가한다.

목표 산출물:

```text
output/poc/hwpx2hwp/task949/stage5/hwpx-h-01/contract_violation_hints.md
```

이 문서는 다음 용도로 사용한다.

```text
1. 한컴 HWP oracle과 rhwp generated HWP의 주요 contract 차이 후보 파악
2. TABLE / PIC / SHAPE_COMPONENT / CTRL_HEADER 중심의 구현 우선순위 도출
3. missing record와 changed payload가 집중되는 구간 확인
4. 다음 probe 또는 구현 task를 만들기 위한 근거 정리
```

## 2. Stage 5 범위

구현할 것:

```text
1. `hwp5-inventory-diff`에 `--report hints` 옵션 추가
2. hints report는 Markdown으로만 출력
3. LCS alignment 결과를 기반으로 후보를 요약
4. role/control별 changed/missing/extra count 출력
5. missing record 목록 출력
6. TABLE / PIC / SHAPE_COMPONENT / CTRL_HEADER 후보 목록 출력
7. payload changed가 많은 tuple role 순위 출력
```

구현하지 않을 것:

```text
1. HWPX inventory 직접 비교
2. probe HWP 자동 생성
3. HWPX -> HWP 저장기 수정
4. 한컴 판정 자동화
5. contract 확정 판정
```

## 3. CLI 확장

기존:

```bash
rhwp hwp5-inventory-diff <oracle.hwp> <generated.hwp> \
  [--align index|lcs] [--format jsonl|md] [--section N] [--out <path>]
```

추가:

```bash
--report diff|hints
```

의미:

```text
diff:
  기존 JSONL/Markdown diff row 출력

hints:
  사람이 읽는 후보 요약 Markdown 출력
  내부적으로 --align lcs 사용을 권장한다.
```

기본값은 `diff`다.

## 4. hints report 구성

```text
# HWP5 Contract Violation Hints

## Input
## Alignment Summary
## Top Role/Control Buckets
## Missing Records
## Table Candidates
## Picture/Shape Candidates
## CtrlHeader Candidates
## Payload Changed Hotspots
## Next Probe Suggestions
```

## 5. 후보 분류 규칙

`Table Candidates`:

```text
tuple_role == table
or control_name == Table
```

`Picture/Shape Candidates`:

```text
tuple_role == pic
or tuple_role == shape_component
or control_name == GenShape
```

`CtrlHeader Candidates`:

```text
tuple_role == ctrl_header
```

`Missing Records`:

```text
alignment_status == missing
```

`Payload Changed Hotspots`:

```text
changed_fields contains payload_hash
group by tuple_role + control_name
```

## 6. 검증 명령

```bash
cargo build

./target/debug/rhwp hwp5-inventory-diff \
  samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  output/poc/hwpx2hwp/task903/web_save_repro/hwpx-h-01-web-save.hwp \
  --align lcs \
  --report hints \
  --out output/poc/hwpx2hwp/task949/stage5/hwpx-h-01/contract_violation_hints.md
```

## 7. 완료 기준

```text
1. cargo build 통과
2. --report diff 기존 동작 유지
3. --report hints 산출물 생성
4. contract_violation_hints.md에 missing / table / picture / ctrl header 후보가 출력됨
5. Stage 5 보고서 작성
```

## 8. 다음 단계

Stage 6에서는 hints report를 바탕으로 실제 구현/probe task를 분리한다.

후보:

```text
1. HWPX -> HWP 저장기의 TABLE control tuple mapping
2. PIC/SHAPE_COMPONENT payload mapping
3. DocInfo BinData/ID mapping 검증
4. missing CTRL_DATA / FORBIDDEN_CHAR 계열 처리
```
