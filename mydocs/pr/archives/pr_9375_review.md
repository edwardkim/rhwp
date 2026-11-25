# PR #9375 검토 — 이월 표와 각주 구분선

## 최종 판정

**식량안보 141쪽의 점수를 유지했다.** 핵심 이월 표는 #9388에서 반영됐다. 이번 통합 후보는 각주 구분선 띠를 추가로 변경하며, Native·fresh WASM 점수는 이전 통합 후보와 각 쪽에서 같다.

[원 PR #9375](https://github.com/edwardkim/rhwp/pull/9375) · [이슈 #8997](https://github.com/edwardkim/rhwp/issues/8997) · [선행 통합 PR #9388](https://github.com/edwardkim/rhwp/pull/9388)

공백 호스트를 앞쪽에 유지하고 이월 표 뒤의 빈 재시작 줄 및 앞 간격을 보존하는 코드는 `a573163584`를 통해 이미 devel에 있다. 후보 트리에서 #9375가 새로 바꾸는 제품 파일은 `src/renderer/pagination.rs`와 `src/renderer/layout/picture_footnote.rs`다. 두 경로가 구분선 띠에 `min(선 굵기, 0.25mm)`를 사용하며 실제 획은 원래 굵기로 그린다.

한컴 2020 Print 141쪽으로 재측정한 최신 devel의 식량안보 90~93쪽은 Native 100.00·100.00·99.86·100.00%, fresh WASM 100.00·100.00·99.84·100.00%다. 원 PR이 기존 미달로 열거한 4·6·68·85쪽의 최신 devel Native도 각각 100.00·100.00·99.97·100.00%다. 따라서 구분선 띠의 추가 효과는 이미 보정된 최신 devel과 비교해야 한다. 식량안보는 기존 한컴 2020 Print를 사용하고, 제주는 원 PR의 2020 Print와 이전 통합에서 차트 누락을 해결한 2024 Print를 구분한다.

## 현재 devel과 비교

식량안보의 Native·fresh WASM 141쪽에서 2px raw 표시값이 양쪽 모두 동일하다. 전쪽 최저는 각각 98.25%와 98.18%다. 각주 21 문장의 상단 y는 Native 렌더 트리에서 양쪽 861.1px이다. Native PNG 141쪽 중 23쪽의 바이트 해시가 달라, 점수 동일을 픽셀 동일이나 모든 문서에서의 무영향으로 해석하지 않는다.

- 92쪽 Native [review](../assets/planet6897_20261125/food/native/review_092.png) · [overlay](../assets/planet6897_20261125/food/native/overlay_092.png)
- 92쪽 fresh WASM [review](../assets/planet6897_20261125/food/wasm/review_092.png) · [overlay](../assets/planet6897_20261125/food/wasm/overlay_092.png)
- [각주 문장 위치](../assets/planet6897_20261125/claim-native-structural-evidence.json)

기존 비교는 `0fbee2c04e`와 통합 후보 `53b15ed46f`의 결과다. #9298 반영 뒤 다시 잰 식량안보 보고서 Native 141쪽의 rhwp PNG·PDF PNG·렌더 트리는 이전 후보와 각각 바이트 단위로 같았다. 전쪽 점수와 92쪽의 직접 판독 이미지를 재사용했다. [Native 재측정 기록](../assets/planet6897_20261125/native-rebase-verification.json)

[원 PR·통합 SHA 및 빌드 출처](../assets/planet6897_20261125/review-provenance.json) · [입력·기준 PDF 출처](../assets/planet6897_20261125/print-provenance.json)

## Merge 후 contributor PR comment 계획

[Visual Sweep 게시 기준](../../manual/verification/visual_sweep_guide.md#github-merge-comment)

아래 5쪽을 대조했다. 90% 하한에 걸린 쪽은 0쪽이다.
raw는 2px 이웃 실루엣 판정값이며, 픽셀·내용 픽셀 일치율은 자동 보조값이다.
페이지별 상세 근거의 배치·획·그림 판독을 함께 사용한다.

| 문서·기준·출력 | 쪽 | raw | 픽셀 일치 | 내용 픽셀 일치 | 증적 |
| --- | ---: | ---: | ---: | ---: | --- |
| food · Hancom PDF 2020 · Native | 90 | 100.00% | 96.84319% | 64.56498% | [review](../assets/planet6897_20261125/final-claim-pages/food/review_090.png) · [overlay](../assets/planet6897_20261125/final-claim-pages/food/overlay_090.png) |
| food · Hancom PDF 2020 · Native | 91 | 100.00% | 96.65955% | 55.15183% | [review](../assets/planet6897_20261125/final-claim-pages/food/review_091.png) · [overlay](../assets/planet6897_20261125/final-claim-pages/food/overlay_091.png) |
| food · Hancom PDF 2020 · Native | 92 | 99.86% | 94.78598% | 43.88199% | [review](../assets/planet6897_20261125/final-claim-pages/food/review_092.png) · [overlay](../assets/planet6897_20261125/final-claim-pages/food/overlay_092.png) |
| food · Hancom PDF 2020 · Native | 93 | 100.00% | 99.07371% | 37.01725% | [review](../assets/planet6897_20261125/final-claim-pages/food/review_093.png) · [overlay](../assets/planet6897_20261125/final-claim-pages/food/overlay_093.png) |
| jeju-2024 · Hancom PDF 2024 · Native | 48 | 100.00% | 96.98788% | 70.87724% | [review](../assets/planet6897_20261125/final-claim-pages/jeju-2024/review_048.png) · [overlay](../assets/planet6897_20261125/final-claim-pages/jeju-2024/overlay_048.png) |

병합 SHA를 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/`에 넣고
위 링크의 저장소 경로를 이어 review와 overlay를 이미지로 표시한다. devel 반영, raw PNG 형식·해시를
확인한 뒤 `--body-file`로 게시하고 GitHub API에서 본문과 이미지 URL을 대조한다.
