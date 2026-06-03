# v0.7.13 PATCH 릴리즈 결과 보고서

- 일자: 2026-05-26
- 방식: 1 방식 + 패턴 A
- 기준: v0.7.12 (`1899ef9b`) -> v0.7.13 (`ba87d20e`)

## 1. 결정

PATCH 릴리즈. HWPX 렌더링/저장 호환성 개선, 외부 기여자 PR 흡수, Chrome 확장 로컬 파일
처리 개선을 포함한다. 공개 API의 하위 호환성을 깨지 않는 버그 수정 및 문서 보강 범위이므로
PATCH 버전으로 진행한다.

## 2. 버전 동기화

| 파일 | 0.7.12 -> 0.7.13 |
|------|------------------|
| `Cargo.toml` | ✓ |
| `rhwp-studio/package.json` | ✓ |
| `rhwp-studio/package-lock.json` | ✓ |
| `rhwp-vscode/package.json` | ✓ |
| `rhwp-vscode/package-lock.json` | ✓ |
| `npm/editor/package.json` | ✓ |
| `rhwp-chrome/package.json` | `0.2.2 -> 0.2.3` |
| `rhwp-chrome/package-lock.json` | stale `0.1.0` -> `0.2.3` |
| `rhwp-chrome/manifest.json` | `0.2.2 -> 0.2.3` |
| `rhwp-chrome/content-script.js` | `0.2.2 -> 0.2.3` |
| `rhwp-chrome/dev-tools-inject.js` | `0.2.2 -> 0.2.3` |
| `rhwp-firefox/package.json` | `0.2.2 -> 0.2.3` |
| `rhwp-firefox/package-lock.json` | stale `0.1.1` -> `0.2.3` |
| `rhwp-firefox/manifest.json` | `0.2.2 -> 0.2.3` |

`Cargo.lock`의 `rhwp` package entry는 이미 `0.7.13` 상태이며 추가 diff가 없다.

## 3. CHANGELOG

- `CHANGELOG.md` — `## [0.7.13] — 2026-05-26`
- `CHANGELOG_EN.md` — `## [0.7.13] — 2026-05-26`
- `rhwp-vscode/CHANGELOG.md` — `## [0.7.13] - 2026-05-26`

## 4. 주요 포함 범위

- HWPX -> HWP 저장 호환성
  - 표/셀 계약, gradient BorderFill, 셀 안쪽 여백, 셀 배경 이미지 채우기 유형, 메모 컨트롤,
    목차 field marker/page notation, 쪽번호 감추기/새 번호 컨트롤 저장 개선
  - `hwpx-h-*`, `mel-001`, `aift`, `exam_kor`, `exam_social` 계열 손상/중단 케이스 개선
- HWPX 렌더링
  - 바탕쪽, 머리말/꼬리말, 문단번호, 글상자, 그라데이션, 사각형 모서리 곡률, 지문 박스 렌더링 보강
- 레이아웃/페이지네이션
  - treat-as-char 표, 중첩 표, 그림 pushdown/vpos, 미주 다단, 하단 overflow 관련 보정
- Chrome 확장
  - `file://` 로컬 문서 안내 및 중복 다운로드 억제
- 브라우저 확장 배포
  - Chrome/Edge/Firefox 확장 `0.2.3` 패치 배포 준비
  - Chrome 빌드 스크립트가 `dist/`를 먼저 정리하도록 보강해 stale asset 혼입 방지
- PR 처리
  - 외부 기여자 PR cherry-pick/review 결과 반영

## 5. 자기 검증

| 항목 | 결과 |
|------|------|
| `cargo fmt --all -- --check` | 통과 |
| `cargo build` | 통과 |
| `cargo test` | 통과 |
| `cargo clippy -- -D warnings` | 통과 |
| `cargo check --target wasm32-unknown-unknown --lib` | 통과 |
| `cargo test --features native-skia skia --lib` | 통과 |
| `docker compose --env-file .env.docker run --rm wasm` | 통과 |
| `rhwp-studio npm run build` | 통과 |
| `rhwp-chrome npm run build` | 통과 |
| `rhwp-firefox npm run build` | 통과 |
| `rhwp-vscode npm run compile` | 통과 |
| Chrome/Edge 제출용 zip | `rhwp-chrome/rhwp-chrome-0.2.3.zip`, `rhwp-chrome/rhwp-edge-0.2.3.zip` |
| Firefox 제출용 zip | `rhwp-firefox/rhwp-firefox-0.2.3.zip` |
| Firefox AMO source zip | `rhwp-firefox/rhwp-source-0.2.3.zip` |

## 6. 주의 사항

- 2026-05-26 현재 GitHub Actions 장애로 원격 CI 수동 실행이 HTTP 500을 반환했다.
  따라서 릴리즈 전 검증은 로컬 빌드/테스트/WASM 빌드 결과를 기준으로 수행했다.
- `exam_social` 3페이지 홀수 머리말 글상자 높이 차이는 별도 후속 이슈로 분리한다.

## 7. 릴리즈 절차

1. `local/devel`에서 버전 bump + CHANGELOG + 본 보고서 작성
2. 로컬 검증 완료 후 릴리즈 커밋
3. `devel`에 `local/devel` 병합
4. `devel`에서 재검증 후 `origin/devel` push
5. `main`에 `devel` 병합
6. `main` 검증 후 `origin/main` push
