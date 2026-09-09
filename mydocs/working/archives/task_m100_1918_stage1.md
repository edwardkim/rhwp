# Stage 1 완료보고서 - Task #1918

**이슈**: #1918 표 입력/삭제 시 워터마크·정적 이미지 레이어 포함 페이지가 매 입력마다 고비용 재렌더링됨
**브랜치**: `local/task1918`
**단계**: Stage 1 - 재현 계측과 fast path 경계 고정
**작성일**: 2026-07-05

---

## 1. 수행 내용

구현 변경 전 기준 계측을 다시 수행했다.

- CLI 렌더 벤치 재측정
- `samples/복학원서.hwp`의 셀 편집 명령 비용과 렌더 비용 분리 재측정
- `rhwp-studio` 브라우저 런타임에서 `window.__wasm` 메서드를 임시로 감싸 page-local text edit 1회 호출 수 계측
- fast path 적용 경계 확인

이번 단계에서 저장소 소스 코드는 수정하지 않았다. 브라우저 계측 스크립트는
`/private/tmp/rhwp_stage1_probe.mjs`에만 두었고 PR 산출물에 포함하지 않는다.

## 2. CLI 기준 벤치

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
| `253E164F57A1BC6934-empty.hwp` | 369.5KB | 2 | 3.1ms | 0.1ms | 1092.9ms | 63.8ms | 1159.9ms |
| `143E433F503322BD33.hwp` | 71.5KB | 1 | 2.6ms | 1.4ms | 208.1ms | 21.4ms | 233.4ms |
| `복학원서.hwp` | 112.0KB | 1 | 4.5ms | 1.4ms | 667.8ms | 34.9ms | 708.6ms |
| `table-001.hwp` | 22.5KB | 1 | 1.4ms | 0.6ms | 3.6ms | 14.2ms | 19.7ms |

해석:

- 일반 표 샘플은 render 3.6ms 수준이다.
- 이미지성 정적 요소가 있는 세 샘플은 render 비용이 크게 높다.
- `253E164F57A1BC6934-empty.hwp`는 표 셀 이미지 채움이 flow 내부에 있어 가장 비싸다.

## 3. 편집 명령 vs 렌더 비용 분리

명령:

```bash
/private/tmp/rhwp-lag-analysis/bench_edit samples/복학원서.hwp
```

결과:

```text
edit_only_median_ms=0.62
render_after_edit_median_ms=679.11
delete_plus_render_median_ms=678.90
```

해석:

- 표 셀 텍스트 삽입/삭제 명령 자체는 1ms 미만이다.
- 사용자가 느끼는 지연은 편집 후 페이지 재렌더링 비용이다.
- #1918의 1차 수정 지점은 문서 편집 명령이 아니라 page-local refresh 렌더 경로다.

## 4. Studio 런타임 계측

임시 계측:

- Vite dev server: `http://127.0.0.1:7700`
- 계측 스크립트: `/private/tmp/rhwp_stage1_probe.mjs`
- 방식:
  - `window.__wasm.renderPageToCanvasFiltered`
  - `window.__wasm.getPageLayerTree`
  - `window.__wasm.getPageOverlayImages`
  - `window.__eventBus.emit`
  위 메서드를 브라우저 런타임에서 감싸 호출 횟수와 시간을 수집했다.
- 각 문서는 로컬 파일 bytes를 직접 읽어 `window.__wasm.loadDocument`로 로드했다.
- 초기 렌더와 이미지 decode 재시도 타이머가 섞이지 않도록 로드 후 3.2초 대기하고 카운터를 리셋했다.
- 이후 `insertTextInCell` 1회와 `document-page-invalidated { pageIndex: 0, reason: 'text-edit' }` 1회를 발생시켰다.

결과:

| 샘플 | elapsed | `renderPageToCanvasFiltered` | layer kind | `getPageLayerTree` | `getPageOverlayImages` |
|------|---------|------------------------------|------------|--------------------|------------------------|
| `복학원서` | 671.8ms | 4회 | flow 1, background 1, behind 1, front 1 | 1회 | 0회 |
| `253E-empty` | 596.5ms | 1회 | flow 1 | 1회 | 0회 |
| `143E` | 539.1ms | 1회 | flow 1 | 1회 | 0회 |
| `table-001` | 514.3ms | 1회 | flow 1 | 1회 | 0회 |

주의:

- elapsed에는 계측 스크립트의 안정화 대기 500ms가 포함되어 있다. 절대 시간보다 호출 수와 layer kind를
  주요 지표로 본다.
- `renderPageToCanvasFiltered` 내부 시간은 `복학원서` 129.0ms, `253E-empty` 93.6ms,
  `143E` 27.3ms, `table-001` 4.8ms였다.
- 현재 `PageRenderer`는 `getPageOverlayImages`를 사용하지 않고, 매번 `getPageLayerTree`를 호출한다.

## 5. 핵심 발견

### 5.1 `복학원서.hwp`: overlay 반복 렌더 병목

`복학원서.hwp`는 text edit invalidation 1회에 다음 네 레이어를 모두 렌더한다.

- `flow`
- `background`
- `behind`
- `front`

따라서 구현계획서의 `overlay canvas 재사용`은 이 샘플에 직접 효과가 있을 가능성이 높다.

### 5.2 `253E164F57A1BC6934-empty.hwp`: flow 내부 이미지 채움 병목

`253E-empty`는 text edit invalidation 1회에 `flow`만 렌더한다. 이전 dump 기준으로 이 문서는
1x1 표 셀의 `BorderFill` 이미지 채움이 핵심 구조다.

즉 이 샘플은 behind/front overlay가 아니라 **flow 내부의 정적 이미지 채움**이 비용을 만든다.
따라서 background/behind/front overlay canvas 재사용만으로는 이 케이스를 해결할 수 없다.

후속 단계에서 다음 중 하나를 추가 검토해야 한다.

- flow 내부 이미지 채움의 변환/디코드/그리기 캐시 강화
- flow 렌더 내부에서 정적 이미지 op와 편집 텍스트 op를 분리하는 더 넓은 static plane 설계
- 최소한 BinData/effect/fill-mode 기반 이미지 변환 결과 캐시로 flow 렌더 비용을 낮추는 접근

### 5.3 `143E433F503322BD33.hwp`: flow 중심 + OLE/rawSvg 관련 케이스

이 샘플도 계측상 page-local text edit 1회에서는 `flow`만 렌더했다. OLE/rawSvg가 포함되어 있지만,
이번 edit 시나리오에서는 overlay layer로 분리되지 않았다.

따라서 이 샘플은 `복학원서`보다 `253E-empty`와 같은 "flow 내부 정적 객체 비용" 관점으로 보는 편이
안전하다. 단, OLE/rawSvg decode 재렌더 안전망 회귀 여부는 계속 검증 대상에 둔다.

## 6. fast path 경계 확정

Stage 2 이후 fast path는 다음 조건에서만 허용한다.

- 이벤트: `document-page-invalidated`
- payload: `{ pageIndex, reason: 'text-edit' }`
- `pageIndex`가 정수이고 현재 page count 범위 안에 있음
- page count가 기존 `CanvasView.pages.length`와 동일함
- Canvas2D backend 경로

다음 경우는 기존 full refresh 또는 기존 렌더 경로를 유지한다.

- `document-changed`
- page count 변화
- page index 불명확 또는 범위 밖
- header/footer/footnote 편집
- zoom/DPR/page size 변경
- CanvasKit backend
- overlay 또는 flow static signature를 계산할 수 없는 경우

## 7. 구현계획 보정 필요점

승인된 구현계획의 큰 방향은 유지하되, Stage 1 결과 때문에 다음 보정이 필요하다.

1. Stage 2의 `overlay summary 경량화`는 그대로 진행한다. `복학원서`에서 `getPageLayerTree` 1회와
   overlay 재렌더 3회를 줄이는 기반이다.
2. Stage 3의 `text edit overlay canvas 재사용`은 `복학원서` 계열을 먼저 줄이는 좁은 최적화로 둔다.
3. #1918 전체 완료 기준에는 `253E-empty` 계열도 포함되어야 하므로, Stage 4 또는 별도 보정 Stage에서
   **flow 내부 정적 이미지 채움 비용**을 다루도록 구현계획을 보완해야 한다.
4. `BorderFill` 이미지 채움은 그림 컨트롤 워터마크와 다른 경로이므로, `Picture`/overlay 전용 최적화로
   이슈를 닫지 않는다.

## 8. 검증 산출물

- CLI 벤치: 통과
- 편집/렌더 분리 벤치: 통과
- 브라우저 런타임 계측: 통과
- dev server: 계측 후 종료
- 소스 변경: 없음

## 9. 다음 단계 제안

Stage 2는 계획대로 `getPageOverlayImages` 기반 summary 경량화와 fallback 정리를 진행한다.
다만 Stage 2 착수 전에 구현계획서에 Stage 1 발견 사항을 반영해, Stage 4 범위를
`이미지 decode 재렌더 안전망 분리`에서 `flow 내부 정적 이미지 비용 완화`까지 확장하는 보정을 권장한다.

---

**승인 요청**: Stage 1 결과와 구현계획 보정 방향을 승인하면, 구현계획서를 보정한 뒤 Stage 2에 착수한다.
