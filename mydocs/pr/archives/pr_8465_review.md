# PR #8465 리뷰 — fix(typeset): 용지 밖으로 오프셋된 문단 기준 자리차지 개체는 흐름을 밀지 않는다 (#8462)

## 최종 판정

**메인터너 보정 후 수용 가능**
메인터너가 통합 후보의 실제 전쪽 증적과 회귀 근거를 보강했습니다. 원 PR에 대응하는 입력·쪽과 반례는 아래에 기록했습니다.
원 head와 리베이스·메인터너 보정을 적용한 통합 후보를 구분합니다. 실제 적용 커밋은 아래 표에 기록했습니다.
제출 전 필수 검증의 완료 근거를 아래에 기록했습니다. 작업지시자는 준비 완료 후 PR 생성·CI 모니터링을 승인했습니다.
아래 값은 명시한 코드 SHA의 실측 결과이며 원 PR의 승인이나 닫기를 뜻하지 않습니다.


최신 통합 code head `acbc0bfeb5660133b0732ce77eae16bd50b29f9f`의 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/41965861415)와 CodeQL·Render Diff·Adapter·Proptest가 모두 성공했습니다.
작업지시자가 CI 완료 뒤 문서 기록·병합·후속 처리를 승인했습니다. 원 head는 그대로 보존하고 보정된 [통합 PR #8622](https://github.com/edwardkim/rhwp/pull/8622)로 처리합니다.
이 문서는 code head CI 성공 뒤의 문서 전용 trailing 기록입니다. push 뒤 trailing head의 실제 fast-pass·필수 검사·최신 mergeability 확인을 병합 조건으로 둡니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8465](https://github.com/edwardkim/rhwp/pull/8465) · `planet6897` → `devel` |
| 원 head | `e7588ab2ab55a852b699cb81ce65bb2e0f4c588c` |
| 현재 통합 적용 커밋 | `f92e429cb414e54ff8e37e10fa8782c0d7b593f7` |
| 이전 검토 기준 devel | `22947f6a90c998e95fa51238d5b907a0ebd37162` |
| 이전 필수 검증 제품 코드 | `a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973` |
| 초기 대표 PNG producer / 당시 base | `4b149b6cdfc514b9858ed644be2cf8120c731cd4` / `7fdd5441ff1e77c083818e920a48089f294de35e` |
| reviewer | `jangster77` 요청. `2026-11-05 03:16:02 UTC` 원격 재조회에서 원 head 불변·Open·non-Draft 확인 |
| 기본 경로 | `maintainer_general` |
| 보조 경로 | `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch`, `post_merge` |
| 관련 이슈 | [#8462](https://github.com/edwardkim/rhwp/issues/8462) |

## 변경과 검토 범위

원 PR의 변경 주장은 **fix(typeset): 용지 밖으로 오프셋된 문단 기준 자리차지 개체는 흐름을 밀지 않는다 (#8462)**입니다. 해당 주장에 연결된 입력과 쪽을 아래에 분리해 적었습니다.
적층 PR에 중복 포함된 선행 커밋과 이 PR의 실제 신규 커밋은 최종 통합 기록에서 구분합니다.

## 검증 입력과 전쪽 결과

| 입력 · 독립 Print PDF | Native 전쪽 TSV · 최저 raw | fresh WASM 전쪽 TSV · 최저 raw |
| --- | --- | --- |
| [156552561-offpage-textbox.hwpx](<../../../samples/issue8462/156552561-offpage-textbox.hwpx>) · [156552561-offpage-textbox-2022.pdf](<../../../pdf/issue8462/156552561-offpage-textbox-2022.pdf>) | [TSV · c183](<../assets/planet6897_20261103/current-c183a1ac61/native/pr8465-156552561-offpage-textbox-79437ef0.tsv>) · 98.01% (3쪽) | [TSV · c183](<../assets/planet6897_20261103/current-c183a1ac61/wasm/pr8465-156552561-offpage-textbox-79437ef0.tsv>) · 98.10% (3쪽) |
| [156633674-offpage-logo.hwpx](<../../../samples/issue8462/156633674-offpage-logo.hwpx>) · [156633674-offpage-logo-2022.pdf](<../../../pdf/issue8462/156633674-offpage-logo-2022.pdf>) | [TSV · c183](<../assets/planet6897_20261103/current-c183a1ac61/native/pr8465-156633674-offpage-logo-364abf78.tsv>) · 99.97% (1쪽) | [TSV · c183](<../assets/planet6897_20261103/current-c183a1ac61/wasm/pr8465-156633674-offpage-logo-364abf78.tsv>) · 100.00% (1쪽) |

96 DPI, 색상 경계 보정 전 2px 실루엣입니다. 검증 입력과 Print PDF는 같은 행에서 대응합니다.
위 전쪽 TSV는 각 링크에 표시한 실제 producer c183 또는 ba2의 완료 증거입니다. 대표 PNG는 해당 이미지의 원래 producer를 유지하며 최신 재출력과 구분합니다.
원본·PDF SHA·Print 절차·전쪽 누락·쪽수 대조는 [증거 목록](../assets/planet6897_20261103/README.md)에 연결합니다.

## 조판 원칙 준수 검토

적용 대상입니다. 원본 저장값·독립 Print를 근거로 공통 규칙, 측정과 실제 배치의 소비 지점, 내용 소속·순서 및 적용되지 않는 반례를 아래에서 대조합니다.

## 구현 주장과 검증 증거의 대조

문단 기준 자리차지 개체의 실제 점유가 용지 밖에 있으면 보이는 본문 흐름을 그 개체 높이만큼 밀지 않는다.

typeset/controls/shape_flow.rs와 controls.rs에서 원본 오프셋·개체 영역과 본문 점유를 나눠 판단한다. paint를 숨기거나 잘못된 좌표를 clamp해 해결하지 않는다.

글상자와 로고 두 입력에서 본문·개체 원점을 비교한다. 본문과 실제로 겹치는 정상 자리차지 개체는 계속 흐름을 밀어야 한다는 대조군을 유지한다.

관련 정식 회귀 파일: [issue_8462_offpage_float_pushdown.rs](../../../tests/cases/issue_8462_offpage_float_pushdown.rs)

## 검증 입력 커밋 확인

이번 통합의 [입력·PDF SHA-256 대조표](../assets/planet6897_20261103/input-integrity.tsv)에 있는
148개 실제 검증 파일과 현재 Git 파일의 바이트가 일치함을 다시 확인했습니다.
위 개별 입력은 저장소의 동일 파일을 재사용합니다.

## 원 PR 주장 쪽의 시각 증적

| 입력 · 쪽 · 검토 대상 | Native raw · review / overlay | fresh WASM raw · review / overlay |
| --- | --- | --- |
| `156552561-offpage-textbox.hwpx` 3쪽 · 용지 밖 글상자가 본문 표를 추가 쪽으로 밀지 않음 | 98.01% · [review](../assets/planet6897_20261103/pr_8465/pr8465-claim-p003-156552561-offpage--native-review.png) / [overlay](../assets/planet6897_20261103/pr_8465/pr8465-claim-p003-156552561-offpage--native-overlay.png) | 98.10% · [review](../assets/planet6897_20261103/pr_8465/pr8465-claim-p003-156552561-offpage--wasm-review.png) / [overlay](../assets/planet6897_20261103/pr_8465/pr8465-claim-p003-156552561-offpage--wasm-overlay.png) |
| `156633674-offpage-logo.hwpx` 1쪽 · 용지 밖 오프셋 | 99.97% · [review](../assets/planet6897_20261103/pr_8465/pr8465-claim-p001-156633674-offpage--native-review.png) / [overlay](../assets/planet6897_20261103/pr_8465/pr8465-claim-p001-156633674-offpage--native-overlay.png) | 100.00% · [review](../assets/planet6897_20261103/pr_8465/pr8465-claim-p001-156633674-offpage--wasm-review.png) / [overlay](../assets/planet6897_20261103/pr_8465/pr8465-claim-p001-156633674-offpage--wasm-overlay.png) |

## 현재 대표 페이지 직접 판독

[판독 기록·실제 PNG SHA-256](../assets/planet6897_20261103/direct-reading-current.md)에 같은 원본·쪽의 관측을 남겼습니다.

- native 3쪽: 개인정보보호법 상자와 본문 뒤 정책브리핑·OPEN 로고가 같은 쪽에서 보존된다. 붙임 제목·후속 표와 로고가 겹치지 않으며 표의 행 순서와 내용은 대응한다. 일부 글꼴 획 차이는 남는다.
- native 1쪽: 보도자료 제목·본문·담당 부서 표 뒤 정책브리핑·광고 배너·OPEN 로고를 대조했다. 페이지 밖 앵커 때문에 사라지던 하단 그림의 보존과 내용 순서·비겹침을 확인했다.
- wasm 3쪽: 개인정보보호법 상자와 본문 뒤 정책브리핑·OPEN 로고가 같은 쪽에서 보존된다. 붙임 제목·후속 표와 로고가 겹치지 않으며 표의 행 순서와 내용은 대응한다. 일부 글꼴 획 차이는 남는다. WASM에서는 일부 굵은 글자의 획 표현도 PDF와 다르며 배치 주장과 구분한다.
- wasm 1쪽: 보도자료 제목·본문·담당 부서 표 뒤 정책브리핑·광고 배너·OPEN 로고를 대조했다. 페이지 밖 앵커 때문에 사라지던 하단 그림의 보존과 내용 순서·비겹침을 확인했다. WASM에서는 일부 굵은 글자의 획 표현도 PDF와 다르며 배치 주장과 구분한다.


<!-- planet6897-20261103-generated-review -->

## 최신 통합 검증 상태

현재 base는 `d4a72efd7c91d1e25092ed30b9d44e43cbebb9f0`입니다. 직전 문서 갱신 뒤 PR #8618의 이슈 라벨링 workflow·계약 검사·운영 문서 3개가 추가됐습니다.
184개 패치가 충돌 0건·range-diff 모두 `=`로 리베이스됐습니다. 제품·테스트·fixture·검증 도구의 내용은
리베이스 전과 같으므로 완료 증거를 재사용합니다. 실제 측정 source는
`ba2ebfd34a4ebf402bce8e002254b672a09ef764`이며 문서 commit을 새 렌더 제품으로 표시하지 않습니다.
[현재 적용 SHA 39개](../assets/planet6897_20261103/integration-commits.md)를 확인합니다.

- [최소 글꼴 환경의 필수 실행 41개](../assets/planet6897_20261103/rebase8609/rust-regression-validation.md)는 모두 exit 0입니다.
  전체 nextest 12,583 PASS·45 SKIP, Native Skia lib 4,040 PASS·12 IGNORE,
  CI Native 모듈 21개와 doctest 8 PASS·3 IGNORE를 포함합니다.
- [최신 Native 영향 10입력·43쪽](../assets/planet6897_20261103/rebase8609/native-own-validation.md)과
  [fresh WASM 같은 43쪽](../assets/planet6897_20261103/rebase8609/wasm-own-validation.md)은 전쪽 90% 이상,
  직전 후보 대비 정밀 raw 0.1%p 초과 하락·누락·쪽수 불일치 0건입니다.
- [최신 필수 편람 Native 383쪽](../assets/planet6897_20261103/rebase8609/handbook-native-validation.md)은
  최저 표시 raw 99.96%, 기준 표시값 하락·fallback 0건·exit 0입니다.
- [최신 실제 API·브라우저](../assets/planet6897_20261103/rebase8609/runtime-validation.md)는 실제
  271 assertion·17 Worker·12쪽 렌더/저장과 가져오기·편집 산출물 대조를 완료했습니다.
- 기존 c183의 [Native 80입력·406쪽](../assets/planet6897_20261103/current-c183a1ac61/native-validation.md)과
  [fresh WASM 전406쪽](../assets/planet6897_20261103/current-c183a1ac61/wasm-validation.md)은 양쪽 90% 이상입니다.
  충돌 없는 리베이스 뒤 전쪽 측정을 반복하지 않는 지시에 따라 원래 producer를 명시해 재사용합니다.
- 보호 PR #8389·#8397의 [c183 Native](../assets/planet6897_20261103/current-c183a1ac61/pr8389_8397/native-validation.md)와
  [c183 fresh WASM](../assets/planet6897_20261103/current-c183a1ac61/pr8389_8397/wasm-validation.md)은
  각 17문서·1,770쪽·원 TSV 2,153행에서 표시값 하락·누락 0행입니다.

원 PR별 주장에 대응하는 입력·쪽의 대표 review/overlay는 아래 원래 출처를 유지합니다.
최신 영향 입력의 그림은 최신 source로 별도 표시하며, 다른 문서의 대표 그림으로 대체하지 않습니다.
[직접 판독 원장](../assets/planet6897_20261103/direct-reading-current.md)과
[입력/PDF·compat·글꼴·명령](../assets/planet6897_20261103/visual-reproduction.md)을 연결합니다.
회귀 측정 주석 갱신은 실제 assertion·helper 본문과 구분했고 주석만 바뀐 전체 검사는 반복하지 않았습니다.

[새 base Native 대조](../assets/planet6897_20261103/rebase8609/native-base-preservation.md)와
[fresh WASM 대조](../assets/planet6897_20261103/rebase8609/wasm-base-preservation.md)는 각각 14입력·41쪽을 완료했습니다.
양쪽 모두 raw 하락 자체가 0쪽이며 90% 미만·누락·쪽수 차이·제품 해시 오류 0건입니다.
문서-only 리베이스의 [내용 동일성](../assets/planet6897_20261103/latest-rebase.md)을 확인해 완료 결과를 재사용했습니다.
추가 전체 회귀는 필요하지 않습니다. 제출 전 필수 검증은 완료했고 통합 PR #8622의 정확한 code head CI가 모두 성공했으며 이번 trailing head는 별도로 확인합니다.
원 PR 39개는 원격 재조회에서 head 불변·Open·non-Draft·reviewer 요청을 확인했습니다.
통합 PR 생성 전 본인 계정의 우선 처리할 열린 PR은 0건이었습니다. 원격 push·통합 PR 생성과 code head CI는 완료했습니다. 이 후속 문서 작성 시점에는 병합·원 PR 댓글·닫기를 아직 수행하지 않았습니다.
작업지시자는 준비 완료 후 PR 생성·CI 모니터링을 승인했습니다. 문서-only trailing commit의 병합·링크·기존 오늘할일 보존을 확인한 뒤 push하고 새 head CI를 확인합니다.

## Merge 후 contributor PR comment 계획

[통합 PR #8622](https://github.com/edwardkim/rhwp/pull/8622)의 실제 merge SHA와 devel 반영을 확인한 뒤 원 PR을 통합 완료로 정리합니다.
[이슈별 처리 범위](../assets/planet6897_20261103/source-followup-scope.md)에 따라 해결 이슈와 남은 별도 범위를 구분합니다.
[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결하고, 위 전쪽 TSV와 같은 원본·PDF·주장 쪽의 review 및 별도 overlay를 사용합니다.
실제 그림은 직접 판독 원장의 producer·페이지·해시를 유지하며 다른 문서의 대표 그림으로 대신하지 않습니다.
원 PR의 기여와 메인터너 보정 이유·검증 결과·남은 차이를 한국어 존댓말로 설명합니다.
이미지 URL은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/...` 형식으로 고정하고 게시 전 응답의 이미지 형식·SHA-256을 원본과 대조합니다.
기존 같은 검토 댓글이 있으면 수정하고 없을 때만 처음 게시합니다. 게시 뒤 API 본문과 이미지 표시를 재확인하며 contributor fork branch는 삭제하지 않습니다.

[최종 code CI와 오탐 처리](../assets/planet6897_20261103/code-head-ci-completion.md)를 함께 기록했습니다. 현재 기록 시점에는 merge·원 PR close/comment를 아직 실행하지 않았으며, 실제 원격 완료 사실은 원 PR 타임라인과 통합 PR에 남깁니다.

[후속 댓글의 실제 원본·쪽·Native/WASM PNG 대응표](../assets/planet6897_20261103/source-comment-evidence.md#pr-8465)를 고정했습니다. 이미지 수·출력 경로·해시를 게시할 댓글과 대조합니다.
