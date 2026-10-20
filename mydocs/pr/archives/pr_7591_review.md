---
kind: investigation
status: historical
canonical: mydocs/pr/archives/pr_7591_review.md
last_verified: 2026-10-11
---

# PR #7591 리뷰 — 미주 단 수용과 잉크 하단

## 최종 판정

**메인터너 보정 후 수용 가능.** 원 기여와 누적 메인터너 보정을 포함한 통합 후보의 판정이다.
최종 통합은 아래에 기록한 남은 검증 조건을 완료한 뒤 진행한다.

미주 수용 판정의 실제 잉크 끝·부분 문단 소유를 원 기여와 누적 보정에 반영했다. 사용자 지정 시험지 검사는 별도 작업 범위로 분리했고, 일반 미주 검사는 전체 회귀에 포함했다.

## 현재 검증 근거 — 2026-10-11

| 항목 | 확인한 상태 |
| --- | --- |
| 검토 branch | `review/planet6897-20261008` |
| 통합 PR | [#7832](https://github.com/edwardkim/rhwp/pull/7832) → `devel`; 원 기여와 메인터너 보정 함께 제출 |
| rebase 후 통합 code head | `9b4817473b2a3515ea27eefef9d20731258433e4` |
| 최신 upstream base | `920dc29284e8a6e624861bfccf68903e19ae309b` |
| 전체 회귀 | 검증 source `7dd9572b7fecbc602c9e5ab47c4a6c9d6422b4bd`: 10,215PASS·실패0·skip45 |
| lint·정책 | fmt·Native/WASM/all-target Clippy·workspace build·base 고정 manifest/unit-tier PASS |
| Studio | 1,834PASS·실패0·skip2, TypeScript 검사 PASS |
| optimized fresh WASM | code source `03a987f4a`: 빌드exit0, `pkg/`·Studio `public/` JS/WASM SHA-256 일치 |
| 시각 선행 조건 | table-text·stored-vpos·hwp-multi·해외투자3입력의 Native/fresh WASM 전쪽 TSV 모두90% 이상. table-text·stored-vpos 변경 영역 확대 판독 완료 |
| 승인된 이관 | #7813 giant-cell, #7830 sample2 HWP, #7831 가정폭력 조사. 각 출력 문제는 후속 이슈에서 해결 |
| rebase 후 재검사 | 오늘할일만 충돌했고 양쪽 기록을 보존했다. 사용자 지시에 따라 통과한 전체 회귀·lint·Studio를 반복하지 않음 |

전체 결과·시각 점수·source와 로그는 [남은 회귀 보정 기록](../../plans/planet6897_20261008_remaining_regressions.md#현재-시각-검증-상태-정정--2026-10-11)에 연결한다.
위 시각 결과는 해당 입력과 기록한 renderer source의 증거이며, 원 PR 전체 영향 목록의 전수 통과를 주장하지 않는다.

## #7768 재확인과 통합 조건

#7768 원본 전1쪽의 Native/fresh WASM 최저92.73966%, 누락0을 확인했다.
양 경로 raster bytes와 review/overlay의 canonical gate `passed`를 확인하고 캡션·표·
체크박스3개·후행 용지 규격 줄을 확대 판독했다. [공개 시각 증적](../assets/issue7768/README.md)을
통합 PR의 `Closes #7768` 근거로 연결한다.

- 최신 upstream의 확장 프로그램 #7703·기존 오늘할일을 보존해 rebase했다. 충돌은 오늘할일 한 파일뿐이며 Rust/test/Cargo tree는 rebase 전과 같다.
- 사용자 지시에 따라 rebase 후 추가 로컬 테스트를 하지 않는다. 별도 로컬 Native Skia3종은 미실행으로 기록하고 통합 PR의 Native Skia CI 결과를 확인한다.
- 남은 통합 조건은 제출 head의 GitHub CI·CodeQL·Native Skia 및 mergeability 확인이다. 원격 CI 완료·merge를 미리 보고하지 않는다.

<details>
<summary>이전 검토 이력 — 당시 판정과 단계별 실행 기록</summary>

초기 검토에서 확인한 결함·증거 부족과 단계별 재검증 기록이다. 당시 보류 판단은 아래 이력의 source와 실행 범위에 한정하며, 현재 판정은 문서 상단을 따른다.

## 접수·적용 범위

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자·base | [#7591](https://github.com/edwardkim/rhwp/pull/7591) / planet6897 / devel |
| 최초 검토 head | `d5cc92dba07eb7c7db768d7a14fd72fb47e72417` |
| 최신 보완 head | `41c64bd552b5f44eaf340112628f340f1745b6a3`; 기존 검증을 이 head의 통과로 재사용하지 않음 |
| 최초 적용 commit | `c08ce6cf47b2aa6d07c0fb7d350f618b170ab848` |
| 이 PR의 적용 뒤 head | `cba840e0febbf392772b798a3b19832e9ecf4765`; 기능 commit 22개, 원 저자·cherry-pick -x 보존 |
| 누적 검증 source head | `90e2c6fd2c614ba0ea7667355c4ed2f3617f6563` |
| 최신 base 정렬 뒤 code head | `ae786f75b2cee72bd5576f6473bd51d9a8216f7c`; 제품/검증 입력 tree 동일 확인 |
| 검토 branch·reviewer | `review/planet6897-20261008` / jangster77; 원 PR reviewer assign 선행 |
| 원 PR 참고 상태 | draft=False, MERGEABLE / CLEAN; 작성 시점 snapshot이며 merge 직전 판정을 대신하지 않음 |
| 경로 | collaborator 외부 PR의 주 작업공간 누적 cherry-pick 9.1.1; intake/template/local_validation/visual_fixture_evidence 및 대형 PR 보조 절차 |

관련 이슈와 해결 표현은 원 PR 본문의 범위를 유지한다. 이번 보류 기록에 새로운 closes 표현을 추가하지 않는다. 원 PR 제공 before/after와 corpus 수치는 참고이며 통합 head의 독립 실행과 구별한다.

## 구현 주장·소비 경로·검증 판정

미주 수용 판정이 `typeset/notes/endnotes/measure.rs:194`의 `measure_endnote_render_ink_fit` → `simulate_endnote_column_extent:263` → scratch `LayoutEngine`의 실제 배치 결과를 소비한다. 전체 문단의 커서·잉크 끝과 부분 문단의 잉크 끝을 구별하고, 0.25px 허용치로 `Fits / SplitAt / NextColumn`을 결정한다. 뒤 제목 원점과 빈 줄 상자·구분선의 흐름 소비는 별도 분기이므로 helper 공유만으로 모두 검증됐다고 판정하지 않는다.

**미검증:** 원 PR이 보고한 전체 범위의 90% 미만 쪽과 미주 제목·본문 혼합 단, 각주/구분선·빈 줄의 직접 비교를 이 통합 head에서 전수 재확인하지 못했다. 전체 회귀의 endnote 검사 PASS는 독립 PDF 일치의 대용이 아니다. 원 PR의 overflow baseline 증가를 정상 변화로 승인하지 않으며 기존 테스트·증적은 보존한다. 대형 PR 경로를 적용하고 공통 통합 후보의 시각 gate 실패도 유지한다.

공통 빌드·회귀·글꼴 등록·383쪽 시각 결과와 입력/기준 해시·증적 보존·merge 후 댓글 계획은 [#7611 공통 통합 검증](pr_7611_review.md#공통-통합-검증과-증거-보존)에 연결한다. 이 PR의 조건부 수용이 아니라 미충족·미검증 해소 후 재검토 계획이다.

## 최신 보완 적용과 재검증 범위

2026-10-08 최신 `upstream/devel` `c7f357efbca7c967380c242b1b0e0c94b42043bb` 위로 누적 branch를 리베이스했다. 이미 병합된 #7619의 중복 커밋 5개를 제외하고, 기존 병합 리뷰와 오늘할일을 보존했다. 리베이스 직후 `d7617672df2700b95479e94a39980adb2a2066e5`의 `src`·`tests`·Cargo 파일은 리베이스 전 커밋 `760e86891142b16b3b84f9f9e436e7dc79c05055`와 동일하다.

새 기능 보완 `405418404874eb6bfa8f1354794f07d6f4118a14`와 대표 PNG 증적 `41c64bd552b5f44eaf340112628f340f1745b6a3`를 저자·출처를 유지한 `cherry-pick -x`로 추가했다. 적용 커밋은 각각 `13b3715d1`·`587eeff76`이며, contributor의 devel merge 커밋은 추가하지 않았다. 새 미주 첫 문단의 소유 경계, 모든 수용 판정의 scratch 렌더, 저장 기준선·글자 상자의 소비 경로를 다시 검증해야 한다. 새 테스트 4개와 contributor의 PNG는 새 통합 head의 독립 검증 완료를 뜻하지 않는다. 이 단계에서는 Git 충돌·공백·base 정렬만 확인했으며, 새 source의 회귀·Native/fresh WASM·최종 Clippy는 미검증이다.

이후 위 보완 검사 4개와 #1375 검사 2개는 최신 통합 source에서 집중 실행하여 6 PASS를 확인했다. 다만 사용자는 시험지 회귀를 다른 작업에서 다시 추가하기 위해 모두 제거하도록 지시했다. 해당 4개 신규 검사도 제거하며, 코드 보완과 contributor 증적은 유지한다. 세부 source SHA·명령·결과·검증 범위 철회는 [공통 시험지 회귀 처리 기록](pr_7611_review.md#사용자-지정-시험지-회귀-처리)을 따른다. 삭제 후 회귀 수 감소를 결함 수정이나 이 PR의 시각 통과로 판단하지 않는다. 삭제 후 fmt·세 Clippy·workspace build·base 고정 manifest/unit tiers는 PASS했다. 삭제 후 전체 nextest와 최신 통합 source의 독립 Native/fresh WASM 시각 검증은 미완료이며 원 PR의 머지 보류를 해소하지 않는다.

</details>
