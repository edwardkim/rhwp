---
kind: guide
status: active
canonical: mydocs/manual/README.md
last_verified: 2026-07-17
---

# rhwp 온보딩 가이드

rhwp는 Rust로 HWP/HWPX/HWP3 문서를 공통 IR로 변환하고 네이티브와 브라우저에서 렌더링·편집하는
프로젝트다. 이 문서는 저장소 진입점만 제공하며 세부 명령과 절차를 중복하지 않는다.

## 먼저 읽을 문서

1. 저장소 루트의 [`AGENTS.md`](../../AGENTS.md)
2. [전체 문서 지도](../README.md)
3. [manual 문서 지도](README.md)와 [tech 문서 지도](../tech/README.md)
4. 작업 종류에 맞는 canonical 문서

주요 진입점:

- 개발 환경과 빌드: [개발 환경 가이드](dev_environment_guide.md)
- CLI: [CLI 명령어 매뉴얼](cli_commands.md)
- PR 준비와 검토: [PR 리뷰·통합 워크플로우](pr_review_workflow.md)
- 시각 검증: [시각 검증 문서 지도](verification/README.md)
- 파서 경계: [포맷 파서와 공통 Document IR 경계](../tech/parser_architecture.md)

## 로컬 시작

```bash
git remote add upstream https://github.com/edwardkim/rhwp.git  # fork를 clone한 경우 최초 1회
git fetch upstream
git switch -c <work-branch> upstream/devel
cargo build
cargo test
wasm-pack build --target web --out-dir pkg
```

브라우저 편집기를 실행할 때는 다음을 사용한다.

```bash
cd rhwp-studio
npm ci
npx vite --host 0.0.0.0 --port 7700
```

## 기여 흐름

1. 관련 GitHub issue와 열린 PR을 확인한다.
2. 최신 `upstream/devel`에서 작업 브랜치를 만든다.
3. 저장소 역할과 요청 범위에 맞는 계획·검증 절차를 따른다.
4. focused test와 필요한 시각 검증을 수행한다.
5. 쓰기 가능한 원격의 작업 브랜치에 push하고 `devel` 대상 PR을 만든다.
6. CI와 리뷰가 끝난 뒤 권한과 승인 범위에 맞게 merge·후속 처리를 수행한다.

오늘할일, PR review 문서, maintainer 보정은 기여자·collaborator·maintainer 역할에 따라 다르다. 임의로
일반화하지 않고 [PR 리뷰·통합 워크플로우](pr_review_workflow.md)를 따른다.

## 기본 원칙

- 사용자 또는 다른 도구의 미커밋 변경을 임의로 되돌리지 않는다.
- HWP/HWPX/HWP3 포맷별 해석은 파서 경계에서 끝낸다.
- 구조 회귀 테스트 통과와 한컴 시각 충실도는 같은 판정이 아니다.
- 문서 이동 시 상대 링크와 canonical 관계를 함께 검증한다.
- 토큰, 개인 주소, 재배포할 수 없는 폰트는 커밋하지 않는다.
