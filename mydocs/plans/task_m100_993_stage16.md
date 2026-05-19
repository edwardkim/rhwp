# Task M100-993 Stage 16 계획서

## 0. 계획 정정

Stage 16 계획의 "최종 후보 생성" 표현은 폐기한다.

작업지시자 판정에 따라 최종 기준은 Stage 15의 다음 통과 파일이다.

```text
output/poc/hwpx2hwp/task993/stage15_first_para_text_marker_contract/02_oracle_para_text_guard.hwp
```

이 파일은 1페이지 고스트 선뿐 아니라 2페이지 인원 현황 표도 정상 처리되는 기준 파일이다.

Stage 16은 TAB extension byte 정합성 확인으로만 취급한다. Stage 16 산출물이 셀 높이 잔여 문제를
가지면 최종안으로 채택하지 않는다.

## 1. 목적

Stage 16은 추가 probe 단계가 아니다.

Stage 15에서 이미 다음 사실이 확정되었다.

```text
1. 첫 문단 PARA_TEXT 전체를 정답지로 투영한 02_oracle_para_text_guard.hwp만 통과했다.
2. marker 순서만 보정한 03_marker_order_candidate.hwp는 통과하지 못했다.
3. marker 순서는 정답지와 같아졌으므로, 남은 차이는 TAB extension payload다.
```

따라서 Stage 16의 목적은 `HWPX <hp:tab>`을 HWP5 `PARA_TEXT` TAB extension으로 정확히
materialize할 수 있는지 확인하는 것이다.

## 2. 확정된 차이

HWPX 원본:

```xml
<hp:tab width="4000" leader="0" type="1"/>
```

현재 generated TAB extension:

```text
[0x0fa0, 0x0000, 0x0001, 0x0000, 0x0000, 0x0000, 0x0000]
```

정답 HWP TAB extension:

```text
[0x0fa0, 0x0000, 0x0100, 0x0020, 0x0020, 0x0020, 0x0009]
```

적용할 규칙:

```text
1. width는 그대로 보존한다.
2. leader는 그대로 보존한다.
3. type은 raw 값이 아니라 상위 byte 위치로 저장한다.
   - type=1 -> 0x0100
4. reserved/fill 영역 3개 word는 0이 아니라 space code 0x0020으로 채운다.
5. 마지막 word는 0이 아니라 TAB code 0x0009를 반복한다.
```

## 3. 구현 범위

수정 대상:

```text
src/parser/hwpx/section.rs
```

대상 로직:

```text
- hp:tab 파싱
- read_text_content_with_tabs()
```

기존 구조는 유지한다.

```text
1. HWPX parser가 tab_extended를 구성한다.
2. serializer는 Paragraph.tab_extended를 그대로 직렬화한다.
3. 첫 문단 전용 특수 처리는 만들지 않는다.
```

## 4. 생성 산출물

생성 폴더:

```text
output/poc/hwpx2hwp/task993/stage16_tab_extension_final/
```

판정 파일:

| file | 한컴 판정 유형 | 1페이지 고스트 선 | 1x1 표 배경 | 셀 대각선 | 조직도 셀 줄나눔 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task993/stage16_tab_extension_final/mel-001.hwp` |  |  |  |  |  |  |  | TAB extension 후보, 최종안 아님 |
| `output/poc/hwpx2hwp/task993/stage16_tab_extension_final/tb-org-02.hwp` |  |  | - |  |  |  |  | 조직도 guard |

## 5. 성공 기준

```text
1. mel-001.hwp에서 1페이지 고스트 선이 제거된다.
2. gradient BorderFill로 해결한 2페이지 첫 1x1 표 배경이 유지된다.
3. 셀 대각선 처리가 유지된다.
4. tb-org-02 조직도 셀 줄나눔 개선이 유지된다.
5. 파일손상 없이 열린다.
6. rhwp-studio 로딩/렌더링이 회귀하지 않는다.
```

## 6. 실행 검증

```text
cargo check
cargo run --bin rhwp -- convert samples/hwpx/mel-001.hwpx \
  output/poc/hwpx2hwp/task993/stage16_tab_extension_final/mel-001.hwp
cargo run --bin rhwp -- convert samples/hwpx/tb-org-02.hwpx \
  output/poc/hwpx2hwp/task993/stage16_tab_extension_final/tb-org-02.hwp
```

필요 시 trace로 TAB extension만 확인한다.

```text
cargo run --bin rhwp -- hwp5-first-para-control-probe \
  samples/hwpx/hancom-hwp/mel-001.hwp \
  output/poc/hwpx2hwp/task993/stage16_tab_extension_final/mel-001.hwp \
  --out-dir output/poc/hwpx2hwp/task993/stage16_tab_extension_final/trace \
  --section 0 \
  --triplet-matrix
```

## 7. 승인 요청

이 계획은 Stage 16에서 `hp:tab` 확장 payload 정합성을 확인하는 데 한정한다. 완료 기준은
Stage 15 `02_oracle_para_text_guard.hwp`의 통과 상태를 adapter 산출물이 재현하는 것이다.
