# PR #330 처리 결과 보고서

## PR 정보

| 항목 | 내용 |
|------|------|
| PR 번호 | [#330](https://github.com/edwardkim/rhwp/pull/330) |
| 작성자 | [@ivLis-Studio](https://github.com/ivLis-Studio) (ivLis) |
| 제목 | Add Firefox Extension |
| 처리 | **Close** (정중한 안내) |
| 처리일 | 2026-04-25 |

## PR 내용

`rhwp-firefox/` 디렉토리에 Firefox WebExtension Manifest V3 패키지 16 파일 신규 추가:
- `manifest.json` (60줄)
- `background.js`, `content-script.js` (375줄), `options.js`, `firefox-compat.js`
- `build.mjs`, `vite.config.ts`, `package.json`, `package-lock.json`
- `sw/context-menus.js`, `sw/download-observer.js`, `sw/message-router.js`, `sw/thumbnail-extractor.js` (385줄), `sw/viewer-launcher.js`
- `README.md`
- `.gitignore` (수정)

## Close 사유

### 1. 영역 정면 충돌
`rhwp-firefox/` 디렉토리는 이미 `devel` 브랜치에 v0.2.1 까지 발전된 상태로 존재. 16 파일 모두 동일 경로에 새 파일 추가 시도 → **머지 불가**.

### 2. AMO 제출 진행 중
우리 측 v0.2.1 이 AMO (Mozilla Add-ons) 제출 검토 중이며, 같은 영역에 외부 코드 머지 시 제출 흐름과 파편화.

### 3. AMO 제출 전제 조건 누락
PR 의 manifest 에 다음이 누락:
- `browser_specific_settings.gecko.id`
- `data_collection_permissions`
- `_locales/` 다국어 디렉토리
- `PRIVACY.md` (AMO 제출 필수)

우리 기존 manifest 는 이를 모두 포함 (`feedback_amo_submission_gotchas.md` 메모리 규칙 준수).

### 4. base=main 으로 PR 작성 (우리 측 안내 부족 일부)
- GitHub 저장소 default branch = `main`
- README 에 base 브랜치 안내 없음 (CONTRIBUTING.md 에만 있음)
- 외부 기여자가 main 만 보고 "rhwp-firefox/ 가 없어 추가" 라고 판단

→ 작성자 책임 + 우리 안내 부족 양쪽 사유.

## 후속 작업 (메인테이너)

### 1. README 개선
다음 외부 기여자도 같은 혼란을 겪지 않도록 **README.md / README_EN.md 의 잘 보이는 위치에 base=devel 안내** 추가 (별도 task).

### 2. AMO 워닝 별도 task
PR #330 과 무관하지만, 본 검토 과정에서 AMO 워닝 (`mydocs/feedback/amo-warning-01.md`) 확인:
- `manifest.json` 의 `strict_min_version` (112) vs `data_collection_permissions` (Firefox 140+ 필요) 모순 (2건)
- `wasm/rhwp.js` 의 `Function` 생성자 (1건, wasm-bindgen 표준)
- `assets/viewer-*.js` 의 `eval` / `innerHTML` / `document.write` (다수, Vite 번들 dependency 영역)

→ `strict_min_version` 상향 + dependency 코드 검토 별도 task 후보.

## 코멘트

[close comment](https://github.com/edwardkim/rhwp/pull/330#issuecomment-4319890378) 에서:
- 충돌 영역 명시 (devel 에 이미 v0.2.1 존재)
- AMO 제출 단계 안내
- base=main 이슈에 우리 측 안내 부족도 인정
- 향후 기여 환영 영역 4가지 제안 (워닝 해결 / 다국어 / UX / 보안)

## 참고 링크

- [PR #330](https://github.com/edwardkim/rhwp/pull/330)
- [Close comment](https://github.com/edwardkim/rhwp/pull/330#issuecomment-4319890378)
- AMO 워닝: `mydocs/feedback/amo-warning-01.md`
- 메모리 규칙: `feedback_amo_submission_gotchas.md`
