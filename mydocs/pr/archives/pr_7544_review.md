---
kind: review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-07
---

# PR #7544 리뷰 — 표 뒤 Enter의 빈 줄 소속 보존

## 최종 판정

**기술 검토 승인 — 로컬 필수 게이트와 코드 head Full CI 충족, 병합 보류 해제.**
검증 source는 `5308d4087d5e7ebdc38e14ff715f6b2621d0e108`, 비교 base는
`7076f836e2300f7d760d74b58c468cc0f73098a2`다. 본인 PR의 self-review이며
GitHub self-Approve나 branch protection 우회를 사용하지 않는다.

동일 source/test의 제출 head `92706070bfad1c46e9a42e62246069b5d6da56f1`에서
[Full CI run 37584835800](https://github.com/edwardkim/rhwp/actions/runs/37584835800)이 성공했다.
CI preflight·Build & Test와 CodeQL·Render Diff·Adapter inter-diff·Proptest를 포함한
최신 checks는 **32 success / 5 정책상 skipped**, 실패·대기0이다.
조회 시 base는 위 SHA와 같고 Open, non-draft, `MERGEABLE` / `CLEAN`이었다.
이 후행 commit은 문서 4개만 정정하며 source/test/이미지는 바꾸지 않는다.
후행 head의 preflight·Build & Test·mergeability는 push 뒤 PR 업데이트 댓글에서
정확한 SHA로 확인한다. 실제 merge·issue close는 아직 수행하지 않았다.

## 2026-10-07 보류 해제 보정과 최종 로컬 검증

보류 원인이었던 새로운 저장 종료 guide의 0 전진 계획을 제외했다.
기존 표 꼬리 정리의 적용을 **바로 앞 완료 PartialTable의 단일 종료 문단**으로
제한해, 일반 본문 줄을 거친 반복 Enter의 확정 쪽·문단 소속을 보존한다.
기존 종료 문단의 의미 자체를 재설계하거나 시각 일치를 새로 주장하지 않는다.
앞 빈 줄의 누적 drift로 다음 줄을 Hidden 처리하던 분기는 제거했다.
문서 ID·표 크기·쪽 번호 조건, 좌표 clamp, 기준값·허용치 변경은 없다.

생산 `FormattedParagraph` → `paragraph/flow.rs` fit/이월 → `PageItem::FullParagraph`
→ `section.rs` 확정 → `state/finalize.rs` → 실제 layout·cursor lookup을 확인했다.
표의 컷/유닛·요구/예약 높이와 줄 원점은 바꾸지 않는다. 직전 항목이 본문 문단이면
표의 직접 종료 문단이 아니므로 기존 사후 정리가 그 확정 쪽을 지우지 않는다.
완료 표의 직접 종료 문단, 명시적 쪽/구역 나눔, 저장 reset 경로는 기존 처리를 유지한다.

| 독립 PDF 경계 | 수정 전 devel Native | 보정 Native / fresh WASM | 전쪽 최저 실루엣 |
| --- | --- | --- | --- |
| 10행·160%·Enter32 | 1쪽 | 1 / 1쪽 | 양쪽100% |
| 10행·160%·Enter33 | 1쪽(기준2쪽) | 2 / 2쪽 | 양쪽100% |
| 10행·160%·Enter40 | 2쪽 | 2 / 2쪽 | 양쪽100% |
| 30행·300%·Enter9 | 1쪽(기준2쪽) | 2 / 2쪽 | 양쪽100% |

같은 기존 회귀 source의 정리 범위 보정 전4PASS/2FAIL, 보정 후6PASS를 확인했다.
실제 저장 입력의 위1→2쪽 대조도 같은 원문·PDF로 실행했다. 환경/빌드 실패는 RED로 세지 않았다.
Native/fresh WASM 각각 전체7쪽의 미달·누락0, 대표 두 경계의 일반 gate는 `passed`다.
표 외곽·행/열 경계·처음/끝 위치와 빈2쪽을 직접 판독했고 표 중복이 없다.
표 선이 PDF보다 밝은 차이는 남으며 엄격 내용 픽셀은 약0.02%/7.46%다.
실루엣은 보조값이며 전체 피델리티100% 판정이 아니다. 96dpi print·고정2px 관용으로
비교했고 글꼴 예외·마스킹·DPI/기준값 완화는 없다.

| 필수 실행 | source `5308d4087d` 결과 |
| --- | --- |
| fmt, Native/WASM32/workspace all-target Clippy, workspace build | 모두 PASS |
| suite prepare·manifest check, 고정 base `7076f836e2` | PASS; 파생 파일 미제출 |
| 전체 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --no-fail-fast` | 10,511 PASS / 0 FAIL / 50 skipped,208.594초 |
| Native Skia `--features native-skia --lib` | workspace 합4,109 PASS / 0 FAIL / 13 ignored |
| Native Skia 누락 그림·직접 PDF | 2 PASS / 4 PASS |
| root locked wrapper fresh web WASM | 최적화까지 PASS; Docker daemon 불가로 host 경로 사용 |

workspace Clippy 첫 실행은 디스크 소진으로 실패했고 컴파일 동시성1의 재실행은 통과했다.
공유 target을 보존했다. source-side cfg(test), Studio/npm source는 변경하지 않아 해당
unit-tier·npm 게이트는 비해당이다. 이번 source의 Studio DOM 재촬영은 하지 않았으며
기존 Chrome 이미지는 당시 source의 이력으로만 보존한다.

### 저장본 범위 분리 대조

어구21쪽·교육과정413쪽·정상 RowBreak18쪽, **452쪽 전체**를 devel과 새 후보의
Native끼리 및 WASM끼리 각각 비교했다. 쪽수·내용·geometry를 포함한 모든 render tree가
동일했다. 기준 source `cf2b0508fc`의 production/test는 병합 devel `7076f836e2`와 동일하다.
이것은 새 guide의 의미·기존 교육과정413/415쪽 차이를 통과시킨 증거가 아니다.
미검증 의미를 추가하던 guide 계획이 이번 diff에서 제외됐고, 이 저장본들의 실제 출력은
변하지 않았다. #7207 잔여·#7445 전체 보정은 이번 PR에 포함하지 않는다.

### 입력·증적과 판정 범위

입력 커밋 확인은 **충족**이다. `tests/fixtures/issue7486_table_enter/` 4개 원문과
`pdf/issue7486_*-2020.pdf` 4개 기준은 검증 source의 Git blob과 실행 파일이 동일하다.
파일별 전체 SHA-256은 [Native 전체7쪽 TSV ZIP의 README](https://github.com/user-attachments/files/33141202/pr7544-native-enter-silhouettes.zip)와
[기존 입력 manifest](../../working/assets/issue7486-table-enter/validation.json)에 있다.
ZIP SHA-256은 `b7952364fe1a9e4d7f088ed0f81c1b942f1a90eb2c9a13de5bb3df71054f053d`이며
공개 다운로드의 동일 바이트를 확인했다. 새 TSV·로그·진단 JSON은 ignored output에 보존한다.

| 조판 검토 | 판정·증거 |
| --- | --- |
| 근거·일반성 | 충족: 독립 PDF의1/2/2/2쪽, 수정 전1/1/2/1쪽과 소속 누락; cleanup의 원래 표 종료 범위 밖인 본문 흐름 보호 |
| 측정·배치·점유 | 충족: 기존 fmt/fit/advance와 배치 결과 보존; guide 원점·0 전진 계획 제거 |
| 분할·이어받기 | 컷/높이 변경 비해당; 최종 조각 뒤 일반 문단의 쪽 소속·표 단일 표시·뒤 빈 쪽은 실행 확인 |
| 독립성·새 회귀 | 충족: API 생성본·독립 Print PDF 전7쪽×2backend, 같은 회귀의 RED/GREEN, 정상 Enter32/40 |
| 기준값 변경 | 비해당: baseline/golden/래칫·허용치 변경 없음 |
| 미검증 | 성능 정량 benchmark·현재 source Studio DOM 직접 촬영; 기존 저장 guide의 일반 사양 재해석·교육과정 전체 개선은 구현/완료 주장 밖 |

대표 최신 이미지: [Native Enter33 review](../assets/pr7544-enter-20261007/native-table33-review-p1.png)·
[overlay](../assets/pr7544-enter-20261007/native-table33-overlay-p1.png),
[fresh WASM Enter33 review](../assets/pr7544-enter-20261007/wasm-table33-review-p1.png)·
[overlay](../assets/pr7544-enter-20261007/wasm-table33-overlay-p1.png),
[Native Enter9 review](../assets/pr7544-enter-20261007/native-table30-review-p1.png),
[fresh WASM Enter9 review](../assets/pr7544-enter-20261007/wasm-table30-review-p1.png),
[Native 빈2쪽](../assets/pr7544-enter-20261007/native-table33-review-p2.png)·
[fresh WASM 빈2쪽](../assets/pr7544-enter-20261007/wasm-table33-review-p2.png).
[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)의 절차를 따랐다.
이미지는 위 source를 새로 출력했다. 작업지시자의 본문 보존 요청에 따라 최신 구현·수치·
head 고정 이미지는 PR 댓글에 게시하고 원 본문은 제출 당시 기록으로 유지한다.

### Merge 후 comment 계획

최신 head의 성공 CI·mergeability 확인과 별도 병합 지시를 만족하면 merge SHA로 고정한
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/pr7544-enter-20261007/wasm-table33-review-p1.png`
및 Native/overlay를 댓글에 다시 남긴다. 표 선 밝기와 현재 source Studio 미촬영 범위를 명시하고,
API로 한글·실제 줄바꿈·이미지 URL을 재조회한다. #7486의 기재된 잔여 Enter 경로만 해결하며
#7207/#7445의 전체 종료 표현은 사용하지 않는다. 실제 merge·issue close는 아직 수행하지 않았다.

## 이력 — 아래 기록은 각각 당시 source의 결과

## 2026-10-07 최소 선행 병합 후 범위 분리 진단

원래 요청은 #7544의 리뷰·병합이다. #7207 전체나 #7445 교육과정 전체 보정으로
확대하지 않고, 이 PR 안의 Enter와 저장 종료 guide만 분리 가능한지 조사했다.
미검증 helper를 단순 삭제한 결과를 통과한 후보로 제출하지 않았다.

| 로컬 분리 후보 | 실제 결과 | 의미와 미검증 |
| --- | --- | --- |
| guide를 제외하고 기존 끝 쪽 제거 복원 | 기존 Enter 6개 중4PASS/2FAIL; 표 뒤 Enter 문단의 page owner가 사라짐 | 원래 Enter 결함을 다시 검출. 그대로 분리·병합할 수 없음 |
| guide만 제외하고 새 끝 쪽 보존 유지 | 기존 Enter6PASS; Native 어구21쪽 유지·교육과정413→414쪽 | 기준 교육과정415쪽과 여전히 다름. 무영향 분리나 전체 출력 개선 완료로 판정하지 않음 |

두 후보는 source parent `4d9889a7f4`의 로컬 진단이다. 기대값·새 fixture·baseline을
추가하거나 완화하지 않았다. fresh WASM·Visual Sweep은 진단 후보에서 미실행이다.
둘째 후보의 쪽수는 같은 기존 입력과 로컬 출력 조건의 Native `info --json`에서
확인했다. 단순 쪽수 변화로 시각 오류나 개선을 확정하지 않는다. 진단 source는
원래 후보로 복원했고 원격 source push·Approve·merge는 수행하지 않았다.
실행 로그·분리 patch·요약은 ignored `output/pr-review/issue7207-7544-focus-20261007`에
보존한다. 현재 base `7076f836e2` 대비 manifest 정책 검사는 PASS이며 전체 Rust/lint와
저장 guide의 독립 기준 확인은 남아 있다.

저장 guide의 의미를 독립 출력으로 입증하거나 기존 실제 저장본 경로를 바꾸지 않는
Enter 수정으로 범위를 다시 나누어야 한다. 기준415쪽과 영향 출력 확인을 생략한 채
통합하지 않는다. 기존 교육과정 결함을 #7544의 신규 회귀라고 단정하지 않으며,
그 전체 수정은 다른 담당 범위로 유지한다.

## 2026-10-07 범위 분리 후 로컬 후보 재검증

Rust source의 병합 충돌은 없었고 오늘할일 add/add는 양쪽 기록을 보존해 해소했다.
최신 `InlineFlowPlan`의 추가 필드 6개를 저장 guide 생성자에 기존 text-only plan과
같은 기본값으로 초기화했다. guide 수용 규칙·0 전진·분할표 소유 규칙은 추가 변경하지 않았다.
#7207의 저장 프레임 정렬 소비 보정은 [별도 계획과 검증](../../plans/task_m100_7207_impl.md)에 연결한다.

| 검증 범위 | 정확한 후보의 결과 | 판정 |
| --- | --- | --- |
| 반복 Enter 원문4개·독립 PDF | Native/fresh WASM 각각 전체7쪽, 최저100%, 누락0 | 해당 소속 계약 충족; 표 선의 색상 차이는 남음 |
| 어구 실제 저장본·동일 원문 PDF | Native/fresh WASM 각각 전체21쪽, 최저93.22327%(7쪽), 90% 미만/누락0 | 해당 정량 기준 충족; 잔여 괘선 차이로 #7207 전체 종료는 아님 |
| 교육과정 guide 영향171/172쪽 | Native 66.38064/24.14817%; 기준415쪽/실제413쪽 | 미충족; `re_review_required`. 이번 source의 fresh WASM guide와 전쪽 대응은 미검증 |
| 기존 Enter·간격·저장/재열기 검사6개 + #7207 지도 검사3개 | 실제9PASS | 해당 회귀 검사 충족 |
| fmt·base `48ff4bb935` 고정 manifest 정책 | PASS | 충족 |
| 전체 Rust 회귀·세 Clippy·workspace build·원격 CI | 새 후보에서는 미완료 | 미검증; 과거 head의 통과를 재사용하지 않음 |

정확한 후보의 fresh WASM17쪽과 Native171쪽 review, Native Enter33쪽 review를
직접 판독했다. 지도·캡션 위치는 복원됐지만 교육과정171쪽의 내용 소속은 기준 PDF와
다르다. 같은 내용의 PDF 쪽 번호를 바꾸어 통과시키지 않았다. 반복 Enter7쪽 및
어구21쪽의 통과는 저장 guide의 0 전진 의미나 교육과정 쪽 대응을 입증하지 않는다.
따라서 현재 PR의 승인·통합 가능 판정으로 확대하지 않는다.

| 대표 경계 | Native review·overlay | fresh WASM review·overlay |
| --- | --- | --- |
| 어구17쪽 | [review](../../working/assets/issue7486-table-enter/recheck_20261007_native-eogu17_review.png) · [overlay](../../working/assets/issue7486-table-enter/recheck_20261007_native-eogu17_overlay.png) | [review](../../working/assets/issue7486-table-enter/recheck_20261007_wasm-eogu17_review.png) · [overlay](../../working/assets/issue7486-table-enter/recheck_20261007_wasm-eogu17_overlay.png) |
| Enter33 첫 쪽 | [review](../../working/assets/issue7486-table-enter/recheck_20261007_native-enter33_review.png) · [overlay](../../working/assets/issue7486-table-enter/recheck_20261007_native-enter33_overlay.png) | [review](../../working/assets/issue7486-table-enter/recheck_20261007_wasm-enter33_review.png) · [overlay](../../working/assets/issue7486-table-enter/recheck_20261007_wasm-enter33_overlay.png) |
| guide171쪽 실패 | [review](../../working/assets/issue7486-table-enter/recheck_20261007_native-guide171_review.png) · [overlay](../../working/assets/issue7486-table-enter/recheck_20261007_native-guide171_overlay.png) | 이번 source에서 미검증 |

입력/PDF·source와 대표 PNG 해시 및 전쪽 최저값은 [새 재검증 원장](../../working/assets/issue7486-table-enter/recheck_20261007.json)에
연결한다. 원문·PDF는 repository blob과 대조했으며 새 변환 없이 기존 독립 출력을 재사용했다.
Native `release-test`와 루트 wrapper의 새 host web WASM(wasm-opt 성공)을 사용했다.
Docker 표준 경로 통과로 보고하지 않는다. 정확한 명령·패키지 해시·TSV·로그는
ignored `output/pr-review/issue7207-7544-focus-20261007`과 해당 임시 sweep 경로에 보존한다.
사용자 제공 파일의 데이터·경로·메타데이터는 제출용 증적에 포함하지 않는다.
원격 push·PR 본문 변경·댓글·승인·merge·이슈 종료는 수행하지 않았다.

## 2026-10-03 최종 판정 — 당시 이력

**머지 보류 — 실제 저장본의 Native/fresh WASM 시각 gate 미달.** 재검토 head `3edcbdbe172767d135ce420e459a41fc9b44f12e`의 Full CI와 관련 checks는 성공했다. 이후 빠졌던 실제 저장본 2개의 동일 원문 한컴 PDF 비교를 실행했고, 두 입력 모두 `pr_review_gate=re_review_required`였다. 반복 Enter 합성 계약의 통과와 실제 저장본의 출력 실패를 구분한다. 기존 승인 기록은 검증 범위를 충분히 구분하지 못해 바로잡는다. 본인 PR의 self-review이며 GitHub Approve event가 아니다.

반복 Enter 합성 입력 4개의 Native/fresh WASM 전체 7쪽 최저 실루엣은 100%였다. 이 결과는 아래 실제 저장본 2개의 실패를 면제하지 않는다. 표 선 색상은 한컴 PDF보다 밝으며 전체 피델리티 100%로 해석하지 않는다. Docker daemon 연결 불가로 표준 Docker WASM은 미실행이고 host release `--no-opt` fallback을 사용했다.

## 2026-10-03 실제 저장본 재검증 — 미충족

검증한 production/test source는 `97772d5d40787e77c3238debc2b2576b11713476`이며 원격 head `3edcbdbe…`와 코드가 같다. 실행 당시 문서 commit은 `0e1dd5d4…`였다. root wrapper로 fresh WASM을 다시 만들고 root/public 해시 일치를 확인했다. Native와 fresh WASM은 같은 원문·PDF를 96dpi print profile·고정 2px 관용으로 비교했다. 글꼴 예외·마스킹·허용치 변경은 없다. 명령·입력/출력 SHA-256·backend provenance·전후 PNG 해시는 [재검증 원장](../../working/assets/issue7486-table-enter/stored-guide-recheck.json)에 있다.

| 실제 입력 | 한컴 PDF / Native / fresh WASM | 전체 쪽 측정과 영향 페이지 | 판정 |
| --- | --- | --- | --- |
| `task2097/18095317_eogu_geumji.hwp` | 21 / 21 / 21쪽 | 양쪽 backend 전21쪽 TSV 최저 **14.72666% (19쪽)**, 90% 미만16쪽, 누락0. 영향20/21쪽은 **18.38669% / 19.92770%** | 미충족; 대표 gate `re_review_required` |
| `task2287/1342000_edu_curriculum_map.hwp` | **415 / 413 / 413쪽** | 양쪽 전체 TSV는 쪽수 불일치로 측정 전 실패. 전체 최저값은 미산출. 같은 쪽171/172 비교는 **66.33053% / 24.00083%** | 미충족; 대표 gate `re_review_required`; 전쪽 일치는 미검증 |

어구 문서는 [동일 원문의 한컴 PDF](../../../pdf/18095317_eogu_geumji-2020.pdf)를 확보했다. 한컴 `11.0.0.9136`, 전처리 없음, one-up 출력, 21쪽/PDF1.4이며 PDF SHA-256은 `98a9378f5b3440cc8c56c03dd483e2af95a194b9340ebc916c08474ceea65eab`다. 교육과정은 [기존 정상 한컴 PDF415쪽](../../../pdf/task2287/1342000_edu_curriculum_map-hwp-2020.pdf)를 재사용했다. 파일명은 2020 bucket이나 실제 Creator는 Hwp2022이며 버전 이름만으로 기준을 폐기하지 않았다.

대표 Native/fresh WASM review와 standalone overlay를 직접 판독했다. 어구21쪽은 이전 세로 지도가 다시 배치되고 마지막 `[부도5]` 지도는 아래로 밀려 용지 밖까지 이어진다. 기준 PDF21쪽은 마지막 지도만 본문 상단에 있다. 교육과정171쪽은 본문 행·성취기준 코드의 쪽 소유가 다르며 현재171쪽 내용은 기준PDF172쪽에 대응한다. 기준 페이지를 재번호하거나 잘라 점수를 올리지 않았다. 글꼴 이름으로 그림 배치·쪽 소유 차이를 면제할 수 없다.

| 실패 대표 | Native review·overlay | fresh WASM review·overlay |
| --- | --- | --- |
| 어구21쪽 | [review](../../working/assets/issue7486-table-enter/blocked_native_eogu_review_021.png) · [overlay](../../working/assets/issue7486-table-enter/blocked_native_eogu_overlay_021.png) | [review](../../working/assets/issue7486-table-enter/blocked_wasm_eogu_review_021.png) · [overlay](../../working/assets/issue7486-table-enter/blocked_wasm_eogu_overlay_021.png) |
| 교육과정171쪽 | [review](../../working/assets/issue7486-table-enter/blocked_native_curriculum_review_171.png) · [overlay](../../working/assets/issue7486-table-enter/blocked_native_curriculum_overlay_171.png) | [review](../../working/assets/issue7486-table-enter/blocked_wasm_curriculum_review_171.png) · [overlay](../../working/assets/issue7486-table-enter/blocked_wasm_curriculum_overlay_171.png) |

수정 전 base `e1ecaa248…`의 Native binary로 같은 영향 페이지를 다시 비교했다. 어구20/21쪽은 점수와 PNG 바이트가 동일하다. 교육과정은 수정 전에도413/415쪽과 같은 내용 대응 차이가 있으며171/172쪽은66.32949%/24.00083%다. 172쪽 PNG는 동일하고171쪽은46 RGB 픽셀 차이가 있어 완전 동일로 보고하지 않는다. 이 비교는 기존 결함의 존속을 보여주며 #7544가 새로 만든 렌더링 회귀라는 판정은 아니다. 기존 결함이라는 분류도 현재 gate 실패를 해소하지 않는다.

사용자는 **실패 증적·보류 사유를 #7544에 반영하고 기존 출력 결함은 분리**하도록 승인했다. 어구 독립 출력 판정은 [#7207](https://github.com/edwardkim/rhwp/issues/7207), 교육과정 전체 피델리티는 [#7445의 기존 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5874264216)에 연결한다. 이번 증적 보정에서 renderer·회귀·baseline은 변경하지 않는다. 저장 guide의 0 전진 의미 계약은 독립 시각 실패 때문에 여전히 미검증이며, 동일 원문 출력 개선과 새 head 재검증 전에는 승인·병합하지 않는다.

## 2026-10-03 초기 재검토 발견 사항 — 재검증 전 기록

### [P1] 저장본 종료 guide 분기를 독립 출력으로 검증해야 한다 — 필수 증거 부족

`src/renderer/typeset/paragraph/empty.rs:25-70`의 새 판단은 fit하지 않는 마지막 빈 문단을 분할 표의 종료 guide로 수용한다. `paragraph.rs:623-635`에서 정상 이월을 생략하고, `state.rs:612-626` 및 `inline_flow.rs:61-82`에서 저장 vpos를 실제 배치 원점으로 쓰면서 흐름 전진은 0으로 유지한다. 이는 문단 소속 보존 외에 줄 위치·점유의 해석을 바꾸는 경로다.

제출된 4개 합성 HWPX와 대응 PDF, 대표 PNG·90% gate는 반복 Enter 경로에 대한 증거다. guide 판단은 직전 항목이 `PartialTable`이고 바로 다음 마지막 문단이어야 하므로, 표 뒤에 여러 빈 본문 문단을 만든 그 사례들의 통과를 guide 분기의 증거로 대신할 수 없다. 실제 발동 입력은 `samples/task2097/18095317_eogu_geumji.hwp`와 `samples/task2287/1342000_edu_curriculum_map.hwp`다. 현재 기록은 21/413쪽 유지, 저장 LineSeg 값, 기존 관련 회귀와 용지 밖 원장 통과다. `validation.json`의 `guidePlacementDiagnostic`도 어구 문서의 줄 좌표를 저장 vpos로 계산해 확인하며, 동일 원문의 독립 PDF와 직접 대조한 결과가 아니다. 어구 문서의 기존 마지막 표 off-canvas 1건은 남아 있다.

저장 줄이 본문 안에 있다는 사실과 쪽수 회귀 통과만으로, 이 빈 문단이 전진하지 않는 종료 guide인지 또는 표 뒤에서 실제 공간을 차지해야 하는 빈 줄인지 확정할 수 없다. 두 실제 입력의 변경 페이지에서 표 끝·빈 문단 원점·마지막 내용과 다음 구역 소속을 동일 원문 한컴 PDF의 Native/fresh WASM 출력과 대조하고 직접 판독해야 한다. 쪽수 주장은 전체 쪽 대응을 확인한다. 적용되는 실루엣·대표 이미지 gate와 입력 커밋 요건도 함께 충족한 뒤 재판정한다. 자료가 부족한 현재 상태를 새 결함 재현이나 시각 통과로 기록하지 않는다.

근거: [조판 원칙 검토 §2.7](../../manual/pr_review/intake_and_review.md#27-조판-원칙-준수-검토), [시각 검증 §3.5](../../manual/pr_review/visual_fixture_evidence.md#35-시각-검증-원칙). 저장 LineSeg 재사용과 편집 후 재조판의 증거를 구분하고, 실제 호출 경로의 미검증 범위를 승인으로 바꾸지 않는다는 공통 계약을 적용한다.

### 완료한 재검토

- 정확한 head `3edcbdbe…`의 [Full CI run 37045552995](https://github.com/edwardkim/rhwp/actions/runs/37045552995) 실제 로그에서 `CLASSIFICATION_STATUS: full`, lint·Native Skia·Archive A/B/C/D·Build & Test 성공을 확인했다. CodeQL, Render Diff, Adapter inter-diff, Proptest 및 CI Impact Policy도 성공이며 pending/failure가 없다. 정책상 skip과 성공은 구분한다.
- 같은 production/test source인 `97772d5d…`와 최종 head 사이 source/test 차이는 없다. 로컬 전체 Cargo·lint는 기존 실행과 정확한 head의 Full CI를 재사용했으며 중복 실행하지 않았다.
- 별도 review checkout에서 이번 리뷰의 경계·관련 회귀를 다시 실행했다. `cargo nextest run --locked --cargo-profile release-test --tests --no-fail-fast -E 'test(/issue_7486_enter_overflow_opens_page|issue_2097_band_fill|issue_6981_split_straddle_row_height|issue_7226_rowspan_only_row_cut|issue_6761_stored_vpos_rewind_page_break|off_canvas_does_not_grow_partition_12/)'`는 21 PASS / 0 FAIL / 10277 미선택, 11.176초다. shared `target/pr-review`를 재사용했으며 source를 변경하지 않았다. 로그는 ignored `output/pr-review/issue7486-table-fix-20261003/review7544-focused.log`에 있다.
- `section.rs`에서 명시적 경계와 저장 reset 처리는 일반 문단 flow 이전에 실행됨을 확인했다. `paragraph/flow.rs`의 조기 반환 순서만으로 저장 쪽 나눔 회귀라고 판단하지 않았다.
- guide 원점 생산 → `ColumnContent.inline_flow_plans` → layout `FullParagraph` → `layout_inline_flow_plan`의 실제 호출 경로를 대조했다. 같은 plan이 전달되는 것은 확인했지만, 독립 출력과의 일치까지 입증하는 근거로 확대하지 않았다.
- 원격 base `e1ecaa248…` / head `3edcbdbe…`의 merge simulation은 exit0, tree `b353863c3a4ae77cea2049af9db1fb8a4780d92e`다. 공백·상대 링크22건·기존 오늘할일228개 보존을 통과했다.

위 초기 재검토는 당시 로컬에만 기록했다. 이후 사용자 승인에 따라 실제 저장본 실패 증적·보류 기록을 동일 PR branch로 게시한다. GitHub Approve·merge·이슈 종료 승인은 포함하지 않는다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#7544](https://github.com/edwardkim/rhwp/pull/7544) / postmelee / devel |
| 기준 base | `e1ecaa248ecf7f667d8fccab4d9938e70a253392` |
| production/test 검증 source | `97772d5d40787e77c3238debc2b2576b11713476` |
| 제출 code candidate·재검토 head | `c0b075ca93b284af6d6c54975b92e7e7ae3531dd` → `3edcbdbe172767d135ce420e459a41fc9b44f12e`; 검증 source 이후 증적·문서만 변경 |
| 관련 이슈 | [#7486](https://github.com/edwardkim/rhwp/issues/7486), `Fixes`; #7487/#7539의 잔여 표 경로 |
| reviewer·시점 상태 | 본인 PR이므로 reviewer 미지정 / 재검토 시점 Open, non-draft, MERGEABLE / mergeStateStatus=CLEAN / head `3edcbdbe…` CI 성공; 실제 저장본 시각 gate 실패로 판정은 보류 |
| 라우팅 | collaborator self-merge §8.2.1; archive self-review + 오늘할일을 trailing 문서 commit으로 포함 |

## 변경과 검토 범위

`section.rs`의 사후 빈 쪽 삭제와 `trailing_disposition`의 앞 빈 줄 drift 기반 Hidden 처리를 제거한다. 이미 fit·배치한 새 빈 줄의 쪽·문단 소속을 유지한다. 저장본의 완료 PartialTable 직후 종료 guide는 유효한 실제 LineSeg와 본문/zone 기하를 검증한 뒤 같은 쪽에 소유시키며, 그 줄의 저장 원점과 표가 소비한 흐름 끝을 공유 plan으로 전달한다.

생산 결과 `FormattedParagraph` → `paragraph/flow.rs` fit/실패 이월 → `PageItem` → `section.rs` finalize → cursor lookup을 대조했다. 종료 guide는 `is_stored_table_closing_guide` → `place_stored_empty_guide` → `ColumnContent.inline_flow_plans` → layout `FullParagraph` plan → `layout_inline_flow_plan`으로 연결된다. `start`는 저장 vpos를 현재 zone 기준으로 변환한 실제 줄 원점이며 `end`는 기존 표 흐름 끝이다. 마지막 guide가 원점을 다시 추측하거나 흐름 끝을 clamp하지 않는다.

표 cut·rowspan·유닛 예약/실제 table paint는 변경하지 않는다. table coordinator가 모든 조각을 소비한 뒤의 문단 소유와 최종 페이지 할당을 고친다. 일반 본문 뒤 반복 Enter, 저장 줄이 본문 밖인 경우, 명시적 쪽/구역 나눔·상단 reset, 합성/무효 LineSeg, 다단은 새 guide 경로에 들어오지 않는다. 셀 내부 편집과 Studio source는 변경하지 않았다.

## 조판 원칙 판정

| 항목 | 판정·근거 |
| --- | --- |
| 구현 근거와 일반성 | 반복 Enter는 충족. 종료 guide의 의미 해석은 미검증. 실제 저장본 2개의 독립 출력 비교는 실행했으나 출력이 미달해 해당 해석의 정확성을 입증하지 못했다. 문서 ID·표 행 수 예외, clamp는 없다. |
| 측정·배치 일관성 | 동일 plan 전달은 충족. 반복 Enter는 동일 fmt advance의 fit/배치를 보존한다. 종료 guide의 저장 줄 원점과 흐름 끝은 layout까지 공유하지만 그 원점·0 전진이 독립 출력과 맞는지는 미검증. |
| 분할·이어받기 계약 | table 컷/rowspan/요구·예약 높이 변경은 비해당. 기존 #7226/#6981의 쪽수·후속 구역·셀 포함과 #2097/#6761의 페이지 핀, off-canvas partition12는 통과했다. 최종 컷 뒤 guide의 독립 시각 계약은 미검증. |
| 줄 소속과 점유 높이 | 반복 Enter는 충족. 빈 글자의 표시·줄 상자·흐름 전진을 구분하며 정상 이월한다. 저장 종료 guide를 전진 없이 수용하는 의미 계약은 미검증. |
| 사례와 증거의 독립성 | 반복 Enter는 충족. 합성 계약 4개·동일 원문 PDF 전체7쪽을 확인했다. 실제 저장본 2개의 독립 PDF는 확보·재사용했고 Native/fresh WASM 비교 결과는 미충족이다. 쪽수·CI 통과로 대체하지 않는다. |
| 기준값 변경 | 비해당. baseline·golden·허용치 변경 없음. 첫 전체 회귀 4 FAIL, 중간 guide의 off-canvas 1 FAIL은 원인 보정으로 해소했으며 래칫을 완화하지 않았다. |
| 주장과 검증 범위 | 부분 충족. 아래 증거는 반복 Enter 범위를 입증한다. 실제 저장본 출력은 미충족, 종료 guide 의미·교육과정 전쪽 대응·Docker 표준 WASM·별도 정량 benchmark는 미검증. |

## 검증 입력과 실행 결과

입력 커밋 확인은 **충족**이다. 실제 사용한 HWPX 4개는 `tests/fixtures/issue7486_table_enter/`, 대응 한컴 PDF 4개는 `pdf/issue7486_*-2020.pdf`이며 `3f1e5c596`부터 검토 history에 있다. 검증 source commit에서 실행 파일과 Git blob의 SHA-256을 대조했다. 파일로 사용한 합성 입력과 비교 PDF는 외부 경로에만 두지 않았다. 실제 기존 문서 8개 및 #2097/#2287 저장본도 기존 repository blob과 대조했다. 경로별 SHA-256·실제 source/backend 출처는 [validation.json](../../working/assets/issue7486-table-enter/validation.json)에 있다.

[원인·전후 실행 보고서](../../report/task_m100_7486_table_report.md)와 [단계 기록](../../working/task_m100_7486_table_stage1.md)을 근거로 사용한다.

| 명령·범위 | 결과 |
| --- | --- |
| 별도 review checkout `--prepare`; `cargo fmt --all -- --check` | PASS |
| Native / WASM32 lib / workspace all-target Clippy `--locked`, `-D warnings`; workspace build | 모두 PASS |
| suite 정책 `--check --base-ref e1ecaa248…`; manifest 계약 검사 | PASS |
| 작은 경계 + off-canvas partition12 | Summary [  10.540s] 21 tests run: 21 passed, 10277 skipped |
| 관련 focused | Summary [  10.999s] 20 tests run: 20 passed, 10278 skipped |
| `cargo nextest run --locked --cargo-profile release-test --tests --no-fail-fast` | Summary [ 249.784s] 10248 tests run: 10248 passed (6 slow), 50 skipped |
| `cargo test --locked --profile release-test --features native-skia --lib` | test result: ok. 3927 passed; 0 failed; 13 ignored; 0 measured; 0 filtered out; finished in 44.50s; test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s; test result: ok. 165 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s; test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s |
| `run-rust-test.mjs issue_2225_missing_picture_placeholder -- … --features native-skia` | Summary [   1.186s] 2 tests run: 2 passed, 224 skipped |
| `run-rust-test.mjs render_p37_direct_pdf_export -- … --features native-skia` | Summary [   0.956s] 4 tests run: 4 passed, 219 skipped |
| root `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt` | fresh host release WASM PASS; root/public/서버 SHA-256 일치 |
| Chrome Enter/Undo/Redo / API 소속 진단 / 8개 저장 문서 대조 | 56 PASS / 720 owner 누락0 / 쪽수 변화0 |
| 반복 Enter 합성 입력 전쪽 Native/fresh WASM sweep / 직접 판독 | 전체7쪽×2backend 최저100%, 대표4gate passed; 실제 저장본은 상단 실패 결과와 구분 |

모든 Cargo는 같은 절대 `target/pr-review`를 순차 재사용했다. sweep 명령은 `pr-sweep-guide.sh`의 `--silhouette-only` 전체 쪽 및 `--pages 1,2` 대표 review 경로이며 각 실행의 provenance와 입력 해시는 validation.json에 보존했다.


새 회귀 최종 검사본을 동일 review checkout에서 base에 적용했을 때 기존 4개 PASS, 새 2개는 문단34/문단10 소속 없음으로 FAIL했고 보정 뒤 6 PASS다. 환경·빌드 실패를 수정 전 결함 FAIL로 계산하지 않았다. 정식 회귀 추가 전 `75b433ccb`의 Native/dev WASM 최저 100%·직접 판독 선행 증거를 보존했으며 최종 source에서 새 release host WASM과 Native로 다시 전쪽 비교·캡처했다.

API 12개 조합의 720 Enter 소속 누락 0, Chrome 실키 Enter/Undo/Redo 56 PASS, 기존 저장 문서 8개 쪽수 변화 0이다. Chrome은 사용자 세션과 별도의 headless 실행이며 사용자가 로컬 결과를 직접 재검증했다. 조작 없이 새 쪽 DOM 캐럿·viewport·100% 스크롤을 검사한다. 합성 계약 진단을 12개 조합 전부의 한컴 출력 검증으로 확대하지 않는다.

소스의 cfg(test), npm/editor, Studio source, capability, sample 원장은 변경하지 않아 해당 별도 게이트는 비해당이다. 실행 로그와 파생 suite는 ignored output에 있으며 제출 diff에 stage하지 않았다.

## 시각 증적과 남은 차이

| 경계 | Native review·overlay | fresh WASM review·overlay |
| --- | --- | --- |
| 10행·160%·Enter33 | [review](../../working/assets/issue7486-table-enter/native_table33_review_001.png) · [overlay](../../working/assets/issue7486-table-enter/native_table33_overlay_001.png) | [review](../../working/assets/issue7486-table-enter/wasm_table33_review_001.png) · [overlay](../../working/assets/issue7486-table-enter/wasm_table33_overlay_001.png) |
| 30행·300%·Enter9 | [review](../../working/assets/issue7486-table-enter/native_table30_review_001.png) · [overlay](../../working/assets/issue7486-table-enter/native_table30_overlay_001.png) | [review](../../working/assets/issue7486-table-enter/wasm_table30_review_001.png) · [overlay](../../working/assets/issue7486-table-enter/wasm_table30_overlay_001.png) |

Enter32/33/40의 1/2/2쪽, 30행 Enter9의 2쪽 전체를 96dpi print·2px 관용으로 비교했다. 양쪽 표 외곽·행/열 경계·시작/끝 위치와 빈 2쪽을 직접 판독했다. 모든 TSV 최저 100%, 원값 100%, boundary reconciliation 추가 픽셀0; 대표4출력 pr_review_gate=passed이다. 표 선 밝기 차이가 남으며 엄격 내용 픽셀은 약0.02%/7.46%다. 평균·실루엣으로 색상 차이를 면제하지 않는다. 글꼴 예외·영역 마스킹·관용치 변경 없음.

PR 본문에는 최신 head SHA의 실제 raw URL로 대표 review·overlay8개와 Studio4개를 표시한다. 게시 후 이미지 다운로드 해시와 브라우저의 실제 표시를 확인한다. source SHA와 asset이 추가된 제출 head SHA를 구분한다.

## Merge 후 contributor PR comment 계획

본인 PR이므로 원 기여자 PR 본문 수정·대신 Approve는 비해당이다. 별도 병합 승인을 받으면 이 PR과 #7486의 후속 처리에서 merge SHA·성공 CI URL 및 [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#pr-body-visual-evidence)을 연결하고 실제 대표 PNG를 merge SHA로 고정해 재게시한다. raw URL은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/working/assets/issue7486-table-enter/native_table33_review_001.png`와 동일 WASM/overlay 경로를 사용한다. 표 선 밝기와 Docker fallback 제한을 함께 적고 `--body-file` 게시 뒤 API로 본문·이미지를 재조회한다. 이 계획은 이번 턴의 게시·merge 승인으로 간주하지 않는다.

현재는 실제 저장본 시각 gate 미달로 위 merge 후 게시 계획을 실행하지 않는다. 합성 입력의 일치를 전체 실제 문서 일치로 표현한 기존 예정 문안은 철회한다. 출력 개선 뒤 정확한 새 head의 증거로 다시 판정하고 merge 문안을 작성한다.


## 원격 제출·병합 사전 확인

code candidate `c0b075ca93b284af6d6c54975b92e7e7ae3531dd`를 upstream 작업 branch에 정상 push했고 Open PR #7544를 생성했다. 검증 source 이후 `src/`, `crates/`, `tests/`, Cargo·scripts·Studio source diff는 없다. force-push·GitHub Approve·merge·이슈 close는 수행하지 않았다.

candidate push 전에 base `e1ecaa248…`, head `c0b075ca9…`의 `git merge-tree --write-tree`는 exit0, tree `7d5a8d3de1d6b8a64a2317e4d48f9acb78c9deb8`이며 공백·상대 링크9건·기존 오늘할일228개 파일 보존을 통과했다. 이 문서와 오늘할일 trailing commit도 같은 절차를 push 전에 적용하고, 원격 head/base 재조회 및 최종 본문/이미지 API·브라우저 확인 결과를 PR 준비 로그에 보존한다. 작성 시점 CI URL: https://github.com/edwardkim/rhwp/actions/runs/37045229606 . 성공 결과로 기록하지 않는다.
