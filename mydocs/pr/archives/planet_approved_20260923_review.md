# planet6897 승인분 통합 후보 검토

## 최종 판정

**로컬 검증 통과 — 새 통합 PR 제출 가능.** Native 대표 캡처 12쪽, 전체 release-test 10,193개, Native Skia, Rust lint·빌드, fresh WASM을 이 후보에서 통과했다. 실제 merge는 새 PR의 최종 head CI 확인 뒤 판정한다. 기존 #7380의 결과를 새 후보의 통과 근거로 복사하지 않았다.

## 대상과 출처

- 기준: `upstream/devel` `c84a8f8e1a97acaf8626b28a14417344a8578f41`.
- 검토 브랜치: `codex/planet-approved-clean-20260924`.
- 닫힌 혼합 후보: [#7380](https://github.com/edwardkim/rhwp/pull/7380). 이 PR의 head·CI·시각 증적은 새 후보의 최종 승인 근거로 재사용하지 않는다.
- 적용: [#7357](pr_7357_review.md), [#7358](pr_7358_review.md), [#7359](pr_7359_review.md), [#7365](pr_7365_review.md), [#7369](pr_7369_review.md), [#7370](pr_7370_review.md), [#7373](pr_7373_review.md), [#7374](pr_7374_review.md), [#7378](pr_7378_review.md). 개별 리뷰에 원 contributor head와 판정·merge 후 comment 계획을 기록한다.
- 제외: #7366·#7371은 90% Visual Sweep gate 미달로 재검토 중이다. #7372는 원 PR 개별 판정은 승인이나 #7371 변경에 의존하므로 별도 묶음으로 남긴다.
- 남은 세 PR의 재검토 순서·해제 조건은 [별도 후속 계획](planet_deferred_20260924_plan.md)에 기록한다.

## 변경 범위

원 PR 9개의 변경과 #7365 TAC 표의 ViewText 시작 inset·선언 행 높이·단위 문구 위치·제목/목록 같은 쪽 배치, #7358 p28 마지막 표 조각의 상단 바깥 여백, #7359 p14 페이지 첫 문단의 저장 간격을 위한 메인터너 보정을 포함한다. Visual Sweep의 한국어 review 코멘트, 2px 이웃 실루엣 집계·90% 제출 gate, `AGENTS.md`·`CLAUDE.md`·`CONTRIBUTING.md`와 연결된 개발자 지침도 포함한다. 보류된 1×1 RowBreak·표 fragment 후속 보정은 포함하지 않는다.

## 검증

| 검사 | 새 후보 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | 최종 코드 기준 통과 |
| `python3 -m unittest scripts/tests/test_visual_sweep.py` | 67 PASS |
| generated-suite manifest / unit-tier 정책 | 1,414 sources / 6,063 static tests 및 4,205 unit tests 확인 |
| generated-suite 정책 자체 테스트 | 23 PASS |
| release-test 전체 | 최종 보정 후보에서 **10,193 PASS / 실패 0 / skip 50** (521.763초) |
| #7358 p28 표 위치 회귀 | 수정 전 FAIL → 보정 후 1 PASS; 두 표 괘선의 PDF 좌표 확인 |
| #7359 p14 첫 문단·표 위치 회귀 | 수정 전 FAIL → 보정 후 1 PASS; 표 첫 괘선 178.5 → 193.2px, PDF 193px |
| #7365 TAC 표 focused | 전체 실행 안에서 6 PASS |
| Native Skia | 전체 lib 3,930 PASS (13 ignored), workspace lib 182 PASS; 실물 #2225 2 PASS, p37 4 PASS |
| Clippy·workspace 빌드·fresh WASM | 기본/wasm32/workspace all-targets Clippy, workspace 빌드, 저장소 루트 `scripts/wasm-pack-locked.sh --target web --out-dir pkg` 통과 |
| Native Visual Sweep | 최종 보정 binary SHA-256 `a220756502525135bd866292aed0163a297edd50c20cadfba7963ee926ea5133`로 대표 12쪽을 재산출했다. 모두 90% gate 통과, #7359 p14는 68.17% → 97.48%. [증적 목록과 수치](../assets/planet_approved_20260923/README.md) |

이전 혼합 head에서 얻은 통과 수치나 캡처를 여기의 완료 결과로 세지 않는다. 최종 새 후보 head CI와 실제 merge SHA가 확인되기 전까지 통합 merge 승인은 유보한다.

## 리뷰어에게 보여줄 대표 이미지

| 원 PR | 대표 Native review | overlay |
| --- | --- | --- |
| #7357 | [63↔62](../assets/planet_approved_20260923/pr7357_native_review_rhwp063_pdf062.png) | [63↔62](../assets/planet_approved_20260923/pr7357_native_overlay_rhwp063_pdf062.png) |
| #7358 | [28](../assets/planet_approved_20260923/pr7358_native_review_028.png) | [28](../assets/planet_approved_20260923/pr7358_native_overlay_028.png) |
| #7359 | [14](../assets/planet_approved_20260923/pr7359_native_review_014.png) | [14](../assets/planet_approved_20260923/pr7359_native_overlay_014.png) |
| #7365 | [3](../assets/planet_approved_20260923/pr7365_native_review_003.png) | [3](../assets/planet_approved_20260923/pr7365_native_overlay_003.png) |
| #7370 | [10](../assets/planet_approved_20260923/pr7370_native_review_010.png) | [10](../assets/planet_approved_20260923/pr7370_native_overlay_010.png) |
| #7373 | [95↔94](../assets/planet_approved_20260923/pr7373_native_review_rhwp095_pdf094.png) | [95↔94](../assets/planet_approved_20260923/pr7373_native_overlay_rhwp095_pdf094.png) |
| #7374 | [1](../assets/planet_approved_20260923/pr7374_native_review_001.png) | [1](../assets/planet_approved_20260923/pr7374_native_overlay_001.png) |
| #7378 | [5](../assets/planet_approved_20260923/pr7378_native_review_005.png), [6](../assets/planet_approved_20260923/pr7378_native_review_006.png), [7](../assets/planet_approved_20260923/pr7378_native_review_007.png), [8](../assets/planet_approved_20260923/pr7378_native_review_008.png) | [5](../assets/planet_approved_20260923/pr7378_native_overlay_005.png), [6](../assets/planet_approved_20260923/pr7378_native_overlay_006.png), [7](../assets/planet_approved_20260923/pr7378_native_overlay_007.png), [8](../assets/planet_approved_20260923/pr7378_native_overlay_008.png) |

#7369는 PDF fixture 교체만 수행하므로 별도의 Visual Sweep 이미지를 판정 근거로 삼지 않는다. #7358·#7359는 표 기하 차이를 메인터너 보정으로 해결해 예외 없이 gate를 통과했다. 대표 이미지의 각도·표 경계·행 순서를 직접 판독하고, 점수만으로 승인하지 않는다.
