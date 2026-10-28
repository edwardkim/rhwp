# PR #7370 리뷰 — 터미널 fragment box 저장 cell 잔여 높이

## 최종 판정

**원 PR 개별 승인.** terminal fragment의 기본 잔여 높이 계약을 고정했다. 이번 승인분 후보에는 #7371의 trailing blank 문단 보정을 포함하지 않으므로, 새 후보의 전체 회귀와 해당 쪽 Visual Sweep을 별도로 확인한다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#7370](https://github.com/edwardkim/rhwp/pull/7370) |
| 작성자 | `planet6897` |
| 관련 이슈 | [#7095](https://github.com/edwardkim/rhwp/issues/7095) |
| 원 contributor head | `9f5a1a6ba3f` |
| 승인분 통합 PR | 새 `upstream/devel` 기반 PR 준비 중 (`codex/planet-approved-clean-20260924`) |
| reviewer | `jangster77` |

## 변경과 통합 순서

분할된 표의 마지막 fragment box가 저장된 cell의 잔여 높이를 반영하도록 한다.

#7371과 #7372의 선행 변경이지만, 두 후속 PR은 이번 후보에서 제외하고 #7370만 적용했다.

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
| Native Visual Sweep 대표 쪽 | 현 후보 binary SHA-256 `a220756502525135…`로 재산출 완료; 99.573% |

## 증적과 범위

추적된 증적 경로: `mydocs/pr/assets/issue_7095_terminal_box`

이 기록은 원 PR의 최신 head를 최신 `upstream/devel` 기준으로 누적 적용한 통합 후보를 대상으로 한다.
원 PR 단독 CI와 통합 후보 CI는 구분하며, source PR의 merge 가능 상태만으로 통합 후보의 최종 head를
승인하지 않는다.

## 최신 Native Visual Sweep 증적

승인분 통합 후보의 review 전용 실행 파일과 **이미 추적된** 기준 PDF로 다시 산출했다.
새 PDF 변환은 하지 않았다.

| 항목 | 값 |
| --- | --- |
| 입력 | `samples/issue7062/tac_object_host_line_height.hwp` |
| 기준 PDF | `pdf/tac_object_host_line_height-2020.pdf` |
| 입력 SHA-256 | `2cf764c89943a23eff17fb8ac5ccaa1958711216b15d5eb29a9a469b97d23abb` |
| 원본 기준 PDF SHA-256 | `f90ea6915a842ac2266f4dd737b2829bbb3b72b927b1658577f6ca8c8b9b6051` |
| 입력 커밋 확인 | 둘 다 후보의 `HEAD` tree에 포함됨 (`git cat-file -e HEAD:<path>` 확인) |
| 비교 쪽 | 10 |
| 산출물 | 아래 링크 |

- [pr7370_native_review_010.png](../assets/planet_approved_20260923/pr7370_native_review_010.png)
- [pr7370_native_overlay_010.png](../assets/planet_approved_20260923/pr7370_native_overlay_010.png)

p10 pixel 90.007%, 엄격 내용 픽셀 50.525%, 2px 이웃 관용 내용 실루엣 99.573%. 끝 조각 box·뒤 표의 세로 위치를 직접 판독한다.

모든 PR의 실행 명령과 증적 목록은 [승인분 통합 후보 Native Visual Sweep 증적](../assets/planet_approved_20260923/README.md)에 있다.

## Merge 후 contributor PR comment 계획

최종 head CI와 실제 merge SHA가 확정된 뒤 원 PR에 한국어로 다음을 게시한다.

- 실제 merge SHA와 새 통합 PR의 최종 head CI URL, 이 PR의 원 contributor head를 구분해 기록한다.
- 위의 변경 계약·적용 순서·검증 범위와 남은 차이가 있으면 그 차이를 함께 설명한다.
- [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결한다.
- 이 PR의 최신 Native review·overlay를 실제 merge commit 아래 경로로 Markdown 이미지로 모두 표시한다.

  - `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/planet_approved_20260923/pr7370_native_review_010.png`
  - `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/planet_approved_20260923/pr7370_native_overlay_010.png`

  위 PNG는 `samples/issue7062/tac_object_host_line_height.hwp`와 이미 추적된 `pdf/tac_object_host_line_height-2020.pdf`를 재사용해 최종 통합 후보에서 다시 산출한 것이다.
