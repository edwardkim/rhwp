# PR #7236 최종 검토 — 줄 밖 독립 수식 텍스트 추출

## Metadata와 처리 경로

2026-09-20 작성 시점 참고값이다. merge 전에 최신 head·CI·mergeability를 재조회한다.

| 항목 | 값 |
| --- | --- |
| PR | [#7236](https://github.com/edwardkim/rhwp/pull/7236), Task #6527: 줄 밖 독립 수식도 페이지 텍스트로 추출 |
| 작성자 | lpaiu-cs |
| Issue | [#6527](https://github.com/edwardkim/rhwp/issues/6527), OPEN |
| Base | devel; 최신 통합 확인 기준 `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db` |
| 검증한 contributor head | `95d463a96373ab38cb2ccd45809f49f63e84bfa9` |
| Head 저장소·branch | `lpaiu-cs/rhwp`, `fix/6527-standalone-equation-text` |
| 원 변경 규모 | 1 commit, 2 files, +55 / -4 |
| 상태 | Ready, MERGEABLE / CLEAN, maintainer_can_modify=true |
| 기존 원격 리뷰 | [postmelee COMMENTED](https://github.com/edwardkim/rhwp/pull/7236#pullrequestreview-5256573072), 동일 source SHA |

- base route: collaborator_external_pr, 원 contributor PR에 문서만 추가하는 경로.
- modifiers: intake_and_review, local_validation, review_only_fast_pass.
- loaded documents: pr_review_workflow.md, pr_review/README.md와 위 경로 문서.
- source commit은 원 작성자 lpaiu-cs의 위 commit 하나다. collaborator는 이 리뷰 Markdown만
  single-parent 후속 commit으로 추가하며 원 commit의 author·내용·history를 변경하지 않는다.

## 변경과 검토 결과

`extract_page_text_native`의 `collect_page_lines`가 `TextLine` 외에 독립 `Equation`을 만나도
기존 `collect_line_text`로 자기 노드의 스크립트를 수집한다. `TextLine` 처리 후 반환하는 경계가
유지되어 줄 안 수식을 재귀 순회로 다시 출력하지 않는다. 포맷별 특례나 수식 내용의 재해석은 없다.

공개 sample16 물리 6쪽에서 줄 밖 수식 두 개를 누락하던 호출 경로와 회귀 assertion을 대조했다.
테스트는 수식 49·81자, 독립/inline 구조, 각각 한 번 출력, 제목 뒤 읽기 순서를 확인한다.
기대 계약은 이슈의 두 공개 입력과 기존 수식 텍스트화 동작에 근거한다.

수정이 필요한 코드 결함은 발견하지 못했다. 이슈에 별도 관찰로 적힌 제목 누락과 Markdown 추출은
이번 수식 누락 수정의 완료 기준이 아니며, 해결됐다고 주장하지 않는다.

## 완료한 검증

Ready 전환 이후 source SHA가 바뀌지 않았다. 아래 직접 검증과 동일 head의 CI를 재사용하며,
이번 문서 추가를 제품 코드의 새 검증으로 표시하지 않는다.

| 검증 | 실행·결과 |
| --- | --- |
| focused 회귀 | `node scripts/rust-test-suite-manifest.mjs --prepare` 후 `node scripts/run-rust-test.mjs issue_6527_standalone_equation_text -- --cargo-profile release-test --target-dir <review-target>` 실행. 2 passed, 200 unrelated filtered, exit 0. 파생 manifest 검사 통과 |
| CI | [Full CI 35202322042](https://github.com/edwardkim/rhwp/actions/runs/35202322042) 성공. Rust lint·Native Skia·test archive 회귀와 최종 Build & Test 통과 |
| 보조 CI | [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/35202322008), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/35202321460) 성공 |
| 현재 base 통합 | `git merge-tree --write-tree 722fb38af 95d463a96` 통과. tree `b146b08d7f33a2c49419b5405770b6bfa58edc93`, 공백 검사 통과 |
| 동작 기반 검증 | 실제 `DocumentCore::extract_page_text_native(5)` 호출로 독립 수식과 inline 대조군을 검사했다. 수정 전 독립 수식 테스트 실패·inline 통과는 contributor PR 본문에 기록된 음성 대조이며 reviewer 직접 재실행 결과와 구분한다. |

환경은 macOS arm64, rustc 1.93.1, Node 24.15.0이다. 코드·테스트를 보정하지 않았으며, 정확한 원 head의
녹색 Full CI와 focused 결과를 확인했으므로 광범위 Rust/Native Skia 전체 회귀를 로컬에서 중복 실행하지
않았다. 새 Rust 변경에 대한 lint 게이트를 생략한 경우가 아니다.

## 입력 커밋·조판 원칙

검증 입력 커밋 확인: **충족**. 다음 기존 공개 파일의 실제 바이트 해시가 검증한 contributor commit의
Git blob 또는 LFS oid와 일치함을 확인했다. 개인 첨부나 새 fixture를 입력으로 쓰지 않았다.

| 입력·역할 | SHA-256 |
| --- | --- |
| `samples/hwp3-sample16.hwp`, 줄 밖 수식 원본 | `559fd94860cf836d5800436d56055c23a73e59eedef3da273a5ba365e6ed9e16` |
| `samples/hwp3-sample16-hwp5.hwpx`, 줄 안 수식 대조군 | `49e3e809eb41e22b2c059383db32b0cf038787269b5c523d1ff59d1a52b4340c` |

조판 원칙과 직접 Visual Sweep: **비해당**. 변경은 render tree를 읽어 페이지 텍스트를 추출하는
query에 한정되며 줄 구성·측정·배치·paint·baseline을 변경하지 않는다. 파일명이 rendering.rs인
것만으로 출력 조판 변경으로 분류하지 않는다. 한컴 GUI/PDF와의 시각 일치를 검증했다는 주장은 없다.

## 최종 판정과 남은 단계

- 최종 판정: **승인**. 위 source 범위의 코드 검토와 적용 검증을 충족한다.
- 문서 추가 후 최신 head의 required CI 통과와 mergeability 재확인이 병합 전 조건이다.
- 사용자는 이번 단계에서 문서 commit·원 PR push만 승인했다. CI 통과를 알려준 뒤 최신 head를
  재조회하여 기존 COMMENTED와 별도로 최종 APPROVE를 제출하고 병합한다. 이번 단계에서는 CI를
  계속 감시하거나 APPROVE·merge·issue close를 실행하지 않는다.
- 단일 PR이며 추가 코드 보정·통합·복수 선택 단계가 없어 별도 review_impl/report는 생략한다.
  오늘할일을 새로 만들지 않고 검토·검증·후속 조건을 이 리뷰에 기록한다.
- merge 후 실제 merge SHA·devel 포함과 이슈 완료 기준 충족을 확인하고 필요한 종료 코멘트를 남긴다.
  기여자 fork branch는 정리 대상으로 삼지 않는다.
