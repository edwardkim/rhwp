# PR #7585 리뷰 — #7337 중첩 TAC 표의 저장 줄 배치 통합

## 최종 판정

**승인 — 검증한 코드 후보 `2f01f2788ee6dbf71eaa0157c3481d66048f6dd9`.**

원 기여의 재현·X 누적 진단을 수용하고 메인테이너가 저장 줄 소속·실제 Y 원점·바깥여백을 보정했습니다. 아래 exact-head Full CI 완료 증거를 확인했습니다. 이 문서는 제품 source를 변경하지 않는 후행 기록입니다. 후행 head의 CI와 current-base 충돌 검증을 확인한 뒤 사용자 승인 범위에서 merge합니다.

## 접수와 승인 범위

- [통합 #7585](https://github.com/edwardkim/rhwp/pull/7585), 작성자 edwardkim, base devel. reviewer를 별도로 지정하지 않는 self-review입니다.
- 원 [#7337](https://github.com/edwardkim/rhwp/pull/7337)의 head `371abaac7a00185c8818a16f69c3ec0296990577`과 fork `seongeun82/rhwp:fix/pdf-text-loss`는 보존했습니다. 원 author를 보존한 `c6f87c47bf0f431e1f988877e30449bc8b3bce8e`, 보정 source `64b3d366c3d6563b2d9ed1eba1eb869d99c0eb7a`를 분리했습니다.
- 초기 base e0f6b094, push 전 최신 base `ea5ef5f6a8e405f5109407e46acb4e9a0455c24a`를 반영했습니다. 원 contributor fork에 devel을 병합하지 않았습니다.
- 기본 경로 collaborator_self_merge, 보조 intake_and_review, local_validation, visual_fixture_evidence, review_only_fast_pass, post_merge를 적용했습니다. 원 PR은 maintainer_general/first_time_contributor/rework_and_exceptions 경로로 접수했습니다.
- 2026-10-05 사용자가 보정 후 PR 처리 진행을 승인했습니다. 관련 #7336은 이미 닫혀 있으며 포괄적인 새 이슈 종료 표현을 사용하지 않습니다.

## 조판 검토와 범위

[원 PR의 독립 기대값·생산/측정/배치 소비 경로·수정 전후·검증 기록](pr_7337_review.md)을 정본으로 연결합니다. 같은 정보를 새 보고서로 복제하지 않습니다.

| 판정 | 실제 근거와 한계 |
| --- | --- |
| 충족 | 원 HWP의 control UTF-16 ts0/ts8과 유효한 저장 LineSeg, 독립 한컴 2020 PDF를 기준으로 줄 소속을 확인했습니다. 측정의 그룹과 배치의 폭·정렬·vpos가 같은 저장 소속을 소비하고, 뒤의 TextLine Y 덮어쓰기도 해당 소유 줄을 선택합니다. |
| 충족 | 부모가 직접 소유한 두 표의 전체 프레임 포함·세로 순서·안내/실적 내용 보존을 실제 최종 RenderNode에서 검사합니다. 같은 검사와 입력이 devel에서 2 FAIL, 보정 후 2 PASS입니다. |
| 충족 | 정상 같은 줄·그림 혼재·앞 공백/여백/줄 소속·#7518 분할 대조군 42 PASS. Native/fresh WASM 전 6쪽과 실제 Studio 출력에서 외곽·내용·뒤 행을 직접 확인했습니다. |
| 비해당 | 페이지 컷·예약 예산·rowspan 유닛 소비 규칙을 바꾸지 않았습니다. 저장 LineSeg 없는 새 너비 기반 줄 나눔은 구현 범위가 아닙니다. |
| 미검증 | 원 PR의 한컴 2024 제품 주장 자체는 확인하지 않았습니다. 판정에 사용한 독립 기준은 입력 전처리 없는 실제 한컴 2020 출력입니다. |

## 로컬·시각 검증

[입력/PDF/source/명령/결과/PNG 해시](../assets/pr7337/validation.json)에 실제 실행 기록을 남겼습니다. 검증 source는 64b3d366c이며 이후 base 반영에서 src/crates/Cargo의 동일성을 확인했습니다. 바뀐 의존성은 새 Vite 8.3.2 환경에서 다시 검증했습니다.

전체 Rust 10,355 PASS / 0 FAIL / 기존 50 skip, fmt·3 Clippy 경로·workspace build·고정 base manifest PASS, Native Skia library 4,109 PASS 및 integration 2+4 PASS, 새 쪽수 원장 16 partition PASS입니다. 새 fixture를 실제 보안 detector에 포함했고 신규 양수 래칫 행은 없습니다. 기존 baseline·golden·공차를 완화하지 않았습니다.

fresh 최적화 WASM의 pkg/public 해시가 일치하며 실제 Windows CDP 13/13 PASS입니다. Native/fresh WASM 전 6쪽의 2px 관용 실루엣 최저 99.30154%, 6쪽 99.9944%, 두 gate passed입니다. [Native review](../assets/pr7337/native_review_006.png)·[overlay](../assets/pr7337/native_overlay_006.png), [WASM review](../assets/pr7337/wasm_review_006.png)·[overlay](../assets/pr7337/wasm_overlay_006.png)를 직접 판독했고 PR 본문에도 head SHA 고정 이미지로 표시했습니다. 전체 6쪽과 실제 Studio 화면은 같은 asset 폴더에 보존했습니다.

엄격 내용 픽셀 지표 50.99281%와 남은 획/raster 차이를 공개하며, 관용 지표를 원시 픽셀 일치로 표현하지 않습니다. 1px 진단 공차의 프레임 최대 이탈은 570.8→1.4px이며 기본 2px body 래칫의 신규 발생은 0입니다. 이번 PR에 85% 예외를 적용하지 않았습니다.

## 코드 후보 Full CI

- 정확한 후보 `2f01f2788ee6dbf71eaa0157c3481d66048f6dd9`의 [CI 37270249144](https://github.com/edwardkim/rhwp/actions/runs/37270249144), [CodeQL 37270249063](https://github.com/edwardkim/rhwp/actions/runs/37270249063), [Render Diff 37270248850](https://github.com/edwardkim/rhwp/actions/runs/37270248850), [Adapter 37270249116](https://github.com/edwardkim/rhwp/actions/runs/37270249116), [Proptest 37270249055](https://github.com/edwardkim/rhwp/actions/runs/37270249055)가 모두 성공했습니다. 같은 repository·source branch·PR event·SHA를 확인했습니다.
- preflight는 `fast_pass=false / no-green-build-candidate`로 Full을 실행했습니다. trusted job 성공을 Full 실행 재사용으로 해석하지 않았습니다.
- archive A/B/C/D는 각각 3,870/2,218/2,024/2,049 PASS, 합계 **10,161 PASS / 0 FAIL / 기존 50 skip**입니다. 원 CI 실패가 있던 text-overlap partition 5와 중첩 표 검사 2개도 성공했습니다. Native Skia·Frontend package gates·Lint·Build & Test가 모두 성공했습니다.
- 실제 CI checkout `957d2423c911537959df00c2b8c79a019d4750db`, tree `02ca9daff3820f5231f3e83412aaa49672cd6d38`의 부모는 최신 base ea5ef5f6와 code candidate 2f01f2788이며, `git merge-tree --write-tree` 결과와 같습니다. lint job 로그의 checkout SHA까지 대조했습니다.
- 33개 check는 30 success / 3 skipped, CI Impact Policy success, 실패·대기 0, MERGEABLE/CLEAN입니다. [run·job·artifact·로그 해시·head 증거](../assets/pr7337/ci_candidate_2f01f2788.json)에 기록했습니다. 이 판정은 후행 head의 검증을 대체하지 않습니다.

이 뒤의 기록 commit은 archive review·CI 증거·오늘할일만 포함합니다. source/test/fixture/workflow/baseline/sample은 바꾸지 않습니다. 최신 head의 실제 preflight 재사용 근거와 required Build & Test 완료, MERGEABLE/CLEAN, 최신 base merge-tree를 확인합니다.

## Merge 후 contributor PR comment 계획

원 #7337의 [안내 계획](pr_7337_review.md#merge-후-contributor-pr-comment-계획)에 따라 첫 기여 감사와 원 기여/추가 보정을 한국어 존댓말로 구분해 설명합니다. #7585와 실제 merge SHA·CI·Visual Sweep 정본, 같은 asset의 merge SHA 고정 Native/WASM review·overlay 이미지 4개를 게시하고 API에서 UTF-8 본문을 재확인합니다. 실제 devel 포함과 duration 갱신 결과를 확인한 뒤 원 #7337을 대체 완료로 닫습니다. merge 방식으로 author 이력을 유지합니다. 이번 작업의 전용 branch/worktree/임시 로그는 소유·clean·활성 작업·영구 증적을 확인한 뒤 정리하고, contributor fork·공유 target·사용자 IDE에 열린 #7518 worktree는 보존합니다.
