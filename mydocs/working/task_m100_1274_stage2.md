# Task 1274 Stage 2

## 대상

- 문서: `samples/3-11월_실전_통합_2022.hwp`
- PDF: `pdf/3-11월_실전_통합_2022.pdf`
- 페이지: 11쪽, 0-index `p=10`
- 로그: `LAYOUT_OVERFLOW_DRAW section=0 pi=537 line=0 y=2622.4 col_bottom=1092.3 overflow=1530.2px`

## 1차 판단

Stage1 전체 sweep에서 가장 큰 overflow 후보는 `3-11월_실전_통합_2022.hwp` 11쪽의 `pi=537`이다. 수치가 1530px로 매우 크므로 단순 미세 trailing 문제가 아니라, 미주/partial 흐름에서 특정 문단 또는 그림/수식의 y 기준이 잘못 복원된 가능성이 높다.

## 진행 계획

1. 11쪽 비교 PNG와 12쪽 continuation을 확인한다.
2. `dump-pages -p 10`, `dump-pages -p 11`, `dump`로 `pi=537`과 주변 문단 구조를 확인한다.
3. 기존 `task 1189`, `task 1209`, `task 1261` 미주 보정과 같은 공통 로직으로 처리 가능한지 확인한다.
4. 수정 후 11쪽/12쪽 PNG, overflow 로그, 관련 통합 테스트를 갱신한다.

## 조사 결과

- `dump-pages samples/3-11월_실전_통합_2022.hwp -p 10`에서 `pi=537`은 텍스트가 없는 host 문단이고, 같은 문단의 `ci=0` non-TAC 그림은 별도 `PageItem::Shape`로 존재했다.
- 디버그 overlay에서는 실제 그림이 오른쪽 단 상단 `y≈183px`에 그려지는데, `FullParagraph pi=537` 경로가 저장 vpos 기준 `y≈2622px`에 보이지 않는 빈 TextLine을 만들었다.
- 따라서 1530px overflow는 실제 렌더 콘텐츠가 아니라 빈 그림 host 문단을 FullParagraph와 Shape item 두 경로로 모두 처리한 데서 생긴 오탐이었다.

## 수정

- `src/renderer/layout.rs`
  - 텍스트가 없고 non-TAC 그림/도형 컨트롤이 별도 `PageItem::Shape`로 존재하는 문단을 판별하는 공통 helper를 추가했다.
  - 해당 host 문단은 `FullParagraph`에서 `layout_paragraph`를 태우지 않고 `para_start_y`만 기록한다.
  - 실제 그림/도형 렌더링은 뒤따르는 `Shape` PageItem 경로가 담당하므로 위치와 출력은 유지된다.
- `tests/issue_1139_inline_picture_duplicate.rs`
  - `pi=537` 그림 bbox가 오른쪽 단 상단에 남는지 확인했다.
  - 같은 문단에 phantom `TextLine`이 생기지 않는지 확인했다.
  - `pi=537` 콘텐츠 하단이 저장 vpos가 아니라 실제 그림 bbox 하단을 따르는지 확인했다.

## 검증

- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 통과: 47개 테스트.
- `cargo build --bin rhwp` 통과.
- `python3 scripts/task1274_visual_sweep.py --target 2022-11-practice` 통과.

## 시각/로그 확인

- 비교 PNG: `output/task1274/2022-11-practice/compare/compare_011.png`
- contact sheet: `output/task1274/2022-11-practice/contact_sheet.png`
- 페이지 수: SVG 21 / PDF 21 / compare 21
- `pi=537`의 `overflow=1530.2px` 로그는 사라졌다.
- 남은 11쪽 후보는 `pi=553`의 `14.4px` overflow이며 다음 스테이지에서 별도로 분석한다.
