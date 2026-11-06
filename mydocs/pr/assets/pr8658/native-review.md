# PR #8658 Native 선행 검토 증거

base `08963f6825956984d38b0d9392f44777db7107ab`, 원 head `218c8856ad0a338fd1820c43e62b40dbcdcb86bb`, 검토 code `759f35f2fbccc2682356dc22cb31b1068b57e178`이다. 각 producer는 `cargo build --locked --profile release-test --features native-skia --bin rhwp --target-dir /home/edward/mygithub/rhwp/target/pr-review`로 실제 빌드했다.

기준은 저장 제품에 맞는 한컴 Desktop Print다. 원 HWPX와 CENTER/선언 높이 대조군은 2024, 원 HWP5는 2020이다. 해당 PDF와 입력은 검토 branch의 `pdf/issue8576`, `samples/issue8576`, `tests/fixtures/pr8658`에 보존한다. 96dpi, print profile, 2024/2022 compat를 각각 명시했다. 원 글꼴 hamchod-r.ttf와 hamchob-r.ttf의 명시적 공급으로 설치 글꼴 가정을 피했다. 글꼴 프로그램은 검증 전용이며 커밋하지 않았다.

## 실제 전쪽 결과

| 입력 | 쪽수 | base Native raw 최저 | 후보 Native raw 최저 |
| --- | --- | --- | --- |
| cell-square-six-pictures-35x25 | 1 | 60.48% | 100.00% |
| cell-square-six-pictures-60x30-grows | 1 | 34.98% | 100.00% |
| split-row-picture-only-para | 2 | 85.72% | 100.00% |
| cell-square-six-pictures-center | 1 | 50.86% | 100.00% |
| cell-square-six-pictures-exact-fit | 1 | 45.59% | 83.83% |

Native 전쪽 원문은 `native/base/*.tsv`, `native/candidate/*.tsv`다. 실루엣 점수는 2px 이웃·색상 경계 보정 전 보조값이며 최종 수용이나 사람의 판정 정확도가 아니다. 원본/대조군의 전체 쪽수는 각각 대응 Print와 일치한다.

원 4쪽과 CENTER 대조군의 100%를 선언 높이 경계의 미달에 상쇄하지 않는다. 후보의 마지막 괘선과 뒤 END는 독립 Print보다 위에 있다. 같은 영역 2배 확대는 [tail zoom](exact-fit-tail-zoom.png), 같은 페이지의 [review](native/review/cell-square-six-pictures-exact-fit/review_001.png)와 [overlay](native/review/cell-square-six-pictures-exact-fit/overlay_001.png)를 따른다. 이는 원 PR에 이미 명시된 미해결 경계이고, base보다 점수가 개선돼 새 회귀로 보고하지 않았다.

## 직접 overlay 지표

- cell-square-six-pictures-35x25 p1: pixel_match=99.68581, visual_accuracy_proxy=67.76831, raw=100.0.
- split-row-picture-only-para p1: pixel_match=98.26495, visual_accuracy_proxy=30.27022, raw=100.0.
- split-row-picture-only-para p2: pixel_match=99.03925, visual_accuracy_proxy=20.21068, raw=100.0.
- cell-square-six-pictures-exact-fit p1: pixel_match=99.58467, visual_accuracy_proxy=58.76978, raw=83.83107.

fresh optimized WASM·집중 회귀 등 남은 실행은 아직 이 Native 증거에 포함하지 않는다. 원격에 게시하지 않은 로컬 검토 자료다.
