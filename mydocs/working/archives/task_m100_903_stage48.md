# Task m100 #903 Stage 48 작업 기록

## 1. 목적

Stage47 판정으로 다음 두 축은 분리되었다.

```text
이미지 출력:
  DocInfo BIN_DATA metadata가 직접 원인

큰 표/개체 배치:
  CTRL_HEADER payload가 직접 전제
```

Stage47 `06_stage46_06_plus_ctrl_list_para_headers.hwp`는 한컴에서 열리고,
이미지와 큰 표 배치는 회복되었지만 일부 셀 내부 텍스트 배치가 아직 틀렸다.

Stage48은 이 잔여 문제를 `PARA_TEXT`, `PARA_CHAR_SHAPE`, `PARA_LINE_SEG`,
`TABLE` residual payload 중 어느 축이 해결하는지 확인하기 위한 probe다.

## 2. 기준 파일

baseline:

```text
output/poc/hwpx2hwp/task903/stage47_header_prereq_probe/06_stage46_06_plus_ctrl_list_para_headers.hwp
```

success source:

```text
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/12_without_6596.hwp
```

baseline은 Stage47에서 다음 상태였다.

```text
한컴 열기 성공
이미지 출력 성공
큰 표 배치 회복
일부 셀 텍스트 배치 틀림
```

success source는 Stage40에서 정상 기준으로 사용한 파일이다.

## 3. Residual diff 요약

생성된 diff:

```text
output/poc/hwpx2hwp/task903/stage48_residual_text_layout_probe/stage48_residual_diff.md
```

요약:

| item | success | baseline |
|---|---:|---:|
| section0 bytes | 225288 | 225712 |
| record count | 7879 | 7879 |
| differing comparable records | 186 | 186 |

tag별 차이:

| tag | name | count |
|---:|---|---:|
| 67 | PARA_TEXT | 55 |
| 68 | PARA_CHAR_SHAPE | 77 |
| 69 | PARA_LINE_SEG | 42 |
| 77 | TABLE | 12 |

이 단계에서는 `CTRL_HEADER`, `LIST_HEADER`, `PARA_HEADER`, `DocInfo BIN_DATA` 계열은
다시 보지 않는다. 이미 앞 stage에서 직접 원인 또는 비원인으로 분리된 축이다.

## 4. 생성 파일

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage48_residual_text_layout_probe/
```

생성 파일:

```text
01_plus_para_text.hwp
02_plus_para_char_shape.hwp
03_plus_para_line_seg.hwp
04_plus_table_all.hwp
05_plus_text_char_shape.hwp
06_plus_text_char_line_seg.hwp
07_plus_line_seg_table.hwp
08_plus_text_char_line_seg_table.hwp
```

모든 HWP 파일 크기는 `375808` bytes이며, rhwp 내부 reload는 모두 `pages=9`로 통과했다.

보조 산출물:

```text
residual_text_layout_detail.md
stage48_residual_diff.md
```

## 5. Probe 구성

| variant | graft payload |
|---|---|
| 01_plus_para_text | PARA_TEXT |
| 02_plus_para_char_shape | PARA_CHAR_SHAPE |
| 03_plus_para_line_seg | PARA_LINE_SEG |
| 04_plus_table_all | TABLE |
| 05_plus_text_char_shape | PARA_TEXT + PARA_CHAR_SHAPE |
| 06_plus_text_char_line_seg | PARA_TEXT + PARA_CHAR_SHAPE + PARA_LINE_SEG |
| 07_plus_line_seg_table | PARA_LINE_SEG + TABLE |
| 08_plus_text_char_line_seg_table | PARA_TEXT + PARA_CHAR_SHAPE + PARA_LINE_SEG + TABLE |

## 6. 검증 명령

```bash
cargo test --test hwpx_to_hwp_adapter task903_stage48_generate_residual_text_layout_probe -- --nocapture
```

결과:

```text
pass
```

## 7. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

| variant | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| 01_plus_para_text | 성공 | 성공 | 일부 셀 텍스트 배치 틀림 | 성공 |  |
| 02_plus_para_char_shape | 성공 | 성공 | 일부 셀 텍스트 배치 틀림 | 성공 |  |
| 03_plus_para_line_seg | 성공 | 성공 | 일부 셀 텍스트 배치 틀림 | 성공 |  |
| 04_plus_table_all | 성공 | 성공 | 일부 셀 텍스트 배치 틀림 | 성공 |  |
| 05_plus_text_char_shape | 성공 | 성공 | 일부 셀 텍스트 배치 틀림 | 성공 |  |
| 06_plus_text_char_line_seg | 성공 | 성공 | 일부 셀 텍스트 배치 틀림 | 성공 |  |
| 07_plus_line_seg_table | 성공 | 성공 | 일부 셀 텍스트 배치 틀림 | 성공 |  |
| 08_plus_text_char_line_seg_table | 성공 | 성공 | 일부 셀 텍스트 배치 틀림 | 성공 |  |

판정 포인트:

```text
1. 한컴에서 계속 성공으로 열리는지
2. 이미지 출력이 유지되는지
3. 큰 표 배치가 Stage47 06 수준 이상인지
4. 일부 셀 텍스트 배치 오류가 사라지는 variant가 있는지
```

## 8. 판정 해석 기준

```text
03에서 셀 텍스트 배치가 회복:
  PARA_LINE_SEG payload가 직접 원인이다.

02 또는 05에서 회복:
  PARA_CHAR_SHAPE run payload가 텍스트 배치나 폭 계산에 영향을 준다.

04 또는 07에서 회복:
  TABLE residual payload가 셀 내부 텍스트 배치에도 필요하다.

08만 정상:
  PARA_TEXT, PARA_CHAR_SHAPE, PARA_LINE_SEG, TABLE payload가 조합으로 필요하다.

모두 실패:
  Stage40 success와 Stage48 08의 diff를 다시 계산해 남은 record를 확인한다.
```

## 9. 현재 결론

Stage48 생성 기준으로는 residual 후보가 다음 네 tag로 좁혀졌다.

```text
PARA_TEXT
PARA_CHAR_SHAPE
PARA_LINE_SEG
TABLE
```

이제 한컴 시각 판정으로 잔여 셀 텍스트 배치 문제의 직접 payload를 확정한다.

## 10. 작업지시자 판정 반영

모든 variant가 다음 공통 상태를 보였다.

```text
한컴 열기 성공
이미지 출력 성공
rhwp-studio 성공
일부 셀 텍스트 배치 틀림 유지
```

사용자 관찰:

```text
일부 셀 텍스트 배치가 틀린 것은 셀내 텍스트가 위쪽으로 baseline이 잡혀
셀 위쪽 영역 때문에 클리핑되는 현상이다.
```

Stage48 파일 해시는 서로 달랐으므로 variant 생성 자체는 서로 다른 payload를 가진다.
하지만 `PARA_TEXT`, `PARA_CHAR_SHAPE`, `PARA_LINE_SEG`, `TABLE` residual payload를
단독 또는 조합으로 graft해도 클리핑은 회복되지 않았다.

따라서 Stage48의 해석은 다음이다.

```text
기각:
  잔여 셀 텍스트 클리핑의 직접 원인이 Section0의
  PARA_TEXT/PARA_CHAR_SHAPE/PARA_LINE_SEG/TABLE record payload 차이라는 가설

유지:
  이미지 출력 원인 = DocInfo BIN_DATA metadata
  큰 표/개체 배치 원인 = CTRL_HEADER payload

새 가설:
  셀 텍스트 클리핑은 Section0 record payload가 아니라,
  DocInfo 쪽의 참조 대상 값 또는 HWPX -> IR 파서의 model field 누락일 가능성이 높다.
  우선순위 후보는 ParaShape/CharShape/Style/FontFace 또는 cell 내부 paragraph 참조값이다.
```

다음 stage는 새 HWP record를 더 graft하기보다, 클리핑이 발생한 셀의 참조 ID를 추적해서
정답 HWP와 생성 HWP의 DocInfo 참조 대상 값을 비교해야 한다.
