# PR #2224 검토 — CanvasKit readiness gate + public opt-in 강화 (P34)

- 작성자: seo-rii (CanvasKit 시리즈 P28~P33 연속 기여자, 7건 merged)
- base: devel / head: `render-p34` / MERGEABLE, CI 12항목 전부 SUCCESS
- 범위: 프론트 전용 — rhwp-studio(렌더 백엔드/진단/캔버스 풀) + e2e 하니스 +
  render-diff workflow + renderer_baseline.py. Rust 비접촉. 검토일: 2026-07-12

## 변경 요약 (16파일, diff 2,559줄)

1. **진단 구조화**: page별 render diagnostics snapshot + `getRendererDiagnostics`
   조회, expected unsupported를 prefix 매칭 → **exact allowlist**로 (새 replay
   gap 자동 은폐 차단).
2. **opt-in 강화**: CanvasKit은 URL query 전용 — `BACKEND_STORAGE_KEY`
   (localStorage) 제거. 요청(backend/mode/surface + source)과 실효 선택값·
   initialization fallback을 분리 노출, 초기화 미완/실패를 Canvas2D로 오인하지
   않음 (`initialized:false` + null effective backend).
3. **fallback 견고화**: default surface 예외 시 software surface 복구 + 사유
   기록, 내부 fallback이 DOM canvas를 교체하면 **canvas pool 소유권 이관**.
4. **readiness gate**: 대표 corpus(paragraph/table/image) `--readiness-only`
   runtime/visual gate — Chromium build `1660786` 고정, 실패 시에도 JSON/MD
   artifact 선기록. ink 비교를 greedy → **최대 일대일 매칭**으로 (threshold
   완화 없이 알고리즘 수정 — 실측 0.008372→0.006718 사례 명기).

비전환 명시: 기본 렌더러 Canvas2D 유지, CanvasKit 기본 전환/전 코퍼스 parity
선언 없음 — 단계적 opt-in 기조 유지 (P30 계획 문서와 정합).

## 로컬 검증 (devel 22ae99cc 결합, npm 위생 절차)

| 항목 | 결과 |
|------|------|
| `npm ci` 선행 (stale 교정) | 완료 |
| `npm test` | **189 / 실패 0** |
| `npm run build` | 성공 (dist 산출) |
| `python3 -m py_compile scripts/renderer_baseline.py` | 통과 |
| render-diff workflow YAML parse | 통과 |
| e2e/readiness (Chromium 고정 빌드 필요) | CI로 갈음 — Render Diff preflight·Canvas visual diff SUCCESS, 작성자 실측 3/3 |

Rust 게이트는 비접촉이므로 CI Build & Test/Native Skia SUCCESS 로 충분.

## 검토 의견

- localStorage 백엔드 키 제거는 확장 보안 가이드(browser_extension_dev_guide)
  방향과 정합 — 외부 주입 지속화 경로 축소.
- exact allowlist 전환·최대 매칭 채택 모두 "게이트를 넓히지 않고 정확하게"
  라는 올바른 방향. threshold 완화 대신 알고리즘 수정을 택한 판단 근거가
  본문에 수치로 명시되어 있어 검증 가능.
- canvas pool 소유권 이관은 #2188/#2191에서 다진 direct replay 경로의 실사용
  안정화 — 시리즈 연속성 정합.

## 판단

**merge 권고.** 시리즈 문맥·CI 전green·로컬 검증 통과, 기본 동작(Canvas2D)
비변경으로 위험 낮음.
