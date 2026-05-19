# Task M100-993 Stage 16 작업 기록

## 0. 정정

작업지시자 판정 기준으로 `Stage 15`의 다음 파일을 통과 기준으로 고정한다.

```text
output/poc/hwpx2hwp/task993/stage15_first_para_text_marker_contract/02_oracle_para_text_guard.hwp
```

이 파일은 1페이지 고스트 선뿐 아니라 2페이지 인원 현황 표도 정상 처리된다.

Stage 16에서 생성한 `mel-001.hwp`는 TAB extension byte를 정답지와 맞춘 후보이지만,
셀 중 과도하게 높이가 커지는 잔여 문제가 있으므로 최종안으로 채택하지 않는다.

```text
채택 기준:
  Stage 15 02_oracle_para_text_guard.hwp

채택하지 않는 것:
  Stage 16 mel-001.hwp를 최종안으로 보는 해석
```

따라서 Stage 16의 의미는 "TAB extension byte 정합성 확인"에 한정한다. 전체 #993 최종 구현은
`Stage 15 02`의 통과 상태를 adapter 산출물이 재현하도록 맞추는 방향으로 이어간다.

## 1. 목적

Stage 16은 첫 문단 `hp:tab`을 HWP5 `PARA_TEXT` TAB extension으로 정확히 materialize해
1페이지 시작 부분 고스트 선 문제를 닫는 단계다.

Stage 15 판정 결과:

```text
02_oracle_para_text_guard.hwp만 통과
```

따라서 marker 순서만으로는 충분하지 않고, 같은 `PARA_TEXT` 내부의 TAB extension payload까지
정답지와 같아져야 한다.

## 2. 적용한 규칙

HWPX 원본:

```xml
<hp:tab width="4000" leader="0" type="1"/>
```

기존 generated TAB extension:

```text
[0x0fa0, 0x0000, 0x0001, 0x0000, 0x0000, 0x0000, 0x0000]
```

정답 HWP TAB extension:

```text
[0x0fa0, 0x0000, 0x0100, 0x0020, 0x0020, 0x0020, 0x0009]
```

Stage 16 적용 규칙:

```text
1. width는 그대로 저장한다.
2. leader는 그대로 저장한다.
3. type은 상위 byte 위치로 저장한다.
   - type=1 -> 0x0100
4. ext[3..=5]는 space code 0x0020으로 채운다.
5. ext[6]은 TAB code 0x0009로 채운다.
```

## 3. 구현 위치

수정 파일:

```text
src/parser/hwpx/section.rs
```

구현 내용:

```text
1. hp:tab 확장값 생성을 parse_tab_extension() 헬퍼로 통합했다.
2. parse_paragraph()의 직접 tab 파싱 경로와 read_text_content_with_tabs() 경로가 같은 규칙을 사용한다.
3. serializer는 기존처럼 Paragraph.tab_extended를 그대로 직렬화한다.
```

## 4. 생성 파일

생성 폴더:

```text
output/poc/hwpx2hwp/task993/stage16_tab_extension_final/
```

판정 파일:

| file | 한컴 판정 유형 | 1페이지 고스트 선 | 1x1 표 배경 | 셀 대각선 | 조직도 셀 줄나눔 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task993/stage16_tab_extension_final/mel-001.hwp` |  |  |  |  |  |  |  | TAB extension 후보, 최종안 아님 |
| `output/poc/hwpx2hwp/task993/stage16_tab_extension_final/tb-org-02.hwp` |  |  | - |  |  |  |  | 조직도 guard |

파일 크기:

| file | size |
|---|---:|
| `mel-001.hwp` | 162,304 bytes |
| `tb-org-02.hwp` | 7,680 bytes |

## 5. trace 확인

trace 생성 폴더:

```text
output/poc/hwpx2hwp/task993/stage16_tab_extension_final/trace/
```

첫 문단 `PARA_TEXT`에서 oracle/generated의 TAB 주변 byte가 일치한다.

```text
oracle:
  09 00 a0 0f 00 00 00 01 20 00 20 00 20 00 09 00

generated:
  09 00 a0 0f 00 00 00 01 20 00 20 00 20 00 09 00
```

marker stream도 정답지와 같은 순서다.

```text
SectionDef -> ColumnDef -> PageNumPos -> PageHide -> TAB -> Header
```

## 6. 실행한 검증

```text
cargo check
cargo run --bin rhwp -- convert samples/hwpx/mel-001.hwpx \
  output/poc/hwpx2hwp/task993/stage16_tab_extension_final/mel-001.hwp
cargo run --bin rhwp -- convert samples/hwpx/tb-org-02.hwpx \
  output/poc/hwpx2hwp/task993/stage16_tab_extension_final/tb-org-02.hwp
cargo run --bin rhwp -- hwp5-first-para-control-probe \
  samples/hwpx/hancom-hwp/mel-001.hwp \
  output/poc/hwpx2hwp/task993/stage16_tab_extension_final/mel-001.hwp \
  --out-dir output/poc/hwpx2hwp/task993/stage16_tab_extension_final/trace \
  --section 0 \
  --triplet-matrix
```

결과:

```text
success
```

## 7. 판정 요청

Stage 16 산출물은 최종안으로 판정하지 않는다. 다음 사실만 기록한다.

```text
1. TAB extension byte는 정답지와 일치한다.
2. 그러나 Stage 16 mel-001.hwp는 셀 높이 잔여 문제 때문에 최종안이 아니다.
3. 최종 기준은 Stage 15 02_oracle_para_text_guard.hwp다.
4. Stage 15 02는 2페이지 인원 현황 표가 정상 처리된다.
5. 다음 구현은 Stage 15 02의 통과 상태를 adapter 산출물이 재현하도록 맞춘다.
```
