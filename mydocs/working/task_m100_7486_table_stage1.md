---
kind: working
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-07
---

# #7486 — 표 뒤 Enter의 확정 쪽 소유 보존

## 현재 단계 — 2026-10-07

source `5308d4087d`에서 새 저장 guide 해석을 제외하고 Enter 쪽 소속 보호로 범위를 고정했다.
독립 PDF Native/fresh WASM 전7쪽 최저100%, 전체10,511 PASS·필수 lint·Skia3종·fresh WASM 통과다.
저장본448쪽은 각 backend에서 devel과 모든 tree가 동일하다. 최신 원격 CI·mergeability는 확인 전이다.
[최신 self-review](../pr/archives/pr_7544_review.md#2026-10-07-보류-해제-보정과-최종-로컬-검증)를 정본으로 사용한다.

## 이전 단계 이력 — 2026-10-03 제출과 저장 guide 실험

기준: `upstream/devel` `e1ecaa248ecf7f667d8fccab4d9938e70a253392`.
Branch: `codex/table-enter-page-ownership`. 최초 범위는 로컬 보정·검증 후 직접 재검증이었다.
사용자가 수정 결과를 확인했고 2026-10-03 PR 제출 준비를 승인했다. 원격 제출은 필수 검증 뒤 진행한다.

## 근거와 수정 방향

실제 편집 API로 만든 A4·10pt·160%·10×2 위아래 표 뒤에서 Enter를 반복했다.
수동 XML·LineSeg 수정 없이 `createTable` → `splitParagraph` → `exportHwpx`로 입력을 보존했다.
수정 전 Enter33~39의 문단34~40은 페이지 소속이 없고 Enter40에서 다시 나타난다.
같은 Enter32/33/40 원문의 독립 한컴 PDF는 각각 1/2/2쪽이다.
저장 제품은 입력 `info`의 `hancom-office-2020`이며 2024 엔진으로 표시하지 않는다.

가시 텍스트 없음은 줄 공간 점유 없음이 아니다. `discard_terminal_blank_only_page`는 이미 fit/배치가
확정한 끝 쪽을 삭제하면서 문단 소속도 제거했다. 저장 vpos가 표 높이를 제외하고 있으면 #7487의
저장 줄 overflow 예외로도 이를 구제할 수 없다. 사후 삭제 자체를 제거한다. 특정 표 크기·문서 ID·
저장 vpos에 새 예외를 덧붙이거나 출력 좌표를 clamp하지 않는다. guide 흡수는 기존 문단 배치 판단에 남긴다.

생산→소비 경로: table 측정과 문단 `FormattedParagraph` → `paragraph/flow.rs`의 fit 예산·whole-fit →
실패 시 `place_after_failed_fit`/`place_split_paragraph`의 동일 line advance →
`TypesetState::advance_column_or_new_page` → 확정 `ColumnContent`/`PageContent` →
`section.rs`의 섹션 끝 확정 → 페이지 번호·HF 부착 → layout 및 `getCursorRect`.
Enter33 진단에서는 앞 쪽 used height 878.7067px, 다음 쪽 문단34의 advance 21.3333px를
이미 확정한 뒤 마지막 `pages.pop()`이 그 소속을 지웠다. 표 컷·요구 높이·예약 높이·rowspan·paint는
바꾸지 않는다. 이 수정의 계약은 확정 결과 보존이며 분할 알고리즘 변경이 아니다.

## 선행 검증과 다음 단계

ignored `output/pr-review/issue7486-table-fix-20261003/`에 원문·독립 PDF·전후 CLI 진단을 보존한다.
Native 선행 진단에서 Enter33은 1→2쪽, 저장 문서 8개 대조군은 쪽수 변화가 없다
(`page-count-controls.json`). 이 대조는 한컴 시각 일치 완료나 전체 회귀 통과를 의미하지 않는다.
최종 source의 Native/fresh WASM 전체 페이지 sweep와 직접 판독이 끝나기 전 새 정식 회귀를 추가하지 않는다.
관련 최저 실루엣 90% 이상에서만 소속·내용 보존 회귀를 추가하고 수정 전 FAIL/후 PASS를 확인한다.
실제 Studio 키 Enter·Undo·Redo 및 캐럿/viewport를 확인한 뒤 사용자 재검증용 서버를 유지한다.
PR 전 전체 Rust lint·release-test·Native Skia 게이트는 별도 PR 준비 단계에서 완료해야 한다.

추가 진단에서 30행·300%·Enter9는 사후 쪽 삭제를 제거해도 `trailing_disposition`의
`prior_trailing_drift && previous_item_is_empty_para` 분기로 숨겨졌다. 앞 줄의 누적 간격과
새 줄의 점유는 별개이므로 이 Hidden 분기를 제거한다. 문단 자체의 미세 overflow 및 각주 예약
흡수는 유지한다. 이 입력도 원본을 보존해 독립 한컴 PDF와 대조하며, 표 행 수를 가드에 쓰지 않는다.

로컬 검증 결과와 사용자 재검증 안내는 [결과 보고](../report/task_m100_7486_table_report.md)에 연결했다.
4개 원문 전체 7쪽 × Native/fresh WASM 최저 실루엣 100%를 먼저 충족한 뒤 정식 회귀 2개를 추가했다.
동일 최종 검사본은 base에서 4 PASS/2 FAIL, 보정 후 6 PASS/0 FAIL이다. 사용자 재검증을 마쳤으며
PR 전 전체 검증·원격 제출은 결과 보고에서 이어 기록한다.


## 2026-10-07 범위 분리 후 로컬 재검증 후보

- base route: collaborator_self_merge.
- modifiers: intake_and_review, local_validation, visual_fixture_evidence.
- loaded documents: pr_review_workflow.md, pr_review/README.md 및 위 네 자식 문서.
- 원격 #7544 head `e22b099a28`는 변경하지 않는다.
- 최신 base `48ff4bb935`와 #7207의 별도 정렬 후보 `cf2b0508fc`를 선행으로 둔
  로컬 브랜치 `codex/pr7544-focus-recheck-20261007`에서 원 PR을 병합한다.
  최소 선행 후보는 이후 #7627로 `7076f836e2`에 병합됐다. 아래 source 검증은
  `2b034275a5`의 실행이며 최신 devel을 정렬한 로컬 `4d9889a7f4`와 source/test가 같다.
  현재 원격 PR의 통과 결과로 표현하지 않는다.
- Rust source의 textual conflict는 없고 `20261003.md` add/add만 양쪽
  기존 작업 기록을 모두 보존해 해소했다. 교육과정 전체 개선은 이 후보에
  포함하지 않으며 #7445의 담당 작업을 인수하지 않는다.
- 이 기록은 준비 단계다. 정확한 후보에서 Enter·저장/재열기·실제 저장본
  영향 검증을 실행하며 판정은 여전히 머지 보류다. 전체 Rust/lint/CI와
  원격 push·review·merge 완료를 주장하지 않는다.

### 재검증 결과와 남은 범위

로컬 code head `2b034275a53ae7d65c9a4670982d9a23fddc60fa`에서 최신
`InlineFlowPlan` 필드 초기화 호환성만 추가 보정했다. 반복 Enter4개 원문 전체7쪽은
Native/fresh WASM 최저100%, 실제 어구21쪽은 양쪽 최저93.22327%다.
기존 Enter/저장 재열기6개와 #7207 소속3개는 실제9PASS이며 fmt 및
base `48ff4bb935` 고정 manifest 정책도 통과했다.

교육과정 Native413쪽/독립 PDF415쪽 차이는 남고 guide171/172쪽은
66.38064/24.14817%다. Native171쪽을 직접 판독해 내용 소속 차이를 확인했다.
이번 source의 fresh WASM guide·교육과정 전쪽 대응과 전체 Rust/lint/CI는
미검증이다. 이전 head나 어구의 통과를 이 경로의 증거로 대신하지 않는다.
저장 guide의 0 전진 의미는 독립 출력으로 입증되지 않아 승인·통합은 보류한다.
교육과정 전체 구현은 이 후보에 포함하지 않는다.

정확한 후보의 수치·대표 PNG·입력/source 해시는
[최신 리뷰](../pr/archives/pr_7544_review.md#2026-10-07-범위-분리-후-로컬-후보-재검증)와
[재검증 원장](assets/issue7486-table-enter/recheck_20261007.json)에 연결한다.
새 sweep에는 기존 독립 PDF를 재사용했고 사용자 제공 파일은 로컬 검증에만 사용했다.
원격 PR source·초기 본문·Approve·merge 및 이슈 상태는 유지했다. 이후
[최신 검토 댓글](https://github.com/edwardkim/rhwp/pull/7544#issuecomment-6031830291)을 게시했고
[검토 기록](../pr/archives/pr_7544_review.md)에 최소 선행 병합과 분리 실험4PASS/2FAIL,
6PASS·교육과정414/415쪽 및 진단 source 복원·미실행 범위를 기록했다.
