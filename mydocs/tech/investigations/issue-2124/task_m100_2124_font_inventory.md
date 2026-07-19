---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-2124/README.md
last_verified: 2026-07-16
---

# Task M100 #2124 — frontend font inventory

- 이슈: #2124
- 단계: Stage 4 — font inventory
- 작성일: 2026-07-10
- 기준 브랜치: `upstream/devel`
- 기준 커밋: `3077f96d1f9931c50d6d62be77b389d4f66470a9`
- 관련 문서:
  - `mydocs/tech/font_fallback_strategy.md`
  - `web/fonts/FONTS.md`
  - `THIRD_PARTY_LICENSES.md`
  - `web/fonts/SourceHanSerifK-OFL.txt`

## 1. 목적

이 문서는 프론트 리팩터링 전에 폰트 파일, 참조 경로, 라이선스 근거, 빌드 복사 경로를 고정하기 위한
Phase 0 기준선이다. #2124에서는 폰트 위치나 파일 구성을 변경하지 않는다.

특히 `/web` legacy 폴더 제거나 `rhwp-studio` 이전은 #2125 이후의 별도 리팩터링 후보이며, 이 문서는
그 판단 전에 지켜야 할 계약과 현재 참조 지점을 기록한다.

## 2. 현재 폰트 저장소

| 항목 | 값 |
|------|----|
| canonical font directory | `web/fonts` |
| 파일 수 | `.woff2` 36개 |
| WOFF2 총 크기 | 22,630,940 bytes |
| 라이선스 요약 | `web/fonts/FONTS.md`, `THIRD_PARTY_LICENSES.md` |
| Source Han Serif Korean OFL | `web/fonts/SourceHanSerifK-OFL.txt` |
| studio public 경로 | `rhwp-studio/public/fonts -> ../../web/fonts` symlink |

`mydocs/tech/font_fallback_strategy.md`는 폰트 폴백 전략의 기준 문서다. 다만 문서 안의 과거 파일 수
언급은 현재 실제 파일 수와 다를 수 있으므로, #2124의 inventory 기준은 이 문서의 파일 목록과 실제
`web/fonts/*.woff2` 스냅샷이다.

각 WOFF2와 라이선스 문서의 정확한 byte 수 및 SHA-256은
`mydocs/metrics/frontend/2026-07-11/metrics.json`의 `fontAssets`에 보존한다. 파일 목록 아래의 표시
크기는 사람이 읽기 위한 반올림 값이며 무결성 비교에는 사용하지 않는다.

## 3. 파일 목록

| 파일 | 크기 |
|------|------|
| `Cafe24Ssurround-v2.0.woff2` | 383K |
| `Cafe24Supermagic-Regular-v1.0.woff2` | 748K |
| `D2Coding-Bold.woff2` | 1.5M |
| `D2Coding-Regular.woff2` | 1.4M |
| `GowunBatang-Bold.woff2` | 437K |
| `GowunBatang-Regular.woff2` | 483K |
| `GowunDodum-Regular.woff2` | 397K |
| `Happiness-Sans-Bold.woff2` | 329K |
| `Happiness-Sans-Regular.woff2` | 285K |
| `Happiness-Sans-Title.woff2` | 341K |
| `HappinessSansVF.woff2` | 599K |
| `LatinModernMath-Regular.woff2` | 383K |
| `NanumGothic-Bold.woff2` | 407K |
| `NanumGothic-ExtraBold.woff2` | 368K |
| `NanumGothic-Regular.woff2` | 333K |
| `NanumGothicCoding-Bold.woff2` | 522K |
| `NanumGothicCoding-Regular.woff2` | 509K |
| `NanumMyeongjo-Bold.woff2` | 562K |
| `NanumMyeongjo-ExtraBold.woff2` | 678K |
| `NanumMyeongjo-Regular.woff2` | 490K |
| `NotoSansKR-Bold.woff2` | 546K |
| `NotoSansKR-ExtraLight.woff2` | 579K |
| `NotoSansKR-Regular.woff2` | 529K |
| `NotoSerifKR-Bold.woff2` | 1.0M |
| `NotoSerifKR-Regular.woff2` | 949K |
| `Pretendard-Black.woff2` | 782K |
| `Pretendard-Bold.woff2` | 773K |
| `Pretendard-ExtraBold.woff2` | 775K |
| `Pretendard-ExtraLight.woff2` | 717K |
| `Pretendard-Light.woff2` | 739K |
| `Pretendard-Medium.woff2` | 760K |
| `Pretendard-Regular.woff2` | 748K |
| `Pretendard-SemiBold.woff2` | 767K |
| `Pretendard-Thin.woff2` | 679K |
| `SourceHanSerifK-OldHangul-subset.woff2` | 234K |
| `SpoqaHanSans-Regular.woff2` | 269K |

## 4. 참조 경로

| 소비자 | 참조 방식 | 리팩터링 주의점 |
|--------|-----------|----------------|
| `rhwp-studio` | `rhwp-studio/public/fonts` symlink가 `web/fonts`를 가리킴 | Vite public path와 `font-loader.ts`의 `/fonts/...` URL을 함께 확인해야 한다. |
| `rhwp-studio/src/core/font-loader.ts` | open-source webfont와 일부 외부 CDN font를 `FontFace`로 등록 | `disableExternalWebFonts`, critical font, `font-display: swap`, unicode range를 보존한다. |
| `rhwp-studio/src/core/font-substitution.ts` | Hancom 계열 글꼴명을 open-source fallback chain으로 치환 | 실제 렌더링 폭과 대체 chain 순서가 contract다. |
| `rhwp-chrome/build.mjs` | `web/fonts/*.woff2` 전체를 `dist/fonts`로 복사 | extension package 크기와 manifest `web_accessible_resources`를 함께 본다. |
| `rhwp-firefox/build.mjs` | `web/fonts/*.woff2` 전체를 `dist/fonts`로 복사 | Chrome과 복사 정책이 맞아야 한다. |
| `rhwp-safari/build.sh` | Chrome dist를 기반으로 Safari dist를 구성 | Safari 단독 font copy가 아니라 Chrome build 결과에 의존한다. |
| `rhwp-vscode/webpack.config.js` | 선택된 font 파일만 `dist/media/fonts`로 복사 | VS Code webview CSP와 `asWebviewUri` 경로를 보존한다. |
| `npm/editor/README.md` | self-hosting 안내에서 Studio build의 `web/fonts/`를 직접 설명 | canonical 경로 변경 시 downstream 문서도 같은 PR에서 갱신한다. |

## 5. VS Code 선택 복사 목록

`rhwp-vscode`는 모든 font를 복사하지 않고 다음 subset을 사용한다.

| 파일 |
|------|
| `NotoSerifKR-Regular.woff2` |
| `NotoSerifKR-Bold.woff2` |
| `NotoSansKR-Regular.woff2` |
| `NotoSansKR-Bold.woff2` |
| `Pretendard-Regular.woff2` |
| `Pretendard-Bold.woff2` |
| `D2Coding-Regular.woff2` |
| `NanumGothic-Regular.woff2` |
| `NanumMyeongjo-Regular.woff2` |
| `GowunBatang-Regular.woff2` |
| `GowunDodum-Regular.woff2` |

이 subset은 VS Code package 크기와 렌더링 품질 사이의 현재 타협이다. Phase A에서 폰트 위치를 옮기는
경우에도 이 subset 정책은 별도 검토 없이 확장하거나 축소하지 않는다.

## 6. 폰트 리팩터링 금지선

| 금지 항목 | 이유 |
|-----------|------|
| `web/fonts` 파일 삭제 또는 이름 변경 | studio, extension, VS Code가 파일명을 직접 참조한다. |
| 라이선스 문서 누락 | 폰트는 binary asset이라 license traceability가 필수다. |
| OS font 우선 전략 제거 | `font_fallback_strategy.md`의 기본 전략이다. |
| `font-display: swap` 제거 | 초기 렌더링 지연과 blank text 회귀를 만들 수 있다. |
| external webfont 설정 의미 변경 | `disableExternalWebFonts`는 offline/privacy 환경의 guardrail이다. |
| `NotoSansKR-ExtraLight` 등 세밀한 대체 font 임의 제거 | 글꼴 폭과 굵기 근사에 영향을 준다. |
| `/web` legacy 정리와 font 이동을 한 PR에 섞기 | runtime asset contract 변경과 dead-code cleanup이 섞이면 회귀 원인 추적이 어렵다. |

## 7. 후속 후보

| 후보 | 처리 단계 |
|------|-----------|
| `web/fonts`를 `rhwp-studio` 또는 공유 asset package로 이전 | #2125 이후 별도 계획 필요 |
| `rhwp-chrome`/`rhwp-firefox`의 전체 font copy와 VS Code subset copy 정책 정렬 | Phase A 또는 Phase B 후보 |
| font reference map 자동 검증 | smoke gate 보강 후보 |
| legacy `/web` 중 font 외 파일 제거 또는 보관 위치 이동 | #2125에서 dependency scan 이후 판단 |

## 8. Stage 4 확인

확인한 항목:

- `web/fonts` 파일 수와 크기.
- `rhwp-studio/public/fonts` symlink.
- Chrome/Firefox/Safari/VS Code의 font copy 경로.
- `npm/editor/README.md`의 self-hosting 경로 안내.
- `font-loader.ts`, `font-substitution.ts`의 fallback 정책.
- 폰트 라이선스 문서 위치.

미실행 항목:

- 렌더링 픽셀 diff.
- extension package size 비교.
- font subset 축소 실험.

미실행 항목은 #2124 Stage 4의 asset 변경 없는 기준선 범위를 넘으므로 smoke manifest의 실제 font 이동
PR gate로 분리한다. 현재 snapshot에서는 extension dist가 정확히 36개 WOFF2를 포함하는 자동 검사를
별도로 수행했다.
