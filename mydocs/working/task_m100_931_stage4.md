# Task #931 Stage 4 완료 보고서

## 1. 목적

`rhwp-studio` 실제 화면에서 `samples/복학원서.hwp`를 로드하고 25%까지 축소했을 때 BehindText 워터마크 overlay가 canvas 표시 배율을 따라가는지 최종 검증했다.

## 2. 검증 환경

- 브랜치: `local/task931`
- 웹 서버: `http://127.0.0.1:7700/`
- 직접 검증 URL: `http://127.0.0.1:7700/?url=/samples/%EB%B3%B5%ED%95%99%EC%9B%90%EC%84%9C.hwp&filename=%EB%B3%B5%ED%95%99%EC%9B%90%EC%84%9C.hwp`
- 문서: `samples/복학원서.hwp`
- 브라우저 검증: Codex in-app Browser
- 반복 실측: headless Chrome + `/private/tmp/rhwp-watermark-analysis/stage1-measure.mjs`
- viewport: 1280x720(in-app Browser), 1600x1000(headless Chrome)

## 3. 검증 결과

### 3.1 빌드

```bash
cd rhwp-studio
npm run build
```

결과: 통과

Vite chunk size warning은 기존 WASM/번들 크기 경고이며 이번 변경과 무관하다.

### 3.2 실제 브라우저 조작 검증

in-app Browser에서 직접 검증 URL을 열고 상태 표시줄의 줌 아웃 버튼을 눌러 `100% → 90% → 80% → 70% → 60% → 50% → 40% → 30% → 25%`까지 이동했다.

25% 상태의 DOM 실측:

| 대상 | 측정값 |
|------|--------|
| canvas rect | `198 × 280.5` |
| behind overlay layer rect | `198 × 280.5` |
| 워터마크 img rect | `123.7578 × 123.9297` |
| 워터마크 left/top | `34.4267 / 67.56` |
| overlay overflow | `hidden` |
| console error/warn | 없음 |

결론:

- 뒤쪽 overlay layer가 canvas와 같은 표시 크기를 유지한다.
- 워터마크 이미지 bbox가 25% 배율로 축소된다.
- 회색 작업 영역에 원본 크기 워터마크가 노출되지 않는다.

### 3.3 반복 실측 스크립트

```bash
cd rhwp-studio
node /private/tmp/rhwp-watermark-analysis/stage1-measure.mjs
```

sandbox 내부에서는 Chrome 프로세스가 실행되지 않아 승인된 실행 경로로 재실행했다.

반복 실측 결과:

| 줌 | canvas rect | behind overlay rect | 워터마크 rect |
|----|-------------|---------------------|---------------|
| 100% | `793 × 1122` | `793 × 1122` | `495.03 × 495.72` |
| 85% | `674 × 954` | `674 × 954` | `420.78 × 421.36` |
| 25% | `198 × 280` | `198 × 280` | `123.75 × 123.92` |

스크린샷:

- `/private/tmp/rhwp-watermark-analysis/stage1_zoom_100.png`
- `/private/tmp/rhwp-watermark-analysis/stage1_zoom_85.png`
- `/private/tmp/rhwp-watermark-analysis/stage1_zoom_25.png`

반복 실측 스크립트의 console에는 `Failed to load resource: net::ERR_FAILED` 2건이 남았다. 이는 스크립트가 외부 CDN 폰트 요청을 의도적으로 차단한 결과이며, in-app Browser 검증에서는 앱 관련 error/warn이 없었다.

## 4. 직접 검증용 서버

요청에 따라 rhwp-studio dev server를 실행한 상태로 유지했다.

```bash
cd rhwp-studio
npm run dev -- --host 127.0.0.1 --port 7700
```

사용자는 아래 URL로 동일 문서를 바로 열 수 있다.

```text
http://127.0.0.1:7700/?url=/samples/%EB%B3%B5%ED%95%99%EC%9B%90%EC%84%9C.hwp&filename=%EB%B3%B5%ED%95%99%EC%9B%90%EC%84%9C.hwp
```

## 5. Stage 4 결론

Stage 4 목표를 충족했다.

- `npm run build`가 통과했다.
- 실제 브라우저에서 25% 줌까지 축소해도 BehindText 워터마크가 canvas 배율을 따라간다.
- 지연 재렌더 이후에도 overlay layer와 이미지 bbox가 정상 유지된다.
- 직접 검증용 웹 서버를 실행 중이다.

## 6. 승인 요청

최종 보고서 검토 및 Task #931 완료 승인 요청.
