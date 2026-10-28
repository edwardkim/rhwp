# #7084 Stage 6 — descriptor 불일치의 제한된 자동 복구

- Issue: #7084
- 승인: 2026-09-14 작업지시자 「다음 절편 진행을 승인합니다」
- 기준: `439fb8317` (Stage 5 기록), 제품/테스트 `281fb2826`
- 범위: Studio의 Canvas2D descriptor 불일치에 한정한 1회 자동 재준비·재그리기.

## 근거와 보호 계약

Stage 5의 통제 오류 주입에서 painter는 불일치한 측정을 거부했지만 CanvasView는 재준비하지 않았다.
이는 자연 발생한 폰트 장애의 보고가 아니라 승인된 오류 복구 계약의 누락이다.
이미 검증한 Rust 측정·grapheme 보호·WASM은 변경하지 않는다.

1. 명시적인 descriptor mismatch만 복구한다. 일반 오류·CanvasKit fallback은 기존 경로를 유지한다.
2. 실패한 paint가 반환된 뒤 문서·폰트 세대·view revision·backend를 확인한다.
3. 의심되는 측정 cache를 비우고 재측정한다. 복구 자체는 외부 폰트 세대를 증가시키지 않는다.
4. 같은 문서/폰트 세대의 여러 페이지 오류를 합쳐 최대 1회만 시도한다. 성공·실패·취소 뒤
   같은 세대에서 반복하지 않는다. 실제 문서 교체/외부 폰트 변경은 새 세대다.
5. 비동기 준비 뒤 다시 유효성을 확인하고 페이지 정보·surface·caret 갱신을 기존 경로로 수행한다.
   지속 불일치와 준비 실패는 진단을 유지하고 자동 반복하지 않는다.

## 검증 계획

- 제품 recovery 클래스의 transient/persistent/중복 요청/오래된 문서·세대/일반 오류 계약 검사.
- TypeScript 및 Studio 전체 npm 테스트.
- 기존 Docker release WASM + 실제 Chrome에서 HWP/HWPX 오류 주입→자동 준비·화면 갱신 확인.
- 기존 원본 정상 동작과 미지원 묶음 경계 smoke, 필요 시 Render Diff.
- Rust/WASM bytes 불변을 확인하여 Stage 5 검증과 이번 Studio 검증의 범위를 구분한다.

원격 push·PR·GitHub 댓글은 이번 범위가 아니다.

## 결과 — 승인된 자동 복구 절편 완료

제품/테스트 커밋: `230f801136a4fc73725b847fe56c49b2c11c88bd`.
최종 검증 뒤 제품 소스는 변경하지 않았다. 계획에 남았던 CanvasView 자동 복구를 연결했다.
`CanvasMetricRecovery`가 문서/폰트 세대별 예산을 소유하며, CanvasView는 현재 view의 유효성과
기존 페이지·surface·layout 갱신을 담당한다. `WasmBridge.invalidateCanvasMetrics()`는
측정 자료만 무효화한다. 외부 폰트 변경 API만 폰트 세대를 증가시킨다.

### 실제 Chrome 통제 검사

Chrome `152.0.7977.83`, 기존 두 HWP/HWPX 원본을 각각 여섯 조건으로 실행해 **12/12 통과**했다.
측정 때는 정상 descriptor, paint 때만 다른 descriptor를 읽게 하여 실제 Rust guard가 오류를 반환하게 했다.
일반 오류만 별도의 오류 주입이며, 나머지는 실제 painter·CanvasView·WASM metric session을 실행한다.

| 조건 (각 HWP/HWPX) | 자동 재준비 | 화면 전체 갱신 | 확인 결과 |
| --- | ---: | ---: | --- |
| 일시 불일치 | 1 | 1 | 보충 측정 활성, 2쪽 surface 재부착·정상 paint |
| 지속 불일치 | 1 | 1 | 후속 실패에도 재준비 증가 없음, 대상 쪽 그리기 거부/오류 유지 |
| 폰트 준비 중 view 교체 | 1 | 0 | 오래된 guard false, 새 화면을 다시 그리지 않음 |
| 폰트 준비 중 실제 문서 재로드 | 1 | 0 | 문서 세대 변경, 오래된 결과 적용/재그리기 거부 |
| 폰트 준비 중 외부 폰트 무효화 | 1 | 0 | 폰트 세대 변경, 오래된 결과 적용/재그리기 거부 |
| 일반 렌더링 오류 | 0 | 0 | 보충 측정/복구 예산을 건드리지 않음 |

지속 오류에서는 이모지가 없는 다른 쪽은 그려질 수 있다. 이를 대상 쪽의 복구 성공으로 집계하지 않는다.
표의 갱신 1회는 시도 횟수이며, 지속 오류를 정상 렌더링으로 바꾸었다는 뜻이 아니다.
외부 폰트 무효화 검사만 의도적으로 세대를 증가시키며 자체 복구는 세대를 유지한다.

음성 대조는 소스 파일을 되돌리지 않고 브라우저 메모리에서 recovery 연결만 비활성화했다.
종료 코드 1과 22개 실패 assertion을 얻었다. 일시 오류의 준비·화면 복원 누락을 실제로 검출하며,
이를 새 제품 회귀로 분류하지 않는다. 이후 원래 연결로 최종 12개 시나리오를 재실행해 통과했다.

### 최종 게이트와 재현 경로

증적 루트: `output/7084/stage6/`.

| 검사 | 결과 | 로그/원자료 |
| --- | --- | --- |
| `node --test rhwp-studio/tests/canvas-metric-{recovery,session}.test.ts` | 15 PASS (아래 전체 npm의 부분집합) | `focused.log` |
| Docker 표준 WASM | PASS, 2분 56초 | `docker-wasm.log` |
| Studio `npx tsc --noEmit` | PASS | `typescript.log` |
| Studio `npm test` | 1,687 PASS / 2 skipped / 0 FAIL | `npm-test.log` |
| Studio `npm run build` | PASS | `production-build.log` |
| `CHROME_CDP=http://localhost:19222 npm --prefix rhwp-studio run e2e:canvas-metric-recovery` | 12개 조건 PASS | `browser-recovery-final.log`, `browser-recovery.json` |
| 위 명령에 `RHWP_METRIC_RECOVERY_NEGATIVE=1` | 예상 실패, 종료 코드 1 | `browser-recovery-negative.log/.json` |
| `node output/7084/stage6/studio-connected-check.mjs` | 원본 2건·서식 6건·portable 복원·외부 폰트 재준비 PASS | `connected/runtime.json` |
| `node output/7084/stage6/check-run-boundary.mjs` | HWP/HWPX × VS16/ZWJ 4건, 서식 분할 뒤 부분 요청/적용 0 | `run-boundary.json` |
| Studio `node e2e/canvas-render-diff.test.mjs --mode=headless` | 3/3 PASS | `render-diff.log` |
| `python3 scripts/check_e2e_manifest.py` | 132 tracked / 132 등록, PASS | `e2e-manifest.log` |
| `git diff --check` | PASS | 커밋 전후 확인 |

production build의 CanvasKit `fs/path` 외부화 및 큰 chunk 경고는 별도 기록하며 빌드 오류가 아니다.
이번 범위에서 bundle 구조를 바꾸거나 경고 상한을 완화하지 않았다.

최종 WASM은 **11,080,648 bytes**,
SHA-256 `9a7500e19b1bf9e195eb7331697aed5edfc65214b77cda38f0ef0891ba58ca77`이다.
실제 Chrome이 HTTP로 읽은 WASM bytes/hash도 일치했다(재검증 응답 304, body 확보).
Stage 5와 Rust/Cargo/build 입력 차이가 없고 새 Docker 산출물도 byte-identical하다.
Rust 전체 회귀·세 Clippy·Native Skia와 직접 PDF는 Stage 5 증적을 유지하며 이번에 다시 실행했다고 쓰지 않는다.

원본 두 파일 모두 이모지 paint 가로/세로 배율 1/1, 비활성 대조 0.364245/1을 유지했다.
64% 장평·첨자·120% 굵게/기울임의 기존 적용과 portable 내보내기 후 문맥 복원도 유지했다.
Render Diff는 KTX 0.01593%, 나머지 두 입력 0%로 기존 결과와 같다. 기준값은 바꾸지 않았다.
확인 이미지: `connected/hwp-after.png`, `connected/hwpx-after.png`.
기존 메인테이너의 원본 시각 판정을 유지하며 새 한컴 PDF를 생성하거나 재판정을 대신하지 않았다.

## 다음 순서와 제한

이번 절편의 구현/검증은 완료했다. 다음은 Stage 1~6의 결과·지원 경계·비용을 최종 보고서로
통합하고 완료 판정을 요청하는 절차다. 원격 push·PR·이슈 close는 수행하지 않았다.

- 자연 발생한 모든 OS/font 오류나 CanvasKit 복구를 검증했다는 주장이 아니다.
- 동일 세대는 성공·실패·취소 뒤에도 자동 예산을 다시 채우지 않는다. 외부 폰트 변경/문서 교체는
  새 세대로 처리한다. 지속 오류는 숨기지 않는다.
- 이번 변경은 실패 catch 경로에만 재준비를 연결한다. 정상 paint에서 추가 문자별 측정은 하지 않는다.
  새 오류 복구의 비용을 정상 화면의 비용 증가율로 환산하지 않는다. Stage 5 비용 관측과 실제 heap
  미계측 한계는 최종 보고서에도 유지한다.
- 새 폰트 배포·전체 VS16/ZWJ 조형·타 backend 보충 폭 활성화·다른 문서 결함은 추가하지 않았다.

## 용어

- descriptor: Canvas가 해석한 font 설정 문자열. 실제 폰트 파일 식별과 다르다.
- generation(세대): 문서 교체 또는 외부 폰트 변경을 구별하는 번호.
- view revision: 편집/렌더러 선택 전후 비동기 결과의 유효성을 구별하는 번호.
- transient / persistent: 한 번 발생한 뒤 사라지는 오류 / 재시도 후에도 지속되는 오류.
