---
title: "#4823 Stage18 한양중고딕 Canvas2D 웹폰트 별칭"
issue: 4823
stage: 18
status: complete
---

# Stage18: 한양중고딕 Canvas2D 웹폰트 별칭

## 배경

PR #4857의 Canvas visual diff는 `table-border-style`에서만 실패했다. CI의
CanvasKit 사전 분석은 이 문서의 필수 글꼴을 `한양중고딕`으로 식별했다.

CanvasKit 등록 계획은 `한양중고딕`을 `HY중고딕`으로 대체하지만, Canvas2D의
문서 웹폰트 선택은 문서 요청명과 카탈로그 이름이 일치해야 한다. 카탈로그에는
`HY중고딕`만 있어 Linux CI의 Canvas2D는 같은 대체 face를 등록하지 못했다.

## 변경

- `한양중고딕`을 `HY중고딕`과 동일한 `NotoSansKR-Regular.woff2` face로 등록했다.
- 문서 요청명을 카탈로그에 유지하는 회귀 테스트를 추가했다.

## 검증 결과

문서 웹폰트 준비 단계에서 Canvas2D는 `한양중고딕` 요청명으로
`NotoSansKR-Regular.woff2` face를 선택한다. 로컬 baseline의 잉크 마스크 차이는
CI에서 face가 누락됐을 때의 `0.0131384`에서 `0.0053428`로 줄었지만, 기존 `0.005`
예산을 조금 초과했다. 남은 Canvas2D/CanvasKit rasterization 차이는 다음 단계에서
fixture 예산의 근거로 분리 검토한다.

## 검증 범위

- Studio TypeScript 검사와 테스트
- Studio production build
- `table-border-style` Canvas baseline 비교

Rust, WASM, 문서 fixture와 렌더러 구현은 변경하지 않는다.
