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

---

## CI/CD 워크플로우 (GitHub Actions)

### 자동 실행되는 워크플로우

| 파일 | 트리거 | 역할 |
|------|--------|------|
| `.github/workflows/ci.yml` | push/PR (main, devel) | cargo build + test + clippy 검증 |
| `.github/workflows/deploy-pages.yml` | main push, 태그 | WASM 빌드 → rhwp-studio 빌드 → GitHub Pages 배포 |
| `.github/workflows/npm-publish.yml` | **GitHub Release 생성** | WASM 빌드 → @rhwp/core + @rhwp/editor + VSCode 익스텐션 일괄 배포 |

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
```

> **중요**: GitHub Release를 생성하면 5곳 모두 자동 배포된다. 수동 `npm publish`나 `publish.sh`를 실행하지 않는다.

### GitHub Secrets 설정

GitHub Actions에서 사용하는 시크릿 (Settings → Secrets and variables → Actions):

| Secret 이름 | 용도 |
|------------|------|
| `NPM_TOKEN` | npm 배포 인증 (@rhwp/core, @rhwp/editor) |
| `VSCE_PAT` | VS Code Marketplace 배포 인증 |
| `OVSX_PAT` | Open VSX 배포 인증 |

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

| 영역 | 2026-04-23 현재 |
|------|----------------|
| 라이브러리 (Cargo.toml) | `0.7.3` |
| rhwp-chrome / Edge | `0.2.1` (Chrome Web Store / Edge Add-ons 심사 통과) |
| rhwp-safari | `0.2.1` |
| rhwp-firefox | `0.1.1` (AMO 등록 준비 중) |

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
cargo build && cargo test        # 네이티브 빌드 + 941개 테스트 (2026-04-23 기준)
docker compose --env-file .env.docker run --rm wasm   # WASM 빌드
```

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

# devel → main merge
git checkout devel && git merge local/devel && git push origin devel
git checkout main && git merge devel && git push origin main
```

> main push 시 CI/CD가 자동 실행된다:
> - `ci.yml` → build + test + clippy 검증
> - `deploy-pages.yml` → GitHub Pages 데모 사이트 자동 배포

### 5단계: GitHub Release 생성 → npm @rhwp/core 자동 배포

```bash
git tag v0.7.3
git push origin v0.7.3
gh release create v0.7.3 --title "v0.7.3 — 제목" --notes "릴리즈 노트"
```

> **Release 생성 시 `npm-publish.yml` 자동 실행:**
> 1. WASM 빌드
> 2. `scripts/prepare-npm.sh` 실행
> 3. `npm publish --access public --provenance`
>
> 수동으로 `cd pkg && npm publish`를 실행하지 않는다.

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

---

## 토큰 관리

### 로컬 배포용 (`.env`)

| 토큰 | 발급처 | 용도 |
|------|--------|------|
| VSCE_PAT | [Azure DevOps](https://dev.azure.com) → Personal Access Tokens | VSCode 익스텐션 배포 |
| OVSX_PAT | [open-vsx.org](https://open-vsx.org) → Access Tokens | Open VSX 배포 |
| npm_token | [npmjs.com](https://www.npmjs.com) → Access Tokens | @rhwp/editor 수동 배포 |

### CI/CD 자동 배포용 (GitHub Secrets)

| Secret | 용도 |
|--------|------|
| NPM_TOKEN | @rhwp/core 자동 배포 (npm-publish.yml) |

> GitHub Secrets 설정: Settings → Secrets and variables → Actions → New repository secret

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

### 배포 순서

- [ ] devel push → CI 통과 확인
- [ ] main merge + push → GitHub Pages 배포 확인
- [ ] GitHub Release 생성 → Actions 탭에서 `Publish All Packages` 실행 확인
- [ ] @rhwp/core npm 배포 확인
- [ ] @rhwp/editor npm 배포 확인
- [ ] VS Code Marketplace 배포 확인
- [ ] Open VSX 배포 확인

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

사전 조건: `~/.npmrc`에 npm 토큰 설정

### npm @rhwp/editor

```bash
cd npm/editor
npm publish --access public
```

> 수동 배포 시 CI/CD 자동 배포와 버전이 충돌하지 않도록 주의한다.
> 이미 배포된 버전이면 PATCH를 올려야 한다.

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

- GitHub Secrets에 `NPM_TOKEN`이 설정되어 있는지 확인
- 토큰 만료 여부 확인 (npmjs.com에서 재발급)
- Actions 탭에서 `npm-publish.yml` 실행 로그 확인

### Open VSX 배포 실패

- OVSX_PAT 토큰 만료 확인 (open-vsx.org에서 재발급)
- `npx ovsx publish` 수동 실행으로 에러 메시지 확인
