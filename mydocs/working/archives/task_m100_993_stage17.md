# Task M100-993 Stage 17 작업 기록

## 1. 목적

Stage 15의 통과 기준 파일을 adapter 산출물로 재현한다.

기준 파일:

```text
output/poc/hwpx2hwp/task993/stage15_first_para_text_marker_contract/02_oracle_para_text_guard.hwp
```

이 기준 파일은 단순히 1페이지 고스트 선만 해결한 파일이 아니다. 작업지시자 판정에 따르면
2페이지 "인원 현황" 표도 정상 처리되는 파일이다.

## 2. 정정한 판단

폐기한 판단:

```text
Stage 16의 TAB extension 정합성만 맞추면 Stage 15의 통과 상태를 재현할 수 있다.
```

정정한 판단:

```text
Stage 15 통과 상태 = 첫 문단 PARA_TEXT contract + pi=22 인원 현황 표 LIST_HEADER contract
```

Stage 16은 TAB extension byte를 oracle과 같게 만들었지만, Stage 9에서 전역 적용한 셀
`LIST_HEADER width_ref bit 0` 때문에 `mel-001`의 8x12 인원 현황 표에서 한컴이 병합 셀 높이를
과도하게 계산했다.

## 3. 적용한 구현

수정 파일:

```text
src/document_core/converters/hwpx_to_hwp.rs
src/parser/hwpx/header.rs
```

변경 내용:

```text
1. raw_list_extra 13바이트 materialization은 유지한다.
2. width_ref bit 0은 모든 셀에 전역 적용하지 않는다.
3. table.col_count >= 30 인 고열 수 micro-grid 표에만 width_ref bit 0을 세운다.
4. 일반 데이터 표에서는 width_ref bit 0을 clear한다.
5. HWPX BorderFill의 slash/backSlash/diagonal을 HWP5 대각선 attr/payload로 materialize한다.
```

의도:

```text
tb-org-02 / 조직도 10x65 표:
  width_ref bit 0 + raw_list_extra 유지

mel-001 pi=22 인원 현황 8x12 표:
  raw_list_extra 유지
  width_ref bit 0 미설정
```

## 4. 생성 산출물

생성 폴더:

```text
output/poc/hwpx2hwp/task993/stage17_stage15_reproduction/
```

생성 파일:

| file | size | rhwp reload |
|---|---:|---|
| `output/poc/hwpx2hwp/task993/stage17_stage15_reproduction/mel-001.hwp` | 162,304 bytes | ok, pages=20 |
| `output/poc/hwpx2hwp/task993/stage17_stage15_reproduction/tb-org-02.hwp` | 7,680 bytes | ok, pages=1 |

판정표:

| file | 한컴 판정 유형 | 1페이지 고스트 선 | 1x1 표 배경 | 셀 대각선 | 인원 현황 표 높이 | 조직도 셀 줄나눔 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task993/stage17_stage15_reproduction/mel-001.hwp` | 성공 | 제거 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | Stage 15 기준 재현 성공 |
| `output/poc/hwpx2hwp/task993/stage17_stage15_reproduction/tb-org-02.hwp` | 성공 | - | - | 성공 | - | 성공 | 성공 | 성공 | 조직도 guard 성공 |

## 5. 내부 검증

실행:

```text
cargo check
cargo test cell_list_header_contract
```

결과:

```text
success
```

기존 warning은 출력되었지만 Stage 17 대상 테스트는 통과했다.

## 6. 계약 확인

### 셀 대각선 BORDER_FILL

작업지시자 판정 중 Stage 17 첫 산출물에서 셀 대각선이 누락되었다.
이는 의도한 결과가 아니다. Stage 11에서 확인한 대각선 계약을 실제 HWPX header 파서 경로에
완전히 반영하지 못한 누락이다.

원인:

```text
1. HWPX hh:diagonal/@type 값을 숫자로 파싱하고 있어 `SOLID`가 HWP5 line type 1로 매핑되지 않았다.
2. HWPX hh:backSlash 요소를 처리하지 않아 BORDER_FILL attr의 backSlash 방향 bit가 세팅되지 않았다.
3. `0.1 mm` 대각선 폭이 HWP5 diagonal width 1이 아니라 0으로 떨어졌다.
```

수정:

```text
1. hh:diagonal / hh:slash / hh:backSlash의 type을 HWP5 BorderLineType code로 매핑한다.
2. hh:slash / hh:backSlash 존재 시 BORDER_FILL attr의 diagonal direction bit를 materialize한다.
3. 대각선 width는 HWP5 payload에서 0이 되지 않도록 최소 1로 보존한다.
```

검증:

```text
output/poc/hwpx2hwp/task993/stage17_stage15_reproduction/diagonal_check/diagonal_borderfill_diff.md
```

결과:

```text
BORDER_FILL #4
oracle    attr=0x0040, slash=0, back=2, type=1, width=1, color=0x00000000
generated attr=0x0040, slash=0, back=2, type=1, width=1, color=0x00000000
```

### 첫 문단 PARA_TEXT

trace:

```text
output/poc/hwpx2hwp/task993/stage17_stage15_reproduction/first_para_trace/first_para_control_trace.md
```

결과:

```text
oracle PARA_TEXT hash    = 42497e5ee91926d7
generated PARA_TEXT hash = 42497e5ee91926d7
```

첫 문단 control marker stream은 oracle과 같은 순서로 구성된다.

```text
SectionDef -> ColumnDef -> PageNumPos -> PageHide -> TAB -> Header
```

### pi=22 인원 현황 표 LIST_HEADER

검증:

```text
output/poc/hwpx2hwp/task993/stage17_stage15_reproduction/pi22_probe_check/stage10_generation.md
```

결과:

```text
03_pi22_oracle_list_extra.hwp:
  list extra oracle = 0
```

즉 Stage 17 생성본은 `pi=22` 인원 현황 표의 LIST_HEADER width_ref/raw_list_extra 축에서
oracle graft가 추가로 필요하지 않은 상태다.

## 7. rhwp 페이지 배치

`mel-001.hwp`의 rhwp 기준 2페이지 배치:

```text
FullParagraph  pi=21  "󰊲  인원 현황(’25.11.30. 기준)"
Table          pi=22  8x12  630.4x146.1px
FullParagraph  pi=23  "  * 별정직, 전문경력관..."
FullParagraph  pi=24  "󰊳  예산 현황"
Table          pi=25  12x5
```

rhwp 기준으로는 인원 현황 표와 예산 현황 표가 같은 2페이지에 배치된다.

## 8. 작업지시자 판정

작업지시자 판정:

```text
테스트 통과
```

통과 조건:

```text
1. mel-001.hwp의 2페이지 인원 현황 표 높이가 Stage 15 02처럼 정상인지
2. 1페이지 고스트 선이 제거되었는지
3. tb-org-02.hwp의 조직도 셀 줄나눔이 2글자씩 유지되는지
```

## 9. 결론

Stage 17은 Stage 15의 통과 파일을 실제 adapter 구현으로 재현한 단계로 확정한다.

해결된 축:

```text
1. HWPX gradient BorderFill -> HWP5 BORDER_FILL payload
2. HWPX cell LIST_HEADER raw_list_extra materialization
3. micro-grid 표 한정 width_ref bit 0 materialization
4. 일반 데이터 표 width_ref bit 0 clear
5. 첫 문단 control marker/TAB stream 보존
6. HWPX slash/backSlash/diagonal -> HWP5 셀 대각선 contract
```

따라서 `mel-001.hwpx`의 파일손상/표 배치/고스트 선/대각선/배경 문제와 `tb-org-02.hwpx`의
조직도 셀 줄나눔 guard는 Stage 17 기준 통과로 처리한다.
