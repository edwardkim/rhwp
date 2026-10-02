# PR #7448 리뷰 — 이슈 폼과 선택값 기반 자동 라벨링

## 최종 판정

**승인 — 로컬 검증과 검토 head의 GitHub Full CI 통과.** 검증 head는
`84679025cd52e29b09ec351a180d5bc65b67b07b`이며 작업지시자가 병합과 후속 처리를 승인했다.
이 판정 갱신 commit을 포함한 최신 head의 required check와 mergeability를 다시 확인한 뒤 일반 merge한다.
실제 GitHub 폼 표시·자동 라벨 이벤트는 기본 브랜치 반영 뒤 검증하며 #7447은 자동 종료하지 않는다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#7448](https://github.com/edwardkim/rhwp/pull/7448) / postmelee / devel |
| 관련 이슈 | [#7447](https://github.com/edwardkim/rhwp/issues/7447), Refs로 연결 |
| 구현 source | `caa0f95f1e7a53c34641becfc65a1d6195c412f7` |
| 최초 PR head | `be9935509972220f78d3224650b4d3667e93e055` |
| 기준 base | `443844b593c62a722cf9cc3d9d0256e94ab88cb8` |
| 작성 시점 참고값 | 2026-09-28 KST, Open, MERGEABLE / CLEAN, 검토 head CI 성공 |
| 규모 | 최초 PR 13 files, +787/-54. 이 검토 문서는 후행 기록 |
| 검토 방식 | collaborator 작성자 자체 검토, reviewer 미지정 |
| 원격 경로 | 사용자 승인에 따라 origin fork 사용. upstream 작업 branch 기본 경로의 명시적 예외 |

라우팅: collaborator_self_merge. 보조: intake_and_review, review_template, local_validation,
review_only_fast_pass, post_merge. 위 문서 및 pr_review_workflow와 pr_review/README를 읽고 적용했다.
병합·후속 처리와 이슈 코멘트가 승인됐다. 관리자 우회·main 반영·issue close는 수행하지 않는다.

## 변경과 검토 범위

Markdown 2개를 폼 4개로 대체하고 사용 경로 5개만 기존 라벨에 연결한다. OS는 본문 정보다.
기존·수동 라벨은 삭제하지 않고 제거 이력이 있으면 자동 재부여도 하지 않는다.
일반 본문은 건너뛰지만 알려진 제목 구조를 복사한 본문은 동일하게 분류한다. 폼 출처 인증을
주장하지 않으며 허용 목록·기존 라벨 존재 확인으로 쓰기 범위를 제한한다.

실제 workflow script의 입력 처리, 최신 본문 조회, 제거 이력 pagination, 기존 라벨 확인,
추가 직전 상태 재확인과 addLabels 경로를 검토했다. checkout·본문 실행·본문 로그 출력·라벨 삭제는 없다.
읽기와 쓰기는 원자적이지 않으므로 직전 재조회 이후 동시 수정 가능성은 남는다.

## 검증 입력과 결과

상세 명령과 관측값은 [운영 변경 기록](../../working/task_m100_7447_issue_forms.md)에 있다.

- Node 테스트 16개 통과: 실제 github-script 본문 실행, 4개 폼 × 5개 경로,
  수정·재실행·수동 라벨/제거 이력 보존, 오래된 payload, 처리 중 변경, 미지 입력과 API 실패.
- CI 배선 Python 검사 3개 통과. 새 테스트 실행 1줄을 기존 Lint job에 추가했다.
- 새 workflow actionlint와 YAML 파싱 통과. 기존 CI의 SC2016 1건은 기준 base에서도 동일하며
  ShellCheck를 제외한 Actions 구문 검사는 통과했다. 기존 경고를 신규 결함으로 보고하지 않는다.
- 최초 push 전 base/head merge simulation은 exit 0, tree `d16207db5dd0e34e10440fd9832d1ba5b0ef040a`였다.
- 조판 원칙·Visual Sweep·HWP/HWPX/PDF 입력 커밋 확인: **비해당**. 제품 렌더링·fixture 변경 없음.
- 제품 Rust/WASM/Studio 로컬 검증: 비해당.
- 검토 head의 [CI Full 실행](https://github.com/edwardkim/rhwp/actions/runs/36352740724)은
  Lint, Native Skia, frontend package, Archive A/B/C/D와 Build & Test를 통과했다.
  [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/36352740775),
  [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/36352740669),
  [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/36352740714)도 성공했다.
  판정 갱신 뒤의 최신 head 결과는 merge 직전에 별도 확인한다.

## 남은 적용 검증과 처리 순서

1. 이 문서를 같은 PR에 후행 commit으로 반영하고 최신 head의 CI와 mergeability를 확인한다.
2. workflow 변경을 포함한 fork PR이므로 review-only fast-pass를 가정하지 않는다.
3. 승인된 merge 이후에도 기본 브랜치 main 반영 뒤 실제 chooser·폼·opened/edited
   라벨 추가, 수동 제거 보존, Blank issue·보안 신고 경로를 확인해야 #7447 전체 완료를 판단할 수 있다.
4. 되돌리기는 폼·workflow 변경 revert이며 이미 부여한 라벨을 일괄 삭제하지 않는다.

시각 증적은 비해당이다. 별도 implementation 계획서는 필요하지 않다.
merge 후 이슈에는 PR·merge SHA·최신 CI·로컬 검증 요약을 남기고, 구현 완료/main 적용 대기 상태로
OPEN을 유지한다. PR에는 동일한 결과와 이슈의 후속 검증 경로를 연결한다. 검토 기록은 이미 PR에
포함되므로 추가 기록 PR을 만들지 않고 merge SHA와 실제 상태는 GitHub 코멘트에 남긴다.
