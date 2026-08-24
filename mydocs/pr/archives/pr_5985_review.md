---
kind: pr-review
status: trailing-docs-pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-24
---

# PR #5985 검토 - HWP MCP 엔진 검증 기준 보정

## 접수 메타데이터

| 항목 | 작성 시점 확인값 |
| --- | --- |
| PR / 작성자 | [#5985](https://github.com/edwardkim/rhwp/pull/5985) / [@jangster77](https://github.com/jangster77) |
| 관련 issue | 없음 |
| base / code candidate | `devel` / `d7a79d400a527ccfed8380c73a33462aacfaa28c` |
| code candidate 변경 규모 | 2 files, +30 / -13 |
| 작성 시점 상태 | non-draft; `MERGEABLE`, GitHub Actions는 trailing 문서 push 뒤 최신 head에서 재확인 필요 |
| reviewer | 작성자 본인 self-review, 별도 reviewer 미지정 |

GitHub mergeability와 CI 상태는 작성 시점 참고값이다. 이 trailing 기록 commit의 최신 head가
required check를 통과하고 작업지시자가 병합을 승인한 뒤에만 merge한다.

## 변경과 판단

- 최신 HWP 2024 client의 `--engine 2020|2024` 요청값과 비동기 `start`·`status` 응답의
  `engine` 일치를 논리 엔진 선택 증적으로 정한다.
- `server.engine`은 concrete backend 식별자일 수 있으므로 논리 엔진 선택의 판정 기준에서 분리한다.
- Hancom Office 2020 저장본 `samples/kps-ai.hwp`의 실제 비동기 변환 결과를 HWP MCP 사용법과
  시각 fixture 증적 절차에 반영한다.

renderer, layout, fixture 출력, 기준 PDF는 변경하지 않았다. `visual_fixture_evidence.md`는 검토 절차를
보정한 문서일 뿐 새 시각 결과를 주장하지 않으므로 visual sweep은 요구하지 않는다.

## 로컬 검증

- `cargo fmt --all`과 `cargo fmt --all -- --check`를 exit code 0으로 통과했다.
- `git diff --check`를 통과했다.
- 최신 HWP 2024 client로 `samples/kps-ai.hwp`를 `--engine 2020`으로 비동기 변환했다.
  - `rhwp info --json`은 `lastSavedWith.product: hancom-office-2020`, 버전 `11.0.0.8808`을 반환했다.
  - `start`는 `queued`, `status`는 `succeeded`·`completed` 및 `engine: 2020`을 반환했다.
  - `download`한 PDF는 634,466 bytes였고 `%PDF-` 서명 및 client/server SHA-256 일치를 확인했다.

인증 token, server URL, 환경 파일 내용, 비동기 job ID는 기록하지 않았다.

## 최종 판정

**수용 권고, trailing CI 대기.** 문서가 최신 client의 명시적 논리 엔진과 concrete backend 식별자를
혼동하지 않도록 보정했다. 최신 trailing head의 GitHub Actions가 성공하고 작업지시자가 병합을 승인한 뒤
merge한다.
