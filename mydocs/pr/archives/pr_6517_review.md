# PR #6517 검토 - 한양중고딕 SVG/Studio 별칭

- 원 PR head: e634e8ba6bad03fd4d471d7a086cfbd9a60a1655
- 통합 cherry-pick: 76532b4da0e720026fb24211ad0c382884d3b970
- 통합 기준: 76532b4da0e720026fb24211ad0c382884d3b970

## 판정: 메인터너 보정 됨 수용 가능

## 확인한 범위

renderer alias와 rhwp-studio font substitution/wasm bridge 및 test를 변경한다.

## 검증 및 증적

공통 전체 회귀, Native Skia, 배포용 WASM이 통과했다.

원 PR은 『별표 7』 run의 before/after/oracle 비교를 제공한다. PDF 경로는 이 변경의 범위 밖이다.

## 다음 조건

설치 글꼴이 있는 Windows 검증 호스트에서 current-head SVG/Studio 출력의 glyph gap과 실제 fallback font-family를 oracle·PDF와 대조한다.

공통 검증 세부 내용은 pr_6489_6517_planet6897_integration_evidence.md를 따른다.
## 2026-08-31 메인터너 보정 검증

**최종 판정: 메인터너 보정 됨 수용 가능.**

- 비공개 `3146683` HWPX는 Hancom Office 2020 저장본이므로 Hancom `2020` profile 기준 PDF를 사용했다: SHA-256 `0b642d299ba6ca8c0e69c56e5a71539eca5d93c3acb0d766b73bc31c7a58e248`.
- Windows 검증 worktree에서 #6496/#6517 alias commit을 적용해 native-skia release PNG를 생성했다. 실제 font file `H2GTRM.TTF`는 `HYGothic-Medium`, `H2MJSM.TTF`는 `HYSinMyeongJo-Medium`이며 SVG는 `HYGothic` chain을 방출한다.
- Windows raster에 글리프 누락, table 경계 이탈, 행 충돌은 없고 단일 페이지 sweep도 해당 structural flag 없이 완료됐다. Hancom과의 픽셀/굵기 차이는 남아 있으므로 이를 픽셀 동일성 통과로 주장하지 않는다.
- Windows rhwp와 Hancom PNG는 각각 `maintainer-20260831/pr6517-windows-rhwp.png`, `maintainer-20260831/pr6517-hancom2020.png`에 보존했다.
