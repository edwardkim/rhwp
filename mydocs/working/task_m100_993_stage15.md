# Task M100-993 Stage 15 작업 기록

## 1. 목적

Stage 14 판정에서 `PARA_TEXT` 단독 투영 케이스가 1페이지 고스트 라인을 제거했다.

성공 케이스:

```text
03, 05, 07, 08
```

공통점:

```text
모두 첫 문단 PARA_TEXT를 포함한다.
```

Stage 15는 첫 문단 `PARA_TEXT` 안에서 어떤 값이 고스트 라인을 만든 것인지 구현 가능한 규칙으로
분리한다.

## 2. 입력

HWPX 원본:

```text
samples/hwpx/mel-001.hwpx
```

정답 HWP:

```text
samples/hwpx/hancom-hwp/mel-001.hwp
```

기준 생성물:

```text
output/poc/hwpx2hwp/task993/stage11_borderfill_diagonal_contract/04_diagonal_attr_payload.hwp
```

생성 폴더:

```text
output/poc/hwpx2hwp/task993/stage15_first_para_text_marker_contract/
```

## 3. HWPX 원본 순서

`samples/hwpx/mel-001.hwpx`의 첫 문단 XML 순서는 다음과 같다.

```xml
<hp:run charPrIDRef="11">
  <hp:secPr>...</hp:secPr>
  <hp:ctrl>
    <hp:colPr .../>
  </hp:ctrl>
</hp:run>
<hp:run charPrIDRef="11">
  <hp:ctrl>
    <hp:pageNum .../>
  </hp:ctrl>
  <hp:ctrl>
    <hp:pageHiding .../>
  </hp:ctrl>
  <hp:t>
    <hp:tab width="4000" leader="0" type="1"/>
  </hp:t>
</hp:run>
<hp:run charPrIDRef="7">
  <hp:ctrl>
    <hp:header ...>...</hp:header>
  </hp:ctrl>
  <hp:t/>
</hp:run>
```

따라서 HWP `PARA_TEXT` marker stream의 기대 순서는 다음이다.

```text
SectionDef -> ColumnDef -> PageNumPos -> PageHide -> TAB -> Header
```

## 4. 원인 확인

현재 HWPX parser의 `parse_ctrl()`는 `pageNum`, `pageHiding`, `header/footer`를 `controls`에는 넣지만
`text_parts`에는 위치 marker를 넣지 않았다.

그 결과:

```text
1. HWPX XML에서는 PageNumPos/PageHide가 TAB 앞에 있다.
2. 하지만 IR의 char_offsets에는 PageNumPos/PageHide 위치가 반영되지 않는다.
3. serializer는 char_offsets gap을 기준으로 controls를 배치한다.
4. 따라서 TAB이 ColumnDef 뒤로 당겨지고, PageNumPos/PageHide/Header는 TAB 뒤로 밀린다.
```

Stage 12/14에서 본 차이와 일치한다.

```text
oracle:
  SectionDef -> ColumnDef -> PageNumPos -> PageHide -> TAB -> Header

generated:
  SectionDef -> ColumnDef -> TAB -> PageNumPos -> PageHide -> Header
```

## 5. 적용한 구현 후보

`src/parser/hwpx/section.rs`의 `parse_ctrl()`에서 다음 control을 파싱할 때 `text_parts`에 8 code-unit
marker를 추가했다.

```text
- header
- footer
- pageNum
- pageHiding
```

이 marker는 실제 HWP control code를 직접 저장하지 않는다. HWPX parser 단계에서는 "이 위치에
HWP 확장 control marker가 있었다"는 위치 정보만 보존하고, 실제 code/id는 serializer의
`control_char_code_and_id()`가 `Control` variant를 기준으로 결정한다.

## 6. 생성 파일

| file | 한컴 판정 유형 | 1페이지 고스트 선 | 표/셀 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task993/stage15_first_para_text_marker_contract/01_stage14_baseline.hwp` | 미기록 | 존재, 미통과 | 미기록 | 미기록 | Stage 11/14 기준 |
| `output/poc/hwpx2hwp/task993/stage15_first_para_text_marker_contract/02_oracle_para_text_guard.hwp` | 정상 | 제거, 통과 | 2페이지 인원 현황 표 정상 | 미기록 | Stage 14 `03` guard, 현재 통과 기준 |
| `output/poc/hwpx2hwp/task993/stage15_first_para_text_marker_contract/03_marker_order_candidate.hwp` | 미기록 | 존재, 미통과 | 미기록 | 미기록 | HWPX parser marker 위치 후보 |

생성 결과:

| file | size | rhwp reload |
|---|---:|---|
| `01_stage14_baseline.hwp` | 162,304 bytes | 기존 기준 |
| `02_oracle_para_text_guard.hwp` | 162,304 bytes | 기존 guard |
| `03_marker_order_candidate.hwp` | 162,304 bytes | ok, pages=20 |

## 7. trace 확인

trace 생성 폴더:

```text
output/poc/hwpx2hwp/task993/stage15_first_para_text_marker_contract/trace_candidate/
```

후보 파일의 첫 문단 `PARA_TEXT` marker 순서:

```text
generated candidate:
  SectionDef -> ColumnDef -> PageNumPos -> PageHide -> TAB -> Header
```

이 순서는 정답지와 같다.

남은 차이:

```text
TAB 확장 데이터가 정답지와 다르다.
```

정답지와 후보의 TAB 주변 byte 비교:

```text
oracle:
  09 00 a0 0f 00 00 00 01 20 00 20 00 20 00 09 00

candidate:
  09 00 a0 0f 00 00 01 00 00 00 00 00 00 00 00 00
```

따라서 Stage 15 후보는 `control marker order` 축만 해결한 파일이다. 한컴 판정에서 여전히 고스트
라인이 남는다면, 다음 축은 `TAB extension materialization`으로 분리한다.

## 8. 실행한 검증

```text
cargo check
cargo run --bin rhwp -- convert \
  samples/hwpx/mel-001.hwpx \
  output/poc/hwpx2hwp/task993/stage15_first_para_text_marker_contract/03_marker_order_candidate.hwp
cargo run --bin rhwp -- hwp5-first-para-control-probe \
  samples/hwpx/hancom-hwp/mel-001.hwp \
  output/poc/hwpx2hwp/task993/stage15_first_para_text_marker_contract/03_marker_order_candidate.hwp \
  --out-dir output/poc/hwpx2hwp/task993/stage15_first_para_text_marker_contract/trace_candidate \
  --section 0 \
  --triplet-matrix
```

결과:

```text
success
```

## 9. 판정 포인트

```text
03_marker_order_candidate.hwp에서 고스트 선이 제거되면:
  원인은 page/header 계열 control marker 위치 누락으로 확정한다.

03_marker_order_candidate.hwp에서 고스트 선이 남으면:
  Stage 14의 PARA_TEXT 양성 원인은 marker 순서가 아니라 TAB extension까지 포함한 문제다.
  다음 단계에서 HWPX <hp:tab> -> HWP TAB extension materialization을 분리한다.
```

## 10. 한컴 판정 결과

작업지시자 판정:

```text
02_oracle_para_text_guard.hwp만 통과
```

해석:

```text
1. 첫 문단 PARA_TEXT 전체를 정답지로 투영한 02는 고스트 라인을 제거한다.
2. 02는 2페이지 인원 현황 표도 정상 처리한다.
3. 따라서 02는 단순 고스트 선 guard가 아니라 현재 통과 기준이다.
4. HWPX parser marker 위치만 보정한 03은 통과하지 못했다.
5. 따라서 Stage 15의 marker-order 보정은 방향상 필요하지만 충분조건은 아니다.
```

정정:

```text
이전 해석처럼 "02는 고스트 선만 통과한 guard"로 보면 안 된다.
02_oracle_para_text_guard.hwp는 2페이지 표 배치까지 정상인 기준 파일이다.
따라서 다음 구현은 Stage 16의 TAB extension 부분 후보가 아니라, 02의 통과 상태를 재현해야 한다.
```

Stage 15 후보의 marker stream은 정답지와 같아졌다.

```text
SectionDef -> ColumnDef -> PageNumPos -> PageHide -> TAB -> Header
```

하지만 이 사실만으로는 02의 정상 상태를 재현하지 못한다.

```text
1. 02_oracle_para_text_guard.hwp는 정상 처리된다.
2. 03_marker_order_candidate.hwp는 정상 처리되지 않는다.
3. Stage 16에서 TAB extension byte를 맞춘 것만으로도 02의 정상 상태는 재현되지 않는다.
4. 따라서 02와 현재 adapter 산출물의 차이를 직접 비교해 남은 contract를 찾아야 한다.
```

폐기할 해석:

```text
Stage 15의 통과 조건 = marker 순서 + TAB extension만 맞추면 충분하다.
```

유지할 해석:

```text
Stage 15의 통과 기준 = 02_oracle_para_text_guard.hwp의 실제 산출 상태다.
adapter는 이 상태를 재현해야 한다.
```
