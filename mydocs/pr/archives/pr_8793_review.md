# PR #8793 self-review — v0.9.0 최종 배포 기록과 npm 안내

## 최종 판정

**승인 — 최종 배포 기록·가이드의 정합성과 제출 head의 문서 전용 CI를 확인했다.**
검토 head는 `9453bcbf6e6368af622d42b87c7565d5d6e75624`다. 이 검토 기록을 반영한
trailing head의 CI·병합 가능 상태는 병합 직전에 별도로 확인한다.
작업지시자의 “이번 릴리즈 작업을 마무리 합시다” 지시에 따라 병합·동기화·종료 정리를 진행한다.
이 지시는 과거 태그 CI 실패를 성공으로 재분류하거나 추가 재실행하는 예외로 해석하지 않는다.

## 접수 정보와 범위

- [PR #8793](https://github.com/edwardkim/rhwp/pull/8793), 작성자 edwardkim, base devel
  `4aa8f5fc2b550145a724155cdb7cdd0b0a2576f3`.
- collaborator_self_merge + intake_and_review·local_validation·review_only_fast_pass·post_merge 적용.
- 제품·workflow·권한 설정 변경 없이 `mydocs/`의 배포 보고서·최종 JSON 증적·종료 정리 기록과
  npm publisher 가이드를 제출했다. 검토 기록과 현재 조회 증거를 같은 PR에 보충한다.
- 완료된 원 PR의 후속 기록 경로이므로 오늘할일·원 PR 댓글·이슈 close를 반복하지 않는다.
  기여자 33명과 기존 기여 이력은 보존하며 새 기여 명단을 임의로 만들지 않는다.

## 검토와 로컬 검증

1. 14개 패키지의 고유 이름·버전·archive SHA-256·크기를 확인했다. 최초 2개와 나머지 12개의
   source SHA가 각각 정식 태그와 승인된 메타데이터 보정 commit임을 보고서·원장과 대조했다.
   docs.rs 14건, Rust·CLI·WASM 공개 소비와 Docker 32개 명령의 관측 시각·검증 범위를 구분했다.
2. npm provenance의 실제 workflow는 `.github/workflows/release-binary.yml`이다.
   현재 workflow 호출 구조와 [npm 공식 문서](https://docs.npmjs.com/trusted-publishers/#troubleshooting)를
   대조해 자동 배포 caller와 독립 수동 실행, environment·Secret 구분을 검토했다.
3. 배포 성공과 태그 CI 실패를 분리했다. 과거 관측을 새 실행 결과로 바꾸지 않고,
   종료 정리된 `output/`의 경로는 실행 당시 식별 정보라고 명시했다.
4. 변경 Markdown 4개의 링크·공백과 메타데이터가 있는 2개 문서, JSON 파싱·소스 분리·체크섬
   형식 검사를 통과했다. 전체 메타데이터 검사의 기존 5개 문서·20개 누락은 base와 파일 내용이
   같음을 확인했다. 이번 변경에는 포함하지 않았다.
5. 최종 증적의 credential/email 패턴과 민감 필드를 점검했고 로컬 Docker 환경의 ignore를
   확인했다. 폰트·환경·입력·인증 파일을 이 PR에 추가하지 않았다.
6. 조판 원칙·검증 입력 커밋 확인·새 시각 회귀는 **비해당**이다. 실행 코드·문서 입력·출력
   기준을 수정하지 않고 완료된 배포 실행의 기록만 보존하므로 실제 조판·렌더 소비 경로는
   바뀌지 않는다. `local_validation.md` 4.3에 따라 Cargo·Native·WASM을 반복 실행하지 않았다.

## GitHub 검증

- [CI](https://github.com/edwardkim/rhwp/actions/runs/42494246476) success.
  실제 preflight 로그는 `all-review-only-no-code-impact`, `DETECTED_FAST_PASS: true`이며,
  Build & Test는 `CI preflight fast-pass accepted.`로 성공했다. heavy worker는 정상 skipped다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/42494246507),
  [Adapter](https://github.com/edwardkim/rhwp/actions/runs/42494246424),
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/42494246635)는 preflight success와 정상 skip을 확인했다.
- PR head의 `CI Impact Policy` 상태는
  [최종 정책 감사](https://github.com/edwardkim/rhwp/actions/runs/42495757398)를 근거로 success다.
  workflow_run 감사의 실행 source와 상태를 게시한 PR head는 구분했다.
- 조회 시 MERGEABLE/CLEAN이었다. 현재 check 목록·실행 SHA·스토어 조회는
  [종료 검토 증거](../assets/release-0.9.0-execution/closeout-review.json)에 보존했다.

## 릴리즈 종료 범위와 남은 외부 상태

게시·공개 소비 검증·최종 가이드·기록 반영을 이번 작업의 종료 범위로 삼는다.
GitHub Release·Pages·npm·crates.io·VS Code·Open VSX와 Firefox의 검증 결과는
[최종 보고서](../../report/release_0_9_0_execution_report.md)를 따른다.
2026-11-09 조회에서 Firefox API는 0.9.0, Chrome 공개 페이지는 0.8.7이며,
Edge의 공개 버전은 추출할 수 없었다. Chrome·Edge는 제출 완료 후 공개 전환 확인 대상으로 남는다.

태그 CI attempt 2의 Binaryen 다운로드 실패와 Native·Archive B 시간 초과는 그대로 남긴다.
Release Binary attempt 4의 성공이나 이 문서 PR의 성공으로 해당 검사를 통과 처리하지 않는다.
추가 배포·재실행·정책 변경은 이번 마무리에 포함하지 않는다. 병합 뒤에는 devel 동기화와
이 PR 전용 branch·임시 output 정리만 수행하고, 로컬 소비 검증 환경과 다른 작업공간은 보존한다.
