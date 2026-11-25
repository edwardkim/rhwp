# PR #9411 검토 — planet6897의 문단 정렬, 이월 표, KAF 렌더링 수정

## 최종 판정

**승인.** 개별 검토에서 가운데 정렬 말미 공백, 이월 표와 각주 구분선, KAF 중첩 표의
저장 쪽 경계, KAF 부록의 WMF와 글 상자 배치를 확인했다. KAF는 통합 과정에서 축소된
독립 글 상자의 줄 소속, 마지막 줄 잘림, 단일 DIB 전체 자르기를 인쇄본과 대조하고 보정했다.

| 원 PR | 검토와 시각 증적 |
| --- | --- |
| [#9374](https://github.com/edwardkim/rhwp/pull/9374) | [인천 직급보조비 표 1쪽](pr_9374_review.md) |
| [#9375](https://github.com/edwardkim/rhwp/pull/9375) | [식량안보 90~93쪽 및 제주 대조군](pr_9375_review.md) |
| [#9386](https://github.com/edwardkim/rhwp/pull/9386) | [KAF 중첩 표 33~44쪽](pr_9386_review.md) |
| [#9400](https://github.com/edwardkim/rhwp/pull/9400) | [KAF 부록의 글 상자·WMF 6쪽](pr_9400_review.md) |

## 최종 후보 검증

- 코드 검증 head `503106c21f7d2137aeece39eaed32d9f078d2bf0`의 GitHub Full CI가 성공했다.
  로컬 전체 nextest는 14,122건 통과·45건 제외, Native Skia는 4,225건 통과·12건 제외다.
  [코드 head CI](https://github.com/edwardkim/rhwp/actions/runs/45619233681)
- 보호 문서 23종·46조건·4,046쪽의 Native와 최적화한 fresh WASM 표시값이 정본과 모두 일치했다.
  편람 HWPX Native 383쪽을 포함한다. KAF 원 PR과 최종 후보를 같은 새 한컴 2020 인쇄본·글꼴
  환경에서 비교해 표 44쪽, 부록 54쪽의 양쪽 경로에서 하락 0쪽을 확인했다. 최초 인쇄본과
  새 인쇄본의 차이 및 31쪽 CID 참조 오류는 개별 검토에 기록했다.
  [보호 검증](../assets/planet6897_20261125/final-validation/README.md) ·
  [KAF 최종 전쪽 결과](../assets/planet6897_20261125/kaf-print-repair/README.md)
- 공급 자료 갱신 head `f8f48aa9b80e856fe197464dcbec17e3655156e1`의 별도 CI가 성공했다.
  이번 기록과 함께 반영한 `HMKMM_MAC.TTF`는 해시 `c7e500aa94e50829339db9d37ca81dae66e663234abed7178d1ee941d0c82e2f`다.
  동일 바이너리·WASM 패키지·Print PDF·공급 순서에서 영향 문서 9종의 Native·WASM 2,296쪽과
  글꼴 공급 조합 추가 392쪽을 다시 측정했다. 공식 표시값과 출력 PNG 2,688장이 직전 결과와
  일치했다. 나머지 14종은 입력·기준 PDF 및 공급 파일 해시를 다시 확인했다.
  [공급 갱신 CI](https://github.com/edwardkim/rhwp/actions/runs/45626663955) ·
  [문서별 허용 해시](../../verification/protected-font-supply/manifest.json)
- `HMKMM.otf`는 화학물질 표시 기준 WASM 82쪽에서 100.00%→99.99%로 하락해 허용 목록에서 제외했다.
  기준 점수와 기존 인쇄본은 유지했다.

원 PR 댓글에 병합 SHA로 고정한 해당 페이지의 review 및 overlay를 표시하는 계획을 각 검토 기록에 남겼다.
