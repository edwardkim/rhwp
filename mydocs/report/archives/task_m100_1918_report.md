# 최종 보고서 - Task #1918

**이슈**: #1918 표 입력/삭제 시 워터마크·정적 이미지 레이어 포함 페이지가 매 입력마다 고비용 재렌더링됨
**브랜치**: `local/task1918`
**작성일**: 2026-07-05

---

## 1. 문제

워터마크, 배경 이미지, OLE/RawSvg 같은 정적 이미지성 요소가 있는 문서에서 표 셀에 글자를 입력하거나
삭제하면 편집 명령 자체보다 편집 후 페이지 재렌더링이 지배적인 비용을 만들었다.

Stage 1 계측으로 두 병목을 분리했다.

| 샘플 | 병목 |
|------|------|
| `samples/복학원서.hwp` | text edit refresh마다 `flow/background/behind/front` overlay가 모두 반복 렌더링됨 |
| `samples/253E164F57A1BC6934-empty.hwp` | behind/front overlay는 없고 flow 내부 정적 이미지 채움이 반복 렌더링됨 |
| `samples/143E433F503322BD33.hwp` | flow 내부 OLE/RawSvg 계열 정적 객체 비용 |

## 2. 구현 요약

### Stage 1 - 재현 계측

- 편집 명령 비용과 렌더 비용을 분리했다.
- `복학원서.hwp`의 셀 편집 명령은 1ms 미만이지만 렌더 후처리는 약 679ms였다.
- Studio text-edit refresh 기준 호출 수를 계측했다.

### Stage 2 - overlay summary 경량화

- `PageRenderer`가 전체 `PageLayerTree`보다 먼저 `getPageOverlayImages` summary를 사용하도록 바꿨다.
- Rust summary API가 replay-plane 기준으로 `hasBehind`, `hasFront`, `imageCount`, `rawSvgCount`를 계산하도록 보정했다.
- 전체 tree fallback은 유지했다.

### Stage 3 - text edit overlay canvas 재사용

- `CanvasView`가 `reason: 'text-edit'`를 `PageRenderer`까지 전달한다.
- text-edit fast path에서 background/behind/front overlay canvas key가 같으면 재사용한다.
- 일반 refresh, zoom, resize, full document refresh, CanvasKit 경로는 기존 동작을 유지한다.

### Stage 4 - flow 내부 정적 이미지 분리

- WASM Canvas2D 필터에 `flow-dynamic`, `flow-static`을 추가했다.
- text-edit fast path에서 조건이 맞으면 동적 본문과 정적 flow 이미지/RawSvg를 분리한다.
- 기존 WASM 산출물에서 새 필터가 없으면 기존 `flow`로 fallback한다.
- image/rawSvg delayed rerender 정책이 static flow와 overlay reuse 정책을 보존하도록 분리했다.

### Stage 5 - 회귀 검증과 보고

- 전체 Rust/Studio 테스트와 E2E를 통과시켰다.
- compact overlay summary 테스트를 새 flow count 필드 포함 계약에 맞게 보정했다.
- Docker daemon 부재로 새 WASM runtime 성능 probe는 완료하지 못했고, 제한 사항으로 남겼다.

### Stage 6 - Docker WASM runtime 검증

- Colima Docker daemon을 6GiB/4CPU로 구동해 Docker 전용 WASM 빌드를 완료했다.
- 새 `pkg/` 기준으로 Studio build, renderer contract, 브라우저 probe, #1456 RawSvg/OLE E2E를 통과시켰다.
- `253E-empty`, `143E`가 실제 runtime에서 `flow-dynamic` + `flow-static` 경로를 타고,
  반복 입력 2회차부터 `flow-dynamic`만 다시 렌더링함을 확인했다.

### Stage 7 - 빠른 연속 입력 coalescing

- `CanvasView`가 `text-edit` page invalidation을 `requestAnimationFrame` 단위로 합치도록 바꿨다.
- 같은 페이지에 빠르게 들어온 여러 text-edit invalidation은 최신 상태만 한 번 렌더한다.
- `PageRenderer`가 text-edit fast path에서 page layer summary cache를 재사용하도록 바꿨다.
- `복학원서.hwp` 20회 연속 입력+무효화 probe의 key handler 평균이 63.24ms/key에서 0.12ms/key로 내려갔다.

### Stage 8 - fast path 안전성 보강

- page-local text edit을 작은 같은 셀 삽입/삭제로 더 좁혔다.
- 긴 단일 paste, 줄바꿈/탭 삽입, 큰 삭제, 편집 전후 page index 변화는 full refresh로 보낸다.
- text-edit에서 정적 overlay/static flow를 재사용한 경우 800ms idle 검증 렌더를 예약한다.
- 새 text-edit, full refresh, zoom/resize/reset, canvas release에서는 idle 검증 타이머를 취소한다.
- Stage 8 probe에서도 key handler 평균은 모든 샘플에서 1ms 미만을 유지했다.

## 3. 주요 변경 파일

| 파일 | 내용 |
|------|------|
| `rhwp-studio/src/view/canvas-view.ts` | text-edit invalidation context 전달 |
| `rhwp-studio/src/view/canvas-view.ts` | Stage 7: text-edit invalidation requestAnimationFrame coalescing |
| `rhwp-studio/src/view/canvas-view.ts` | Stage 8: static layer idle verification refresh |
| `rhwp-studio/src/view/page-renderer.ts` | overlay summary 우선 사용, overlay canvas 재사용, static flow 분리, delayed rerender policy 분리 |
| `rhwp-studio/src/view/page-renderer.ts` | Stage 7: text-edit layer summary cache |
| `rhwp-studio/src/view/page-renderer.ts` | Stage 8: static layer verification result |
| `rhwp-studio/src/engine/input-edit-invalidation.ts` | Stage 8: page-local text edit 범위 제한 |
| `rhwp-studio/src/engine/command.ts` | Stage 8: text edit payload hint |
| `rhwp-studio/src/engine/input-handler.ts` | Stage 8: text edit payload/page index hint 전달 |
| `rhwp-studio/src/core/wasm-bridge.ts` | `flow-dynamic`, `flow-static` bridge 계약 추가 |
| `src/document_core/queries/rendering.rs` | overlay summary에 flow image/rawSvg count 추가 |
| `src/renderer/web_canvas.rs` | `LayerFilter::FlowDynamic`, `LayerFilter::FlowStatic` 추가 |
| `src/wasm_api.rs` | `renderPageToCanvasFiltered` layer kind 확장 |
| `tests/issue_850_answer_sheet_name_hit_test.rs` | compact summary 계약 보정 |
| `tests/issue_938.rs` | overlay summary flow count 회귀 가드 추가 |
| `rhwp-studio/tests/render-backend.test.ts` | PageRenderer static overlay/static flow 정책 정적 계약 테스트 추가 |
| `rhwp-studio/tests/input-edit-invalidation.test.ts` | Stage 8: 긴 paste/페이지 이동/큰 삭제 full refresh 가드 |

## 4. 검증

통과한 명령:

```bash
cargo test
```

```bash
cd rhwp-studio
npm test
npm run build
npm run e2e:renderer-contract
```

```bash
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
VITE_URL="http://127.0.0.1:7700" \
node e2e/issue-1456-chart-rerender.test.mjs --mode=headless
```

```bash
target/debug/rhwp bench \
  samples/253E164F57A1BC6934-empty.hwp \
  samples/143E433F503322BD33.hwp \
  samples/복학원서.hwp \
  samples/table-001.hwp \
  -n 5
```

```bash
docker-compose --env-file .env.docker run --rm wasm
```

```bash
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
VITE_URL="http://127.0.0.1:7700" \
node /private/tmp/rhwp_stage1_probe.mjs --mode=headless
```

```bash
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
VITE_URL="http://127.0.0.1:7700" \
node /private/tmp/rhwp_stage6_probe_twice.mjs --mode=headless
```

```bash
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
node /private/tmp/rhwp_task1918_rapid_input_probe.mjs --mode=headless
```

Native bench 결과는 Stage 1과 같은 범위였고, 일반 표 샘플 `table-001.hwp`는 여전히 render 3.4ms 수준이다.

## 5. 성능 관찰

브라우저 probe 기준:

| 샘플 | Stage 1 filtered render | Stage 6 새 WASM filtered render |
|------|-------------------------|-------------------------|
| `복학원서.hwp` | `flow/background/behind/front` 4회 | `flow` 1회 |
| `253E164F57A1BC6934-empty.hwp` | `flow` 1회 | 1회차 `flow-dynamic + flow-static`, 2회차 `flow-dynamic`만 |
| `143E433F503322BD33.hwp` | `flow` 1회 | 1회차 `flow-dynamic + flow-static`, 2회차 `flow-dynamic`만 |
| `table-001.hwp` | `flow` 1회 | `flow` 1회 |

반복 입력 probe 결과:

| 샘플 | 1회차 layer kind | 2회차 layer kind | 2회차 filtered render 시간 |
|------|------------------|------------------|----------------------------|
| `253E-empty` | `flow-dynamic` 1, `flow-static` 1 | `flow-dynamic` 1 | 5.5ms |
| `143E` | `flow-dynamic` 1, `flow-static` 1 | `flow-dynamic` 1 | 10.3ms |

Stage 7 빠른 연속 입력 probe 결과:

| 샘플 | Stage 6 key 평균 | Stage 7 key 평균 | Stage 7 filtered render |
|------|------------------|------------------|--------------------------|
| `복학원서.hwp` | 63.24ms/key | 0.12ms/key | `flow` 1회 |
| `253E-empty` | 15.72ms/key | 0.04ms/key | `flow-dynamic` 1회, `flow-static` 1회 |
| `143E` | 10.69ms/key | 0.09ms/key | `flow-dynamic` 1회, `flow-static` 1회 |
| `통합재정통계(2011.10월).hwp` | 6.55ms/key | 0.30ms/key | `flow` 1회 |

Stage 7 probe에서는 20회 invalidation이 page당 1회 렌더로 합쳐졌고, text-edit 구간에서
`getPageOverlayImages`는 호출되지 않았다.

Stage 8 안전성 보강 후 빠른 연속 입력 probe 결과:

| 샘플 | Stage 7 key 평균 | Stage 8 key 평균 | Stage 8 filtered render |
|------|------------------|------------------|--------------------------|
| `복학원서.hwp` | 0.12ms/key | 0.24ms/key | `flow` 2회, `background/behind/front` 각 1회 |
| `253E-empty` | 0.04ms/key | 0.13ms/key | `flow-dynamic` 1회, `flow-static` 1회, idle `flow` 1회 |
| `143E` | 0.09ms/key | 0.24ms/key | `flow-dynamic` 1회, `flow-static` 1회, idle `flow` 1회 |
| `통합재정통계(2011.10월).hwp` | 0.30ms/key | 0.15ms/key | `flow` 1회 |

Stage 8에서는 정적 레이어 문서에 idle 검증 렌더가 추가되지만, 입력 이벤트 경로의 평균 비용은 1ms 미만이다.

## 6. Docker WASM 검증

Docker Desktop 앱은 없었지만 Colima가 설치되어 있었다.

- 초기 Colima: 2CPU, 1.913GiB memory
- 1차 WASM 빌드: `rustc`가 `SIGKILL`로 종료
- 보정: `colima start --memory 6 --cpu 4`
- 보정 후 Docker info: 4CPU, 5.772GiB memory
- 2차 WASM 빌드: 성공

`pkg/`와 `rhwp-studio/dist/`는 git ignored 산출물이므로 PR 커밋에는 포함하지 않는다.

## 7. 판단

이슈의 두 병목을 모두 처리했다.

- `복학원서.hwp` 계열: background/behind/front overlay 반복 렌더 제거
- `253E-empty`, `143E` 계열: flow 내부 정적 이미지/OLE를 `flow-static`으로 분리하고 반복 입력에서 재사용
- 빠른 연속 입력: text-edit invalidation coalescing으로 키 입력 이벤트가 매번 canvas 렌더를 기다리지 않음
- PR 전 안전성: 긴 paste/페이지 이동/큰 삭제는 full refresh로 되돌리고, 정적 레이어 재사용은 idle 검증 렌더로 회복

#1456 RawSvg/OLE 첫 로드 재렌더 E2E도 새 WASM runtime에서 통과했다.
PR 전 runtime 성능 검증까지 완료된 상태다.

## 8. 후속 이슈 후보

Stage 8에서 PR 범위의 최소 안전장치는 넣었지만, 다음 일반화 작업은 별도 이슈로 분리하는 것이 적절하다.

- 같은 셀 편집이 실제 페이지 로컬인지 layout/page signature로 판정하는 일반화
- static layer summary key에 bbox/signature를 포함해 위치 변화까지 감지하는 정밀 캐시 키
- 다양한 HWP fixture 기반 자동 성능/시각 회귀 probe 정식화
