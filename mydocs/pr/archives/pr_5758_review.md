---
kind: pr-review
status: active
canonical: mydocs/manual/pr_review_workflow.md
---

# PR #5758 - 점유된 Docker GID에서도 builder 소유권을 설정한다

## 라우팅과 메타데이터

```text
base route: collaborator external PR
modifiers: intake_and_review.md, local_validation.md,
  collaborator_external_pr.md, post_merge.md
code candidate: 00e95071e2e5557baeddd53eec0a8d10482f27c5
merge commit: cd761884cc580fa721961105043fa0b5b1a9c6b1
```

| 항목 | 결과 |
| --- | --- |
| PR | [#5758](https://github.com/edwardkim/rhwp/pull/5758) |
| Issue | 연결된 이슈 없음 |
| 작성자 | `kjh0523` (rhwp 첫 GitHub 기여) |
| base / head | `devel` / `fix/docker-build-on-macos` |
| 범위 | `Dockerfile` 1개, +6 / -2 |
| 병합 | 2026-08-20, administrator merge, `cd761884c` |

## 변경 범위와 판정

- `groupadd -g ${GID} builder || true`는 지정한 숫자 GID가 이미지 안에서 이미 쓰이면
  `builder` 그룹을 만들지 못한다. 기존 `chown builder:builder`는 이 경우 존재하지 않는
  그룹 이름을 다시 참조해 실패했다.
- 두 `chown`을 `builder:${GID}`로 바꿔, 새 그룹 생성 여부와 관계없이 숫자 GID에
  소유권을 설정한다. `/home/builder`와 `.cargo`를 함께 보정해 후속 단계의 동일한 실패도
  방지한다.
- 기본 `1000:1000` 경로를 망가뜨리지 않고, macOS 기본 GID 20처럼 Debian 계열에서 이미
  점유된 GID를 명시한 Docker 사용자 경로만 고친다.

## 검증과 한계

- Ubuntu 컨테이너에서 `UID=501, GID=20`을 재현했다. 기존 그룹 이름 방식은
  `chown: invalid group: 'builder:builder'`로 실패했고, 숫자 GID 방식은
  `uid=501(builder) gid=20(dialout)` 및 두 대상 경로의 `501:20` 소유권으로 성공했다.
- 호환 경로는 `UID=1001, GID=1000`으로 확인했다. 두 대상 경로가 `1001:1000`이 되어
  기본적인 비점유 GID 동작도 유지했다.
- 전체 `rust:latest` Dockerfile build는 registry pull이 진행되지 않는 로컬 Docker 환경 문제로
  완료하지 못했다. 이 기록은 위 Dockerfile 소유권 명령의 Linux 컨테이너 검증만 근거로 하며,
  전체 이미지 빌드 성공을 주장하지 않는다.
- [CI run 32350721366](https://github.com/edwardkim/rhwp/actions/runs/32350721366)는
  preflight, Lint, Native Skia, frontend package, archive builder, slow/regular test worker와
  `Build & Test` aggregate가 성공했다.
- Rust CodeQL은 administrator merge 시점에도 진행 중이었다. 작업지시자의 명시적
  `admin merge` 지시로 해당 대기를 예외 처리했으며, 성공으로 간주하지 않는다.

## 최종 판정

**병합 완료.** 실제 점유 GID 재현에서 실패 원인과 두 소유권 경로의 보정이 확인됐고, code CI의
핵심 Rust aggregate도 성공했다. 이 검토 기록과 오늘할일은 구현 PR과 분리한 docs-only PR로
보존한다.

