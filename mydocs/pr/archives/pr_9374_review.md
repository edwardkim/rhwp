# PR #9374 검토 — 가운데 정렬 문단 끝의 공백 폭

## 최종 판정

**가운데 정렬 표 원점 보정을 재현했다.** 같은 한컴 2024 Print에서 Native·fresh WASM 모두 84.36% → 100.00%로 개선됐다. 최종 후보의 두 출력도 같은 100.00%다.

[원 PR #9374](https://github.com/edwardkim/rhwp/pull/9374) · [이슈 #9373](https://github.com/edwardkim/rhwp/issues/9373)

저장 줄이 없는 가운데 정렬 문단에서 표 뒤 연속 공백이 표 원점을 왼쪽으로 당긴다. 변경은 마지막 상자의 공백 폭을 정렬 계산에서만 차감한다. 밑줄 공백, 내용이 없는 줄, 가운데 정렬이 아닌 행은 적용 조건에서 빠진다. 양쪽 여유를 비교하는 표 제약과 밑줄 반례가 테스트에 있다. 저장·편집 위치를 구성하는 상자의 전진량은 유지한다.

첨부 Print의 `Creator`는 `Hwp 2022 12.0.0.4547`이다. 동일 입력으로 MCP 2024를 실행해 데스크톱 한컴 2024 `13.0.0.4225`, `Hancom PDF 1.3.0.550`, PDF 1.4, A4 1쪽을 확보했다. 영수증상 `hancom2024_pdf_driver_one_up`이고 SHA-256은 `e0d117aa663eb8001344f8e3ca5e0d6d2ac3c833f889a06cae9fb2a4847d50c9`다. 원 PR은 2022 Print에서 Native 99.26%, WASM 100.00%를 보고했다. 공식 2024 Print 비교와는 기준 PDF가 다르다.

## 한컴 2024 Print 대조

| 출력 | 쪽수 | devel | 후보 | 1쪽 증적 |
| --- | ---: | ---: | ---: | --- |
| Native | 1 | 84.36% | 100.00% | [review](../assets/planet6897_20261125/allowance/native/review_001.png) · [overlay](../assets/planet6897_20261125/allowance/native/overlay_001.png) |
| fresh WASM | 1 | 84.36% | 100.00% | [review](../assets/planet6897_20261125/allowance/wasm/review_001.png) · [overlay](../assets/planet6897_20261125/allowance/wasm/overlay_001.png) |

표 좌우 경계와 가운데 칸 경계, 아래 비고의 순서를 확인했다. 100.00%는 2px 실루엣 점수다. 표 외곽선의 획과 제목·날짜의 글자 모양은 overlay에 미세한 차이로 남는다. [기준 PDF](../../../pdf/issue497/18561241_allowance_table-2024.pdf)

기존 비교는 `0fbee2c04e`와 통합 후보 `53b15ed46f`의 결과다. #9298이 반영된 `fb98e8bff3` 위로 리베이스하고 같은 PDF로 재측정한 Native 1쪽은 100.00%였다. 이전 후보와 rhwp PNG·PDF PNG·렌더 트리가 각각 바이트 단위로 같아 직접 판독 이미지를 재사용했다. [Native 재측정 기록](../assets/planet6897_20261125/native-rebase-verification.json)

[원 PR·통합 SHA 및 빌드 출처](../assets/planet6897_20261125/review-provenance.json) · [입력·기준 PDF 출처](../assets/planet6897_20261125/print-provenance.json)

## Merge 후 contributor PR comment 계획

[Visual Sweep 게시 기준](../../manual/verification/visual_sweep_guide.md#github-merge-comment)

아래 1쪽을 대조했다. 90% 하한에 걸린 쪽은 0쪽이다.
raw는 2px 이웃 실루엣 판정값이며, 픽셀·내용 픽셀 일치율은 자동 보조값이다.
페이지별 상세 근거의 배치·획·그림 판독을 함께 사용한다.

| 문서·기준·출력 | 쪽 | raw | 픽셀 일치 | 내용 픽셀 일치 | 증적 |
| --- | ---: | ---: | ---: | ---: | --- |
| allowance · Hancom PDF 2024 · Native | 1 | 100.00% | 99.20134% | 47.20243% | [review](../assets/planet6897_20261125/final-claim-pages/allowance/review_001.png) · [overlay](../assets/planet6897_20261125/final-claim-pages/allowance/overlay_001.png) |

병합 SHA를 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/`에 넣고
위 링크의 저장소 경로를 이어 review와 overlay를 이미지로 표시한다. devel 반영, raw PNG 형식·해시를
확인한 뒤 `--body-file`로 게시하고 GitHub API에서 본문과 이미지 URL을 대조한다.
