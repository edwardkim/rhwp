---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-14
---

# PR #7103 — 연속 TAC 표 겹침 검토

**최종 판정: 메인터너 보정 후 수용 가능.** 원 head의 음수 clamp는 그대로 승인하지 않는다.
실제 control의 저장 줄 소속과 조판·배치의 공통 흐름 높이를 보정한 통합 PR #7110을 검증했다.
원 저자와 source SHA를 보존했고, 원 PR #7103은 통합 PR 병합 후 반영 근거와 함께 종료할 대상이다.

## 1. 대상과 경로

| 항목 | 확인값 |
| --- | --- |
| 원 PR / 실제 issue | [#7103](https://github.com/edwardkim/rhwp/pull/7103) / [#7096](https://github.com/edwardkim/rhwp/issues/7096) |
| 작성자 / reviewer | winchoose, 첫 rhwp PR / jangster77 사전 지정 |
| source head | `1c5fd967674e238b3d4e42948c1abe5d4c279f6f` |
| 기준 devel | `cd2d9e8a430e2664181326c684b7491aa451cd31` |
| 체리픽 | `d92086eec6fa0f053e3fc50f7845aedc0202dd27`; 원 SHA `-x`·저자 유지, 충돌 없음 |
| 최초 보정 | `6bb907fa9` |
| 최종 보정 / 통합 code head | `ef489fd0290b96706f8fda265ed2f19b3265601a` |
| 통합 PR | [#7110](https://github.com/edwardkim/rhwp/pull/7110), `fix/7096-tac-line-ownership-20260913` → `devel` |
| 로컬 검토 | `review/pr7103-20260913`, clean 주 작업공간과 전용 `target/pr7103-review-20260913` |
| 최초 기준 PDF·이미지 | `37245bd59`; 최종 비교 패널은 이 trailing commit에서 교체 |
| 절차 | collaborator_external_pr / intake_and_review / first_time_contributor / local_validation / visual_fixture_evidence |

작업지시자는 원 PR 코멘트, 체리픽 메인터너 보정, 검증·통합 PR 생성을 승인했다. owner에게 리뷰를
자동 요청하지 않았다. source fork의 contributor commit을 rewrite하거나 실제 push하지 않았다.

**fork 권한 판단 정정:** 최초 `winchoose/rhwp` repository API의 일반 `push: false`와 PR의
`maintainer_can_modify: true`는 별개다. source 그대로의 no-op dry-run은 쓰기 증거가 아니다.
비LFS 빈 자식 commit `b14e522b2`의 일반 dry-run은 LFS lock 인증 오류였으나, 9.3.0절에 따라
변경 파일·신규 LFS object가 없음을 확인한 `GIT_LFS_SKIP_PUSH=1` fast-forward dry-run은 exit 0이었다.
실제 ref update/LFS upload 성공 증거는 아니다. 체리픽은 승인된 최신 devel 통합 경로이며,
“fork 권한 때문에 체리픽만 가능”이라는 초기 판단을 정정했다.

[원 PR 코멘트](https://github.com/edwardkim/rhwp/pull/7103#issuecomment-5653812325)에 이 구분과 통합 경로를
게시하고, `jangster77`을 fork collaborator의 **Write(push)** 권한으로 초대해 달라는 요청을 추가했다.
API로 게시 내용을 재확인했다. 요청 후 2026-09-14 KST 재조회에서는 **`push: true`**를 확인해
[원 PR 후속 코멘트](https://github.com/edwardkim/rhwp/pull/7103#issuecomment-5654150897)와 통합 PR 설명도 갱신했다. 이미 검증한 통합 이력은 #7110에서 이어간다.

## 2. 보류 사유와 해소

### F1 — 잘못 고른 LineSeg의 간격을 clamp: 해소

빈 carrier의 `control_text_positions()`는 편집 논리 위치다. SectionDef/ColumnDef/Header/Footer는
이 위치를 늘리지 않지만 원시 HWP UTF-16 스트림에서는 각각 8칸을 점유한다.

| control index | 종류 | 원시 UTF-16 | 논리 위치 | 보정 전 투영 | 올바른 LineSeg |
| ---: | --- | ---: | ---: | ---: | ---: |
| 0 | SectionDef | 0 | 0 | 0 | 0 |
| 1 | ColumnDef | 8 | 0 | 0 | 0 |
| 2 | Header | 16 | 0 | 0 | 0 |
| 3 | 제목 TAC 표 | 24 | 0 | 0 | 1 |
| 4 | Footer | 32 | 1 | 8 | 2 |
| 5 | 본문 TAC 표 | 40 | 1 | 8 | 3 |

원시 PARA_TEXT를 독립 해제해 확인한 값이며, 문서별 24/40 상수를 제품 코드에 넣지 않았다.
보정 전 두 표 모두 LineSeg 0을 선택해 `208-(0+7527)=-7319HU=-97.5867px`를 구했다.
원 PR은 이 잘못된 차이만 0으로 clamp했다.

보정은 완전한 빈 control stream 계약에서 원시 위치를 복구하고 공통 `control_line_seg_index`가
사용하도록 했다. 실제 다음 TAC 표의 소유 줄을 찾아 Header/Footer 줄을 건너뛰고, 저장 top·점유 end·
다음 pen을 조판과 배치가 공유한다. 마지막 줄의 signed spacing도 보존한다. 표의 선언 높이+바깥여백이
소유 줄 높이와 같고 실측 높이가 유지되며 같은 단에 모두 들어오는 경우에만 적용한다.
HWPX 구역 머리의 재기준화 축은 개수 역산에서 제외한다. 앞 문단 뒤 저장 시작 위치도 fit/paint가
함께 보존한다. 같은 줄 개체·성장 표·편집/재조판·분할은 기존 경로를 유지한다.
[구현 및 계약](pr_7103_review_impl.md)에 적용·비적용 조건을 기록했다.

### F2 — 비겹침만 확인하고 한컴 경계 검증 없음: 해소

근거 없는 “gap ≤2px”를 제거하고 독립 한컴 PDF 경계를 0.5px 이내로 검증했다.

| p.1 경계 | 한컴 y (pt) | 원 PR y (px) | 보정 y (px, 96dpi) |
| --- | ---: | ---: | ---: |
| 제목 하단 | 116.992 | 153.3 | 156.0 |
| 개인정보 내부 표 상단 | 151.994 | 197.2 | 202.9 |
| 튜터 색상 행 상단 | 286.487 | 376.8 | 382.4 |

표시값은 0.1px 반올림이며 테스트는 원래 f64값으로 판정한다. 원 PR의 약 4pt 잔여 오차가 해소됐다.
새 4개 테스트는 원 clamp 코드를 대입한 대조군에서 **4 FAIL**, 보정 코드에서 **4 PASS**였다.
원 HWP의 구조 제어 순서를 유지한 메모리 변형으로 양수 +600HU와 음수 -100HU 간격도 검증했다.
입력 좌표나 PDF/golden을 바꿔 실물 검사를 통과시킨 것이 아니다.

### F3 — closing 번호·최종 테스트 불일치: 통합 PR에서 해소

원 PR의 `closes #7103`과 삭제된 단위 테스트 실행 설명을 통합 PR에 복사하지 않았다.
통합 PR은 실제 해결 issue에 **Closes #7096**을 지정하고 아래 최종 실행 결과를 기록했다.
원 PR의 원본 설명을 사후 작성자의 기록처럼 rewrite하지 않았다.

## 3. 공통 조판 원칙과 입력 커밋

| 항목 | 최종 근거 | 판정 |
| --- | --- | --- |
| 구현 근거와 일반성 | 완전한 저장 control stream·줄 높이 계약; 문서 ID/좌표 상수 분기 없음 | 충족 |
| 측정·배치 일관성 | 동일 stored plan의 top/advance_end를 typeset와 layout이 사용 | 충족 |
| 줄 소속과 점유 높이 | 공통 소유 줄 조회, 실제 다음 표의 줄, 높이·fit 조건 및 비적용 경로 | 충족 |
| 사례와 증거의 독립성 | 실제 HWP·독립 한컴 PDF, 경계 회귀 4개·음성 대조·관련 TAC 회귀 | 충족 |
| 기준값 변경 | 기존 baseline/golden/래칫 변경 없음; PDF 경계 오차를 확대하지 않음 | 충족 |
| 주장과 검증 범위 | 원 head와 보정 head 구분, 실제 최종 검사/CI, 글꼴·텍스트층 차이 별도 | 충족 |

검증 입력은 [입력 manifest](../assets/issue_7103/README.md)의 Git blob/SHA와 일치했다.
source에 있는 HWP와 기존 HWP/HWPX는 원래 경로로 재사용했다. 한컴 PDF만 중복 부재를 확인해
`pdf/ari-tutoring-application-2020.pdf`로 보존했다. 다운로드 폴더에만 의존하는 입력은 없다.

## 4. 최종 보정 head 검증

| 검사 | 결과 |
| --- | --- |
| Rust lint | prepare → fmt check → native/WASM32/workspace all-target Clippy 및 workspace build → manifest check 모두 통과 |
| focused | #7103 4개 + #6078/#6181/#7049/#6754/#6972, 모두 통과 |
| 전체 release-test | Summary [ 320.358s] 9753 tests run: 9753 passed (2 slow), 51 skipped |
| Native Skia | lib: 4,112 PASS / 13 ignored / 0 FAIL; placeholder: Summary [   1.177s] 2 tests run: 2 passed, 201 skipped; direct PDF: Summary [   0.743s] 4 tests run: 4 passed, 197 skipped |
| WASM | fresh host --no-opt build 성공; Node native/WASM SVG p.1 일치 및 Chrome 실제 WASM 실행·시각 확인 (Docker daemon 부재로 최적화 배포 build 미실행) |
| OVR | devel 대비 5문서 142페이지/48표, 기하 회귀 0, 기존 2px tolerance |
| fidelity | p.1 완료1/누락0, body/각주·표/footer/frame/표셀 겹침·표셀 경계 후보0 |
| Visual Sweep | devel → 원 PR → 보정: flagged 1→0→0, pixel 78.78501→87.38109→92.42056%, ink 13.62244→36.66524→56.36344% |
| 최종 code CI | [CI](https://github.com/edwardkim/rhwp/actions/runs/34765219360) 성공, [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34765219369) 성공, [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/34765219141) 성공, [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/34765219329) 성공, [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/34765219351) 성공, [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/34765219113) 성공 |

기존 source CI 34757985771 성공은 원 head의 증거로만 보존했다. 위 로컬 전체·Native·lint·WASM 및
통합 CI는 메인터너 보정 코드에 대해 다시 수행했다. 파생 suite는 제출하지 않았다.
source-side cfg(test)/Studio/편집 명령/npm/workflow 변경은 없다. 성능은 별도 벤치마크하지 않았다.

[3-way](../assets/pr7103_tac_review_p001_3way.png)는 한컴 / 원 PR / 보정 순서이며,
[OVL](../assets/pr7103_tac_review_p001_ovl.png)은 R=한컴 gray, G=B=보정 gray다.
제목·개인정보·튜터/튜티 표·서명·하단 로고를 페이지 전체에서 직접 확인했다. 글꼴 모양·획 굵기와
PDF 텍스트층의 c/d/e/f/g 각10개 reference-only 항목은 별개이며, 가시 본문 누락 또는 문자 완전 일치로
주장하지 않는다. 5개 OVR 문서는 새로운 한컴 raster 비교가 아니라 기존 문서의 기하 회귀 검사다.

## 5. 남은 원격 절차와 merge 후 코멘트

이 문서·오늘할일·최종 패널은 코드 CI 성공 뒤 같은 통합 PR의 trailing commit으로 추가했다.
작업지시자의 “CI 모니터링 후 후속처리” 승인을 받았다. 최종 trailing head의 CI·fast-pass·
MERGEABLE/CLEAN을 확인한 뒤 병합과 후속 처리를 진행한다. 현재 병합 완료로 기록하지 않는다.

병합 시 원 PR에 통합 PR/merge SHA와 [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md),
입력·검증 범위·대표 3-way/OVL을 알린 뒤 #7103을 종료한다. 이미지는
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/pr7103_tac_review_p001_3way.png`
형식의 고정 SHA 링크를 사용하고 게시 후 API로 재확인한다. #7096의 실제 종료 상태도 확인한다.
post_merge 절차의 devel 동기화·duration refresh·이번 소유 target/branch 정리가 남으며, contributor fork
branch와 공유 산출물은 삭제하지 않는다.
