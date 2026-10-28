# PR #7378 리뷰 — cell unit split의 value1 행 원자성·빈 orphan 방지

## 최종 판정

**원 PR 개별 승인.** 새 승인분 후보에서 `156482639_startup_ir_contest.hwp`와 Hancom 2020 PDF의 Native 5–8쪽을 다시 비교했다. 관용 내용 실루엣 평균 99.629%, 최저 99.355%이고, p6이 새 행에서 시작한다. 이 대표 쪽의 점수만으로 전체 후보의 회귀·CI 통과를 선언하지 않는다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#7378](https://github.com/edwardkim/rhwp/pull/7378) |
| 작성자 | `planet6897` |
| 관련 이슈 | [#7288](https://github.com/edwardkim/rhwp/issues/7288) |
| 원 contributor head | `0f6ebd933425` |
| 승인분 통합 PR | 새 `upstream/devel` 기반 PR 준비 중 (`codex/planet-approved-clean-20260924`) |
| reviewer | `jangster77` |

## 변경과 통합 순서

표 셀 unit 분할에서 value1 행을 원자적으로 유지하고 빈 orphan 조각을 만들지 않도록 한다.

#7357·#7358 뒤에 적용했다. #7357의 0 경계 보호를 전제로 #7288 실물 문서를 검증했다.

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
| Native Visual Sweep 대표 쪽 | 현 후보 binary SHA-256 `a220756502525135…`로 재산출 완료; 최저 99.355%, p5–p8 |

## 증적과 범위

추적된 증적 경로: `mydocs/report/issue7288-cell-unit`

이 기록은 원 PR의 최신 head를 최신 `upstream/devel` 기준으로 누적 적용한 통합 후보를 대상으로 한다.
원 PR 단독 CI와 통합 후보 CI는 구분하며, source PR의 merge 가능 상태만으로 통합 후보의 최종 head를
승인하지 않는다.

## 최신 Native Visual Sweep 증적

승인분 통합 후보의 review 전용 실행 파일과 **이미 추적된** 기준 PDF로 다시 산출했다.
새 PDF 변환은 하지 않았다.

| 항목 | 값 |
| --- | --- |
| 입력 | `samples/issue6132/156482639_startup_ir_contest.hwp` |
| 기준 PDF | `pdf/issue6132/156482639_startup_ir_contest-2020.pdf` |
| 입력 SHA-256 | `40df46aa25961c9360d9f4187d93c0af596739a6b150e9f6da0eae08cf612825` |
| 원본 기준 PDF SHA-256 | `db40972a820e2e882d16b3ac73ca75f5b44fd5ac09dfed33329a2f954f6d1da8` |
| 입력 커밋 확인 | 둘 다 후보의 `HEAD` tree에 포함됨 (`git cat-file -e HEAD:<path>` 확인) |
| 비교 쪽 | 5–8 |
| 산출물 | 아래 링크 |

- [pr7378_native_review_005.png](../assets/planet_approved_20260923/pr7378_native_review_005.png)
- [pr7378_native_overlay_005.png](../assets/planet_approved_20260923/pr7378_native_overlay_005.png)
- [pr7378_native_review_006.png](../assets/planet_approved_20260923/pr7378_native_review_006.png)
- [pr7378_native_overlay_006.png](../assets/planet_approved_20260923/pr7378_native_overlay_006.png)
- [pr7378_native_review_007.png](../assets/planet_approved_20260923/pr7378_native_review_007.png)
- [pr7378_native_overlay_007.png](../assets/planet_approved_20260923/pr7378_native_overlay_007.png)
- [pr7378_native_review_008.png](../assets/planet_approved_20260923/pr7378_native_review_008.png)
- [pr7378_native_overlay_008.png](../assets/planet_approved_20260923/pr7378_native_overlay_008.png)

p5 pixel 91.592%·관용 99.856%, p6 pixel 90.822%·관용 99.610%. 표가 행 한가운데 끊기지 않고 p6이 새 행에서 시작하는지 직접 판독한다.

모든 PR의 실행 명령과 증적 목록은 [승인분 통합 후보 Native Visual Sweep 증적](../assets/planet_approved_20260923/README.md)에 있다.

## Merge 후 contributor PR comment 계획

최종 head CI와 실제 merge SHA가 확정된 뒤 원 PR에 한국어로 다음을 게시한다.

- 실제 merge SHA와 새 통합 PR의 최종 head CI URL, 이 PR의 원 contributor head를 구분해 기록한다.
- 위의 변경 계약·적용 순서·검증 범위와 남은 차이가 있으면 그 차이를 함께 설명한다.
- [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결한다.
- 이 PR의 최신 Native review·overlay를 실제 merge commit 아래 경로로 Markdown 이미지로 모두 표시한다.

  - `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/planet_approved_20260923/pr7378_native_review_005.png`
  - `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/planet_approved_20260923/pr7378_native_overlay_005.png`
  - `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/planet_approved_20260923/pr7378_native_review_006.png`
  - `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/planet_approved_20260923/pr7378_native_overlay_006.png`
  - `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/planet_approved_20260923/pr7378_native_review_007.png`
  - `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/planet_approved_20260923/pr7378_native_overlay_007.png`
  - `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/planet_approved_20260923/pr7378_native_review_008.png`
  - `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/planet_approved_20260923/pr7378_native_overlay_008.png`

  위 PNG는 `samples/issue6132/156482639_startup_ir_contest.hwp`와 이미 추적된 `pdf/issue6132/156482639_startup_ir_contest-2020.pdf`를 재사용해 최종 통합 후보에서 다시 산출한 것이다.
