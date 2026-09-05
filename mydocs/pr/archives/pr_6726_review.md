---
kind: pr_review
status: approved
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-04
pr: 6726
issue: 6711
author: edwardkim
---

# PR #6726 self-review — working 월별 archive 1차 배치

## 결론

**승인.** PR #6726은 #6711 Stage 3-A로 `mydocs/working`의 cutoff 이전 문서 1,119건을
월별 archive 거버넌스에 따라 처리한다. 최종 code candidate
`e3b0b8833f5a04cbe66b0542048309b715752813`을 재검토한 결과 1,111건은 rename이고 8건은
byte-identical archive가 이미 있는 root 중복 제거다. 예상하지 않은 삭제·경로 범위 누출·신규
Markdown 링크 또는 metadata 오류는 발견하지 않았다.

첫 head `f73a68590d`와 첫 정정 head `03417efe2d`는 각각 Gym 경로 계약과 분할 조립된 Rust 경로
계약을 누락해 CI가 실패했으므로 승인 대상에서 제외했다. 두 실패 원인을 root redirect로 우회하지
않고 실제 test·tool·generator·fixture 소비자를 archive 경로로 정정했으며, 최종 candidate의 Full
CI에서 Gym과 Archive A–D 실제 테스트가 모두 성공했다.

이 문서의 `승인`은 작성자 self-review 판정이다. 자기 PR이므로 reviewer 지정이나 GitHub approve
event를 만들지 않는다. 이 review와 오늘할일만 추가한 trailing head의 GitHub Actions,
`MERGEABLE`·`CLEAN`, 최신 `devel` 정합을 다시 확인하고 메인테이너의 별도 merge 승인을 받아야 한다.

## 라우팅과 메타데이터

- 기본 경로: `collaborator_self_merge.md`
- 보조 경로: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`,
  `rework_and_exceptions.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`와 위 자식 문서
- `review_impl`은 추가하지 않는다. 승인된 [수행계획](../../plans/task_m100_6711.md)과
  [Stage 3 보고서](../../working/task_m100_6711_stage3.md)가 실행·재작업·검증 계보를 고정한다.

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#6726](https://github.com/edwardkim/rhwp/pull/6726) / @edwardkim |
| 관련 이슈 | [#6711](https://github.com/edwardkim/rhwp/issues/6711) (`Refs #6711`) |
| base | `devel@f93a4dfab93353dee54d25795be9c186b61a073b` |
| code candidate | `e3b0b8833f5a04cbe66b0542048309b715752813` |
| 규모 | 1,258 files, `+395/-767`, 4 commits |
| 작성 시점 GitHub 상태 | Open, 비 Draft, `MERGEABLE`·`CLEAN`, candidate checks 완료 |
| reviewer | self PR이므로 지정하지 않음 |

변경 파일 수와 줄 합계가 1,000을 넘으므로 대형 PR 경로를 적용했다. GitHub가 rename을 전혀
인식하지 않는 보수적 경로 수는 2,369개로 PR files API 상한 3,000개보다 작다. GitHub REST를
100개씩 전수 조회한 1,258개 파일이 로컬 rename-aware diff와 일치했다.

## 이동·손실 방지 재검토

| 상태 | 수 | 판정 |
| --- | ---: | --- |
| rename | 1,111 | `working`에서 같은 역할의 `archives/`로 이동 |
| 수정 | 138 | incoming 링크, 실행 경로 소비자, 거버넌스·계획·단계 증적 갱신 |
| 삭제 | 8 | 기존 archive와 SHA-256이 같은 root 중복만 제거 |
| 추가 | 1 | Stage 3 결과 보고서 |
| 예상 밖 최상위 범위 | 0 | `mydocs`, `.agents`, `.agents`, `gym`, `scripts`, `tests`, `tools`, `rhwp-studio`만 변경 |

동명 archive가 있던 root 문서 8건은 Stage 3 보고서에 기록한 SHA-256이 각각 기존 archive와
일치한다. 내용이 다른 동명 충돌은 0건이다. 이동 직후 root에는 Stage 3-B 후보 1,120건과 9월
생성 문서가 남았으며, Stage 3-A는 이를 임의로 완료나 폐기로 바꾸지 않는다.

## 경로 계약 회귀와 수정 판정

### 첫 head

`f73a68590d`의
[Gym run 33865831824](https://github.com/edwardkim/rhwp/actions/runs/33865831824)는 2,125건 중
failures 5건, errors 16건으로 실패했다. Python 계약이 이동한 `gym_*.md`를 옛 root에서 열던 것이
원인이었다. Python·Rust 계약 테스트와 생성기·fixture의 실제 소비 경로를 archive로 고치고,
historical `canonical`·과거 증적 문자열은 보존했다.

### 첫 정정 head

`03417efe2d`의
[Archive C job](https://github.com/edwardkim/rhwp/actions/runs/33867686045/job/101007764193)은
`agent_surface_skill_contract::working_doc_closes_issue_5326`에서 실패했다. 이 계약은
`.join("mydocs").join("working")`처럼 경로를 조각으로 조립해 완성 문자열 중심 감사에서 빠졌다.
`archives` 조각을 추가하고 완성 경로와 분할 조립 경로를 별도 검색하도록 월별 archive 정본도
보완했다.

두 실패는 재실행으로 덮어쓰지 않고 원인 계보로 남겼다. 최종 candidate에서는 같은 Archive C
샤드가 4분 8초에 성공했다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| `git diff --check` | 통과 |
| `python3 scripts/check_markdown_links.py` | 609개 canonical 문서, 오류 0 |
| 이동 전후 전체 링크 그래프 | 전체 9,187건·유효 8,620건 보존, 신규·소실 broken link 0 |
| `python3 scripts/check_document_metadata.py` | 기존 4개 문서의 16건만 재현, 신규 0 |
| CI와 같은 Gym 계약 명령 | 2,125 passed, 1 skipped |
| 관련 Python·generator 모듈 | 40개 모듈 통과 |
| `tools.agent_onboarding.test_rhwp_doctor` | 136 passed |
| 변경된 Rust `working` 계약 | 14개 generated suite, 16 passed |
| 누락 계약 focused nextest | 1 passed |
| native Clippy `-D warnings` | 통과 |
| WASM library Clippy `-D warnings` | 통과 |
| workspace build | 통과 |
| workspace all-targets Clippy `-D warnings` | 통과 |
| integration suite manifest | 1,152 sources, 4,879 static attrs, 48/48 targets 정합 |

`test_agent_knowledge_map`, `test_agent_mcp_session`, `test_agent_surface`의 기존 실패는 분리한
`devel@2394c9044c` worktree에서도 동일하게 재현되어 이 PR의 신규 회귀가 아니다. 렌더링·레이아웃,
sample과 PDF bytes는 바꾸지 않아 시각 검증은 비대상이다.

## GitHub Actions와 최신 base

candidate SHA에 대해 다음 workflow가 성공했다.

- CI [run 33868857965](https://github.com/edwardkim/rhwp/actions/runs/33868857965)
- CodeQL [run 33868858034](https://github.com/edwardkim/rhwp/actions/runs/33868858034)
- Proptest [run 33868858029](https://github.com/edwardkim/rhwp/actions/runs/33868858029)
- Adapter inter-diff [run 33868858010](https://github.com/edwardkim/rhwp/actions/runs/33868858010)
- Gym benchmark [run 33868857672](https://github.com/edwardkim/rhwp/actions/runs/33868857672)
- Skill router [run 33868857664](https://github.com/edwardkim/rhwp/actions/runs/33868857664)
- trusted Impact Policy 최종 집계
  [run 33869957354](https://github.com/edwardkim/rhwp/actions/runs/33869957354)

최종 check 집계는 success 32, skipped 5, failure·pending 0이다. `upstream/devel`은 candidate에
병합한 기준선 `f93a4dfab93353dee54d25795be9c186b61a073b`에서 전진하지 않았다.
`git merge-tree --write-tree upstream/devel HEAD`의 결과
`1cbffd86e315bdb0151fe75be9e388c17ddf2439`는 candidate tree와 같다.

## 잔여 위험과 후속 경계

- 저장소 밖 소비자가 옛 root 경로를 사용하면 GitHub history 외부 링크는 깨질 수 있다. 대량
  redirect stub은 만들지 않으며 실제 중요 소비자가 확인될 때 해당 링크나 canonical index를 고친다.
- historical Markdown·metadata 오류는 이번 이동에서 새로 만든 결함이 아니다. 범위를 섞어 일괄
  정정하지 않았다.
- 이 PR은 `working` 1차 배치만 처리한다. #6711은 Stage 3-B와 Stage 4 전수 감사·최종 보고가
  끝날 때까지 close하지 않는다.

## Merge 후 comment 계획

정상 merge commit이 `devel`에 반영되고 merge SHA의 필수 Actions가 성공한 뒤 PR #6726과 이슈
#6711에 다음 사실을 남긴다.

- 정상 merge commit SHA와 검증한 최종 PR head SHA
- 최종 candidate의 Full CI·CodeQL·Proptest·Adapter·Gym·Skill router run 링크
- 첫 head의 Gym 실패와 첫 정정 head의 Archive C 실패, 실제 소비자 경로 정정 결과
- 1,111 rename·동일본 root 중복 8건 제거·1,258개 API 파일·신규 링크/metadata 오류 0건
- 코드 렌더링·sample·PDF bytes 변화가 없어 시각 검증은 비대상이라는 판정
- #6711은 OPEN으로 유지하고 최신 `devel`에서 Stage 3-B를 시작한다는 후속 경계

게시 뒤 API로 한글·선두 BOM·`??` 치환과 merge SHA·run URL을 검증한다. 같은 사실을 이미 담은
maintainer comment가 있으면 중복 게시하지 않는다.

## 최종 판정과 다음 조건

- 판정: **승인**
- 판정 대상: code candidate `e3b0b8833f5a04cbe66b0542048309b715752813`
- trailing 조건: 이 review와 오늘할일만 추가한 최신 head의 GitHub Actions 성공,
  `MERGEABLE`·`CLEAN`, 최신 `upstream/devel` 정합 재확인
- merge 조건: 최신 head SHA 고정과 메인테이너의 별도 merge 승인
- GitHub review: self PR이므로 approve event와 reviewer 지정 없음
- merge 방식: branch protection을 우회하지 않는 정상 merge commit
- merge 뒤: 최신 `devel` 동기화 후 Stage 3-B를 시작하며 #6711은 OPEN으로 유지
