---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6369
---

# PR #6369 review - 표 행 컷의 0.07px 근접 초과만 허용한다

## 검토 판단

**수용 권고.** 초기 `0.5px` 관용이 확정 baseline을 깨뜨린 뒤 `0.1px`으로 좁힌 최신 source를
검토했다. 원 PR이 설명한 두 경계의 줄 이월은 해소하면서, integrated `text_overlap_baseline`과
`issue_2439` 계약을 유지한다.

## 라우팅과 검증

- 원 PR: https://github.com/edwardkim/rhwp/pull/6369
- 작성자 / reviewer: `kevin9327` / `jangster77` review request 등록
- source head: `4ddc51ad9fa95806ca364b279172c0e3a86361c8`
- `issue_2439`: 4/4, 전체 release-test와 Native Skia 회귀를 통과했다.

## 저장 버전별 시각 증적

- `samples/hwpctl_API_v2.4.hwp`는 `hancom-office-2018`이므로 2020 engine으로
  `pdf/hwpctl_API_v2.4-2020.pdf`를 재산출했다. p12-13 sweep은 2/2, flag 0,
  평균 pixel match `95.70364%`.
- `samples/80168_regulatory_analysis.hwp`는 `hancom-office-2024`이므로 2024 engine으로
  `pdf/80168_regulatory_analysis-2024.pdf`를 재산출했다. p121-123 sweep은 3/3, flag 0,
  평균 pixel match `90.59995%`.
- 이 호스트에서 Chrome webfont raster가 즉시 종료하는 race가 있어 visual_sweep의 `rsvg` fallback을
  사용했다. 자동 흐름 지표와 사람 검토 모두 표 행·코드 상자·다음 쪽 이월이 기준 PDF와 일치함을 확인했다.
- 보관 asset: `mydocs/pr/assets/pr_6369_{hwpctl,regulatory}_{info,visual_sweep_summary}.json` 및
  `pr_6369_{hwpctl,regulatory}_p*_review.png`.

## 후속 코멘트

merge 후 원 PR에는 엔진 선택, 각 page range의 flag 0, 대표 image raw URL과 0.1px로 좁힌 이유를 남긴다.
