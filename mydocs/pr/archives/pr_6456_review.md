---
kind: pr-review
status: pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6456
issue: null
author: jangster77
---

# PR #6456 review - HWPX 양식 사전 차단과 재시작 대응 안내

## 라우팅과 metadata

- PR: [#6456](https://github.com/edwardkim/rhwp/pull/6456), base: `devel`.
- base route: `collaborator_self_merge.md`; modifiers: `intake_and_review.md`,
  `local_validation.md`, `review_only_fast_pass.md`.
- 작성자·self-review: `jangster77`; collaborator 본인 PR이므로 reviewer request는 등록하지 않았다.
- 기준: `upstream/devel@a71ebf2141ae3bd1ca3b5d3ea438bbe3454edaf9`.
- code candidate: `bd3241519a594778743137c1f93a39fe18c4bd96`.
- 작성 시점 참고값: Open non-draft, `MERGEABLE`, 1 file, `+34/-0`.
  `mergeStateStatus=BLOCKED`는 required CI 대기 상태이며, CI 결과와 mergeability는 merge 직전에
  다시 확인한다.

## 변경 범위와 판단

`mydocs/manual/mcp_hwp2024Convert_usage.md`만 수정한 review-only PR이다.

- HWPX의 `checkBtn`, `radioBtn`, `comboBox`, `edit` 양식 컨트롤은 현재 무인 HOffice120/HOffice130
  runtime에서 access violation을 재현하지만, 손상 문서가 아니라는 점을 분리했다.
- 해당 입력은 한컴 worker 전에 `failed`로 끝나며 결과 blob·local output이 없고 download를 호출하지
  않아야 한다는 계약을 추가했다. 암호 HWPX의 password preparation 후 재검사도 안내한다.
- `service_restarted`는 같은 job의 반복 polling·download가 아니라 원본 local input으로 새 `start`를
  제출해야 하는 실패임을 명시했다. 여러 endpoint를 병렬 사용하는 wrapper의 start/status/download
  endpoint 고정 규칙도 포함했다.
- server/client source, service 설정, 인증 정보, fixture, PDF 산출물은 변경하지 않는다.

## 로컬 검증

- `git diff --check`: 통과.
- `python scripts/check_markdown_links.py mydocs/manual/mcp_hwp2024Convert_usage.md`: 통과.
- `python scripts/check_document_metadata.py`: 통과.
- `cargo fmt --all`, `cargo fmt --all -- --check`: 통과.
- 문서 전용 변경이므로 Cargo test, MCP 변환 smoke, visual sweep은 실행하지 않았다. 이번 PR은 runtime
  동작을 변경하지 않으며, 배포된 service의 실제 비동기 smoke는 별도 `hwp-convert-2024` 작업에서 확인했다.

## 최종 권고와 후속 조건

**조건부 수용.** HWPX 양식 입력을 손상·timeout·일반 재시도와 혼동하지 않도록 하고, service 재시작 뒤
job 회수 불가 시의 client 동작을 실제 계약에 맞게 설명한다.

- trailing review 기록을 포함한 최신 head의 review-only fast-pass CI와 required aggregate가 성공해야 한다.
- merge 직전에 최신 head의 `MERGEABLE/CLEAN` 상태를 다시 확인한다.
- 작업지시자 승인에 따라 squash merge한 뒤 `upstream/devel` 동기화와 브랜치 정리를 수행한다.
