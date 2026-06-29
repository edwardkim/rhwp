# Task #1521 Stage 4 완료보고서 — 회귀 검증

## 범위

- 작업지시자의 브라우저 확장 수동 로드 검증 통과 결과를 기록했다.
- 수정 source content-script 문법 검사를 재실행했다.
- 로드용 `dist/` content-script 문법 검사와 source 일치성을 확인했다.
- 기존 브라우저 확장 서비스워커 회귀 테스트를 재실행했다.
- 로드용 산출물의 핵심 파일과 manifest 파싱을 확인했다.

## 수동 검증 결과

작업지시자가 2026-06-29에 확장을 직접 로드해 수동 검증을 수행했고, 검증 통과를 확인했다.

Stage 3에서 요청한 핵심 시나리오는 다음 동작의 사용자 환경 확인으로 본다.

- show delay 이전에 링크를 벗어나면 hover card가 생성되지 않음
- 정상 hover 시 hover card가 표시됨
- 링크에서 card로 이동하면 card가 유지됨
- card를 벗어나면 card가 닫힘
- 빠르게 여러 링크를 지나가도 이전 링크의 stale card가 남지 않음
- hover card click의 기존 `open-hwp` 요청 흐름 유지
- 보안 fixture의 data attribute가 실행되지 않고 텍스트로 취급됨

## 자동 검증 결과

### source content-script 문법 검사

통과:

- `node --check rhwp-chrome/content-script.js`
- `node --check rhwp-firefox/content-script.js`
- `node --check rhwp-safari/src/content-script.js`

### dist content-script 문법 검사

통과:

- `node --check rhwp-chrome/dist/content-script.js`
- `node --check rhwp-firefox/dist/content-script.js`
- `node --check rhwp-safari/dist/content-script.js`

### 기존 서비스워커 테스트

통과:

- `node rhwp-chrome/sw/fetch-security.test.mjs`
- `node --test rhwp-chrome/sw/download-interceptor.test.mjs` — 13개 통과
- `node --test rhwp-firefox/sw/download-interceptor.test.mjs` — 11개 통과

참고: `rhwp-firefox/sw/fetch-security.test.mjs`는 현재 저장소에 존재하지 않아 실행 대상에서 제외했다.

### diff 검사

통과:

- `git diff --check`

## 로드용 산출물 확인

Stage 3에서 준비한 로드용 산출물을 재확인했다.

확인한 경로:

- `rhwp-chrome/dist/`
- `rhwp-firefox/dist/`
- `rhwp-safari/dist/`

확인 결과:

- `rhwp-chrome/dist/manifest.json` 파싱 성공: MV3, version `0.2.7`
- `rhwp-firefox/dist/manifest.json` 파싱 성공: MV3, version `0.2.7`
- `rhwp-safari/dist/manifest.json` 파싱 성공: MV3, version `0.2.1`
- 각 dist에 `manifest.json`, `viewer.html`, `background.js`, `content-script.js`, `wasm/rhwp_bg.wasm` 존재
- 각 dist의 `content-script.js`는 대응 source와 byte-for-byte 일치
- `dist/`는 git ignored 상태라 PR/커밋 대상 변경에는 포함되지 않음

## 변경 범위 요약

현재 추적 source 변경은 content-script 3개 파일에 한정된다.

```text
rhwp-chrome/content-script.js     | 104 ++++++++++++++++++++++++++++++-------
rhwp-firefox/content-script.js    | 106 ++++++++++++++++++++++++++++++--------
rhwp-safari/src/content-script.js |  98 ++++++++++++++++++++++++++++-------
3 files changed, 248 insertions(+), 60 deletions(-)
```

## 남은 위험과 처리

| 항목 | 상태 |
|------|------|
| `:hover` stale guard가 정상 hover 표시를 막을 가능성 | 수동 검증 통과로 해소 |
| Chrome/Firefox closed Shadow DOM 때문에 CLI 자동 hover 검증이 제한됨 | 작업지시자 수동 검증으로 보완 |
| 표준 `npm run build`의 Vite config 해석 실패 | Stage 3에서 환경 문제로 분리, 로드용 dist는 별도 경로로 준비 및 검증 완료 |

## 다음 단계

Stage 5에서 최종 결과보고서를 작성하고, 오늘 할일 문서의 상태를 완료 단계로 갱신한다.
