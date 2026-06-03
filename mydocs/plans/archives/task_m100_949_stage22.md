# Task M100 #949 Stage 22 계획서 — hwpx-h-03 drawText 내부 record contract 추적

## 1. 현재 기준

현재 작업 기준:

```text
branch: local/task949
base:   local/devel = origin/devel = 39d90d9d
issue:  #949 [hwpx2hwp] hwp5-inventory / hwp5-inventory-diff P0 구현
```

주의:

```text
기존 task/m100-949-hwp5-inventory 브랜치는 #955 rustfmt 정규화 이전 기준이다.
해당 브랜치의 핵심 커밋은 최신 local/devel에 patch-equivalent 형태로 이미 반영되어 있다.
따라서 후속 작업은 오래된 브랜치를 재사용하지 않고 local/task949에서 진행한다.
```

## 2. Stage 21까지의 결론

Stage 21에서 반영한 계약:

```text
1. hp:rect 계열 shape의 hp:shapeComment를 CommonObjAttr.description으로 보존
2. hp:drawText > hp:subList@vertAlign=CENTER를 LIST_HEADER list_attr bit 5로 materialize
```

정적 확인:

```text
BodyText.Section0#824 CTRL_HEADER GenShape size=60
BodyText.Section0#826 LIST_HEADER size=20 head20=01 00 00 00 20 00 ...
BodyText.Section0#827 PARA_HEADER size=22
BodyText.Section0#833 CTRL_DATA size=76 hash=024e873ad9c2bd92
```

남은 차이:

```text
#826 LIST_HEADER: oracle 33B / generated 20B
#827 PARA_HEADER: oracle 24B / generated 22B
```

## 3. 문제 정의

`hwpx-h-03`의 2페이지 이미지 개체 묶기 실패/파일손상 후보를 이제 `hp:rect > hp:drawText`
내부 record contract로 좁힌다.

Stage 22에서는 다음을 하지 않는다.

```text
- 감으로 HWP record payload를 graft하지 않는다.
- hwpx-h-01 성공 조건을 깨는 후보를 만들지 않는다.
- 한컴 시각 판정 요청만 반복하지 않는다.
```

대신 다음을 수행한다.

```text
- HWPX source node와 oracle HWP record tuple의 대응을 명시한다.
- generated HWP의 누락 byte가 어떤 HWP5 record field 또는 tail인지 분리한다.
- 후보 구현 전 정적 trace를 먼저 만든다.
```

## 4. 작업 범위

### 포함

```text
1. Stage 21 산출물을 최신 local/task949 기준에서 재현 가능한지 확인
2. hwpx-h-03 oracle/generated의 #824~#838 record window 재덤프
3. #826 LIST_HEADER 33B/20B payload를 필드 단위로 디코드
4. #827 PARA_HEADER 24B/22B payload를 필드 단위로 디코드
5. hp:rect/drawText/subList/p/run/pic/t source tree와 HWP record tuple 대응표 작성
6. 구현 후보가 필요한 경우 최소 후보를 계획서 후속 단계로 분리
```

### 제외

```text
- production serializer 대규모 수정
- HWPX to HWP 저장기 전반 재설계
- unrelated rustfmt 정규화
- hwpx-h-01/hwpx-h-02 성공 guard를 깨는 실험
```

## 5. 산출물

출력 경로:

```text
output/poc/hwpx2hwp/task949/stage22_drawtext_record_contract/
```

예상 산출물:

```text
hwpx-h-03_source_tree.md
record_window_824_838_oracle.md
record_window_824_838_generated.md
list_header_826_decode.md
para_header_827_decode.md
drawtext_contract_findings.md
```

완료 보고서:

```text
mydocs/working/task_m100_949_stage22.md
```

## 6. 검증

정적 검증:

```bash
cargo run --bin rhwp -- hwp5-inventory ...
cargo run --bin rhwp -- hwp5-inventory-diff ...
cargo run --bin rhwp -- dump-records ...
```

코드 변경이 발생하는 경우에만 추가 검증:

```bash
cargo fmt --all -- --check
cargo test --quiet <관련 테스트>
cargo clippy -- -D warnings
```

## 7. 승인 요청

Stage 22는 구현 후보를 만들기 전에 `hwpx-h-03`의 `hp:rect > hp:drawText` 내부 HWP record
계약을 정적으로 닫는 단계다.

승인 후 위 범위로 진행한다.
