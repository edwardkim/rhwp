# 셀 줄간격 일괄 적용과 내어쓰기

[이슈 #7407](https://github.com/edwardkim/rhwp/issues/7407)의 재현 입력은 [원본 HWP](../../../../samples/issue6639/rhwp-table-cell-minimal-repro.hwp)와 [한컴이 재저장한 정상 HWPX](../../../../samples/issue6639/issue6639-hancom-160.hwpx)다. 셀 31의 문단 10개에 줄간격 140%를 일괄 적용하면 문단 5의 긴 줄은 ‘가’ 44자를 포함하고 마지막 한 자가 다음 줄에 놓인다. 현재 기준의 편집 경로는 저장 줄에 남은 내어쓰기 플래그를 따라 45자를 한 줄에 두어 마지막 ‘가’의 줄이 사라졌다.

[140% 한컴 2020 인쇄본](../../../../pdf/issue6639/issue6639-reference-140-2020.pdf)은 줄간격만 바꾸고 해당 저장 줄을 제거한 HWPX를 한컴에서 다시 조판해 인쇄한 독립 기준이다. [원본 160% 인쇄본](../../../../pdf/issue6639/issue6639-original-160-2020.pdf)과 [캐시를 제거한 160% 인쇄본](../../../../pdf/issue6639/issue6639-reference-160-2020.pdf)은 PDFium 96 DPI에서 같은 이미지였다. 유효한 저장 줄의 소속은 정상 HWPX도 제공한다. [자료의 출처와 속성 변경 기록](../../../../samples/issue6639/README.md)

수정본의 각 입력은 편집 전 160%, 140% 일괄 적용, HWP와 HWPX 각각 두 차례 저장·재열기, 원래 문단 모양 및 스냅샷으로 160% 복원을 확인했다. 아래 모든 상태는 한 쪽이며, 색상 경계 보정 전 2px 실루엣 일치율은 100.00%다. 확대 비교에서 문단 5의 줄 소속, 앞뒤 문단 순서와 표 경계를 확인했다.

| 입력 | Native TSV | fresh WASM TSV |
| --- | --- | --- |
| 원본 HWP | [160%](native/original/160-initial.tsv) · [140%](native/original/140-batch.tsv) · [HWP 재열기](native/original/140-hwp-reopen-1.tsv) · [HWP 2회차 재열기](native/original/140-hwp-reopen-2.tsv) · [HWPX 재열기](native/original/140-hwpx-reopen-1.tsv) · [HWPX 2회차 재열기](native/original/140-hwpx-reopen-2.tsv) | [160%](wasm/original/160-initial.tsv) · [140%](wasm/original/140-batch.tsv) · [HWP 재열기](wasm/original/140-hwp-reopen-1.tsv) · [HWP 2회차 재열기](wasm/original/140-hwp-reopen-2.tsv) · [HWPX 재열기](wasm/original/140-hwpx-reopen-1.tsv) · [HWPX 2회차 재열기](wasm/original/140-hwpx-reopen-2.tsv) |
| 정상 HWPX | [160%](native/control/160-initial.tsv) · [140%](native/control/140-batch.tsv) · [HWP 재열기](native/control/140-hwp-reopen-1.tsv) · [HWP 2회차 재열기](native/control/140-hwp-reopen-2.tsv) · [HWPX 재열기](native/control/140-hwpx-reopen-1.tsv) · [HWPX 2회차 재열기](native/control/140-hwpx-reopen-2.tsv) | [160%](wasm/control/160-initial.tsv) · [140%](wasm/control/140-batch.tsv) · [HWP 재열기](wasm/control/140-hwp-reopen-1.tsv) · [HWP 2회차 재열기](wasm/control/140-hwp-reopen-2.tsv) · [HWPX 재열기](wasm/control/140-hwpx-reopen-1.tsv) · [HWPX 2회차 재열기](wasm/control/140-hwpx-reopen-2.tsv) |

| 확대 비교 | Native | fresh WASM |
| --- | --- | --- |
| 원본 HWP 140% | [review](native/review_001.png) · [overlay](native/overlay_001.png) · [문단 5 확대](native/140-enlarged.png) | [review](wasm/review_001.png) · [overlay](wasm/overlay_001.png) · [문단 5 확대](wasm/140-enlarged.png) |
| 160% 대조 | [문단 5 확대](native/160-enlarged.png) | [문단 5 확대](wasm/160-enlarged.png) |

Native Skia의 제품 소스는 `85aeed7b3e9437d369bb8e85c92637e1b0fd8a86`이고, 최적화 WASM의 제품 소스는 `46e14637fc41f76251dcd953687955ce02a51800`이다. 리베이스에서 제품 코드와 검증 입력이 동일함을 확인했다. Native에는 저장소 공개 글꼴을, Chrome에는 저장소의 Noto Sans KR을 공급했다.

## 기여자 공통 보호 검사

4개 문서의 각 한 쪽을 Native와 fresh WASM으로 측정했다. 공식 두 자리 raw는 8건 모두 100.00%였으며, 등록 기준 하락과 누락은 없었다. 편람 HWPX의 Native 383쪽은 최저 99.97%로 정본 하락이 없었다.

| 문서 | Native | fresh WASM |
| --- | --- | --- |
| `re-03-latin-only-hancom` | [TSV](contributor/re-03-latin-only-hancom/native/silhouette.tsv) · [review](contributor/re-03-latin-only-hancom/native/review.png) · [overlay](contributor/re-03-latin-only-hancom/native/overlay.png) | [TSV](contributor/re-03-latin-only-hancom/wasm/silhouette.tsv) · [review](contributor/re-03-latin-only-hancom/wasm/review.png) · [overlay](contributor/re-03-latin-only-hancom/wasm/overlay.png) |
| `angle-2400` | [TSV](contributor/angle-2400/native/silhouette.tsv) · [review](contributor/angle-2400/native/review.png) · [overlay](contributor/angle-2400/native/overlay.png) | [TSV](contributor/angle-2400/wasm/silhouette.tsv) · [review](contributor/angle-2400/wasm/review.png) · [overlay](contributor/angle-2400/wasm/overlay.png) |
| `pic-crop-01` | [TSV](contributor/pic-crop-01/native/silhouette.tsv) · [review](contributor/pic-crop-01/native/review.png) · [overlay](contributor/pic-crop-01/native/overlay.png) | [TSV](contributor/pic-crop-01/wasm/silhouette.tsv) · [review](contributor/pic-crop-01/wasm/review.png) · [overlay](contributor/pic-crop-01/wasm/overlay.png) |
| `shape-group-02` | [TSV](contributor/shape-group-02/native/silhouette.tsv) · [review](contributor/shape-group-02/native/review.png) · [overlay](contributor/shape-group-02/native/overlay.png) | [TSV](contributor/shape-group-02/wasm/silhouette.tsv) · [review](contributor/shape-group-02/wasm/review.png) · [overlay](contributor/shape-group-02/wasm/overlay.png) |

## 선행 devel 대조

기준은 `5665e538292a0686a912f6a4d8950c55be89929a`다. Native 실행 파일의 소스 `0adeca439f1481e7245a344b7713d7d563df2ace`와 이 커밋 사이에는 제품 코드·글꼴·검증 입력 변경이 없다. 기준 WASM은 #7910에서 같은 devel을 빌드한 패키지를 사용했다. `wasm-opt`의 종료 결과와 패키지 SHA-256 `1e627a0d2b985118e9a81acf9d63b138d4267823655fde3781002fd6c1d1c842`를 확인했고, 수정본은 SHA-256 `665978390def368e01324a73f55df66df9ef84c530d83e249fd8b921261a6c6d`다.

| 1쪽 전쪽 대조 | 기준 Native | 기준 fresh WASM | 수정본 Native·fresh WASM |
| --- | ---: | ---: | ---: |
| 원본 HWP 140% | [91.62%](devel/native/original/140-batch.tsv) | [91.47%](devel/wasm/original/140-batch.tsv) | 100.00% |
| 원본 HWP 160% | [100.00%](devel/native/original/160-initial.tsv) | [100.00%](devel/wasm/original/160-initial.tsv) | 100.00% |
| 정상 HWPX를 140%로 편집 | [100.00%](devel/native/control/140-batch.tsv) | [100.00%](devel/wasm/control/140-batch.tsv) | 100.00% |

기준 fresh WASM의 원본 140%는 [두 번째 HWP 재열기](devel/wasm/original/140-hwp-reopen-2.tsv)와 [두 번째 HWPX 재열기](devel/wasm/original/140-hwpx-reopen-2.tsv)에서도 각각 91.47%였다. [같은 영역의 한컴 Print와 WASM 수정 전후 확대](devel/wasm/140-before-after.png)

## 제출 시점 devel 재대조

`1a27faacff48ea7329e76ce8eca15aa9ca8aa877`의 Native와 최적화 WASM을 새로 빌드했다. #9304의 그림 저장과 #9302의 Studio 변경이 반영된 기준이다. 후보와 최신 기준의 병합 시뮬레이션에서 새 제품·회귀 코드 43개 경로가 최신 기준과 동일했다.

동일 입력·인쇄본·글꼴·96 DPI에서 두 입력과 두 backend의 각 8가지 상태를 다시 대조했다. 원본 140% 기준은 Native 91.62%, fresh WASM 91.47%였고 HWP·HWPX의 두 번째 재열기도 같은 값이었다. 수정본의 해당 상태와 160% 정상 대조는 모두 100.00%다. 32행 중 기준 대비 하락은 0행이다.

- [정밀값 32행 대조](devel-latest/comparison.tsv)
- [원본 140% Native 기준 TSV](devel-latest/native/original/140-batch.tsv) · [원본 140% fresh WASM 기준 TSV](devel-latest/wasm/original/140-batch.tsv)
- 기준 Native 실행 파일 SHA-256 `fd74472e57b23d4d38622286aa83ee48047fc1788cd67751e73a6c6188c34c8e` · 기준 fresh WASM SHA-256 `40e2f59d7cabacfe361266e63859f0b2c2b9ca04fbf36666c5a63572c3e1ecc1`
