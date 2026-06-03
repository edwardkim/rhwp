# 최종 결과 보고서 — Task #133

**이슈**: [#133](https://github.com/edwardkim/rhwp/issues/133)
**타이틀**: npm 패키지 배포 준비 — @rhwp/core + @rhwp/editor v0.7.2
**마일스톤**: M100
**브랜치**: `local/task133`
**완료일**: 2026-04-13

---

## 0단계: devel → main PR merge

- PR [#134](https://github.com/edwardkim/rhwp/pull/134) 생성 및 merge 완료
- 충돌 해결: `zip 2.4→8.5` (Dependabot), `puppeteer-core 24.40.0`, `vite 8.0.8` 반영
- `deploy-pages.yml` 자동 실행 → GitHub Pages 최신화 (단축키·커맨드 팔레트 반영)
- 로컬 `main` fast-forward 업데이트 완료

## 1단계: 버전 일괄 변경

| 파일 | 이전 | 이후 |
|------|------|------|
| `Cargo.toml` | 0.7.1 | **0.7.2** |
| `rhwp-studio/package.json` | 0.7.1 | **0.7.2** |
| `npm/editor/package.json` | 0.7.0 | **0.7.2** |
| `rhwp-vscode/package.json` | 0.7.0 | **0.7.2** |

## 2단계: Docker WASM 빌드 + prepare-npm.sh

```
Compiling rhwp v0.7.2 (/app)
Finished `release` profile in 30.88s
wasm-opt 최적화 완료
Done in 1m 18s

scripts/prepare-npm.sh → pkg/package.json + README.md ✅
```

- `pkg/rhwp_bg.wasm`: 3.4MB (2026-04-13 22:00 빌드)

## 3단계: npm publish 체크리스트

**@rhwp/core** (`pkg/`):
- [x] `pkg/package.json` name: `@rhwp/core`, version: `0.7.2`
- [x] `pkg/rhwp.d.ts` 신규 API 포함:
  - `setFormValueInCell()` — 셀 내 양식 컨트롤 값 설정
  - `getShowTransparentBorders()` / `setShowTransparentBorders()` — 투명 선 상태
- [x] `pkg/rhwp_bg.wasm` 최신 빌드 확인 (v0.7.2 소스)
- [x] `pkg/` 폴더는 `.gitignore` 대상 — `npm publish pkg/` 직접 실행

**@rhwp/editor** (`npm/editor/`):
- [x] `npm/editor/package.json` version: `0.7.2`
- [x] `DEFAULT_STUDIO_URL`: `https://edwardkim.github.io/rhwp/` (GitHub Pages, 최신 배포됨)
- [x] API (`loadFile`, `pageCount`, `getPageSvg`) 변경 없음

## npm publish 명령 (작업지시자 승인 후 실행)

```bash
# @rhwp/core
cd /home/edward/mygithub/rhwp
npm publish pkg/ --access public

# @rhwp/editor
npm publish npm/editor/ --access public
```

## v0.7.2 반영 기능 요약

| 타스크 | 기능 | @rhwp/core | @rhwp/editor |
|--------|------|-----------|-------------|
| #110-#112 | 양식 컨트롤 파싱 + 셀 API | ✅ WASM 재빌드 반영 | ✅ GitHub Pages 자동 반영 |
| #126 | TypeScript strict 수정 | — | ✅ |
| #127 | quick-xml 0.39, zip 8.5 | ✅ | ✅ |
| #130 | 투명 선 토글 단축키 | — | ✅ |
| #131 | 한컴 단축키 + 커맨드 팔레트 | — | ✅ |
| #132 | VS Code 컨텍스트 메뉴 4개 | — | — (VS Code 전용) |
