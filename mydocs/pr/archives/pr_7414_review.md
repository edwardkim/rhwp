---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-04
---

# PR #7414 검토 — 수정(조판): 한 줄에 안 들어가는 토큰을 잰 폭으로 자른다 — 폴백의 상수 12.0px 제거 (#7407, #7410 위)

## 최종 판정

**메인터너 보정 후 수용 가능.** 원 PR head `ed53392e9f575492f5cd7192f378490cc8c4b6d9`의 직접 병합 판정과 구분합니다. `review/planet6897-green-20261002`의 검증 head `859d1ddefc2f70180fd8009d7124804bdd9d76d8`는 최신 base `1d6bc70767fad365b07afe4ef57972d23b140f2b` 위에서 원 기여·중복 제거·메인터너 보정을 함께 검증했습니다. 원 source head는 접수 이후 바뀌지 않았으며 source CI에 실패·진행 중인 check가 없습니다. 현재 source PR 병합 상태는 `CLEAN`입니다.

- 수용 대상은 보정이 포함된 통합 head입니다. 새 통합 PR의 최신 required CI와 mergeability를 확인한 뒤 승인된 병합·후속처리를 수행합니다. 원 PR 자체를 직접 merge하지 않습니다.
- 대형/낮은 피델리티 입력의 기존 #7445 이관은 보존했습니다. 이 판정은 해당 문서 전체 피델리티 완료나 관련 issue 전체 해결을 뜻하지 않습니다.

## 기여자 중심 최종 검토 — 2026-10-04

### 기여 내용과 메인터너 보정

한 줄에 들어가지 않는 긴 토큰을 상수 폴백 대신 측정한 글자 폭으로 나누신 기여입니다.

선행 칸 줄 상자와 후속 condense·마커 폭 변경을 동일한 가용 폭으로 적용했습니다. run 보존·언어별 글꼴 측정과 정상 반례를 유지했으며 저장 줄 또는 기준 PDF 없이 픽셀 위치만 고정한 검사를 승인 근거로 사용하지 않았습니다.

원 기여의 저자와 출처를 보존했습니다. 해당 PR의 원본 커밋과 현재 리베이스 후 적용 SHA는 [최종 적용 원장](../assets/planet6897_green_20261002/final_applied_provenance.json)에 기록했습니다. 누적 통합의 공통 보정 결과를 이 원 PR만의 단독 결과로 표기하지 않습니다.

### 최종 검증과 조판 원칙

- 검증 head `859d1ddefc2f70180fd8009d7124804bdd9d76d8`: 전체 nextest10,288PASS/0FAIL·50skip(9slow)·497.515초·exit0. release-test/target/pr-review/threads8/no-fail-fast로 수행했습니다.
- Native Skia 라이브러리4,109PASS/0FAIL·13ignored, 누락 그림2PASS, 직접 PDF4PASS이며 모든 명령exit0입니다. Native/WASM Clippy·fmt·workspace build·all-target Clippy·manifest도exit0입니다. fresh WASM은 Mac 로컬 no-opt 대체 빌드이며 Docker 최적화 검증으로 보고하지 않습니다.
- 측정→예약/컷→실제 배치·paint가 같은 원점/여백/내용 소유를 소비하도록 검토했습니다. 원본 저장 글줄 유효성·정상 반례·비겹침·누락/중복을 확인하고 문서ID 예외·좌표clamp·출력 숨김·임계값 변경을 보정으로 사용하지 않았습니다. 이전 픽셀 핀은 독립 시각 근거를 확보한 범위에서 기존 검사의 의미 관계로 교정했습니다.
- [개별 변경 관련 증거](../assets/planet6897_green_20261002/hy_ladder3_current_band_validation.json), [최종 공통 검증](../assets/planet6897_green_20261002/final_validation.json), [원 source head·CI 재조회](../assets/planet6897_green_20261002/final_source_metadata.json), [검증 입력 commit 일치](../assets/planet6897_green_20261002/final_input_commit_check.json).
- 정책연구 Native/fresh WASM215/PDF215쪽: 최저90.30307%·90% 미달0. 첫 조각 보정90쪽99.15148%·94쪽99.88068%, 종료 조각91쪽98.91458%·95쪽99.15957%를 직접 판독했으며 각주 참조와 소유를 유지했습니다. 전체 SVG215쪽 중 변경2쪽을 새 raster로 확인하고 동일 SVG213쪽은 기존 전쪽 PNG 비교를 재사용했습니다. 전215쪽을 새로 raster했다고 보고하지 않습니다.
- [Native 전쪽 TSV](../assets/planet6897_green_20261002/liver_p90_native_whole215.tsv), [fresh WASM 전쪽 TSV](../assets/planet6897_green_20261002/liver_p90_wasm_whole215.tsv), [90쪽 review](../assets/planet6897_green_20261002/liver_p90_native_review.png), [94쪽 review](../assets/planet6897_green_20261002/liver_p94_native_review.png). 정상5문서42쪽은 Native tree/fresh WASM SVG가 기존 승인 증적과 동일합니다. 모든 다른 문서의 전체 피델리티를 이215쪽 지표로 대신하지 않습니다.

### Merge 후 contributor PR comment 계획

통합 PR의 merge SHA와 최신 CI URL을 확정한 뒤 이 원 PR에 다음 내용으로 한국어 존댓말 comment를 게시하고, 통합 PR에서 대체 병합됐음을 링크한 뒤 close합니다. 원본 fork branch는 보존하고 관련 issue를 이 기록만으로 자동 종료하지 않습니다.

> 기여해 주신 변경을 통합 브랜치에서 검토했습니다. 한 줄에 들어가지 않는 긴 토큰을 상수 폴백 대신 측정한 글자 폭으로 나누신 기여입니다. 선행 칸 줄 상자와 후속 condense·마커 폭 변경을 동일한 가용 폭으로 적용했습니다. run 보존·언어별 글꼴 측정과 정상 반례를 유지했으며 저장 줄 또는 기준 PDF 없이 픽셀 위치만 고정한 검사를 승인 근거로 사용하지 않았습니다. 보정된 후보의 전체 회귀10,288건과 Native Skia 검증이 통과했습니다. 수용 범위·잔여 #7445 과제·시각 증적은 개별 검토 문서에 남겼습니다. 통합 PR과 최종 merge SHA·CI 링크를 함께 안내드리겠습니다.

실제 게시에서는 통합 PR/merge SHA/CI URL을 확정 값으로 치환하고 [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment), 위 대표 PNG의 merge SHA 고정 raw URL, 남은 범위를 포함합니다. UTF-8 본문 파일의 `--body-file`로 게시하고 API 재조회로 한글·링크·게시 내용과 close 상태를 확인합니다.

이하 접수·단계 실행 기록은 과거 head별 이력입니다. 과거의 “보류/진행 중”은 위 최종 검증을 대체하지 않습니다.

## 접수·기여자·출처

- 원 PR: https://github.com/edwardkim/rhwp/pull/7414, 작성자 planet6897, base `devel`, 정확한 head `ed53392e9f575492f5cd7192f378490cc8c4b6d9`.
- reviewer jangster77을 지정했습니다. 원 contributor 변경과 메인터너 충돌 보정을 구분해 기록합니다.
- 사전 선택: collaborator_external_pr 9.1.1 체리픽 통합 경로; intake/local_validation/visual_fixture_evidence/multi_pr_update_branch/post_merge 적용.
- source CI는 통합 head 검증을 대체하지 않습니다. 모든 renderer·페이지 변경은 직접 Native/fresh WASM 전쪽 TSV와 영향 경계 review/overlay를 검토합니다.

## 체리픽 계획

| source commit | 판정 | 변경 |
| --- | --- | --- |
| `4cc0ffeedbfefa7a72a1bb03bce69ee26020394f` | stacked/rebased duplicate | 수정(조판): 칸 문단의 줄 상자도 문단 좌우 여백만큼 들여 잡는다 (#7407) |
| `3d18a31ec5000d2d38cc596031e10849d65e74b5` | candidate | 수정(조판): 한 줄에 안 들어가는 토큰을 잰 폭으로 자른다 (#7407) |
| `235aaa6b092f251267ab993cdbc6dc9c15930d50` | candidate | 문서(증적): #7407 A2b 시각 증적과 코퍼스 변화 3건 판독 |
| `ed53392e9f575492f5cd7192f378490cc8c4b6d9` | merge excluded | Merge branch 'devel' into fix/7407-token-fallback-char-width |

## 변경 범위와 검토 계획

- 원 PR 변경 372줄 추가·59줄 삭제·10파일입니다.
- 생산 경로: `src/document_core/commands/document.rs`, `src/renderer/composer/line_breaking.rs`, `src/renderer/layout_frame.rs`.
- 실제 호출 경로의 측정→예약/컷→paint 소비를 추적하고 정상 대조군·원본 저장 정보의 유효성을 확인합니다. 문단·행·개체·각주 소유와 누락/중복을 기존 검사 의미로 판단하며 픽셀 핀을 승인 근거로 사용하지 않습니다.
- 원본·기준 PDF와 검토 commit의 실제 파일/해시 일치를 확인합니다. 통합 출력과 PDF의 전체 쪽수가 다르거나 미달쪽이 있으면 재검토합니다. 원 PR의 부분 개선 주장을 전체 피델리티 완료로 확대하지 않습니다.
- 합성/실물 경계 회귀·전체 nextest threads8·Native Skia3·필수 lint/정책·fresh WASM은 누적 후보에서 순차 수행합니다. 로그는 ignored output에만 저장합니다.

## 실행 결과

고유 source commit 63개 체리픽을 완료했습니다. 출처와 보정은 [적용 원장](../assets/planet6897_green_20261002/applied_commits.json)에 기록했습니다. 현재 후보 `ec5ca7c3057a89c9a82bb59a78956a4d5eee567d`의 Native Clippy는 exit0이며 전체 nextest는 실행 중입니다. 최종 회귀·시각 검증은 미완료입니다.

### 누적 후보 검증 시작

- 검증 코드 head: `ec5ca7c3057a89c9a82bb59a78956a4d5eee567d`. Native Clippy exit0(34.38초). 전체 nextest release-test/threads8/no-fail-fast 실행 중이며 통합 시각 검증은 아직 미완료입니다. 원 PR의 green CI와 구분합니다.

### 누적 후보 검증 상태 갱신 (2026-10-02)

- GitHub를 다시 조회한 결과 선택 당시의 원 PR head와 CI green 상태가 유지됩니다. #7435는 현재도 non-green이며 선택에 포함하지 않았습니다. 누적 후보는 `review/planet6897-green-20261002`, source 고유 커밋63개입니다.
- 최초 전체 nextest는 10,283개 중10,250PASS/33FAIL/50SKIP, exit100으로 완료했습니다. 그 뒤 PR별 보정과 focused 재검증으로 최초 실패28개를 처리했으며 5개가 남았습니다. 전체 재실행 통과로 바꾸어 보고하지 않습니다.
- 각주 빈 번호·합성 사다리·다열 표 대조군·중첩 표 후속 원점의 잔존 5개와 whole fixture 시각 보류를 [현재 검증 기록](../assets/planet6897_green_20261002/review_progress.json)에 기록했습니다. 원 PR별 기존 분석·커밋 출처는 위 내용을 유지합니다.
- **현재 통합 승인/머지 보류**입니다. 원 PR의 green CI는 누적 후보의 실패 또는 미완료 Native/fresh WASM 시각 검증을 대체하지 않습니다. 새 통합 PR 생성·push·머지는 하지 않았습니다.

### 원 PR 최신 head·CI 재확인 — 통합 검증과 구분

- 2026-10-04 API 재조회: 원 PR은 OPEN, head `ed53392e9f575492f5cd7192f378490cc8c4b6d9`로 기존 접수 기록과 같습니다. 원본 저장소의 해당 SHA check 32건은 skipped 20건, success 12건이며 실패·진행 중인 check는 없습니다.
- 현재 통합 후보 `a73100f16`에서 form002는 Native/fresh WASM 전10쪽 최저92.96827%와 기존 관련42건·SVG 묶음7건의 통과를 확인했습니다. 원 PR CI 통과를 통합 후보 전체 통과로 대체하지 않습니다. 76076 실제 물리6쪽의 본문/쪽번호 겹침, 다른 시각 보류 및 최종 전체 회귀·Skia 검증이 남아 있어 최종 승인·PR 제출은 계속 보류합니다.
- [정확한 source SHA별 check 증적](../assets/planet6897_green_20261002/source_ci_refresh_after_form002.json). 원 PR mergeability와 통합 분기 충돌 여부는 별개이며, 원 PR의 직접 병합은 수행하지 않았습니다.
