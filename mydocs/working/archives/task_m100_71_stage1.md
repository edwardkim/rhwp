# Task #71 단계1 완료 보고서: CI/CD 워크플로우 최소 권한 설정

## 수행 내용

CodeQL 경고 6,7,8 (Workflow does not contain permissions) 해소.

## 변경 파일

| 파일 | 변경 내용 |
|------|----------|
| `.github/workflows/ci.yml` | build-and-test, wasm-build job에 `permissions: { contents: read }` 추가 |
| `.github/workflows/npm-publish.yml` | 단일 job → 4개 job 분리 + 각 job에 최소 permissions 명시 |

## npm-publish.yml 구조 변경

| job | permissions | 역할 |
|-----|-----------|------|
| build-wasm | contents: read | WASM 빌드 + 아티팩트 업로드 |
| publish-npm-core | contents: read, id-token: write | @rhwp/core npm 배포 |
| publish-npm-editor | contents: read, id-token: write | @rhwp/editor npm 배포 |
| publish-vscode | contents: read | VSCode Marketplace + Open VSX 배포 |

## 해소된 경고

- 경고 6: ci.yml:16 (build-and-test)
- 경고 7: ci.yml:48 (wasm-build)
- 경고 8: npm-publish.yml:10 (최상위 → job 레벨)

## 커밋

`31d4b92` Task #71 단계1: CI/CD 워크플로우 최소 권한 설정
