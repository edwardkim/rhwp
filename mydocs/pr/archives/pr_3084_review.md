# PR #3084 검토 — pgnp 글자 크기 단위와 장식 공백 교정

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3084](https://github.com/edwardkim/rhwp/pull/3084) |
| 작성자 | planet6897 |
| 관련 이슈 | [#3048](https://github.com/edwardkim/rhwp/issues/3048) |
| base / 규모 | devel / 5 files, +29 -12 |
| 문서 작성 시점 참고값 | 원 PR head 68898247607b64941e82539503c00d7e5e577190, BEHIND 및 MERGEABLE. 이전 head의 Build & Test, CodeQL, Canvas visual diff는 성공으로 확인했다. |
| 통합 적용 | 최신 upstream/devel 866925fa6 위 적용 commit 6dcccecdc2834c831e677143af60f93f1d14ee35 |

## 관련 이슈와 변경 범위

- #3048은 한글 pgnp 쪽 번호의 10pt 의도가 px 필드에 들어가 96dpi에서 7.5pt로 작아지던 단위 혼동과 장식 leader 공백을 교정한다.
- 레이아웃의 쪽 번호 크기를 dpi / 72.0 변환으로 명시하고 KTX 목차 golden 및 관련 검증을 갱신한다.
- 변경은 쪽 번호 출력과 그 회귀 근거에 한정되며, 일반 본문 글자 크기 계산에는 닿지 않는다.

## 렌더 영향과 검증

- 쪽 번호 위치와 글자 크기를 바꾸므로 visual sweep 대상이다.
- 기준 원본은 samples/KTX.hwp, 기준 PDF는 pdf/KTX-2022.pdf다. 두 파일의 SHA-256은 각각 b6c1492152f53e8dd7d4bbbb4faca88866bb8458e9018c70c936cd469ea6fab3, f6fad0448109ee477f7b259e947408314f2c6e12a0defde21d25e01dfb1e9d78다.
- KTX p002 visual sweep 자동 후보는 0건이었다. pixel match는 93.69492%, visual accuracy proxy는 21.58668%다. 후자는 폰트와 anti-aliasing 차이가 큰 보조값이므로 수용 기준으로 사용하지 않았다. 검토 PNG는 mydocs/pr/assets/pr_3129_planet_ktx_p002_review.png에 고정했다.
- 최신 통합 브랜치에서 cargo test --test svg_snapshot 8/8, cargo test --test issue_874_ktx_toc_page_number_right_align 1/1을 통과했다.

## 리스크와 판단

- dpi 변환은 pgnp 경로 전체에 적용된다. 기존 KTX 목차 snapshot과 우측 정렬 테스트가 직접 회귀를 막는다.
- visual sweep 이미지에서 한컴 PDF와 rhwp의 폰트 raster 차이는 남지만, 쪽 번호의 우측 정렬 및 목차 구조는 유지된다.

## 최종 권고

- planet6897 렌더 4건 통합 PR에 포함해 수용 권고.
- 최종 merge 조건은 통합 PR 최신 head의 GitHub Actions 통과와 작업지시자 승인이다.
