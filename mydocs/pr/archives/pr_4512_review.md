---
kind: report
status: active
last_verified: 2026-08-11
---

# PR #4512 검토 — 프로젝트 로드맵과 업스트림 경계 정립

## 라우팅

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md, review_only_fast_pass.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
  pr_review/collaborator_self_merge.md, pr_review/intake_and_review.md,
  pr_review/local_validation.md, pr_review/review_only_fast_pass.md
current head: 190d956332036c5996faf70cd704b2b32774b13f (접수 시점 참고)
```

## Metadata

| 항목 | 접수 시점 참고값 |
| --- | --- |
| PR | [#4512](https://github.com/edwardkim/rhwp/pull/4512) |
| 작성자·self-review 담당 | `edwardkim` |
| base / head | `devel` / `task/4467-project-roadmap` |
| 관련 이슈 | `Closes #4467` |
| 규모 | 18개 파일, +932/-33, 5 commits |
| 상태 | Open, MERGEABLE/CLEAN |
| 1차 트리야지 | assignee `edwardkim`, milestone `v1.0.0`, label `documentation` |

작성자 본인은 GitHub에서 자기 PR의 requested reviewer나 `APPROVE` 대상이 될 수 없다. 작업지시자의
maintainer self-review 승인에 따라 별도 reviewer request 없이 최신 head에 `COMMENTED` review로 검토
결과를 게시한다.

## 변경 범위와 판단

README의 초기 제품 로드맵을 루트 `ROADMAP.md`로 분리하고, 한국어 독자가 현재 위치와 완료 기준을
읽을 수 있도록 다시 구성했다. v1.0 조판과 v2.0 협업의 병행 진행, 40명이 넘는 외부 기여자와 두 명의
collaborator, 업스트림과 다운스트림의 책임 경계를 README·CONTRIBUTING·문서 지도에 연결한다.

Rust·frontend·WASM·renderer·fixture·baseline 변경은 없다. 시각 출력 계약도 바뀌지 않으므로 Cargo,
WASM과 시각 검증은 적용하지 않는다.

## Self-review finding과 보정

접수 head 작성 뒤 최신 `devel`에 CLI 릴리스 아카이브와 deb/rpm/MSI 설치 패키지, 설치 채널 매니페스트,
GHCR CLI 이미지와 GitHub Action이 공식 업스트림 산출물로 추가됐다. 접수 head의 로드맵은 공식 배포
대상을 브라우저·VS Code 확장과 npm만으로 적고 “설치 프로그램”을 넓게 다운스트림으로 분류해 최신
업스트림 계약을 온전히 설명하지 못했다.

보정 head에서는 공용 CLI 배포·자동화 산출물을 업스트림에 추가하고, 다운스트림 범위를 특정 운영체제용
완제품의 앱 셸·제품 설치 프로그램·자동 업데이트·파일 연결로 좁힌다. 이는 프로젝트 앱과 공용 CLI
배포 계약을 구분하며, 새 플랫폼 완제품의 공식 편입에 별도 유지 책임 합의가 필요하다는 원칙은 유지한다.

## 검증

- 접수 candidate `190d956332036c5996faf70cd704b2b32774b13f`의 GitHub CI·CodeQL·Native Skia·
  네 Rust test shard·Build & Test가 모두 성공했다.
- 최신 `upstream/devel` `8dbe982e89e780fe0612a1bc66aa417bbd6356b2`와 접수 head의 merge tree
  `fc2dc6c2e8b672cc0bbabe4465028c04d31d7336`가 충돌 없이 생성됐다.
- 공개 GitHub 집계를 다시 확인해 owner·bot을 제외한 이름 있는 외부 contributor 41명과 write 권한
  collaborator `postmelee`, `jangster77` 두 명이 로드맵 표현과 일치했다.
- 접수 head 기준 Markdown 링크 검사(556개 문서, 변경 파일 18개), 문서 메타데이터 검사(540개 문서),
  `git diff --check`가 통과했다.
- 보정·review 기록 후보에서 Markdown 링크 검사(557개 문서, 변경 파일 19개), 문서 메타데이터
  검사(540개 문서), 배포 경계 표현 검색과 `git diff --check`가 통과했다.
- push 뒤 최신 head의 merge tree와 GitHub required checks를 새로 확인한다.

## 최종 권고

현재 self-review finding은 보정에 포함했다. 최신 head의 문서 검사, mergeability와 GitHub required
checks가 성공하면 blocking finding 없이 merge를 권고한다. self-review 결과는 `COMMENTED` review로
게시하며 실제 merge는 작업지시자의 별도 승인을 조건으로 한다.
