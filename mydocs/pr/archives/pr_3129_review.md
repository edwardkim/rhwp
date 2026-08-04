# PR #3129 검토 — body clip의 흐름 콘텐츠 꼬리 보존

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3129](https://github.com/edwardkim/rhwp/pull/3129) |
| 작성자 | planet6897 |
| 관련 이슈 | [#3127](https://github.com/edwardkim/rhwp/issues/3127) |
| base / 규모 | devel / 3 files, +66 -22 |
| 문서 작성 시점 참고값 | 원 PR head e7b9d01e077688ec90464f00d73405e6e099e638, BEHIND 및 MERGEABLE. 이전 head의 Build & Test, CodeQL, Canvas visual diff는 성공으로 확인했다. |
| 통합 적용 | 최신 upstream/devel 866925fa6 위 적용 commit 0f6310c199e813b9fb6e8a17c670efe8c6331543 |

## 관련 이슈와 변경 범위

- #3127은 body_area 아래로 늘어난 표 등 흐름 콘텐츠가 SVG body clip에 잘려 PDF에서 소실되는 문제를 고친다.
- 흐름 콘텐츠와 부동 그림 subtree의 bbox를 분리한다. 흐름 콘텐츠는 실제 아래 경계까지 clip을 확장하고, 부동 그림은 기존 Task #460 보호 규칙인 body bottom + 10 상한을 유지한다.
- partial TAC table 회귀 테스트를 갱신한다. 범위 밖 renderer 리팩터링은 포함하지 않는다.

## 렌더 영향과 visual sweep

- clipping, 표 꼬리, PDF 출력에 직접 영향이 있어 visual sweep 필수 대상이다.
- 이슈와 원 PR에는 원 주장에 쓰인 20402833 HWP/HWPX 원본이 없었다. 재현성 공백으로 기록하며, merge 후 원 PR 안내에서는 다음 시각 검증 PR에 원본과 기준 PDF를 함께 첨부하도록 요청한다.
- 동등한 기존 샘플 samples/KTX.hwp와 한컴 기준 pdf/KTX-2022.pdf로 p002를 대조했다. visual sweep의 자동 후보는 0건이며, pixel match 93.69492%, visual accuracy proxy 21.58668%다. 이미지 확인은 완료했으나 보조 지표의 낮은 값은 폰트와 anti-aliasing 차이에 기인하므로 단독 수용 근거로 쓰지 않았다.
- export.log는 page=1에서 Table bottom 1050.2, body bottom 1028.0, overflow 22.1px를 기록했다. 생성 SVG KTX_002의 body clip 하단은 1050.1867까지 확장되어 흐름 표의 꼬리를 포함한다.
- 안정 검토 자산은 mydocs/pr/assets/pr_3129_planet_ktx_p002_review.png다.

## 사전 검증

- 최신 통합 브랜치에서 cargo fmt --check 통과.
- cargo test --test issue_1486_hwpx_partial_tac_table 6/6, cargo test --lib renderer::layout::integration_tests::tests::test_634 8/8, cargo check --features native-skia --lib 통과.
- cargo test --test svg_snapshot 8/8과 git diff --check upstream/devel...HEAD도 통과.
- Clippy, doctest, release 전체 검증은 원격 통합 PR 최신 head CI에서 최종 확인한다.

## 리스크와 판단

- 흐름과 부동 객체의 분류가 핵심 위험이다. Image, Group, Path, Ellipse, Rectangle, Line, TextBox, Placeholder, RawSvg만 부동 객체로 분류하고, 그 subtree에는 기존 상한을 적용한다.
- KTX 표의 실제 overflow와 TAC nested table 회귀 테스트가 각각 흐름 clip 확장과 기존 표 경계 보존을 검증한다.

## 최종 권고

- planet6897 렌더 4건 통합 PR에 포함해 수용 권고.
- 최종 merge 조건은 통합 PR 최신 head의 GitHub Actions 통과와 작업지시자 승인이다. merge 후 원 PR과 #3127의 close 상태를 확인하고, 재현 자료 요청을 포함한 감사 코멘트는 별도 승인 후 남긴다.
