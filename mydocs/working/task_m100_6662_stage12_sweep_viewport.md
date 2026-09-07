---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_6662_stage12_sweep_viewport.md
last_verified: 2026-09-06
---

# 열린 이슈 재검증 12단계: 기준 비교 raster 하단 누락

Issue: #6712 검증 기반. 시작 HEAD: `ad72b8ff3`.

## 분석과 계획

- 11단계 비교 PNG에서 하단선과 로고 일부가 잘린 것으로 보였다. 그러나 SVG의
  하단선 y=1049.12, 로고 bottom<=1053.40은 셀 clip bottom=1054.43 및
  Body clip bottom=1065.80 안에 있다. 렌더러 clip 결함으로 단정할 수 없다.
- PNG 크기는 794x1123인데 두 페이지 모두 y=1040 이후 색상 픽셀이 0이다.
  `--headless=new --window-size`의 외부 창 크기와 실제 content viewport 차이를
  먼저 조사한다. 기존 Chrome 종료/file polling 경합도 같은 캡처 경로에 있다.
- 독립 작은 SVG의 네 모서리와 아래 경계가 실제 PNG에 보이는지 재현한다.
  문서 layout/clip을 이 현상에 맞춰 변경하지 않는다. 필요한 경우 기존 browser
  의존성을 활용해 정확한 viewport와 screenshot 완료를 기다리는 방식으로 보정한다.
- 도구 테스트 및 두 가정통신문 sweep 재실행 후 단계 결과를 커밋한다.
  모든 임시 로그/PNG/SVG/JSON은 `/tmp`에 보관하며 커밋하지 않는다.

11단계의 '하단 clip' 표현은 raster 관찰이다. 원인 및 전체 #6712 해결 판정은
이번 단계와 남은 layout 검증 이후에 갱신한다.

## 구현과 검증 결과

- 독립 794x1123 SVG의 1배 PNG에서 하단 파란 모서리가 흰색으로 나왔다.
  2배 시도는 PNG가 저장되고 Chrome exit=0인데도 file polling 순서로 실패했다.
- 기존 Studio `puppeteer-core`를 재사용하여 content viewport/DPR을 명시하고,
  `document.fonts.ready`와 `page.screenshot()` 완료를 기다리도록 교체했다.
  브라우저는 finally에서 닫으며 공유 profile을 사용하지 않는다.
- terminal fallback 이름에 따옴표를 중첩해 `font-family` 속성이 깨지는 문제도
  별도 red 테스트로 확인했다. CSS identifier인 fallback 이름은 따옴표 없이 추가한다.
- Node 단위 테스트 2/2 passed. 실제 Chrome 네 모서리 검사 1 test(1배/2배 subcase)
  passed, 3.073초. 기존 visual sweep Python 테스트 46/46 passed, 3.204초.
  `node --check`와 `git diff --check`도 통과했다.
- Linux Chromium 실측이다. macOS/Windows 실기 검증은 하지 않았으며 OS 장식 높이를
  하드코딩하지 않고 공통 DevTools viewport API를 사용한다.

## 재산출 판단

- 기존과 같은 CLI SHA-256 `8daef59bd762a785983eea06532a52e9d3bd6a19a639284102d5567701872bc9`.
  Rust renderer 변경 없이 두 문서의 1, 2쪽을 새 출력 경로에 산출했다.
- `/tmp/rhwp-6712-stage12-ko-sweep`와 `/tmp/rhwp-6712-stage12-zh-sweep` 모두
  PDF/SVG 2쪽, sweep exit 0. review PNG 4장을 직접 열었다.
- 한국어 하단 테두리 및 로고의 일부 소실은 캡처 결함이었다. 11단계의 그림 소유
  보정은 필요하지만, 이후 관찰한 'clip 소실'은 이 단계에서 정정한다.
- 중국어 2쪽 하단선/문구 교차와 양쪽 문서의 세로 위치 차이는 여전히 존재한다.
  글꼴 공급 교정으로 글자 모양도 달라졌으므로 이전 PNG의 픽셀 일치율과 단순 비교하지 않는다.
  **#6712는 아직 해결 완료가 아니다.** 다음 단계는 실제 저장 줄의 advance를 조사한다.
- 로그와 중간 산출물은 커밋하지 않는다. 최종 코멘트에 필요한 비교 PNG만 전체 판정 후
  선별한다. Rust 전체 회귀/lint는 이 도구 검사로 대체하지 않는다.
