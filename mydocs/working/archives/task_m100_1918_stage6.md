# Stage 6 완료보고서 - Task #1918

**단계**: Stage 6 - Docker WASM 재빌드 및 runtime probe
**이슈**: #1918 표 입력/삭제 시 워터마크·정적 이미지 레이어 포함 페이지가 매 입력마다 고비용 재렌더링됨
**브랜치**: `local/task1918`
**작성일**: 2026-07-05

---

## 1. 작업 요약

Stage 5에서 남았던 새 WASM runtime 검증을 완료했다.

Docker Desktop 앱은 없었지만 Colima가 설치되어 있어 Docker daemon을 Colima로 구동했다.
초기 Colima 설정은 2GiB 메모리라 `rhwp` wasm release 컴파일이 `SIGKILL`로 종료됐다.
Colima를 6GiB/4CPU로 재시작한 뒤 Docker 전용 WASM 빌드를 완료했다.

## 2. Docker / Colima 상태

```bash
colima start
```

- 결과: 성공
- 비고: Docker context가 `colima`로 전환됨

```bash
docker info
```

- 초기 상태: CPUs 2, Total Memory 1.913GiB

```bash
docker-compose --env-file .env.docker run --rm wasm
```

- 1차 결과: 실패
- 원인: `rustc` wasm release 컴파일 중 `SIGKILL`
- 판단: Colima VM 메모리 부족

```bash
colima stop
colima start --memory 6 --cpu 4
```

- 결과: 성공
- 재시작 후 상태: CPUs 4, Total Memory 5.772GiB

```bash
docker-compose --env-file .env.docker run --rm wasm
```

- 2차 결과: 성공
- 출력: `Your wasm pkg is ready to publish at /app/pkg.`

## 3. 새 WASM 산출물 확인

`pkg/`가 Docker 빌드로 갱신됐다.

- `pkg/rhwp.js`
- `pkg/rhwp.d.ts`
- `pkg/rhwp_bg.wasm`
- `pkg/rhwp_bg.wasm.d.ts`

`pkg/`와 `rhwp-studio/dist/`는 git ignored 산출물이므로 PR 커밋 대상에는 포함하지 않는다.
`rhwp-studio`는 Vite alias `@wasm -> ../pkg`를 사용하므로 dev server와 production build 모두 새 `pkg/`를 참조한다.

## 4. Studio build / contract 검증

```bash
cd rhwp-studio
npm run build
```

- 결과: 통과
- 산출 wasm: `dist/assets/rhwp_bg-BPco6KX_.wasm`
- 비고: 기존과 같은 Vite chunk size 경고가 출력됨

```bash
cd rhwp-studio
npm run e2e:renderer-contract
```

- 결과: 통과
- 출력: `renderer backend contract guard passed`

## 5. 브라우저 1회 입력 probe

명령:

```bash
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
VITE_URL="http://127.0.0.1:7700" \
node /private/tmp/rhwp_stage1_probe.mjs --mode=headless
```

결과:

| 샘플 | `getPageOverlayImages` | `getPageLayerTree` | `renderPageToCanvasFiltered` | layer kind |
|------|------------------------|--------------------|------------------------------|------------|
| `복학원서` | 1회 | 0회 | 1회 | `flow` 1 |
| `253E-empty` | 1회 | 0회 | 2회 | `flow-dynamic` 1, `flow-static` 1 |
| `143E` | 1회 | 0회 | 2회 | `flow-dynamic` 1, `flow-static` 1 |
| `table-001` | 1회 | 0회 | 1회 | `flow` 1 |

해석:

- 새 WASM runtime에서 `flow-dynamic`, `flow-static` layer kind가 fallback 없이 동작한다.
- Stage 2의 경량 summary 경로가 적용되어 `getPageLayerTree` 호출은 사라졌다.
- overlay가 없는 flow 내부 정적 이미지/OLE 문서가 static flow split 경로를 탄다.

## 6. 반복 입력 probe

명령:

```bash
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
VITE_URL="http://127.0.0.1:7700" \
node /private/tmp/rhwp_stage6_probe_twice.mjs --mode=headless
```

결과:

| 샘플 | 1회차 layer kind | 2회차 layer kind | 2회차 filtered render 시간 |
|------|------------------|------------------|----------------------------|
| `253E-empty` | `flow-dynamic` 1, `flow-static` 1 | `flow-dynamic` 1 | 5.5ms |
| `143E` | `flow-dynamic` 1, `flow-static` 1 | `flow-dynamic` 1 | 10.3ms |

해석:

- 첫 입력에서 정적 flow canvas를 생성한다.
- 두 번째 입력부터는 정적 flow canvas를 재사용하고 `flow-dynamic`만 다시 그린다.
- #1918의 반복 입력 지연 핵심인 flow 내부 정적 이미지/OLE 재렌더가 실제 runtime에서 제거된다.

## 7. RawSvg/OLE 회귀 검증

```bash
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
VITE_URL="http://127.0.0.1:7700" \
node e2e/issue-1456-chart-rerender.test.mjs --mode=headless
```

- 결과: 통과
- chart A colored ratio: 1.871% > 0.3%
- chart B colored ratio: 2.677% > 0.3%
- chart B/A diff ratio: 4.82% > 2%

## 8. 결론

Stage 5에서 남았던 새 WASM runtime 확인을 완료했다.

- `복학원서.hwp`: overlay 반복 렌더 제거 확인
- `253E164F57A1BC6934-empty.hwp`: flow 내부 정적 이미지 split 및 2회차 재사용 확인
- `143E433F503322BD33.hwp`: OLE/RawSvg 포함 flow static split 및 2회차 재사용 확인
- `table-001.hwp`: 일반 표 편집 경로는 기존 `flow` 1회 유지
- #1456 RawSvg/OLE 첫 로드 재렌더 회귀 없음

PR 전 runtime 성능 검증까지 완료된 상태다.
