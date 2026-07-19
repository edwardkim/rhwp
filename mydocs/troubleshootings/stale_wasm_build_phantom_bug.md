---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# 오래된 WASM 빌드로 인한 유령 버그

## 증상

현재 `devel` 소스의 SVG 출력(`rhwp export-svg`)이나 코드 분석으로는 재현되지 않는
레이아웃/렌더 버그가 rhwp-studio 브라우저 캔버스에서만 보인다.

이 경우 실제 코드 회귀가 아니라, rhwp-studio가 오래된 WASM 산출물이나 Service Worker
캐시를 사용하고 있을 가능성이 있다.

## 원인

`pkg/`의 WASM 빌드 산출물은 git 비추적 파일이다. 소스가 최신이어도 `pkg/rhwp_bg.wasm`
또는 `rhwp-studio/public/rhwp_bg.wasm`이 이전 빌드라면 studio는 과거 코드로 동작한다.

또한 rhwp-studio의 Service Worker가 `.wasm` 파일을 캐시할 수 있으므로, 단순 새로고침만으로
새 WASM이 적용되지 않는 경우가 있다.

## 진단 순서

1. 현재 CLI 기준으로 재현되는지 먼저 확인한다.

```bash
cargo build
target/debug/rhwp export-svg <sample> -o output/check/ -p <page>
```

2. SVG가 정상인데 studio만 비정상이면 WASM 빌드 시점과 브라우저 캐시를 확인한다.

```bash
ls -lh pkg/ rhwp-studio/public/rhwp.js rhwp-studio/public/rhwp_bg.wasm
```

3. WASM을 재빌드한다.

```bash
docker compose --env-file .env.docker run --rm wasm
```

4. 브라우저에서 hard reload를 수행하고, 필요하면 Service Worker와 Cache Storage를 지운다.

- Chrome/Edge DevTools → Application → Service Workers → Unregister
- Chrome/Edge DevTools → Application → Storage 또는 Cache Storage → rhwp 관련 cache 삭제

## 판단 기준

- SVG와 최신 WASM studio가 모두 정상이라면 기존 신고는 stale WASM으로 인한 유령 버그로 본다.
- 최신 WASM에서도 studio만 비정상이면 canvas 렌더 경로를 별도로 조사한다.
- SVG와 studio가 모두 비정상이면 layout/render 공통 경로의 실제 회귀로 본다.

## 사례

PR #1301 검토 중 `samples/3-09월_교육_통합_2022.hwpx` 17쪽 미주 수식 겹침 의심 사례에서
현재 소스의 SVG와 최신 WASM studio는 정상이고, 오래된 WASM 빌드가 원인일 가능성이 확인됐다.
