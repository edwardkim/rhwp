# 이슈 #8661 시각 증거

폭 표가 없는 공급 TrueType(서울한강체·서울남산체)을 원본 글꼴 환경에서도 1em으로 재던 결함의
수정 전후 측정이다.

- base: `a7feb849e1`
- 측정 source: `75555763c4`. 이후 커밋은 주석·문서·CI 호출·증거만 바꾸고 `src/`는 바꾸지 않는다.
- 측정: `scripts/visual_sweep.py`, 96dpi, 판정값 `silhouette_raw_match_percent`,
  Native와 fresh WASM(기본 `--rasterizer png`).
- 파일 이름: `native_base`, `native_head`, `wasm_base`, `wasm_head`.

## 입력과 기준 PDF

| 항목 | 내용 |
| --- | --- |
| 원본 | `samples/issue8661/sign_board_register_card.hwp` (SHA-256 `dd3d1f1c2c36…312cb`, 전체 해시는 아래) |
| 기준 PDF | `pdf/issue8661/sign_board_register_card-print-2020.pdf` (Hwp 2020, Hancom PDF 1.3.0.550, 1쪽) |
| 공급 글꼴 | 서울한강체 M, 서울남산체 M. 서울특별시 배포본 `seoul_font3.zip` |

```text
dd3d1f1c2c3637d0663b90adab21b997d85ef2dd3131ca30ef02ba13772f04b0  sign_board_register_card.hwp
109c95a3ff3f1dca20c05339440d4e3350e3d692d88d3c228d097495feb8cda2  sign_board_register_card-print-2020.pdf
7ab485b98f5b1a1b05cfd04484dd49a62f856be8506223cd99e5ea1a33e400a7  seoul_font3.zip
d27e1b26b55e507bec1045962c954cf426d79605009c720fad1c9ef808e312cb  SeoulHangangM.ttf
b4bb3c3926431fc1cddc37f4e0869546afd1b113c9ed87eecdd53b3dcce1432d  SeoulNamsanM.ttf
```

글꼴 파일은 재배포 조건을 확인하지 않아 저장소에 넣지 않는다. 회귀 검사는 저장소의 라이선스 확인
fixture 글꼴을 쓴다.

## 서명판 문서

| 경로 | base | head |
| --- | --- | --- |
| Native | 90.30 | 100.00 |
| fresh WASM | 90.84 | 100.00 |

전체 1쪽, PDF와 쪽수 일치. `sign_board/`에 TSV, review·overlay PNG, 같은 배율 확대본
`zoom_table_header.png`, `zoom_card_rows.png`가 있다.

확대 판독: base는 표 머리의 `설치일자`, `의무설치 기한`이 글자 폭 1em으로 넓어져 잘못된 자리에서
줄을 나눴다. head는 Native와 WASM 모두 줄바꿈과 글자 전진이 PDF와 같다. Native 글자는 PDF보다 획이
가늘게 보인다. 이 차이는 수정 전후에 같고 이 변경의 범위가 아니다.

## #8662와 결합한 확인

`combined_8662/`: 이 source와 #8662 head `f84e0f3ffe`를 병합한 `38c7884e42`의 측정이다.

| 조건 | Native | fresh WASM |
| --- | --- | --- |
| 2020 PDF, 서울 글꼴 공급 | 100.00 | 100.00 |
| 2022 PDF, 서울 글꼴 미공급 | 99.97 | 100.00 |

#8661만 적용하면 2022 미공급 조건은 수정 전후 모두 37.70 / 36.82로 같다. 글꼴이 없으면 측정이
그대로이므로 이 조건은 이 변경의 대상이 아니고 #8662(#8639)가 해결한다.

## 무회귀 A/B

10k 말뭉치에서 base와 head의 출력을 비교했다. 이 전수 비교는 이전 base `08963f6825`와 이전 후보로
실행했고, SVG 1건과 PNG 14건이 바뀌었으며 쪽수 변화는 없었다. 그 뒤 후보는 PostScript 이름 일치를
측정에서 제외해 영향 범위가 줄었다. 전수 비교는 다시 실행하지 않았고, 바뀐 문서의 바뀐 쪽을 현재
base·head로 다시 측정했다(`ab/<key>/`). 호스트에 설치된 글꼴이 결과를 바꾸지 않도록 숨긴 환경이다.

| key | 말뭉치 문서 ID | 측정 쪽 수 | PDF/rhwp 전체 쪽수 | base·head 최저 Native | base·head 최저 WASM |
| --- | --- | --- | --- | --- | --- |
| d1 | 경찰청 156466145 | 1 | 2/2 | 97.06 → 97.06 | 98.87 → 98.87 |
| d2 | 경찰청 156476871 | 1 | 7/7 | 95.20 → 95.20 | 97.30 → 97.30 |
| d3 | 1270000-201800056 | 1 | 161/161 | 97.51 → 98.42 | 99.06 → 99.98 |
| d4 | 1351000-201400163 | 1 | 6/6 | 97.83 → 99.96 | 97.82 → 99.96 |
| d5 | 6470000-201500053 | 1 | 187/187 | 99.90 → 99.90 | 99.99 → 99.99 |
| i274 | 148760848 | 1 | 14/14 | 49.98 → 50.72 | 47.41 → 48.15 |
| i63 | 경찰청 156454300 | 1 | 2/2 | 54.66 → 54.66 | 54.06 → 54.06 |
| i543 | 1320000-201800025 | 6 | 14/14 | 49.64 → 49.64 | 51.41 → 51.41 |
| i654 | 1051000-202000124 | 32 | 209/209 | 28.38 → 28.39 | 27.31 → 27.31 |
| i530 | 3780000-202300009 | 36 | 54/54 | 86.14 → 86.10 | 86.15 → 86.12 |
| i875 | 1311000-200800002 | 2 | 208/209 | 9.48 → 9.39 | 9.24 → 9.17 |
| i866 | 6440000-202400074 | 25 | 128/129 | 12.20 → 12.20 | 12.63 → 12.63 |
| i717 | 1371000-202100069 | 190 | 246/248 | 7.40 → 7.40 | 7.22 → 7.22 |
| x356 | 3560000-202300004 | 11 | 185/185 | 67.52 → 67.52 | 69.01 → 69.01 |

- 90% 미만 쪽은 i717 137, i866 10, i654 5, x356 3, i543 2, i875 2, i274 1, i63 1, i530 1쪽이다.
  **모두 base에서도 90% 미만이며, head에서 새로 90% 아래로 내려간 쪽은 없다.**
- i875, i866, i717의 쪽수 불일치는 base와 head가 같다.
- x356은 호스트 글꼴을 숨기면 PNG가 base와 같다. 변화는 환경 차이였다.
- 점수가 내려간 쪽은 15쪽이다. Native·WASM 한쪽이라도 0.1 넘게 내려간 쪽은 다음 세 곳이다.

| 문서·쪽 | Native base → head | WASM base → head | 판독 |
| --- | --- | --- | --- |
| i543 2쪽 | 82.32 → 82.38 | 86.05 → 85.58 | 바뀐 글자 폭이 한컴 PDF와 일치 |
| i654 132쪽 | 98.57 → 98.44 | 98.58 → 98.37 | 바뀐 글자 폭이 한컴 PDF와 일치 |
| i875 190쪽 | 9.48 → 9.39 | 9.24 → 9.17 | 쪽 어긋남이 있는 기존 불일치 문서, 판독 미실시 |

i530 40쪽(86.14 → 86.10, 86.15 → 86.12)은 바뀐 제목 줄 `7) 삭감계획 이행평가`가 head에서 PDF에 더
가깝다. i543과 i654의 판독은 PostScript 이름 제외 전 후보에서 했고, 두 쪽의 점수는 현재 head로
다시 쟀다.

### d5 71쪽의 회귀와 수정

중간 후보에서 d5 71쪽이 내려갔다. 한컴은 `HCRBatang` 슬롯을 Haansoft Batang으로 대체했는데, 후보가
PostScript 이름만 같은 HANBatang의 공백 폭을 썼기 때문이다. HWP5 측정을 family 레코드 일치로 제한해
base와 같은 값으로 돌아왔다. `hwp5_postscript_name_does_not_select_the_truetype_program`이 이를
검사한다. 수정을 끄면 실패하고 켜면 통과한다.

### 기준 PDF 예외

- d5: engine 2020 워커가 비정상 종료(exit 3221225477)해 engine 2024로 출력했다. 2020 기준이 없어 쓴
  예외이며 187쪽이 일치한다. 이 PDF는 저장소에 넣지 않았다.
  SHA-256 `c44d00fb3f4305c57ae3760882a8c9d7206ac5ba6e054f9958f6b63a9d2e0179`.
- d6(1073100-202100006, 827쪽): 원본의 OLE 저장소가 112바이트로 비어 engine 2020·2024 모두 거부해
  기준 PDF가 없다. 그래서 d6는 Visual Sweep 점수가 없다. 이전 후보 대비 바뀐 8쪽
  (127, 129, 131, 135, 137, 140, 674, 675)은 현재 base·head PNG가 픽셀까지 같다.

## 기존 검사의 기대값 변경

`unassigned_hwp5_font_bits_do_not_create_truetype_requests`는 폭 표 없는 HWP5 face의 TrueType 요청이
없다고 고정했다. 합성 fixture의 기대값이며 독립 출력 근거가 없었고, 서명판의 한컴 PDF는 그 요청이
필요함을 보인다. 이름을 `truetype_requests_follow_the_declared_face_type_across_sources`로 바꾸고 해당
단언을 뒤집었다. 폭 표가 있는 face의 보호는 `hwp5_face_with_a_metric_table_stays_out_of_the_truetype_request`가
맡는다.

## 회귀 검사

최소 글꼴 환경(저장소 `ttfs/opensource` 글꼴 5개만 노출)의 `75555763c4` 결과다.

| 실행 | 결과 |
| --- | --- |
| 기본 feature | 12766건 전부 통과, 45건 skip |
| `--features native-skia` | 13017건 중 13013건 통과, 4건 실패, 45건 skip |

실패한 4건은 같은 환경의 base `a7feb849e1`에서 같은 위치·같은 메시지로 실패한다. 이 변경의 결과가
아니며, 네 검사 모두 CI 호출 목록에 없다. 같은 네 검사는 열린 PR #8623이 다룬다.

- `native_skia_public_paint_contract::renders_complete_svg_image_larger_than_a_raw_fragment`
- `native_skia_public_paint_contract::text_glyph_fit_uses_layout_advance_without_distribution_spacing`
- `issue_7445_native_bold_tracking::native_bold_tracking_does_not_stretch_the_actual_glyph`
- `issue_7445_negative_spacing_glyph_shape::negative_distribution_keeps_the_native_glyph_shape`

`issue_8661_supplied_ttf_without_metric`은 `native-skia` 전용이라 기존 CI 경로에서 실행되지 않았다.
Native Skia job의 두 프로필에 호출을 하나씩 추가했고, 같은 명령이 로컬에서 7건을 실행해 모두 통과한다.
