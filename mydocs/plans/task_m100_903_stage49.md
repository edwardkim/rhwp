# Task m100 #903 Stage 49 계획

## 1. 목적

Stage48 판정으로 다음 가설은 기각되었다.

```text
잔여 셀 텍스트 클리핑의 직접 원인이
BodyText/Section0의 PARA_TEXT, PARA_CHAR_SHAPE, PARA_LINE_SEG, TABLE record payload 차이다.
```

Stage48의 모든 variant는 서로 다른 payload를 가졌지만, 결과는 모두 같았다.

```text
한컴 열기 성공
이미지 출력 성공
큰 표/개체 배치 성공
일부 셀 텍스트가 셀 위쪽에서 클리핑됨
```

따라서 Stage49의 목적은 record payload graft가 아니라,
셀 안 문단이 참조하는 DocInfo 쪽 값을 비교하고, 클리핑의 직접 원인이 되는 참조 테이블을 좁히는 것이다.

## 2. 현재 확정 사항

```text
이미지 출력:
  DocInfo BIN_DATA metadata

큰 표/개체 배치:
  CTRL_HEADER payload

마지막 페이지 출력:
  DocProperties.section_count

기본 표/셀 배치:
  HWPX paraPr/margin -> ParaShape 매핑

잔여 결함:
  일부 셀 텍스트 baseline이 셀 위쪽으로 잡혀 클리핑됨
```

## 3. 기준 파일

positive:

```text
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/12_without_6596.hwp
```

baseline:

```text
output/poc/hwpx2hwp/task903/stage48_residual_text_layout_probe/08_plus_text_char_line_seg_table.hwp
```

baseline은 Stage48에서 가능한 Section0 residual payload를 모두 적용한 파일이다.
그런데도 클리핑이 유지되었으므로, Stage49는 DocInfo 참조 대상 차이를 본다.

## 4. 작업 범위

### 4.1 비교 리포트 생성

출력:

```text
output/poc/hwpx2hwp/task903/stage49_docinfo_text_clip_probe/stage49_docinfo_ref_diff.md
```

비교 대상:

```text
DocProperties
FaceName
BorderFill
CharShape
ParaShape
Style
TabDef
Numbering
Bullet
```

리포트에는 다음을 포함한다.

```text
1. positive와 baseline의 DocInfo record count/size/hash 차이
2. tag별 차이 요약
3. ParaShape/CharShape/Style의 record index별 차이
4. BodyText에서 참조하는 para_shape_id/char_shape_id/style_id 범위
5. 클리핑 후보와 연관 가능한 참조 ID 우선순위
```

### 4.2 HWP 후보 생성

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage49_docinfo_text_clip_probe/
```

후보:

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

각 후보는 Stage48 `08` baseline을 기준으로 positive의 DocInfo 일부만 graft한다.

## 5. 후보 의미

```text
01_para_shapes_from_positive:
  line spacing, paragraph margin, text alignment 쪽 확인

02_char_shapes_from_positive:
  font size, baseline offset, 장평/자간, 글자 위치 계산 확인

03_styles_from_positive:
  paragraph/char shape 참조를 style이 간접 보정하는지 확인

04_font_faces_from_positive:
  font face 매핑과 글꼴 metric 차이 확인

05_para_char_shapes_from_positive:
  ParaShape + CharShape 조합 확인

06_para_char_styles_from_positive:
  ParaShape + CharShape + Style 조합 확인

07_docinfo_text_metrics_bundle:
  FaceName + CharShape + ParaShape + Style 묶음 확인

08_docinfo_all_non_bindata_from_positive:
  BIN_DATA를 제외한 DocInfo 전체가 원인인지 확인
```

## 6. 하지 않을 것

```text
- Section0 PARA_TEXT/PARA_CHAR_SHAPE/PARA_LINE_SEG/TABLE 재검증
- DocInfo BIN_DATA 재검증
- CTRL_HEADER 재검증
- production 코드 구현
```

Stage49는 잔여 클리핑 원인을 DocInfo 참조 테이블 단위로 좁히는 stage다.

## 7. 검증 명령

```bash
cargo test --test hwpx_to_hwp_adapter task903_stage49_generate_docinfo_text_clip_probe -- --nocapture
```

## 8. 작업지시자 판정 요청

| variant | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| 01_para_shapes_from_positive |  |  |  |  |  |  |
| 02_char_shapes_from_positive |  |  |  |  |  |  |
| 03_styles_from_positive |  |  |  |  |  |  |
| 04_font_faces_from_positive |  |  |  |  |  |  |
| 05_para_char_shapes_from_positive |  |  |  |  |  |  |
| 06_para_char_styles_from_positive |  |  |  |  |  |  |
| 07_docinfo_text_metrics_bundle |  |  |  |  |  |  |
| 08_docinfo_all_non_bindata_from_positive |  |  |  |  |  |  |

## 9. 판정 해석

```text
01에서 회복:
  ParaShape record의 model/raw 값 중 누락 필드가 직접 원인이다.

02에서 회복:
  CharShape record의 font metric 또는 baseline 관련 값이 직접 원인이다.

03에서 회복:
  Style record의 para/char shape 참조 또는 style 속성이 직접 원인이다.

04에서 회복:
  FontFace mapping 또는 글꼴 metric 차이가 직접 원인이다.

05/06/07에서만 회복:
  단일 DocInfo table이 아니라 조합 문제다.

08에서만 회복:
  아직 분리하지 못한 DocInfo record가 원인이다.

모두 실패:
  클리핑 원인은 DocInfo가 아니라,
  BodyText record의 참조 ID 자체 또는 HWPX -> IR 단계의 cell paragraph 구성 문제로 본다.
```

## 10. 성공 기준

```text
1. 셀 텍스트 클리핑을 회복시키는 DocInfo 후보를 찾는다.
2. 회복 후보가 없으면 DocInfo 가설을 기각하고 참조 ID/IR 구성 문제로 전환한다.
3. 다음 production 구현 범위를 DocInfo parser/adapter 또는 BodyText 참조 매핑으로 확정한다.
```
