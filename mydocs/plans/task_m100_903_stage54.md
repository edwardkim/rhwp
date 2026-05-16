# Task m100 #903 Stage 54 계획

## 1. 목적

Stage53 판정으로 현재 구현 산출물의 잔여 실패 원인이 다음 최소 조합으로 좁혀졌다.

```text
1. BIN_DATA metadata/raw_data 보강
2. BodyText CTRL_HEADER payload 보존/합성
```

Stage54의 목적은 이 두 축을 실제 `hwpx -> IR -> hwp` 저장 경로에 반영한
최소 구현 후보를 만들고, 한컴 에디터와 rhwp-studio에서 다시 판정하는 것이다.

## 2. Stage53에서 확정된 근거

Stage53 주요 판정:

| variant | 결과 | 해석 |
|---|---|---|
| `03_current_plus_bindata_ctrl_header` | 한컴 성공, rhwp-studio 성공 | 최소 성공 조합 |
| `05_current_plus_bindata_ctrl_header_vertical_bits` | `03`과 동일 SHA-256, 성공 | vertical bits 추가는 no-op |
| `04_current_plus_parashape_vertical_bits` | baseline과 동일 SHA-256 | Stage52 vertical bits는 이미 산출물에 반영됨 |
| `06_current_plus_parashape_full` | 실패 | ParaShape full 단독은 잔여 문제의 직접 원인이 아님 |
| `07_current_plus_bindata_ctrl_header_parashape_full` | 성공 | 상한 후보이나 최소 구현 후보는 아님 |

따라서 Stage54에서는 ParaShape 전체 raw 보존을 구현하지 않는다.
Stage52의 `align.vertical -> attr1 bits 20..21` 구현도 이미 반영된 것으로 보고
이번 단계의 주 구현 범위에서 제외한다.

## 3. 구현 범위

### 3.1 BIN_DATA 보강

HWPX에서 온 `BinData`가 HWP 저장 시 한컴이 인식할 수 있는 embedded binary
metadata로 직렬화되도록 보강한다.

구현 후보:

```text
current:
  raw_data 없음
  attr=0x0
  status=NotAccessed

target:
  raw_data 있음
  attr=0x101
  status=Success
```

주의점:

```text
- 실제 BinData/BinDataContent 저장소 매핑은 유지한다.
- HWPX 원본의 이미지 바이트를 변환하지 않는다.
- 웹 렌더러용 변환 산출물이 아니라 HWP 저장용 원본 bindata 경로를 우선한다.
- hwp -> hwp 저장 경로의 기존 raw_data 보존 동작은 건드리지 않는다.
```

### 3.2 CTRL_HEADER payload 보강

HWPX에서 생성된 table/object control이 HWP 저장 시 한컴이 기대하는
`CTRL_HEADER` payload를 갖도록 보강한다.

구현 후보:

```text
- table/common object attribute에서 CTRL_HEADER payload를 안정적으로 합성한다.
- 기존 adapter의 raw_ctrl_data 합성 결과가 부족한 경우 누락 필드를 보완한다.
- Stage47/53에서 성공 축으로 확인된 CTRL_HEADER 계열만 우선 반영한다.
- 모든 table attr를 일괄 보존하지 않고, 기존 성공 guard 범위 안에서만 반영한다.
```

주의점:

```text
- LIST_HEADER, PARA_HEADER, PARA_TEXT, PARA_LINE_SEG 전체 raw graft는 하지 않는다.
- TABLE record 전체 raw graft도 하지 않는다.
- 성공 probe를 그대로 복사하는 방식이 아니라 모델 기반 직렬화 경로를 보강한다.
```

## 4. 예상 수정 파일

우선 검토 대상:

```text
src/parser/hwpx/mod.rs
src/document_core/converters/hwpx_to_hwp.rs
src/model/bin_data.rs
src/serializer/doc_info.rs
src/serializer/control.rs
tests/hwpx_to_hwp_adapter.rs
```

문서:

```text
mydocs/working/task_m100_903_stage54.md
```

## 5. 생성 산출물

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage54_minimal_impl_candidate/
```

판정 대상:

```text
output/poc/hwpx2hwp/task903/stage54_minimal_impl_candidate/hwpx-h-01.hwp
```

리포트:

```text
output/poc/hwpx2hwp/task903/stage54_minimal_impl_candidate/stage54_generation.md
output/poc/hwpx2hwp/task903/stage54_minimal_impl_candidate/bindata_after_impl.md
output/poc/hwpx2hwp/task903/stage54_minimal_impl_candidate/ctrl_header_after_impl.md
```

## 6. 검증 명령

최소 후보 생성:

```bash
cargo test --test hwpx_to_hwp_adapter task903_stage54_generate_minimal_impl_candidate -- --nocapture
```

기존 핵심 샘플 회귀 확인:

```bash
cargo test --test hwpx_to_hwp_adapter task888_stage4_generate_local_devel_outputs -- --nocapture
cargo test --test hwpx_to_hwp_adapter task899_stage4_generate_business_overview_outputs -- --nocapture
```

위 명령 이름은 실제 테스트 함수명 확인 후 조정한다.

## 7. 작업지시자 판정 요청

| 파일 | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task903/stage54_minimal_impl_candidate/hwpx-h-01.hwp` |  |  |  |  |  |  |  |

판정 기준:

```text
- 한컴 에디터에서 파일 읽기 오류나 파일손상 없이 열릴 것
- 1페이지 표 안 이미지 2개가 출력될 것
- 2페이지 이미지 개체 묶기가 출력될 것
- 표/셀 배치가 정답지 수준으로 회복될 것
- 셀 텍스트 클리핑이 사라질 것
- 마지막 9페이지가 출력될 것
- rhwp-studio에서도 정상 재열기/렌더링될 것
```

## 8. 실패 시 해석

Stage54 단일 후보가 실패하면 다음처럼 분기한다.

```text
이미지만 실패:
  BIN_DATA raw_data/attr/status 또는 BinDataContent 매핑을 재검토한다.

표 배치만 실패:
  CTRL_HEADER 합성 payload가 Stage53 positive graft와 아직 다르다는 뜻이다.
  다음 stage에서 CTRL_HEADER payload diff를 필드 단위로 좁힌다.

셀 텍스트 클리핑만 실패:
  Stage53의 03 성공 판정과 충돌하므로 generated hash와 실제 판정 파일을 먼저 확인한다.

마지막 페이지만 실패:
  DocProperties.section_count 보정 회귀 여부를 확인한다.
```

## 9. 승인 후 작업 순서

```text
1. 현재 worktree 상태 확인
2. Stage54 작업 기록 문서 생성
3. BIN_DATA 보강 구현
4. CTRL_HEADER payload 보강 구현
5. Stage54 생성 테스트 추가
6. cargo test로 산출물 생성
7. 작업지시자 판정 요청
8. 판정 결과에 따라 구현 범위 확정 또는 다음 probe 계획 작성
```

## 10. devel 반영 전 정정

Stage54 시각 판정 후 `local/devel` 반영 과정에서 기존 3샘플 페이지 회귀를 다시
확인했다. 모든 table의 packed attr를 보존하는 구현은 `hwpx-h-02.hwpx`를
9페이지에서 10페이지로 늘리는 부작용이 있었다.

따라서 최종 적용 범위는 다음처럼 정정한다.

```text
1. BIN_DATA metadata/status materialize
2. table CTRL_HEADER attr 보존은 기존 guard(materialize_hancom_table/materialize_tac_table) 범위 유지
3. HWPX table id/zOrder 파싱
4. HWPX picture shapeComment 파싱
```

이 정정 후 `stage5_all_three_samples_recover_via_unified_entry_point`와
`task903_stage54_generate_minimal_impl_candidate`가 함께 통과했다.
