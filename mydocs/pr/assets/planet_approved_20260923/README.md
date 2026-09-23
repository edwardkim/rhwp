# 승인분 통합 후보 Native Visual Sweep 증적

승인된 원 PR 9개를 분리한 후보와 #7365·#7358·#7359 메인터너 보정본을 `/Users/tsjang/rhwp/target/pr-review/release-test/rhwp`로 재산출했다.
모든 기준 PDF는 이미 저장소에 추적된 파일을 재사용했으며 새 PDF 변환을 하지 않았다.

| 원 PR | 입력 | 기준 PDF | 쪽 | review / overlay |
| --- | --- | --- | --- | --- |
| [#7357](../../archives/pr_7357_review.md) | `samples/task1725/text_footnote_tail_overpagination.hwp` | `pdf/text_footnote_tail_overpagination-2024.pdf` | rhwp 63 ↔ PDF 62; rhwp 64 ↔ PDF 63 | [review 63↔62](pr7357_native_review_rhwp063_pdf062.png) · [overlay 63↔62](pr7357_native_overlay_rhwp063_pdf062.png) · [review 64↔63](pr7357_native_review_rhwp064_pdf063.png) · [overlay 64↔63](pr7357_native_overlay_rhwp064_pdf063.png) |
| [#7358](../../archives/pr_7358_review.md) | `samples/86712_regulatory_analysis.hwp` | `pdf/86712_regulatory_analysis-hwp-2024.pdf` | 28 | [review](pr7358_native_review_028.png) · [overlay](pr7358_native_overlay_028.png) · [표 위치 보정 근거](../../archives/pr_7358_review.md#최신-native-visual-sweep-증적) |
| [#7359](../../archives/pr_7359_review.md) | `samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp` | `pdf/1480000-201900042-chemical-product-labeling-study-2020.pdf` | 14 | [review](pr7359_native_review_014.png) · [overlay](pr7359_native_overlay_014.png) · [표 위치 보정 근거](../../archives/pr_7359_review.md#통합-중-메인터너-보정--페이지-첫-문단의-저장-간격) |
| [#7365](../../archives/pr_7365_review.md) | `samples/task1768/distribution_doc.hwpx` | `pdf/distribution_doc-2024.pdf` | 3 | [pr7365_native_review_003.png](pr7365_native_review_003.png) · [pr7365_native_overlay_003.png](pr7365_native_overlay_003.png) |
| [#7370](../../archives/pr_7370_review.md) | `samples/issue7062/tac_object_host_line_height.hwp` | `pdf/tac_object_host_line_height-2020.pdf` | 10 | [pr7370_native_review_010.png](pr7370_native_review_010.png) · [pr7370_native_overlay_010.png](pr7370_native_overlay_010.png) |
| [#7373](../../archives/pr_7373_review.md) | `samples/task2070/1130000-201900011_D0150004-1-002_2017년기준 시장구조조사.hwp` | `pdf/task2070/1130000-201900011_D0150004-1-002_2017년기준 시장구조조사-2022.pdf` | rhwp 95 ↔ PDF 94 | [pr7373_native_review_rhwp095_pdf094.png](pr7373_native_review_rhwp095_pdf094.png) · [pr7373_native_overlay_rhwp095_pdf094.png](pr7373_native_overlay_rhwp095_pdf094.png) |
| [#7374](../../archives/pr_7374_review.md) | `samples/issues/2809/jubo_20260104.hwp` | `pdf/issue-2809-jubo_20260104-2020.pdf` | 1 | [pr7374_native_review_001.png](pr7374_native_review_001.png) · [pr7374_native_overlay_001.png](pr7374_native_overlay_001.png) |
| [#7378](../../archives/pr_7378_review.md) | `samples/issue6132/156482639_startup_ir_contest.hwp` | `pdf/issue6132/156482639_startup_ir_contest-2020.pdf` | 5–8 | [pr7378_native_review_005.png](pr7378_native_review_005.png) · [pr7378_native_overlay_005.png](pr7378_native_overlay_005.png) · [pr7378_native_review_006.png](pr7378_native_review_006.png) · [pr7378_native_overlay_006.png](pr7378_native_overlay_006.png) · [pr7378_native_review_007.png](pr7378_native_review_007.png) · [pr7378_native_overlay_007.png](pr7378_native_overlay_007.png) · [pr7378_native_review_008.png](pr7378_native_review_008.png) · [pr7378_native_overlay_008.png](pr7378_native_overlay_008.png) |

## 실행 기준

이 디렉터리의 이미지는 승인된 원 PR만 포함한다. 폐기된 통합 PR #7380의 head에서 만든 이미지는
새 후보의 최종 판정에 사용하지 않는다. 모든 대표 PNG는 #7358 표 여백, #7359 페이지 첫 문단,
#7365 TAC 표 보정이 함께 적용된 release-test binary SHA-256
`a220756502525135bd866292aed0163a297edd50c20cadfba7963ee926ea5133`으로 다시 산출했다.
#7359 보정 직전과 PNG SHA-256을 대조하면 #7359의 review·overlay 2장만 바뀌고 다른 22장은
byte-identical하다. 다른 대표 화면에 이번 페이지 첫 문단 보정의 시각 회귀가 없음을 확인했다.
Visual Sweep script SHA-256은 `49c53ca246df4b992cb126d5710503d00a6096ce1f766497c809aba06ece9c73`이다.
현 후보의 실행 원장은 로컬에 보존하며 binary·입력·PDF 해시를 확인했다. 경로는
`output/planet-approved-evidence/<원 PR>-afterfinal/pr<원 PR>/run_manifest.json`이다. 원장의 `git_head`는
메인터너 보정을 같은 commit에 묶기 전 마지막 cherry-pick head다. 이 값만으로 실행 코드의 정확한 버전을 주장하지 않으며,
위 binary·script 해시와 이 PR의 최종 commit에 포함된 보정 코드를 함께 대조한다.

```bash
cd /Users/tsjang/rhwp-planet-approved-clean-20260924
python3 scripts/visual_sweep.py \
  --file-target <pr-key> <입력 HWP/HWPX> <기존 PDF> \
  --rhwp-bin /Users/tsjang/rhwp/target/pr-review/release-test/rhwp \
  --pages <쪽> --dpi 96 --out output/planet-approved-evidence/<pr>-afterfinal
```

검증된 글꼴 디렉터리를 공급하는 별도 실행이라면 먼저 실제로 존재하고 입력 문서의 face를 제공하는지 확인한다. 존재하지 않는 과거
`ttfs/hwp:ttfs/windows` 경로를 넘기면 fallback face로 raster될 수 있다. #7359의
낮은 점수는 실측한 표·문단 위치 차이였으며, 저장 간격을 보정한 결과 예외 없이 통과했다.

#7357과 #7373은 원본 문서의 물리 쪽 색인이 각각 한 장 어긋나므로
`rhwp 63 ↔ 원 PDF 62`, `rhwp 64 ↔ 원 PDF 63`, `rhwp 95 ↔ 원 PDF 94`로 짝지었다.
`pypdf`로 원 PDF 첫 장을 임시 비교본 맨 앞에 한 번 더 넣어 1-based 쪽 인덱스를 맞췄다.
원본 PDF를 수정하거나 새 기준으로 등록하지 않았다. 대응쪽 캡처 제목의 `pdf p063/p095`는 이
임시 비교본 번호이며 원본 PDF에서는 각각 p062/p094다. 원본 PDF SHA-256은 #7357
`d8f47fc772556cc15b444fad61deca8b0f260eb82b3bf1691a36cc41036a69ba`, #7373
`89ff2583598e1bb8ab9d95b754fcc0cf42e3409547aec4dbe01ac0f4207b312e`다.

| 원 PR | 새 후보 2px 이웃 실루엣 최저값 | Visual Sweep gate |
| --- | ---: | --- |
| #7357 | 99.80% | 통과, 물리 쪽 대응 |
| #7358 | 94.35% | 통과, 표 마지막 조각의 상단 여백 보정 후 재산출 |
| #7359 | 97.48% | 통과, 페이지 첫 문단과 표 위치 보정 |
| #7365 | 93.56% | 통과, 메인터너 보정 |
| #7370 | 99.57% | 통과 |
| #7373 | 98.41% | 통과, 물리 쪽 대응 |
| #7374 | 98.31% | 통과 |
| #7378 | 99.36% | 통과, p5–p8 |

## 판독 한계

- `pixel_match`와 엄격 내용 픽셀은 글꼴 래스터·원래 조판 차이를 포함하는 보조값이다. review PNG를 수용 근거로 쓰는 경우에는 2px 이웃 관용 내용 실루엣 일치율이 90% 이상이어야 하며, 미만 쪽은 원인 수정과 새 PNG 재검토 전 보류한다. 실제 글꼴이 완전히 다른 경우만 양쪽 글꼴 정보와 확인 방법을 기록한 증거로 예외를 남길 수 있다. 이 gate를 통과해도 각 PR이 고친 행 경계·표 조각·줄 순서·label clipping의 직접 판독과 회귀 검사는 별도로 필요하다.
- PDF fixture 교체 PR #7374의 Visual Sweep은 현재 renderer와 교체 PDF의 실제 비교 경로를 확인한다. #7369는 기준 PDF 재산출만 수행한 PR이므로 Visual Sweep PNG를 산출하지 않으며 producer·글꼴·쪽수의 원본성은 fixture 검증으로 판정한다.
