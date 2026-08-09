# PR #4153 메인터너 보정 기록

## 목적

이슈 [#4152](https://github.com/edwardkim/rhwp/issues/4152)의 산출물 경로 통일 범위가 OVR 도구와
CONTRIBUTING에만 반영되고 편집-스윕의 Rustdoc·매뉴얼에는 남지 않은 상태를 보완한다.

## commit 구성

| 순서 | SHA | 역할 |
| --- | --- | --- |
| 1 | `06989c41f` | contributor: OVR 기본 산출물과 CONTRIBUTING 안내를 `output/`으로 통일 |
| 2 | `3a7c88b13` | maintainer: 편집-스윕 Rustdoc·매뉴얼의 `out/sweep`을 `output/sweep`으로 통일 |
| 3 | `9edcf0829` | review·오늘할일 기록만 추가 |
| 4 | 이 문서 commit | 최신 devel today 보존·merge simulation 절차를 workflow에 보강 |

contributor 원 commit은 수정하거나 재작성하지 않았다. 보정은 같은 visibility branch
`review/humdrum00001010-20260807`에서 원 head 뒤에 단일 commit으로 추가했다.

## 검증과 push

- 보정 전후 원격 source branch와 PR head가 모두 contributor SHA `06989c41f`임을 확인했다.
- 수정 파일 두 개는 `filter: unspecified`이고 `git lfs status`에 push object가 없었다.
  따라서 `GIT_LFS_SKIP_PUSH=1`을 사용한 dry-run과 실제 push가 성공했다.
- `git diff --check`, targeted Markdown 링크 검사, ignore 경로 검사, `cargo fmt --check`를 통과했다.
- `3a7c88b13`의 GitHub Actions full CI와 CodeQL이 성공했다.
- `9edcf0829`의 review-only fast-pass CI가 성공했다. source에 없는 최신 devel archive 기록을 복사하면
  링크가 깨지는 것을 확인해, source의 기존 항목을 보존하고 변경되지 않은 경계에 현재 PR 기록만 넣은
  merge simulation으로 정합을 확인했다.

## 후속 조건

이번 workflow 보강 commit도 source code를 바꾸지 않는다. push 뒤 최신 PR head가 그 commit과 일치하는지,
review-only fast-pass preflight 및 aggregate가 성공하는지 확인한 뒤 작업지시자 승인에 따라 merge한다.
