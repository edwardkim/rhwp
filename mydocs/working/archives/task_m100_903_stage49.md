# Task m100 #903 Stage 49 작업 기록

## 1. 목적

Stage48 판정으로 `BodyText/Section0` residual payload 가설은 기각되었다.

```text
PARA_TEXT
PARA_CHAR_SHAPE
PARA_LINE_SEG
TABLE
```

위 record payload를 모두 graft해도 셀 내부 텍스트 클리핑은 회복되지 않았다.

Stage49는 클리핑 원인을 DocInfo 참조 대상 쪽에서 찾는 probe다.

## 2. 기준 파일

positive:

```text
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/12_without_6596.hwp
```

baseline:

```text
output/poc/hwpx2hwp/task903/stage48_residual_text_layout_probe/08_plus_text_char_line_seg_table.hwp
```

baseline은 다음 상태다.

```text
한컴 열기 성공
이미지 출력 성공
큰 표/개체 배치 성공
일부 셀 텍스트 클리핑 유지
```

## 3. DocInfo diff 요약

생성된 diff:

```text
output/poc/hwpx2hwp/task903/stage49_docinfo_text_clip_probe/stage49_docinfo_ref_diff.md
```

DocInfo raw summary:

| item | positive | baseline |
|---|---:|---:|
| raw bytes | 26474 | 26474 |
| records | 523 | 523 |
| differing comparable records | 85 | 85 |

DocInfo record count는 모두 동일하다.

| tag | name | positive | baseline |
|---:|---|---:|---:|
| 16 | DOCUMENT_PROPERTIES | 1 | 1 |
| 17 | ID_MAPPINGS | 1 | 1 |
| 18 | BIN_DATA | 5 | 5 |
| 19 | FACE_NAME | 114 | 114 |
| 20 | BORDER_FILL | 82 | 82 |
| 21 | CHAR_SHAPE | 171 | 171 |
| 22 | TAB_DEF | 4 | 4 |
| 23 | NUMBERING | 2 | 2 |
| 25 | PARA_SHAPE | 85 | 85 |
| 26 | STYLE | 58 | 58 |

차이가 난 tag는 하나뿐이다.

| tag | name | differing records |
|---:|---|---:|
| 25 | PARA_SHAPE | 85 |

즉 Stage49 기준으로는 `FACE_NAME`, `CHAR_SHAPE`, `STYLE`, `BORDER_FILL`, `BIN_DATA` 등은
positive와 baseline 사이에 이미 동일하다. 잔여 클리핑과 직접 관련될 수 있는 DocInfo 차이는
`PARA_SHAPE`뿐이다.

## 4. BodyText 참조 ID 요약

positive와 baseline은 같은 문단 수와 거의 같은 참조 범위를 가진다.

| role | top paragraphs | cell paragraphs | style_ids |
|---|---:|---:|---|
| positive | 121 | 1461 | 0, 36, 37 |
| baseline | 121 | 1461 | 0, 36, 37 |

`para_shape_ids` 범위도 동일하다.

```text
0, 2, 3, 4, 9, 15, 16, 32,
39..61,
64..84
```

따라서 참조 ID 자체가 밀린 현상보다는, 해당 ID가 가리키는 `PARA_SHAPE` record 내용 차이가
우선 후보다.

## 5. 생성 파일

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage49_docinfo_text_clip_probe/
```

생성 파일:

```text
01_para_shapes_from_positive.hwp
02_char_shapes_from_positive.hwp
03_styles_from_positive.hwp
04_font_faces_from_positive.hwp
05_para_char_shapes_from_positive.hwp
06_para_char_styles_from_positive.hwp
07_docinfo_text_metrics_bundle.hwp
08_docinfo_all_non_bindata_from_positive.hwp
```

모든 파일은 `375808` bytes이고, rhwp 내부 reload는 모두 `pages=9`로 통과했다.

## 6. Probe 구성과 실제 해시 그룹

계획상 후보는 8개지만, 실제 DocInfo diff가 `PARA_SHAPE`에만 있었기 때문에 결과 파일은 두 해시 그룹으로 나뉜다.

### ParaShape 적용군

```text
01_para_shapes_from_positive.hwp
05_para_char_shapes_from_positive.hwp
06_para_char_styles_from_positive.hwp
07_docinfo_text_metrics_bundle.hwp
08_docinfo_all_non_bindata_from_positive.hwp
```

hash:

```text
704189795d001214a39b4ff065eb0ef1b99396978e6c475341c106adb3d97a86
```

### baseline 동일군

```text
02_char_shapes_from_positive.hwp
03_styles_from_positive.hwp
04_font_faces_from_positive.hwp
```

hash:

```text
c1e1a97c69fbee9ae90c4449cde441d9406a1995945408fbfb27750149bd5485
```

`CHAR_SHAPE`, `STYLE`, `FACE_NAME`은 positive와 baseline 사이에 차이가 없으므로,
해당 후보는 baseline과 동일한 파일이 된다.

## 7. 검증 명령

```bash
cargo test --test hwpx_to_hwp_adapter task903_stage49_generate_docinfo_text_clip_probe -- --nocapture
```

결과:

```text
pass
```

## 8. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

| variant | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| 01_para_shapes_from_positive | 성공 | 성공 | 성공 | 개선 | 성공 |  |
| 02_char_shapes_from_positive | 성공 | 성공 | 성공 | 클리핑 | 성공 | baseline 동일군 |
| 03_styles_from_positive | 성공 | 성공 | 성공 | 클리핑 | 성공 | baseline 동일군 |
| 04_font_faces_from_positive | 성공 | 성공 | 성공 | 클리핑 | 성공 | baseline 동일군 |
| 05_para_char_shapes_from_positive | 성공 | 성공 | 성공 | 개선 | 성공 | 01과 동일 hash |
| 06_para_char_styles_from_positive | 성공 | 성공 | 성공 | 개선 | 성공 | 01과 동일 hash |
| 07_docinfo_text_metrics_bundle | 성공 | 성공 | 성공 | 개선 | 성공 | 01과 동일 hash |
| 08_docinfo_all_non_bindata_from_positive | 성공 | 성공 | 성공 | 개선 | 성공 | 01과 동일 hash |

실질 판정 포인트는 다음 두 파일이다.

```text
01_para_shapes_from_positive.hwp
02_char_shapes_from_positive.hwp
```

`01`에서 클리핑이 사라지면 `PARA_SHAPE` record 차이가 직접 원인이다.
`01`에서도 클리핑이 유지되면, Stage49 기준의 DocInfo 차이도 직접 원인이 아니다.

## 9. 현재 해석

Stage49 생성 결과만 놓고 보면, 잔여 클리핑의 유일한 DocInfo 후보는 `PARA_SHAPE`다.

```text
유력 후보:
  PARA_SHAPE raw payload 85개

기각 후보:
  FACE_NAME
  CHAR_SHAPE
  STYLE
  BORDER_FILL
  BIN_DATA
  BodyText PARA_TEXT/PARA_CHAR_SHAPE/PARA_LINE_SEG/TABLE residual payload
```

따라서 다음 판단은 한컴 시각 판정에서 `01_para_shapes_from_positive.hwp`가
셀 텍스트 클리핑을 회복하는지 여부에 달려 있다.

## 10. 작업지시자 판정 반영

판정 결과 `PARA_SHAPE` 적용군은 모두 셀 텍스트 클리핑이 개선되었다.

```text
01, 05, 06, 07, 08:
  클리핑 개선

02, 03, 04:
  클리핑 유지
```

해시 그룹상 `05~08`은 `01`과 동일하므로, 실제 회복 원인은 `PARA_SHAPE` 단독이다.

최종 해석:

```text
확정:
  셀 텍스트 baseline/클리핑 문제의 직접 원인은 DocInfo PARA_SHAPE record 내용 차이다.

기각:
  BodyText residual PARA_TEXT/PARA_CHAR_SHAPE/PARA_LINE_SEG/TABLE
  DocInfo CHAR_SHAPE
  DocInfo STYLE
  DocInfo FACE_NAME
```

다음 단계는 `PARA_SHAPE` 85개 전체를 그대로 적용하는 방식에서 벗어나,
필드 단위 또는 인덱스 단위로 어느 값이 클리핑 개선에 필요한지 좁히는 것이다.
