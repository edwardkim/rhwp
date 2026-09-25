# PR #7359 리뷰 — Visual Sweep 긴 진단 라벨 줄바꿈

## 최종 판정

**메인터너 배치 보정 후 시각 gate 통과 — 개별 수용 가능.** 기존 p14의 68.175%는 글꼴 예외가 아니었다.
첫 문단과 표 전체가 기준 PDF보다 각각 8px, 약 15px 높게 그려졌다. 저장된 HWP5 첫 줄의
문단 위 간격을 프레임 재조판 뒤에도 보존하자 표 가로선이 기준 좌표에 맞고 2px 이웃
내용 실루엣 일치율이 **97.479%**로 올랐다. 통합 PR의 최종 head 검증과 CI는 별도 판정한다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#7359](https://github.com/edwardkim/rhwp/pull/7359) |
| 작성자 | `planet6897` |
| 관련 이슈 | [#7349](https://github.com/edwardkim/rhwp/issues/7349) |
| 원 contributor head | `afa0334fc9fd` |
| 승인분 통합 PR | 새 `upstream/devel` 기반 PR 준비 중 (`codex/planet-approved-clean-20260924`) |
| reviewer | `jangster77` |

## 변경과 통합 순서

긴 비교 대상명과 진단 key가 review·overlay 캔버스 밖으로 나가지 않도록 줄바꿈과 표시 폭을 조정한다.

최신 devel의 #7377 Visual Sweep 지표 확장과 `scripts/visual_sweep.py`, 테스트 파일에서 충돌했다. 메인터너가 긴 라벨 줄바꿈과 #7377의 2px 관용 내용 실루엣 지표를 함께 보존했다.

## 통합 후보 검증

승인된 9개 원 PR만 `upstream/devel` `c84a8f8e1a97acaf8626b28a14417344a8578f41` 위에
다시 적용한 `codex/planet-approved-clean-20260924` 후보를 대상으로 한다. #7366·#7371과
#7371에 의존하는 #7372는 포함하지 않는다. 이전 #7380 혼합 head의 전체 회귀 결과는 이
후보의 검증 결과로 재사용하지 않는다. 로컬 검증과 새 head CI 결과가 확정되기 전에는
아래 수용 판정은 원 PR의 개별 검토 판정이며 통합 PR의 merge 승인이 아니다.

| 검증 | 새 후보 결과 |
| --- | --- |
| Rust format 및 Visual Sweep Python 계약 | format 통과, Python 67 PASS |
| release-test 전체 `cargo nextest run --locked --cargo-profile release-test --target-dir /Users/tsjang/rhwp/target/pr-review --tests --test-threads 8 --no-fail-fast` | 최종 후보에서 10,193 PASS / 실패 0 / skip 50 (521.763초) |
| #7365 TAC 표 focused 6개 | 위 전체 실행 안에서 **6 PASS** |
| Native Skia 전체 lib 및 실물 fixture 2종 | 최종 후보에서 lib 3,930 PASS (13 ignored)·workspace lib 182 PASS·실물 #2225 2 PASS·p37 4 PASS |
| Clippy·workspace 빌드·fresh WASM | 최종 후보에서 기본/wasm32/workspace all-targets Clippy, workspace 빌드, 저장소 루트 `scripts/wasm-pack-locked.sh --target web --out-dir pkg` 모두 통과 |
| Native Visual Sweep 대표 쪽 | #7359 보정 후 p14 재산출 완료; 2px 이웃 내용 실루엣 97.479%, 예외 없이 gate 통과 |

## 증적과 범위

추적된 증적 경로: `mydocs/pr/assets/issue_7349_sweep_label`

이 기록은 원 PR의 최신 head를 최신 `upstream/devel` 기준으로 누적 적용한 통합 후보를 대상으로 한다.
원 PR 단독 CI와 통합 후보 CI는 구분하며, source PR의 merge 가능 상태만으로 통합 후보의 최종 head를
승인하지 않는다.

## 통합 중 메인터너 보정 — review footer 중복 제거

`make_overlay_page`는 overlay 하단에 한국어 지표를 포함한다. `make_review_panels`가 이를 다시 그려
review PNG에 같은 문장을 두 번 출력하던 것을 제거했다. review는 overlay를 그대로 붙이고 별도 footer를
추가하지 않는다. `test_review_uses_the_overlay_footer_once`는 review와 overlay의 높이가 같아야 한다고
고정해 중복이 다시 생기면 실패한다.

`python3 -m unittest scripts/tests/test_visual_sweep.py`는 **67 PASS**이며, 이 PR의 최신 `review_014`와
각 원 PR 증적을 모두 다시 산출했다.

## 통합 중 메인터너 보정 — 페이지 첫 문단의 저장 간격

실물 HWP5의 p14는 본문 첫 문단 `pi=133`의 저장 첫 줄 `vpos=600HU=8px`, 다음 표 호스트
`pi=134`의 `vpos=3020HU`를 가진다. 기준 PDF에서 첫 문단은 약 140px, 표 제목은 약
173px, 표 첫 괘선은 193px이다. 보정 전 렌더 트리는 각각 132.28px, 157.88px,
178.5px이어서 표 행 높이 자체가 아니라 페이지 상단 간격의 누락이었다.

`src/renderer/layout.rs`에서 페이지 첫 문단 다음에 다음 문단의 자리차지 표와
표 제목이 이어지고 두 저장 줄의 앞 간격이 실제 ladder와 일치하는 경우만 선별한다.
`src/renderer/layout/paragraph_layout.rs`는 그 HWP5 첫 문단이 프레임에서
재조판되어도 저장 시작 간격을 보존한다. 다른 첫 문단까지 넓히면 본문 넘침
baseline이 증가하므로 이 구조로 범위를 좁혔다. 보정 후 좌표는
**140.3px, 172.5px, 193.2px**로 기준에 맞는다.
`tests/cases/issue_7359_page_top_table_spacing.rs`가 원본 103쪽 수와 이 세 좌표를
실물 회귀 계약으로 고정한다. 보정 전 FAIL, 보정 후 1 PASS를 확인했다.

## 최신 Native Visual Sweep 증적

승인분 통합 후보의 review 전용 실행 파일과 **이미 추적된** 기준 PDF로 다시 산출했다.
새 PDF 변환은 하지 않았다.

| 항목 | 값 |
| --- | --- |
| 입력 | `samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp` |
| 기준 PDF | `pdf/1480000-201900042-chemical-product-labeling-study-2020.pdf` |
| 입력 SHA-256 | `398d03a5d5e4d6e857086be532d6d9ed0cec9c8ad06f95c17bbb7f83056ae860` |
| 원본 기준 PDF SHA-256 | `f8e5c0408e221080ede9a9a67b153d02d792d22961c738e46749641f32a32e79` |
| 입력 커밋 확인 | 둘 다 후보의 `HEAD` tree에 포함됨 (`git cat-file -e HEAD:<path>` 확인) |
| 비교 쪽 | 14 |
| 산출물 | 아래 링크 |

- [pr7359_native_review_014.png](../assets/planet_approved_20260923/pr7359_native_review_014.png)
- [pr7359_native_overlay_014.png](../assets/planet_approved_20260923/pr7359_native_overlay_014.png)

p14 pixel 97.160%, 엄격 내용 픽셀 56.515%, 2px 이웃 관용 내용 실루엣 **97.479%**다.
기준 PDF와 표의 세 가로선 및 그림 위치가 일치하는 것을 review·overlay에서 직접 확인했다.
남은 글자 획의 차이는 보조 지표에 나타나지만, 이 건의 표 위치 불일치를 글꼴 차이로
분류하지 않는다. 긴 target label도 캔버스 안에서 접힌다.

모든 PR의 실행 명령과 증적 목록은 [승인분 통합 후보 Native Visual Sweep 증적](../assets/planet_approved_20260923/README.md)에 있다.

## Merge 후 contributor PR comment 계획

최종 head CI와 실제 merge SHA가 확정된 뒤 원 PR에 한국어로 다음을 게시한다.

- 실제 merge SHA와 새 통합 PR의 최종 head CI URL, 이 PR의 원 contributor head를 구분해 기록한다.
- 위의 변경 계약·적용 순서·검증 범위와 남은 차이가 있으면 그 차이를 함께 설명한다. 특히
  p14 표 첫 괘선 178.5→193.2px(기준 PDF 193px), 실루엣 68.17→97.48%의
  메인터너 보정과 기존 글꼴 예외 판정 철회를 명시한다.
- [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결한다.
- 이 PR의 최신 Native review·overlay를 실제 merge commit 아래 경로로 Markdown 이미지로 모두 표시한다.

  - `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/planet_approved_20260923/pr7359_native_review_014.png`
  - `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/planet_approved_20260923/pr7359_native_overlay_014.png`

  위 PNG는 `samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp`와 이미 추적된 `pdf/1480000-201900042-chemical-product-labeling-study-2020.pdf`를 재사용해 최종 통합 후보에서 다시 산출한 것이다.
