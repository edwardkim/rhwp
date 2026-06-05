# 오래된 WASM 빌드로 인한 "유령 버그" (이미 수정된 버그가 studio에서 보임)

## 증상

사용자가 rhwp-studio(브라우저 캔버스)에서 레이아웃/렌더 버그를 신고하나, 현재 devel 소스의 SVG 출력(`rhwp export-svg`)·코드 분석으로는 재현되지 않음.

## 원인

`pkg/`(WASM 빌드 산출물)는 `.gitignore` 대상이라 **git 비추적**. 소스(devel)가 최신이어도 studio는 **이전에 빌드된 옛 WASM**으로 동작한다. Service Worker가 `.wasm`을 캐시(`vite.config.ts`의 `wasm-cache`)하므로 단순 새로고침으로도 갱신 안 될 수 있다.

→ 최근 커밋에서 이미 고친 버그가 studio 화면에는 그대로 남아 "유령 버그"로 신고된다.

## 진단 순서

1. `ls -la pkg/` — `rhwp_bg.wasm`이 없거나(빈 pkg) 오래됐는지 확인.
2. `rhwp export-svg <sample> -p <N>` 고해상도 렌더로 **현재 소스** 기준 재현 여부 확인.
3. SVG가 정상이면 캔버스도 정상이어야 함(렌더 트리 bbox 공유, `svg.rs`↔`web_canvas.rs` 동등). 캔버스만 다르면 그때 캔버스 경로 조사.

## 조치

```bash
# WASM 재빌드 (CLAUDE.md)
cd /home/planet/iop/rhwp
docker compose --env-file .env.docker run --rm wasm   # → pkg/ 갱신
```
- studio **하드 리로드** + Service Worker 캐시 무효화(개발자도구 Application → Service Workers → Unregister, 또는 wasm-cache 삭제).

## studio 헤드리스 재현 레시피 (Linux, 호스트 Chrome 없을 때)

```bash
# 1) Chrome 내려받기 (puppeteer 다운로더, 1회)
npx --yes @puppeteer/browsers install chrome@stable --path ~/.cache/puppeteer

# 2) vite 기동
cd rhwp-studio && npx vite --host 0.0.0.0 --port 7700 &

# 3) headless 스크립트 (e2e/helpers.mjs 활용)
export CHROME_PATH=~/.cache/puppeteer/chrome/linux-*/chrome-linux64/chrome
#   loadApp → loadHwpFile(page,'<sample>') → #scroll-container scrollTop 설정 +
#   scroll 이벤트 dispatch(가상 스크롤 갱신) → 상태표시줄 "N / M 쪽" 확인 → screenshot
```
- 페이지 N 이동: `sc.scrollTop = sc.scrollHeight*frac; sc.dispatchEvent(new Event('scroll',{bubbles:true}))`. 가상 스크롤이라 직접 scrollTop만으론 재렌더 안 됨 — scroll 이벤트 필수.
- 헤드리스 Chrome 폰트 ≠ 사용자 브라우저 폰트일 수 있음(수식 폴백 차이 주의).

## 사례

Task #1297 (3-09월_교육_통합_2022.hwpx 17쪽 [다른 풀이] 수식 겹침). 미주 겹침은 Task #1256/#1257/#1261, PR #1259/#1262에서 이미 수정됨. studio 옛 WASM이 원인. 상세: `mydocs/report/task_m100_1297_report.md`.
