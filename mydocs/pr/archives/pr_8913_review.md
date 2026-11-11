# PR #8913 검토 — 여러 문단 선택의 문단 부호 표시

## 최종 판정

**승인 — 원 기여의 선택 사각형 수정과 정상 대조군 검증을 완료했다.** 메인터너의 제품 코드·테스트 보정은 없다.
작업지시자의 두 PR 머지·후속 처리 승인 뒤 원 head의 CI와 최신 devel 병합을 재확인하고
`243e6417278564228570cc1ccb879d5cc52f43f9`로 원 PR을 merge했다. 후속 기록 반영 뒤 연결 이슈·감사 댓글을 처리한다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#8913](https://github.com/edwardkim/rhwp/pull/8913) / semanticist21 / devel |
| 관련 이슈 | [#8908](https://github.com/edwardkim/rhwp/issues/8908), 해결 범위 전체 |
| 원 code head | `66141a6c485844e37cdd133f7ac92890e9d8a2c3` |
| 최신 검증 base | `e118de8abadd2dffb2798036b9ffd381b96ed7d5` |
| 누적 로컬 실행 source | `18623a34e8a6bea48d2908ee85c34ae786f4feab`, #8913·#8915의 원 SHA 보존 병합 |
| reviewer·접수 상태 | 미지정 Ready PR에 `edwardkim` 지정·API 확인. 2026-11-11 KST 재조회: head 불변, clean |
| 원격 처리 상태 | [승인 리뷰](https://github.com/edwardkim/rhwp/pull/8913#pullrequestreview-5854957180)·원 PR merge 완료. 원 fork push 없음 |

## 원인과 소비 경로

최신 devel의 `cursor_nav.rs`는 이전 기준 `5ea1227505978156082121f1601cb03044a0583f`와
Git blob `4f1b71bce7a62a187c288bb9371fcac5f4ee7c5e`가 같다. 기존 선택 query는 중간 문단의
마지막 줄도 열 오른쪽까지 칠하고 빈 문단·문단 끝에서 시작하는 선택을 누락한다.

원 기여는 `src/document_core/queries/cursor_nav.rs`의 공통 선택 사각형 계산에서 문단 마지막 줄을
글자 끝 caret + 문단 부호 너비 2.5pt까지 표시한다. 본문·셀의 공통 query를 사용하고, 같은 문단의
부분 선택 끝점과 자동 줄바꿈 중간 줄의 열 끝 채우기는 유지한다. 문단 마지막 줄 판정은 실제
`rendered_starts`를 반영한 줄 수를 사용한다. layout tree 재사용 계약도 유지한다.

실제 소비 경로는 `DocumentCore::get_selection_rects_for_target_native` → WASM
`getSelectionRects*` → Studio `InputHandler.updateSelection()` → `SelectionRenderer.render()`다.
Native host의 `native_selection_rects`와 접근성 query도 선택 geometry를 소비한다. 문서 page tree의
측정·배치·paint·저장·export는 이 반환값을 소비하지 않는다. 이 변경은 문서 조판 수정이 아니다.
따라서 Print PDF 실루엣 TSV·편람 383쪽·정본 보호 TSV·Native Skia 출력 3종은 비해당이다.

## 검증 입력과 결과

입력 생성 API·선택 범위는 [inputs.json](../assets/semanticist21-20261111/inputs.json),
실행·producer·체크 목록은 [공통 검증 원장](../assets/semanticist21-20261111/README.md)에 보존했다.

| 검증 | 실제 결과 |
| --- | --- |
| 원 head GitHub CI | [42803126005](https://github.com/edwardkim/rhwp/actions/runs/42803126005), Lint·A–E build/test·집계 성공. 별도 required check 포함 36개는 success 또는 정상 skip·neutral |
| 최소 글꼴 focused Rust | `body_selection_tree_reuse` 4 PASS. 공개 글꼴 5개만 노출, 나머지 304개는 필터 제외 |
| 이번 소스의 WASM API·브라우저 | 본문·빈 문단·mark-only 시작·부분 끝점·collapsed·셀·자동 줄바꿈 등 11/11 PASS |
| 음성 대조 | 이전 실제 WASM은 동일 검사 11개 중 6개 실패. 최신 base의 원인 코드 blob 동일성을 별도로 확인 |
| 실제 Studio 선택 overlay | 본문·셀, zoom 1·0.66의 실제 DOM과 WASM 사각형 투영 8개 assertion PASS |
| 기존 400문단·10쪽 선택 대조군 | 156개 assertion PASS. Undo/Redo·Backspace·스크롤·비동기 복원·문서 교체, pageerror·render error 0건 |
| 문서 출력 불변 대조 | 본문·셀 SVG가 선택 전후 같고, 이전 producer와 후보 사이에도 동일. 문서 fidelity 점수로 환산하지 않음 |
| 최신 base 병합 | 개별 `merge-tree --write-tree` exit 0. #8915와 누적 적용도 clean |

로컬 WASM은 locked wrapper의 **dev 빌드**다. source·JS/WASM 해시와 브라우저 실소비 파일을
기록했으며 선택 API/UI 동작을 검증했다. 최적화·배포 성능·문서 렌더링 검증 결과로 주장하지 않는다.
Studio 검사는 실제 CursorState·InputHandler·WasmBridge·SelectionRenderer를 실행했으며
마우스 수동 조작 검사로 기록하지 않는다.

[녹색 code head 재사용 규칙](../../manual/pr_review_workflow.md#322-녹색-github-code-head의-중복-로컬-전체-회귀-생략)에
따라 원 head의 전체 Rust CI·Lint 결과를 재사용했다. source·test·fixture·workflow 보정이 없고
current-base merge가 clean이므로 전체 nextest를 다시 실행하지 않았다. Native Skia는 이번
query-only 범위에서 원 CI도 요구하지 않았으므로 성공했다고 기록하지 않는다.

## 독립 GUI 증거와 직접 판독

기준은 원 기여에 포함된 Hancom 12.30 Mac의
[본문](../assets/issue_8908_selection_paragraph_mark/hancom-paragraphs.png)·
[셀](../assets/issue_8908_selection_paragraph_mark/hancom-cell.png)·
[자동 줄바꿈](../assets/issue_8908_selection_paragraph_mark/hancom-wrapped.png) 선택 화면이다.
선택 overlay는 Print에 출력되지 않으므로 PDF 기준이 아니다. 10pt·20pt에서 문단 부호 너비가
2.5pt로 관측된 GUI 계약을 검토했다. 글꼴·플랫폼별 글줄 높이와 간격이 다른 화면 전체의 RGB
동일성을 주장하지 않는다.

후보 Studio의 [본문 zoom 1](../assets/semanticist21-20261111/studio/body-zoom1.png)·
[zoom 0.66](../assets/semanticist21-20261111/studio/body-zoom0.66.png)·
[셀](../assets/semanticist21-20261111/studio/cell.png)을 직접 확인했다. 빈 문단 두 줄과 셀의 빈 줄도
짧은 선택 표시가 있고, 문단 끝에서 열 전체로 길게 칠하던 현상이 사라졌다. 부분 끝점과 정상
자동 줄바꿈은 유지했다. 문서 조판의 남은 차이를 이 UI 수정의 완료로 간주하지 않는다.

## Merge 후 contributor PR comment 계획

원 head를 보존해 원 PR을 merge한다. merge SHA 확인 → review archive·오늘할일·증적의
mydocs 운영 기록 반영 → devel 동기화 → #8908 상태 재조회·필요한 완료 댓글/close → 감사 댓글 순서다.
공개 댓글에는 실제 merge SHA, 위 CI, focused 4건·WASM 11건·Studio 8건·정상 대조 156건을
적는다. 선택 UI 비교임을 명시하고 문서 Print 일치율을 붙이지 않는다. 운영 기록 commit으로
고정한 본문·셀 PNG를 게시 전 원본 해시와 대조한다. 한국어 존댓말 `--body-file` 게시 후 API로
본문·이미지 링크를 재확인한다. 이번 이슈 범위에 남은 미해결 사항은 없다.

## 병합 확정 기록

- 원 head `66141a6c485844e37cdd133f7ac92890e9d8a2c3`, merge `243e6417278564228570cc1ccb879d5cc52f43f9`, 병합 시각 `2026-11-11T06:57:23Z`.
- 후속 문서는 maintainer 직접 반영 경로다. 코드·테스트·workflow를 포함하지 않는다.
- 원 기여 head와 fork branch를 보존했다. 문서 운영 기록·devel 동기화 뒤 연결 이슈와 감사 댓글을 처리한다.

- 연결 이슈 #8908는 자동 CLOSED 상태를 API로 확인했다. 운영 기록 반영 뒤 maintainer 완료 댓글을 게시한다.
