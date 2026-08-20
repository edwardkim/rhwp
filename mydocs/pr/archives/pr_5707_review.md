---
kind: pr-review
status: approved
pr: 5707
issue: 5706
---

# PR #5707 검토 기록 - 스킬 라우터와 실렌더 검증

- PR: [#5707](https://github.com/edwardkim/rhwp/pull/5707) `스킬 라우터(요청→intent→capability→skill→그래프)와 rhwp 실렌더 검증`
- 관련 이슈: [#5706](https://github.com/edwardkim/rhwp/issues/5706)
- 작성자: `@kevin9327`, `maintainer_can_modify=true`
- source code candidate: `7044877da5744ce1f0c5759bcc0b482d92d3a2f5`
- 검토 기준: `upstream/devel@1139f28d1` 위 `review/open-prs-20260820`
- 체리픽: `dac58ed13`부터 `40c1c0829`까지 원 PR의 9개 커밋을 순서대로 `-x` 적용했다.
- 메인터너 보정: `9f79ed5a9`가 20개 증적 텍스트의 CRLF·trailing whitespace를 정규화했고, `c264a575d`가 게이트의 stale Cargo binary 자동 발견을 제거했다.
- 라우팅: `collaborator_external_pr` + `intake_and_review` + `local_validation` + `multi_pr_update_branch`

## 검토 범위

- 요청을 intent·capability·skill·execution graph로 해석하는 Python 라우터, skill catalog·probe·동기화 게이트, 그 전용 GitHub Actions workflow를 추가한다.
- 스킬 설명의 `rhwp <command>` 참조는 명시적으로 선택한 후보 바이너리에 대해서만 live-command 계약을 검사한다. 기본 Python-only gate는 우연히 남은 `target/release`나 PATH의 구식 실행 파일에 영향받지 않는다.

## 보정 사유

- 원 PR의 Markdown·CSV 등 20개 텍스트가 CRLF·trailing whitespace를 포함해 `git diff --check`를 실패시켰다. 의미 변경 없이 LF와 줄 끝 공백을 정규화했다.
- 로컬 검토에서 구식 `target/release/rhwp`가 새 `explore`·`armor` 명령을 알지 못해 게이트가 실패했다. 후보 바이너리를 `--rhwp-bin`으로 명시하도록 바꿔 검토 target과 CI 구조 게이트를 분리했다.

## 검증 근거

- 기본 gate를 세 번 반복하고 route, catalog, author, probe, precommit unit test와 catalog sync를 실행해 모두 통과했다.
- `cargo build --release --target-dir target/pr-review --bin rhwp` 뒤 `python3 tools/skill_router/gate_new_skill.py --rhwp-bin target/pr-review/release/rhwp`를 실행했다. 27개 skill을 세 번 scan했고 207개 live command와 189개 참조를 모두 통과했다.
- source head의 Skill router gate, Build & Test, Lint, Native Skia, regular/slow shard, CodeQL, Proptest, adapter inter-diff가 성공했다.

## 결론

**승인 (메인터너 보정 포함).** 라우터·catalog·probe 계약은 반복 실행에서 안정적이며, live-command 검사는 검토 후보 바이너리를 명시해 실제 명령 집합과 대조했다. #5706은 통합 후보 PR의 CI가 성공한 뒤 수용 결과와 함께 닫는다.
