# Task M100 #1487 Stage 3

- 이슈: #1487 확장 옵션에 외부 웹폰트 사용 안 함 설정 추가
- 브랜치: `task_m100_1487`
- 작성일: 2026-06-23
- 상태: 구현/검증 완료

## 배경

Stage 1 구현 후 시각 검증 중 `외부 웹폰트 사용 안 함` 옵션이 켜져 있어도
문서 로드 시 `로컬 글꼴 감지` 모달이 표시되는 것을 확인했다.

이 모달은 외부 CDN 웹폰트 로드 여부가 아니라, 사용자 PC에 설치된 원본 글꼴을
감지할지 묻는 기능이다. 하지만 버튼/설명에 `웹 대체`라는 표현이 있어
`외부 웹폰트 사용 안 함` 옵션과 혼동될 수 있다.

## 구현 방향

- 로컬 글꼴 감지 모달은 유지한다.
- 사용자에게 보이는 `웹 대체` 표현을 `대체 글꼴`로 바꾼다.
- `외부 웹폰트 사용 안 함` 옵션이 켜진 상태에서는 모달 본문에 해당 상태를 표시한다.
- 실제 외부 웹폰트 차단 동작은 Stage 1의 `disableExternalWebFonts` 경로를 유지한다.

## 검증 계획

```bash
cd rhwp-studio && npm test
cd rhwp-studio && npm run build
cd rhwp-chrome && node build.mjs
git diff --check
```

## 구현 내용

- 로컬 글꼴 감지 모달의 사용자 노출 문구를 `웹 대체`에서 `대체 글꼴`로 변경했다.
  - 버튼: `웹 대체로 보기` → `대체 글꼴로 보기`
  - 상태/요약: `웹 대체 사용` → `대체 글꼴 사용`
  - 안내 문장: `웹 대체 글꼴` → `대체 글꼴`
- `showLocalFontsModalIfNeeded()`에 options 인자를 추가했다.
- viewer의 `extensionViewerSettings.disableExternalWebFonts` 값을 로컬 글꼴 감지 모달에 전달했다.
- `외부 웹폰트 사용 안 함`이 켜진 상태에서는 모달 본문에 `외부 웹폰트 사용 안 함: 켜짐` 안내를 표시하도록 했다.
- 문구/상태 표시 회귀 테스트를 추가했다.

## 검증 결과

- `cd rhwp-studio && npm test`: 통과 (130 passed)
- `cd rhwp-studio && npm run build`: 통과
- `cd rhwp-chrome && node build.mjs`: 통과
- `git diff --check`: 통과
