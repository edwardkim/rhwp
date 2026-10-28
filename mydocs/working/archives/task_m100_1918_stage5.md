# Stage 5 완료보고서 - Task #1918

**단계**: Stage 5 - 성능 검증, 회귀 확인, 보고
**이슈**: #1918 표 입력/삭제 시 워터마크·정적 이미지 레이어 포함 페이지가 매 입력마다 고비용 재렌더링됨
**브랜치**: `local/task1918`
**작성일**: 2026-07-05

---

## 1. 작업 요약

Stage 5에서는 Stage 2-4 구현 후 전체 회귀 테스트, 샘플 native bench, Studio 브라우저 E2E,
기존 probe 기반 호출 수 확인을 수행했다.

검증 중 `tests/issue_850_answer_sheet_name_hit_test.rs`의 compact JSON 길이 가드가 실패했다.
Stage 4에서 `getPageOverlayImages` summary에 `flowImageCount`, `flowRawSvgCount`를 추가하면서
JSON 길이가 128 byte가 되었기 때문이다. compact 계약의 본질은 full `PageLayerTree` JSON을
입력 루프에서 피하는 것이므로, 임계값을 256 byte로 보정하고 새 flow count 필드 존재를 함께 검증했다.

## 2. Native bench

명령:

```bash
target/debug/rhwp bench \
  samples/253E164F57A1BC6934-empty.hwp \
  samples/143E433F503322BD33.hwp \
  samples/복학원서.hwp \
  samples/table-001.hwp \
  -n 5
```

결과:

| 파일 | 크기 | 쪽 | parse | layout | render | serialize | total |
|------|------|----|-------|--------|--------|-----------|-------|
| `253E164F57A1BC6934-empty.hwp` | 369.5KB | 2 | 2.9ms | 0.2ms | 1095.3ms | 61.5ms | 1159.9ms |
| `143E433F503322BD33.hwp` | 71.5KB | 1 | 2.7ms | 1.2ms | 208.5ms | 20.8ms | 233.1ms |
| `복학원서.hwp` | 112.0KB | 1 | 4.1ms | 1.3ms | 675.0ms | 33.3ms | 713.7ms |
| `table-001.hwp` | 22.5KB | 1 | 1.4ms | 0.6ms | 3.4ms | 14.1ms | 19.5ms |

해석:

- native full render 비용은 Stage 1 기준과 같은 범위다.
- 이번 작업은 편집 후 Studio page-local refresh의 반복 렌더 비용을 줄이는 것이므로,
  native full render bench는 회귀 기준으로만 해석한다.

## 3. 브라우저 probe

명령:

```bash
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
VITE_URL="http://127.0.0.1:7700" \
node /private/tmp/rhwp_stage1_probe.mjs --mode=headless
```

결과:

| 샘플 | elapsed | `getPageOverlayImages` | `getPageLayerTree` | `renderPageToCanvasFiltered` | layer kind |
|------|---------|------------------------|--------------------|------------------------------|------------|
| `복학원서` | 612.2ms | 1회 | 1회 | 1회 | `flow` 1 |
| `253E-empty` | 606.5ms | 1회 | 1회 | 2회 | `flow-dynamic` 1, `flow` 1 |
| `143E` | 535.6ms | 1회 | 1회 | 1회 | `flow` 1 |
| `table-001` | 508.3ms | 1회 | 1회 | 1회 | `flow` 1 |

주의:

- elapsed에는 probe 안정화 대기 시간이 포함되어 있어 절대 시간으로 해석하지 않는다.
- 현재 로컬 `pkg/`는 Stage 4 Rust WASM 변경을 반영하지 않은 산출물이다.
- `253E-empty`의 `flow-dynamic` 호출은 `invalid layer_kind`로 실패했고, PageRenderer fallback에 따라
  기존 `flow` 렌더로 되돌아갔다.
- dev server 로그:

```text
[PageRenderer] flow-dynamic 렌더 미지원, 기존 flow 렌더로 fallback: invalid layer_kind: 'all' | 'background' | 'flow' | 'behind' | 'front'
```

따라서 이 probe는 호환 fallback 검증과 Stage 3 overlay 재사용 확인으로만 사용한다.
새 `flow-dynamic`/`flow-static` 성능 개선 수치는 Docker WASM 재빌드 후 다시 측정해야 한다.

## 4. 검증 결과

```bash
rustfmt tests/issue_850_answer_sheet_name_hit_test.rs
```

- 결과: 통과

```bash
cargo test issue_850_exam_social_overlay_images_api_stays_compact_for_input_loop \
  --test issue_850_answer_sheet_name_hit_test
```

- 결과: 통과
- 세부: 1 passed, 0 failed

```bash
cargo test
```

- 1차 결과: 실패
  - 실패 테스트: `issue_850_exam_social_overlay_images_api_stays_compact_for_input_loop`
  - 원인: summary 필드 추가로 compact JSON 길이가 128 byte가 되어 기존 `< 128` 가드와 충돌
- 보정 후 재실행 결과: 통과
  - lib tests: 2113 passed, 0 failed, 6 ignored
  - integration/doc tests: 최종 종료 코드 0

```bash
cd rhwp-studio
npm test
```

- 결과: 통과
- 세부: 159 passed, 0 failed

```bash
cd rhwp-studio
npm run build
```

- 결과: 통과
- 비고: 기존과 같은 Vite chunk size 경고가 출력됨

```bash
cd rhwp-studio
npm run e2e:renderer-contract
```

- 결과: 통과
- 세부: renderer backend contract guard passed

```bash
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
VITE_URL="http://127.0.0.1:7700" \
node e2e/issue-1456-chart-rerender.test.mjs --mode=headless
```

- 결과: 통과
- chart A colored ratio: 3.754% > 0.3%
- chart B colored ratio: 2.834% > 0.3%
- chart B/A diff ratio: 5.40% > 2%

## 5. Docker WASM 재빌드 제한

Stage 5에서도 Docker 기반 WASM 재빌드는 완료하지 못했다.

```bash
docker compose version
```

- 실패: Docker CLI에 compose plugin이 없음

```bash
docker-compose version
```

- 결과: Docker Compose version 5.1.3

```bash
docker info
```

- 실패: Docker daemon에 연결할 수 없음

저장소 규칙상 WASM 빌드는 Docker 경로만 사용해야 하므로 로컬 `wasm-pack`으로 우회하지 않았다.

## 6. 결론

- Stage 2-3 효과인 `복학원서.hwp` overlay 반복 렌더 제거는 현재 로컬 runtime에서도 유지된다.
  - Stage 1: `flow/background/behind/front` 4회
  - Stage 5 probe: `flow` 1회
- Stage 4의 `flow-dynamic`/`flow-static` 분리 소스와 TypeScript 정책은 테스트로 검증됐다.
- 새 WASM runtime에서 `253E-empty`, `143E` flow 내부 정적 이미지 비용이 실제로 줄어드는지는
  Docker daemon이 켜진 환경에서 WASM 재빌드 후 재측정해야 한다.
- fallback 경로는 검증됐다. 새 필터가 없는 WASM 산출물에서도 앱은 기존 `flow` 렌더로 되돌아간다.
