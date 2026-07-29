# PR #3520 검토 — 렌더 글리프 치환을 텍스트 추출에 공유

- 검토일: 2026-07-29
- 작성자: [@planet6897](https://github.com/planet6897)
- PR: https://github.com/edwardkim/rhwp/pull/3520
- base / 원본 head: `devel` / `2005086934141ee8f38b8b7afb7d035798788596`
- 규모: +145 / -1, 2 files (GitHub 조회 시점 기준)
- reviewer: `@jangster77` 배정 완료
- 관련 이슈: #3385

## 변경과 검토

`src/renderer/composer.rs`의 텍스트 추출이 렌더와 다른 치환 표를 쓰던 분기를 제거해 PUA가
사용자-visible text surface로 유출되지 않게 한다. 원본은 통합 브랜치에 `c8db0665f`으로 적용했다.
새 CLI 회귀 테스트의 binary 탐색은 collaborator 보정 `639e6250d`으로 nextest archive에서도
동작하도록 했다.

렌더러 파일이지만 SVG/PDF의 geometry·paint path를 바꾸지 않고 추출 문자열 경로만 바꾼다. 그래도
시각 보조 증적을 수행했다.

## 시각 증적

- 원본: `samples/exam_kor.hwp`, SHA-256
  `0315576fb25dd29ad3b6b188ee2539d0e8d31c15b74847be801c2186a97aac69`
- 기준 PDF: `pdf/exam_kor-2022.pdf`, SHA-256
  `80fa2a520dcbd15a15c04eb6284f83bba9c8ff33a535445c5fcd5df1b9e12197` (20 pages)
- 임시 sweep: `output/visual-planet6897-20260729-exam-kor-p1-rebased/`
- page 1만 raster/compare/overlay/review: 1/1, 자동 후보 0건.
- pixel match 90.103%, visual_accuracy_proxy 16.353%. 후자는 서로 다른 글꼴 rasterizer의
  획 안티앨리어싱 차이를 포함한 보조 지표이며, overlay에서 페이지 frame·문단·문항·표의 기하학적
  위치는 일치했다.
- 사람이 확인한 대표 asset:
  `mydocs/pr/assets/pr_3520_text_surface_review_001.png` (SHA-256
  `458016e453045f2ecfddad9ffa58d2f3cfeafa4bc653a891279518ae7164e3d6`).

`--page 1`의 raster 제한 자체는 collaborator 보정 `7ab7e137a`으로 기록했다. 문서 전체의 SVG와
render-tree 추출(20/20)은 유지하되 `rsvg-convert`와 `pdftoppm`은 각 1장만 생성하는 회귀 테스트
4건을 추가했다.

## 검증과 판정

- 해당 회귀 테스트: 3 passed.
- 전체 통합 검증·최신 devel rebase·CI 진행 순서는
  [공통 구현 기록](pr_3503_review_impl.md)에 집계한다.

**권고: 수용.** visual sweep은 text-surface 변경의 보조 증적이고, 최종 조건은 통합 PR의 required
CI 성공이다. 성공 뒤 #3385를 close한다.
