# 배포 가이드

rhwp 프로젝트의 배포 대상과 절차를 정리한다.

---

## 배포 대상

| 대상 | 패키지명 | 배포 방식 | 트리거 |
|------|---------|----------|--------|
| GitHub Pages (데모) | — | CI/CD 자동 | main push 또는 태그 |
| npm WASM 코어 | @rhwp/core | CI/CD 자동 | GitHub Release 생성 |
| npm 에디터 | @rhwp/editor | CI/CD 자동 | GitHub Release 생성 |
| VSCode Marketplace | rhwp-vscode | CI/CD 자동 | GitHub Release 생성 |
| Open VSX | rhwp-vscode | CI/CD 자동 | GitHub Release 생성 |
| Chrome Web Store | rhwp-chrome | 수동 업로드 | 확장 릴리즈 |
| Microsoft Edge Add-ons | rhwp-chrome | 수동 업로드 | 확장 릴리즈 |
| Firefox AMO | rhwp-firefox | 수동 업로드 | 확장 릴리즈 |

---

## CI/CD 워크플로우 (GitHub Actions)

### 자동 실행되는 워크플로우

| 파일 | 트리거 | 역할 |
|------|--------|------|
| `.github/workflows/ci.yml` | push/PR (main, devel) | cargo build + test + clippy 검증 |
| `.github/workflows/deploy-pages.yml` | main push, 태그 | WASM 빌드 → rhwp-studio 빌드 → GitHub Pages 배포 |
| `.github/workflows/npm-publish.yml` | **GitHub Release 생성** 또는 수동 실행 | WASM 빌드 → @rhwp/core + @rhwp/editor + VSCode/Open VSX 익스텐션 배포 |

### CI/CD 자동 배포 흐름

```
코드 작업 완료
  ↓
devel push → CI 자동 실행 (build + test + clippy)
  ↓
main merge + push → GitHub Pages 자동 배포
  ↓
GitHub Release 생성 (태그)
  ↓ npm-publish.yml 자동 실행
  ├─ WASM 빌드
  ├─ npm @rhwp/core 배포
  ├─ npm @rhwp/editor 배포
  ├─ VS Code Marketplace 배포
  └─ Open VSX 배포
  ↓
브라우저 확장 zip 별도 생성
  ├─ Chrome Web Store 수동 업로드
  ├─ Microsoft Edge Add-ons 수동 업로드
  └─ Firefox AMO 확장 zip + source zip 수동 업로드
```

> **중요**: GitHub Release를 생성하면 npm 2종과 VS Code/Open VSX 배포가 자동 실행된다. 수동 `npm publish`나 `publish.sh`를 실행하지 않는다.
> 단, release workflow를 재실행하면서 이미 VS Code/Open VSX 배포가 끝난 경우에는
> `workflow_dispatch`의 `publish_extensions=false` 입력으로 npm publish만 다시 시도한다.
> Chrome/Edge/Firefox 브라우저 확장은 스토어 심사 흐름이 달라 현재 수동 업로드한다.

### GitHub Secrets 설정

GitHub Actions에서 사용하는 시크릿 (Settings → Secrets and variables → Actions):

| Secret 이름 | 용도 |
|------------|------|
| `VSCE_PAT` | VS Code Marketplace 배포 인증 |
| `OVSX_PAT` | Open VSX 배포 인증 |

> npm 배포는 GitHub Actions OIDC 기반 Trusted Publishing을 사용한다.
> `@rhwp/core`, `@rhwp/editor` 각각의 npm package settings에서
> trusted publisher를 `edwardkim/rhwp`, workflow `npm-publish.yml`, allowed action `npm publish`로 등록해야 한다.
> npm package settings의 Environment name을 지정했다면 GitHub Actions publish job의 `environment:`와
> 정확히 같아야 한다. 현재 배포 환경명은 `NPM_TOKEN`이다.
> 여기서 `NPM_TOKEN`은 GitHub Actions environment 이름이며, npm token secret 이름이 아니다.
> npm publish job에는 장기 토큰(`NPM_TOKEN`, `NODE_AUTH_TOKEN`)을 주입하지 않는다.
> `npm publish`는 GitHub Actions OIDC로 인증하며 provenance는 npm이 자동 생성한다.

### 보안 원칙

- 토큰, 2FA 코드, recovery code, store reviewer private note는 커밋하지 않는다.
- GitHub Secrets/Variables 값은 문서나 로그에 복사하지 않는다.
- npm Trusted Publisher를 사용할 때는 `NODE_AUTH_TOKEN`/`NPM_TOKEN` 환경변수 주입을 피한다.
- GitHub Actions environment 이름과 secret 이름을 혼동하지 않는다. 현재 npm environment 이름은 `NPM_TOKEN`이다.
- 브라우저 확장 reviewer note에는 권한 사용 목적만 설명하고 민감정보를 적지 않는다.
- AMO source zip에는 `node_modules/`, `target/`, `dist/`, `output/`, `samples/`, `pdf-large/`를 포함하지 않는다.
- Firefox AMO source upload 제한은 200 MB이므로 전체 Git tree archive를 업로드하지 않는다.
- Chrome/Edge host permission 설명은 manifest의 `permissions`와 `content_scripts.matches` 기준으로 작성한다.
- 배포 산출물 zip에는 개발용 `.env`, 로컬 인증 파일, 개인 폰트, 임시 저장 파일이 포함되지 않았는지 확인한다.

---

## 버전 관리

### 버전 번호 규칙 (Semantic Versioning)

```
v{MAJOR}.{MINOR}.{PATCH}
  │       │       └─ 버그 수정, README 보강, 문서 업데이트
  │       └───────── 기능 추가, 조판 개선, API 추가
  └─────────────────  호환성이 깨지는 변경 (v1.0.0 = 편집 엔진 정합성 확립)
```

### 버전 번호가 관리되는 파일

| 파일 | 패키지 | 예시 |
|------|--------|------|
| `Cargo.toml` | rhwp (Rust) + @rhwp/core 원본 | `version = "0.7.3"` |
| `rhwp-vscode/package.json` | VSCode 익스텐션 | `"version": "0.7.3"` |
| `npm/editor/package.json` | @rhwp/editor | `"version": "0.7.3"` |
| `rhwp-studio/package.json` | rhwp-studio (GitHub Pages 데모) | `"version": "0.7.3"` |

> `pkg/package.json`은 직접 편집하지 않는다. `scripts/prepare-npm.sh`가 `Cargo.toml`에서 버전을 읽어 자동 생성한다.
> `rhwp-studio/package.json` 버전은 빌드 시 `__APP_VERSION__`으로 주입되어 제품정보 대화창에 표시된다.

### 버전 동기화 원칙

- **Cargo.toml이 기준**이다. MINOR 버전은 모든 패키지가 동일하게 맞춘다.
- @rhwp/core 는 Cargo.toml 버전을 그대로 따른다.
- VSCode 익스텐션은 Cargo.toml과 MINOR까지 동일하게 유지한다.
- @rhwp/editor 는 독자적으로 PATCH를 올릴 수 있다 (README 보강 등).
- npm은 한 번 배포한 버전을 덮어쓸 수 없으므로, README만 수정해도 PATCH를 올려야 한다.

### 브라우저 확장 버전 정책 (라이브러리와 이원화)

**rhwp-chrome / rhwp-edge / rhwp-firefox / rhwp-safari** 의 버전은 라이브러리(Cargo.toml) 와 **독립적으로 관리**한다.

| 영역 | 2026-05-26 현재 |
|------|----------------|
| 라이브러리 (Cargo.toml) | `0.7.13` |
| rhwp-chrome / Edge | `0.2.3` |
| rhwp-safari | `0.2.1` |
| rhwp-firefox | `0.2.3` |

#### 이원화 이유

- **배포 주기 독립**: 라이브러리는 기능 추가·버그픽스 주기로, 확장은 스토어 심사 주기(Chrome/Edge/AMO) 로 별도 움직임
- **스토어 요구사항**: 각 스토어가 manifest 의 `version` 을 자체 규칙으로 관리 요구 (예: 4자리, 재사용 불가)
- **사용자 인지 버전**: 확장 사용자에게 보이는 버전은 "확장 버전"이고, 라이브러리 버전은 기술 내부 번호

#### 확장 버전 동기화 파일

**rhwp-chrome/rhwp-edge** (한 코드베이스, 동일 버전):
- `rhwp-chrome/manifest.json` — 스토어 심사 기준
- `rhwp-chrome/package.json`
- `rhwp-chrome/dev-tools-inject.js` 상수
- `rhwp-chrome/content-script.js` 상수

> manifest 하나만 바꾸고 다른 세 곳이 누락되면 UI 일관성 깨짐. v0.2.0 사이클에서 같은 실수가 발생해 hotfix v0.2.1 을 낸 이력 있음.

**rhwp-firefox**:
- `rhwp-firefox/manifest.json`
- `rhwp-firefox/package.json`

**rhwp-safari**:
- `rhwp-safari/src/manifest.json`

#### 확장 버전 올리기 기준

- 스토어 심사 필요한 변경 → PATCH 이상
- UI/동작 변경 없음 (dist 만 재빌드) → 버전 그대로 유지

> 라이브러리 MINOR 업이 확장 버전 업을 강제하지는 않는다. 확장은 WASM을 새로 번들링해도 스토어 메타데이터 변경 필요 시에만 버전 업.

#### 확장 배포 빌드

Chrome Web Store와 Microsoft Edge Add-ons는 `rhwp-chrome` 빌드 산출물을 공유한다.
Firefox AMO는 `rhwp-firefox` 빌드 산출물을 사용한다.

```bash
cd rhwp-chrome
npm run build
cd dist
zip -r ../rhwp-chrome-{version}.zip .
cp ../rhwp-chrome-{version}.zip ../rhwp-edge-{version}.zip

cd ../rhwp-firefox
npm run build
cd dist
zip -r ../rhwp-firefox-{version}.zip .

cd ../..
git archive --format=zip --prefix=rhwp-source/ --output=rhwp-firefox/rhwp-source-{version}-amo.zip HEAD Cargo.toml rust-toolchain.toml rustfmt.toml Dockerfile docker-compose.yml .env.docker.example LICENSE README.md README_EN.md CHANGELOG.md CHANGELOG_EN.md THIRD_PARTY_LICENSES.md assets/fonts src rhwp-studio rhwp-firefox rhwp-shared scripts npm/README.md npm/editor
```

Firefox AMO 제출 시에는 확장 패키지와 함께 검토용 source zip을 업로드한다.
AMO source 업로드 제한은 200 MB 이므로 전체 Git tree를 압축하지 않는다. 전체 archive는
`samples/`, `pdf-large/` 같은 대형 fixture를 포함해 제한을 초과할 수 있다.

source zip은 확장 재빌드에 필요한 경로만 포함한다.

- 포함: `src/`, `rhwp-studio/`, `rhwp-firefox/`, `rhwp-shared/`, `assets/fonts/`, build scripts, manifest/package files
- 제외: top-level `samples/`, `pdf-large/`, `output/`, `target/`, `node_modules/`, extension `dist/`

#### 확장 스토어 제출 문서

스토어 제출 문서는 `mydocs/feedback/`에 버전별로 보관한다.

| 문서 | 용도 |
|------|------|
| `chrome-{version}_kor.md` | Chrome Web Store 한국어 설명/변경사항 |
| `chrome-{version}_eng.md` | Chrome Web Store 영어 설명/변경사항 |
| `edge-{version}_reviewer_notes.md` | Microsoft Edge Add-ons 심사 노트 |

Edge reviewer note에는 다음을 반드시 포함한다.

- `<all_urls>` 또는 content script match pattern이 필요한 이유
- HWP/HWPX 링크 감지, preview badge, 우클릭 메뉴, 로컬 파일 열기 처리 범위
- 새 외부 네트워크 endpoint가 없다는 점
- 문서 처리가 브라우저 내부 WASM에서 수행된다는 점
- 새 권한이 없는 경우 “No new permissions” 명시

Firefox AMO에는 확장 zip과 source zip을 함께 업로드한다. source zip은 재빌드 가능성을 보여주기 위한 자료이며,
대형 샘플/산출물/개인 환경 파일을 포함하지 않는다.

### 버전 올리기 예시

**MINOR 릴리즈** (조판 개선, 새 기능):
```
Cargo.toml:                  0.7.3 → 0.8.0
rhwp-vscode/package.json:    0.7.3 → 0.8.0
npm/editor/package.json:     0.7.3 → 0.8.0
rhwp-studio/package.json:    0.7.3 → 0.8.0
```

**PATCH 릴리즈** (npm README 수정 등):
```
npm/editor/package.json:  0.6.1 → 0.6.2  (다른 파일 변경 없음)
```

### Git 태그

- 태그는 `v{MAJOR}.{MINOR}.{PATCH}` 형식 (예: `v0.6.0`)
- Cargo.toml 기준 MINOR 릴리즈마다 태그를 생성한다
- PATCH 전용 릴리즈(npm README 등)는 태그를 생성하지 않는다

---

## 배포 절차

### 1단계: 코드 검증

```bash
cargo build && cargo test        # 네이티브 빌드 + 테스트
docker compose --env-file .env.docker run --rm wasm   # WASM 빌드
```

macOS 로컬에서 release 검증이 필요한 경우 `cargo test --release --tests` 대신
`cargo test --profile release-test --tests` 를 사용한다. 이유와 실측치는
[개발환경 가이드](dev_environment_guide.md)의 "macOS 로컬 빌드/테스트 검증"을
참조한다.

E2E 테스트:
```bash
cd rhwp-studio
CHROME_CDP=http://localhost:19222 node e2e/edit-pipeline.test.mjs --mode=host
# 16개 테스트 파일 순차 실행
```

### 2단계: 버전 업데이트 + CHANGELOG

**Cargo.toml** (Rust 패키지 + npm @rhwp/core 버전 원본):
```toml
version = "0.8.0"
```

**rhwp-vscode/package.json**:
```json
"version": "0.8.0"
```

**rhwp-vscode/CHANGELOG.md** 새 버전 항목 추가.

**npm/editor/package.json**:
```json
"version": "0.8.0"
```

**rhwp-studio/package.json** (제품정보 대화창 버전 자동 주입):
```json
"version": "0.8.0"
```

### 3단계: README 점검

모든 배포 대상의 README에 다음 항목이 포함되어야 한다:

| 항목 | rhwp-vscode | npm/core | npm/editor |
|------|:---------:|:-------:|:---------:|
| 기능 목록 | O | O | O |
| 폰트 가이드 | — | O (CDN/셀프호스팅) | O (내장 폴백 안내) |
| Third-Party Licenses | O | O | O |
| Trademark 면책 조항 | O | O | O |
| Notice (한컴 공개 문서) | O | O | O |

### 4단계: Git 커밋 + devel/main push

```bash
# 변경사항 커밋
git add -A
git commit -m "v0.7.3 릴리즈 준비"

# local/devel → devel 검증 → origin/devel push
git checkout devel
git merge local/devel
cargo build
cargo test
# macOS release 통합 검증: cargo test --profile release-test --tests
docker compose --env-file .env.docker run --rm wasm
git push origin devel

# devel → main merge → origin/main push
git checkout main
git merge devel
git push origin main
```

> 이 PC의 기본 작업 흐름은 task branch → `local/devel` merge → `devel` merge/test → `origin/devel` push → `main` merge/push 순서다.
> `local/devel`을 원격 `devel`로 직접 push하지 않는다.
> `devel`에서 최소 compile/test/WASM 빌드를 확인한 뒤 원격으로 push한다.
>
> main push 시 CI/CD가 자동 실행된다:
> - `ci.yml` → build + test + clippy 검증
> - `deploy-pages.yml` → GitHub Pages 데모 사이트 자동 배포

### 5단계: GitHub Release 생성 → npm 자동 배포

```bash
git tag v0.7.3
git push origin v0.7.3
gh release create v0.7.3 --title "v0.7.3 — 제목" --notes "릴리즈 노트"
```

> **Release 생성 시 `npm-publish.yml` 자동 실행:**
> 1. WASM 빌드
> 2. `scripts/prepare-npm.sh` 실행
> 3. npm Trusted Publishing(OIDC)으로 `@rhwp/core`, `@rhwp/editor` 배포
> 4. VS Code Marketplace / Open VSX 배포
>
> Trusted Publishing 사용 시 provenance attestation은 npm이 자동 생성한다.
>
> 수동으로 `cd pkg && npm publish`를 실행하지 않는다.
> 이미 VS Code/Open VSX가 배포된 뒤 npm만 재시도해야 하면 Actions에서 `npm-publish.yml`을
> `workflow_dispatch`로 실행하고 `publish_extensions=false`를 선택한다.

### 6단계: 배포 확인 (자동 완료 대기)

GitHub Release 생성 후 Actions 탭에서 `Publish All Packages` 워크플로우가 실행되는 것을 확인한다.

4개 job이 순차 실행된다:
1. **Build WASM** — WASM 빌드 + 아티팩트 업로드
2. **Publish @rhwp/core** — npm 배포
3. **Publish @rhwp/editor** — npm 배포
4. **Publish VSCode Extension** — Marketplace + Open VSX 배포

> 전체 소요 시간: 약 5~10분

### 7단계: 배포 확인

| 대상 | 확인 URL |
|------|---------|
| GitHub Pages | https://edwardkim.github.io/rhwp/ |
| VS Code Marketplace | https://marketplace.visualstudio.com/items?itemName=edwardkim.rhwp-vscode |
| Open VSX | https://open-vsx.org/extension/edwardkim/rhwp-vscode |
| npm @rhwp/core | https://www.npmjs.com/package/@rhwp/core |
| npm @rhwp/editor | https://www.npmjs.com/package/@rhwp/editor |

### 8단계: 브라우저 확장 스토어 업로드

1. `rhwp-chrome` build zip을 Chrome Web Store에 업로드한다.
2. 같은 zip을 `rhwp-edge-{version}.zip`으로 복사해 Microsoft Edge Add-ons에 업로드한다.
3. `rhwp-firefox` build zip을 Firefox AMO에 업로드한다.
4. Firefox AMO에는 `rhwp-source-{version}-amo.zip`도 함께 업로드한다.
5. 각 스토어 reviewer note에는 권한 사용 목적, 개인정보 미수집, 외부 전송 없음, WASM local processing을 명시한다.

---

## 토큰 관리

### 로컬 배포용 (`.env`)

| 토큰 | 발급처 | 용도 |
|------|--------|------|
| VSCE_PAT | [Azure DevOps](https://dev.azure.com) → Personal Access Tokens | VSCode 익스텐션 배포 |
| OVSX_PAT | [open-vsx.org](https://open-vsx.org) → Access Tokens | Open VSX 배포 |
| npm_token | [npmjs.com](https://www.npmjs.com) → Access Tokens | 수동 npm 배포가 필요한 경우에만 사용 |

### CI/CD 자동 배포용 (GitHub Secrets)

| Secret | 용도 |
|--------|------|
| VSCE_PAT | VS Code Marketplace 자동 배포 |
| OVSX_PAT | Open VSX 자동 배포 |

> GitHub Secrets 설정: Settings → Secrets and variables → Actions → New repository secret
> npm 자동 배포는 Secret 대신 npm Trusted Publisher 설정을 사용한다.
> npm package의 Trusted Publisher 설정에서 Environment name을 `NPM_TOKEN`으로 지정했기 때문에
> GitHub Actions에는 같은 이름의 environment가 필요하다. 이는 secret 값이 아니라 environment 이름이다.

---

## 배포 체크리스트

### 배포 전

- [ ] `cargo build` + `cargo test` 통과
- [ ] WASM 빌드 완료 (`pkg/`)
- [ ] E2E 테스트 통과
- [ ] 저작권 폰트가 포함되지 않았는지 확인
- [ ] Cargo.toml, package.json 버전 업데이트
- [ ] CHANGELOG.md 작성
- [ ] README 현행화 (기능, 폰트 가이드, 라이선스, 상표)
- [ ] THIRD_PARTY_LICENSES.md 현행화
- [ ] 확장 스토어 제출 문서 현행화 (`mydocs/feedback/`)
- [ ] 배포 zip에 `.env`, 개인 폰트, token, `node_modules/`, `target/`, `dist/` 불포함 확인

### 배포 순서

- [ ] devel push → CI 통과 확인
- [ ] main merge + push → GitHub Pages 배포 확인
- [ ] GitHub Release 생성 → Actions 탭에서 `Publish All Packages` 실행 확인
- [ ] @rhwp/core npm 배포 확인
- [ ] @rhwp/editor npm 배포 확인
- [ ] VS Code Marketplace 배포 확인
- [ ] Open VSX 배포 확인
- [ ] Chrome Web Store zip 업로드
- [ ] Microsoft Edge Add-ons zip 업로드
- [ ] Firefox AMO extension zip + source zip 업로드

---

## 수동 배포 (폴백)

CI/CD 실패 시 또는 README만 패치 배포할 때 수동으로 배포할 수 있다.

### VSCode 익스텐션

```bash
cd rhwp-vscode
bash publish.sh
```

사전 조건: `.env`에 `VSCE_PAT`, `OVSX_PAT` 설정

### npm @rhwp/core

```bash
bash scripts/prepare-npm.sh
cd pkg
npm publish --access public
```

> 원칙적으로 수동 npm publish는 사용하지 않는다.
> 2FA/OIDC 문제 조사 중 메인테이너가 직접 수행해야 하는 긴급 폴백에서만 사용한다.
> 이 경우에도 토큰을 shell history, 문서, 로그에 남기지 않는다.

### npm @rhwp/editor

```bash
cd npm/editor
npm publish --access public
```

> 수동 배포 시 CI/CD 자동 배포와 버전이 충돌하지 않도록 주의한다.
> 이미 배포된 버전이면 PATCH를 올려야 한다.
> npm Trusted Publishing이 정상 동작하는 경우 이 경로는 사용하지 않는다.

---

## 트러블슈팅

### VSCE_PAT 오류

```
❌ VSCE_PAT가 .env에 설정되지 않았습니다
```

- `.env` 파일에서 `VSCE_PAT=` 줄 앞에 개행이 있는지 확인
- Windows 줄바꿈(`\r`)이 포함되었을 수 있음: `cat -A .env`로 확인

### npm publish 버전 충돌

```
You cannot publish over the previously published versions
```

- 이미 배포된 버전. package.json 버전을 올려야 함 (예: 0.6.0 → 0.6.1)
- npm은 한 번 배포된 버전을 덮어쓸 수 없음
- CI/CD 자동 배포와 수동 배포가 충돌한 경우 패치 버전을 올려서 수동 배포

### pkg/ 권한 오류

```
Permission denied: pkg/package.json
```

- Docker 빌드로 `pkg/`가 root 소유로 생성된 경우
- `sudo chown -R $(whoami) pkg/` 로 소유권 변경 후 재시도

### GitHub Actions npm 배포 실패

- npm package settings의 Trusted Publisher 설정 확인
  - package: `@rhwp/core`, `@rhwp/editor` 각각 등록 필요
  - repository: `edwardkim/rhwp`
  - workflow filename: `npm-publish.yml`
  - environment name: `NPM_TOKEN` (npm 설정에서 비워두면 workflow에서도 제거)
  - allowed action: `npm publish`
- workflow `permissions.id-token: write` 설정 확인
- publish job에 `environment: NPM_TOKEN`이 설정되어 있는지 확인
- npm publish step에 `NPM_TOKEN` 또는 `NODE_AUTH_TOKEN` 환경변수가 주입되지 않는지 확인
- `package.json`의 repository URL이 GitHub repository와 일치하는지 확인
- `actions/setup-node`에 `registry-url`을 지정하지 않았는지 확인
- Actions 탭에서 `npm-publish.yml` 실행 로그 확인

### VS Code/Open VSX 재배포 없이 npm만 재시도

이미 VS Code Marketplace / Open VSX 배포가 완료된 상태에서 npm publish만 실패했다면:

1. Actions → `Publish All Packages` → Run workflow
2. `publish_extensions=false` 선택
3. `@rhwp/core`, `@rhwp/editor` job만 재시도

이 절차는 extension marketplace 중복 버전 업로드 오류를 피하기 위한 것이다.

### Open VSX 배포 실패

- OVSX_PAT 토큰 만료 확인 (open-vsx.org에서 재발급)
- `npx ovsx publish` 수동 실행으로 에러 메시지 확인

### Firefox AMO source zip 반려

- source zip 크기가 200 MB 이하인지 확인
- `samples/`, `pdf-large/`, `output/`, `target/`, `node_modules/`, `dist/`가 포함되지 않았는지 확인
- `LICENSE`, `THIRD_PARTY_LICENSES.md`, 빌드에 필요한 `package.json`/lockfile, `Cargo.toml`/`Cargo.lock`이 포함되었는지 확인
- reviewer note에 source zip의 재빌드 범위를 설명한다.

### Edge host permission 경고

Edge의 host permission은 manifest의 `permissions`뿐 아니라 `content_scripts.matches`도 포함한다.

- `<all_urls>` 또는 광범위 match pattern이 필요한 이유를 reviewer note에 명확히 적는다.
- HWP/HWPX 링크 감지, badge 표시, hover preview, 우클릭 메뉴, 로컬 파일 안내 범위를 설명한다.
- 새 네트워크 endpoint가 없고 문서가 외부 서버로 전송되지 않는다고 명시한다.
