# Task M100-1099 Stage 3 작업 기록

## 1. 목적

Stage 2에서 Header/Footer 내부 Table materialization 결함은 제거했지만, 1페이지 축소 샘플은
여전히 한컴 에디터에서 파일손상 판정을 받았다.

이번 단계는 다음 후보를 검증했다.

```text
SectionDef 아래 바탕쪽(master page) LIST_HEADER envelope / 순서 / 확장 바탕쪽 위치 contract
```

## 2. 원인 후보

Stage 2 생성 파일은 rhwp reload 기준으로 세 번째 바탕쪽을 `Even extension`으로 해석했다.

```text
Stage 2:
  [0] Both
  [1] Odd
  [2] Even, is_ext=true, overlap=true, ext_flags=0x0003
```

정답 HWP는 세 번째 바탕쪽을 `Both extension`으로 해석한다.

```text
oracle:
  [0] Both
  [1] Odd
  [2] Both, is_ext=true, overlap=true, ext_flags=0x0003
```

정답지의 extension master page는 SectionDef child group 내부가 아니라 body paragraph stream 뒤쪽에
level 1 `LIST_HEADER`로 배치되어 있었다.

```text
BodyText.Section0#290
  LIST_HEADER level=1
  parent=PARA_HEADER#283@lv0
  head32=01 00 00 00 ... 00 00 03 00 ...
```

## 3. 구현

수정 파일:

```text
src/serializer/control.rs
src/serializer/body_text.rs
```

수정 내용:

```text
1. SectionDef child group에는 일반 master page만 저장한다.
2. `is_extension=true` master page는 body stream tail에 level 1 LIST_HEADER로 저장한다.
3. 기존 HWP raw stream에 이미 level 1 LIST_HEADER extension이 있으면 중복 저장하지 않는다.
```

이 변경은 HWPX `LAST_PAGE` master page를 HWP5 저장 시 한컴 정답지와 같은 extension envelope에
넣기 위한 것이다.

## 4. 생성 파일

```text
output/poc/hwpx2hwp/task1099/stage3_masterpage_envelope_contract/01_stage2_baseline.hwp
output/poc/hwpx2hwp/task1099/stage3_masterpage_envelope_contract/02_masterpage_apply_order_normalized.hwp
```

파일 크기:

| file | size |
|---|---:|
| `01_stage2_baseline.hwp` | 667K |
| `02_masterpage_apply_order_normalized.hwp` | 668K |

## 5. 구조 확인

`02_masterpage_apply_order_normalized.hwp`는 rhwp reload 기준으로 정답지와 같은 바탕쪽 해석이
된다.

```text
[0] Both, is_ext=false, overlap=false, ext_flags=0x0000
[1] Odd,  is_ext=false, overlap=false, ext_flags=0x0000
[2] Both, is_ext=true,  overlap=true,  ext_flags=0x0003
```

또한 candidate inventory에서도 extension master page가 정답지와 같은 body stream tail 위치로
이동했다.

```text
BodyText.Section0#290
  LIST_HEADER level=1
  parent=PARA_HEADER#283@lv0
  head32=01 00 00 00 ... 00 00 03 00 ...
```

비교 산출물:

```text
output/poc/hwpx2hwp/task1099/stage3_masterpage_envelope_contract/oracle.section0.inventory.md
output/poc/hwpx2hwp/task1099/stage3_masterpage_envelope_contract/candidate.section0.inventory.md
output/poc/hwpx2hwp/task1099/stage3_masterpage_envelope_contract/contract_violation_hints.md
```

## 6. 남은 차이

Stage 3 이후에도 정답지와 다음 차이는 남아 있다.

```text
1. DocInfo compatibility 계열 record 누락
   - FORBIDDEN_CHAR
   - COMPATIBLE_DOCUMENT
   - LAYOUT_COMPATIBILITY
   - TRACKCHANGE
2. FACE_NAME / PARA_SHAPE payload 차이
3. GenShape / Picture payload 차이
4. 일부 TABLE payload 차이
```

이번 단계는 바탕쪽 extension envelope의 structural defect만 제거했다. 한컴 파일손상 여부는
작업지시자의 한컴 에디터 판정으로 확정한다.

## 7. 검증

실행:

```text
cargo fmt --check
cargo test document_core::converters::hwpx_to_hwp::tests::header_footer_nested_tables_are_materialized
cargo build --bin rhwp
```

결과:

```text
success
```

## 8. 판정 요청

한컴 에디터 확인 대상:

| file | 한컴 판정 유형 | 바탕쪽 출력 | 지문 박스 출력 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1099/stage3_masterpage_envelope_contract/01_stage2_baseline.hwp` |  |  |  |  |  | baseline |
| `output/poc/hwpx2hwp/task1099/stage3_masterpage_envelope_contract/02_masterpage_apply_order_normalized.hwp` |  |  |  |  |  | extension master page envelope 후보 |

## 9. 다음 판단

```text
1. 02 파일이 한컴에서 정상 로딩되면:
   - 파일손상 원인은 extension master page envelope로 확정한다.
   - 2/3/4페이지 축소 샘플로 확장한다.

2. 02 파일도 파일손상이면:
   - 바탕쪽 extension envelope 수정은 유지한다.
   - 다음 후보는 DocInfo compatibility record 또는 GenShape/Picture payload 차이로 이동한다.
```

## 10. 판정 후 결론

작업지시자 판정:

```text
02_masterpage_apply_order_normalized.hwp:
  - 한컴 에디터에서 열림
  - 1~3항 문제 영역의 문단 경계선이 외곽만 연결되지 않고 문단 시작/끝에도 출력됨
  - rhwp-studio에서는 정상 출력
```

해석:

```text
1. 파일손상 원인은 extension master page envelope로 확정한다.
2. 남은 문제는 한컴 에디터의 문단 테두리 연결 contract 문제로 분리한다.
3. HWPX paraPr border/@connect 값이 HWP5 PARA_SHAPE attr1 bit 28로 저장되는지 확인한다.
```
