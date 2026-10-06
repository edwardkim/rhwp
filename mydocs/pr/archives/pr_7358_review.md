# PR #7358 리뷰 — 선언 frame overflow의 신뢰 상한

## 최종 판정

**메인터너 보정 후 수용 가능.** 원 head의 p28에서는 마지막 RowBreak 표 조각이 저장된 상단
바깥 여백 141HU(1.88px)를 다시 열지 않아 표와 뒤따르는 표가 함께 위로 치우쳤다. 통합 후보에서
조판 예산과 그리기를 함께 보정한 뒤 기준 PDF의 두 표 괘선과 일치했고, Native 2px 이웃 실루엣
일치율은 82.869%에서 **94.348%**로 올라 90% gate를 예외 없이 통과했다. 최신 통합 head의 전체
검증과 CI가 완료되기 전에는 merge하지 않는다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#7358](https://github.com/edwardkim/rhwp/pull/7358) |
| 작성자 | `planet6897` |
| 관련 이슈 | [#7288](https://github.com/edwardkim/rhwp/issues/7288) |
| 원 contributor head | `c4145a97703b` |
| 승인분 통합 PR | 새 `upstream/devel` 기반 PR 준비 중 (`codex/planet-approved-clean-20260924`) |
| reviewer | `jangster77` |

## 변경과 통합 순서

저장된 frame overflow 값을 무제한으로 신뢰하지 않고 실제 조판에 사용할 수 있는 상한으로 제한한다.

#7357·#7378 등 승인분과 함께 적용했다. #7378과 같은 표 분배 경로를 사용하지만 직접 충돌은
없었다. 원 head가 놓친 마지막 분할 행의 상단 여백은 통합 후보에서 메인터너가 보정했다.

`samples/86712_regulatory_analysis.hwp`의 본문 문단 172는 28행 RowBreak 표이고
`outer_margin_top=141HU`다. 28쪽에서는 마지막 행을 셀 내용 중간부터 이어 그리므로
`src/renderer/float_placement.rs`에 native HWP5·자리차지·문단 기준·다중 행·3열 이상·마지막 행
내부 컷·상단 여백 양수·세로 오프셋 0 조건을 둔다. 이 조건을
`src/renderer/typeset/table/continuation/fragment/budget.rs`의 예약과
`src/renderer/layout/table_partial.rs`의 그리기에 같이 적용한다. 일반 RowBreak 표 전체에
바깥 여백을 반복하면 기존 #2097 반례가 깨지므로 범위를 넓히지 않았다.
거대 단일열 표(#2214)는 마지막 행이 여러 쪽을 차지하고, 2열 표(76076 34쪽)는
기존 조각 위치가 이미 PDF와 맞는다. 첫 전수 회귀에서 두 반례가 드러나
열 수 게이트를 추가한 뒤 같은 검증을 다시 수행한다.

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
| #7358 p28 회귀 | 수정 전 FAIL → 보정 후 1 PASS |
| #7365 TAC 표 focused 6개 | 최종 후보 전체 회귀에서 6 PASS |
| Native Skia 전체 lib 및 실물 fixture 2종 | 최종 후보에서 lib 3,930 PASS (13 ignored)·workspace lib 182 PASS·실물 #2225 2 PASS·p37 4 PASS |
| Clippy·workspace 빌드·fresh WASM | 최종 후보에서 기본/wasm32/workspace all-targets Clippy, workspace 빌드, 저장소 루트 `scripts/wasm-pack-locked.sh --target web --out-dir pkg` 모두 통과 |
| Native Visual Sweep 대표 쪽 | 현 후보 binary SHA-256 `a220756502525135bd866292aed0163a297edd50c20cadfba7963ee926ea5133`로 p28 재산출, 94.348% PASS |

## 증적과 범위

추적된 증적 경로: `mydocs/report/issue7288-declared-frame`

이 기록은 원 PR의 최신 head를 최신 `upstream/devel` 기준으로 누적 적용한 통합 후보를 대상으로 한다.
원 PR 단독 CI와 통합 후보 CI는 구분하며, source PR의 merge 가능 상태만으로 통합 후보의 최종 head를
승인하지 않는다.

## 최신 Native Visual Sweep 증적

승인분 통합 후보의 review 전용 실행 파일과 **이미 추적된** 기준 PDF로 다시 산출했다.
새 PDF 변환은 하지 않았다.

| 항목 | 값 |
| --- | --- |
| 입력 | `samples/86712_regulatory_analysis.hwp` |
| 기준 PDF | `pdf/86712_regulatory_analysis-hwp-2024.pdf` |
| 입력 SHA-256 | `ee82c7755617003cb972ba398da9cffadfed24ac0fa068eee1a5347da7658a88` |
| 원본 기준 PDF SHA-256 | `bc1025b0607bbac01fea960997fa54430fd8dcc2604831b02940e7815cbcf84f` |
| 입력 커밋 확인 | 둘 다 후보의 `HEAD` tree에 포함됨 (`git cat-file -e HEAD:<path>` 확인) |
| 비교 쪽 | 28 |
| 산출물 | 아래 링크 |

- [pr7358_native_review_028.png](../assets/planet_approved_20260923/pr7358_native_review_028.png)
- [pr7358_native_overlay_028.png](../assets/planet_approved_20260923/pr7358_native_overlay_028.png)

p28 수정 후 pixel 90.686%, 엄격 내용 픽셀 14.039%, 2px 이웃 관용 내용 실루엣 **94.348%**다.
원본 HWP 표의 `outer_margin_top=141HU`인데 수정 전 이어진 조각은 본문 상단 75.60px에 그려졌다.
한컴 PDF 벡터의 첫 괘선은 77.51px, 뒤따르는 표의 첫 괘선은 541.49px다. 수정 후 render tree는
각각 **77.48px**, **541.85px**이며, 두 표의 열·행 경계도 새 overlay에서 직접 확인했다.
이 차이는 글꼴만으로 설명할 수 없으므로 이전 글꼴 예외 판정을 폐기했다. 남은 글자 래스터
차이는 엄격 픽셀 점수에 나타나지만 90% 실루엣 gate를 통과하는 데 예외가 필요하지 않다.

모든 PR의 실행 명령과 증적 목록은 [승인분 통합 후보 Native Visual Sweep 증적](../assets/planet_approved_20260923/README.md)에 있다.

## Merge 후 contributor PR comment 계획

최종 head CI와 실제 merge SHA가 확정된 뒤 원 PR에 한국어로 다음을 게시한다.

- 실제 merge SHA와 새 통합 PR의 최종 head CI URL, 이 PR의 원 contributor head를 구분해 기록한다.
- 위의 변경 계약·적용 순서·검증 범위와 남은 차이가 있으면 그 차이를 함께 설명한다.
- [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결한다.
- 이 PR의 최신 Native review·overlay를 실제 merge commit 아래 경로로 Markdown 이미지로 모두 표시한다.

  - `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/planet_approved_20260923/pr7358_native_review_028.png`
  - `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/planet_approved_20260923/pr7358_native_overlay_028.png`

  위 PNG는 `samples/86712_regulatory_analysis.hwp`와 이미 추적된 `pdf/86712_regulatory_analysis-hwp-2024.pdf`를 재사용해 최종 통합 후보에서 다시 산출한 것이다.
