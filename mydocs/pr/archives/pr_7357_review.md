# PR #7357 리뷰 — 쪽 경계값 0의 분모 보호

## 최종 판정

**승인.** 0 값 입력의 안전성 보정이며, 통합 전체 회귀와 뒤따르는 #7378의 실제 문서 검증으로 회귀 범위를 확인했다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#7357](https://github.com/edwardkim/rhwp/pull/7357) |
| 작성자 | `planet6897` |
| 관련 이슈 | [#7288](https://github.com/edwardkim/rhwp/issues/7288) |
| 원 contributor head | `bf1fae3803e7` |
| 승인분 통합 PR | 새 `upstream/devel` 기반 PR 준비 중 (`codex/planet-approved-clean-20260924`) |
| reviewer | `jangster77` |

## 변경과 통합 순서

쪽 경계값이 0인 저장 문서에서 행 분배의 분모가 0이 되는 경로를 막고, 기존 정상 쪽 경계의 배치를 유지한다.

같은 #7288 축의 #7378보다 먼저 적용했다. #7378의 행 원자성 검사가 이 보호 위에서 실행된다.

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
| Native Visual Sweep 대표 쪽 | 현 후보 binary SHA-256 `a220756502525135…`로 재산출 완료; 99.805% |

## 증적과 범위

추적된 증적 경로: `mydocs/report/issue7288`

이 기록은 원 PR의 최신 head를 최신 `upstream/devel` 기준으로 누적 적용한 통합 후보를 대상으로 한다.
원 PR 단독 CI와 통합 후보 CI는 구분하며, source PR의 merge 가능 상태만으로 통합 후보의 최종 head를
승인하지 않는다.

## 최신 Native Visual Sweep 증적

승인분 통합 후보의 review 전용 실행 파일과 **이미 추적된** 기준 PDF로 다시 산출했다.
새 PDF 변환은 하지 않았다.

| 항목 | 값 |
| --- | --- |
| 입력 | `samples/task1725/text_footnote_tail_overpagination.hwp` |
| 기준 PDF | `pdf/text_footnote_tail_overpagination-2024.pdf` |
| 입력 SHA-256 | `d335c26cbcd57b157c929ed36596d95a6676bea5624348baae3bff3b55a705d9` |
| 원본 기준 PDF SHA-256 | `d8f47fc772556cc15b444fad61deca8b0f260eb82b3bf1691a36cc41036a69ba` |
| 입력 커밋 확인 | 둘 다 후보의 `HEAD` tree에 포함됨 (`git cat-file -e HEAD:<path>` 확인) |
| 비교 쪽 | rhwp 63 ↔ PDF 62, rhwp 64 ↔ PDF 63 |
| 산출물 | 아래 링크 |

- [pr7357_native_review_rhwp063_pdf062.png](../assets/planet_approved_20260923/pr7357_native_review_rhwp063_pdf062.png)
- [pr7357_native_overlay_rhwp063_pdf062.png](../assets/planet_approved_20260923/pr7357_native_overlay_rhwp063_pdf062.png)
- [pr7357_native_review_rhwp064_pdf063.png](../assets/planet_approved_20260923/pr7357_native_review_rhwp064_pdf063.png)
- [pr7357_native_overlay_rhwp064_pdf063.png](../assets/planet_approved_20260923/pr7357_native_overlay_rhwp064_pdf063.png)

같은 물리 쪽 대조는 PDF가 한 장 더 많은 문서의 매핑 오류로 28.401%였다. 실제 대응쪽은 rhwp 63 ↔ PDF 62의 2px 이웃 관용 실루엣 **99.832%**, rhwp 64 ↔ PDF 63의 **99.805%**다. 잘못된 same-page PNG는 커밋 대상에서 제외했다.

모든 PR의 실행 명령과 증적 목록은 [승인분 통합 후보 Native Visual Sweep 증적](../assets/planet_approved_20260923/README.md)에 있다.

## Merge 후 contributor PR comment 계획

최종 head CI와 실제 merge SHA가 확정된 뒤 원 PR에 한국어로 다음을 게시한다.

- 실제 merge SHA와 새 통합 PR의 최종 head CI URL, 이 PR의 원 contributor head를 구분해 기록한다.
- 위의 변경 계약·적용 순서·검증 범위와 남은 차이가 있으면 그 차이를 함께 설명한다.
- [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결한다.
- 이 PR의 최신 Native review·overlay를 실제 merge commit 아래 경로로 Markdown 이미지로 모두 표시한다.

  - `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/planet_approved_20260923/pr7357_native_review_rhwp063_pdf062.png`
  - `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/planet_approved_20260923/pr7357_native_overlay_rhwp063_pdf062.png`
  - `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/planet_approved_20260923/pr7357_native_review_rhwp064_pdf063.png`
  - `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/planet_approved_20260923/pr7357_native_overlay_rhwp064_pdf063.png`

  위 PNG는 `samples/task1725/text_footnote_tail_overpagination.hwp`와 이미 추적된 `pdf/text_footnote_tail_overpagination-2024.pdf`를 재사용해 최종 통합 후보에서 다시 산출한 것이다.
