# 단계별 완료 보고서 — Task M100 #2215 Stage 2

## 1. 결론

same-page와 cross-page 선택의 최소 후보 계약을 확정했다.

- anchor/focus page hint가 모두 유효하면 두 endpoint page의 inclusive 범위만 조회한다.
- 같은 페이지면 정확히 한 페이지만 조회한다.
- 서로 다른 페이지면 두 페이지 사이의 host-page fragment만 조회한다.
- 제한된 후보는 page-tree cache를 사용한다.
- hint가 없거나 유효하지 않거나 endpoint를 찾지 못하면 기존 전체 host-page 탐색으로
  fallback한다.

기존 positional API는 hint 없는 fallback 동작을 유지한다. Studio drag만 기존
`getSelectionRectsInCellEx`의 optional `startPageHint`/`endPageHint`를 사용하도록 확장하는
것이 가장 작은 호환 경계다.

## 2. 권위 page fragment 경계

`getPageLayerTree()`의 `textSources[].stableSourceKey`를 사용해 실제 셀 문단 fragment를
페이지별로 추출했다.

| 문단 | 이전 페이지 마지막 offset | 다음 페이지 첫 offset | 페이지 경계 |
|-----:|--------------------------:|----------------------:|-----------|
| 17 | 166 | 166 | p0 → p1 |
| 1277 | 78 | 78 | p55 → p56 |
| 2499 | 114 | 114 | p113 → p114 |

일반 cross-page 기준인 문단 1250→1275 선택은 실제 rect가 p54–p55에만 존재한다.

## 3. 비분할 범위 oracle

기존 전체 탐색의 rect JSON과 copy 문자열을 SHA-256으로 고정했다. HWP/HWPX 결과는 아래
대표 범위에서 byte-level hash까지 동일했다.

| 범위 | rect 수 / 페이지 | rect SHA-256 | copy SHA-256 |
|------|------------------|-------------|-------------|
| p5 0..10 | 1 / p0 | `315ad15861e6c39a382dfdb2dc4485f83ea965e75f3fddee6f9f6d88226c6f92` | `151b5a80917cce4535f99fefe973432f2f2023d5957dd357802f4f64e54e9782` |
| p1250 0..1 | 1 / p54 | `404e7c50fcfa18a487d904d32004319cb727a89bbebb5d8db85890bd4df40088` | `2c624232cdd221771294dfbb310aca000a0df6ac8b66b696d90ef06fdefb64a3` |
| p1250:0→p1275:1 | 45 / p54–55 | `baa7e14170b66cc345ca82e7acf504a24ed5604a660c59b5d31d6fbcff1d07b6` | `9d5cebfdc987867aeb8c9b985416f57298734a3f5daef90de29a6e4f6227f0e2` |

비분할 문단과 정상 cross-page 범위는 최적화 전후 raw rect JSON 및 copy 문자열을 그대로
비교할 수 있다.

## 4. split-paragraph에서 발견된 기존 candidate 오류

페이지에 걸친 동일 문단에서는 기존 전체 탐색을 rect page oracle로 사용할 수 없다.
함수는 115쪽을 모두 구축한 뒤 첫 번째로 offset을 포함하는 tree를 선택하므로, 이전 페이지
tree가 다음 fragment의 offset까지 포함할 때 잘못된 페이지 좌표를 반환한다.

| 선택 범위 | 기존 rect | 실제 endpoint cursor |
|-----------|-----------|----------------------|
| p17 166..170 | p0, x=670.9, width=516.2 | end=p1, x=154.7 |
| p1277 78..82 | p55, x=670.9, width=474.0 | end=p56, x=196.9 |
| p2499 114..118 | p113, x=670.9, width=463.8 | end=p114, x=207.1 |

세 rect 모두 페이지 오른쪽 폭을 벗어난다. HWP/HWPX의 기존 hash도 동일하므로 형식 차이가
아니다.

같은 page의 실제 pointer endpoint가 다음 fragment를 가리키는 선택은 endpoint page 한 장만
후보로 전달하면 해당 fragment의 기존 TextRun 좌표 계산을 그대로 사용할 수 있다. 따라서
same-page candidate disambiguation은 #2215에 포함한다.

다만 실제 cross-page 선택은 두 fragment page를 모두 후보로 포함하므로 현재 첫-hit/cursor
bias 규칙이 이전 fragment를 다시 고를 수 있다. page 후보 제한만으로 고쳐지는 경우에는
회귀로 고정하되, 좌표·clip 또는 cursor semantic 변경이 필요하면 별도 정확성 이슈로
분리한다.

다만 hinted page의 tree에서도 endpoint를 찾지 못해 좌표 산식·clip 의미를 바꿔야 한다면
#2215 구현을 중단하고 별도 정확성 이슈로 분리한다.

## 5. Studio가 제공할 수 있는 page hint

`DocumentPosition`은 hit test가 만든 `cursorRect?: CursorRect`를 보존한다.

- 드래그 시작: `setAnchor()`가 현재 pointer-hit position과 `cursorRect.pageIndex`를 복사한다.
- 드래그 focus: `moveTo(hit)`가 새 pointer-hit position을 보존한다.
- `getSelectionOrdered()`가 start/end를 바꿔도 각 position의 cursorRect가 함께 이동한다.
- `CursorState.updateRect()`도 WASM page와 hit-test page가 다르면 hit-test rect를 권위로
  사용하는 기존 계약이 있다.

따라서 실제 mouse drag의 정상 경로는 추가 hit test 없이
`start.cursorRect?.pageIndex`와 `end.cursorRect?.pageIndex`를 전달할 수 있다.

키보드 선택처럼 position에 cursorRect가 없는 경로는 기존 fallback을 사용한다. #2215의
직접 범위는 mouse drag이며, 키보드 선택의 별도 page-state 저장 확대는 포함하지 않는다.

## 6. 후보 페이지 계약

### 6.1 정상 hinted 경로

1. host 문단의 기존 `find_pages_for_paragraph()` 결과를 얻는다.
2. 두 hint가 page count와 host-page 집합에 속하는지 검증한다.
3. `min(startHint, endHint)..=max(startHint, endHint)` 안의 host page만 유지한다.
4. same-page면 한 장, cross-page면 endpoint 사이의 필요한 fragment만 조회한다.
5. 제한된 후보에서 cached page tree를 사용한다.

선택 방향이 역방향이어도 ordered position과 각 position의 page hint가 함께 정렬되므로 같은
계약을 사용한다.

### 6.2 fallback 경로

다음 조건에서는 기존 host-page 전체 후보로 재시도한다.

- start/end hint 중 하나가 없음
- hint가 page count 또는 host-page 집합 밖임
- hinted 범위가 비어 있음
- non-empty 선택인데 start/end endpoint 또는 rect를 찾지 못함

기존 positional API와 hint 없는 `Ex` 호출은 처음부터 fallback 경로를 사용한다. fallback도
가능하면 page-tree cache를 사용하되, 115쪽 순회가 정상 drag 경로에 재진입하지 않는지를
회귀로 고정한다.

## 7. API 호환 경계

신규 positional 인자를 추가하지 않는다. 이미 options-object 변형인
`getSelectionRectsInCellEx`에 다음 optional 키를 추가한다.

```text
startPageHint?: u32
endPageHint?: u32
```

- 키가 모두 있으면 hinted 범위를 시도한다.
- 하나라도 없으면 기존 전체 fallback과 동일하다.
- 기존 `getSelectionRectsInCell`과 기존 `Ex` 호출자는 동작이 바뀌지 않는다.
- Studio `WasmBridge`는 hints가 있을 때만 `Ex`를 호출하고, 없으면 기존 positional API를
  유지한다.

## 8. 구현계획에 반영할 테스트 계약

### Native/WASM

- HWP/HWPX의 비분할 대표 범위에서 rect JSON과 copy hash가 기존 oracle과 동일함
- same-page hinted 범위의 대상 page 수가 1임
- p54→p55 선택의 대상 page 수가 2이며 rect 45개와 copy hash가 동일함
- p1, p56, p114의 same-page split paragraph가 pointer endpoint page에 맞는 rect를 반환함
- cross-page split은 기존 정상 범위에 회귀가 없고, page 후보만으로 정정되지 않는 잔여를
  별도 정확성 후보로 기록함
- invalid/missing hint가 정확한 fallback을 수행함
- 정상 hinted drag가 fallback을 호출하지 않음
- 기존 `issue_658_text_selection_rects` 2건 유지

### Studio E2E

- 실제 mouse drag의 anchor/focus page hint 전달 확인
- 첫·중간·후반 same-page에서 focus offset과 highlight가 pointer를 추종함
- split paragraph 및 cross-page drag가 각 페이지에 visible rect를 그림
- mouseup 후 선택과 copy 문자열 유지
- 기존 drag autoscroll 회귀 유지
- 드래그 중 pagination·Canvas page refresh 0회
- 문서화된 로컬 환경에서 callback p95 < 50ms, 반복 long task 0건

UI 자동화는 앱 내 브라우저 런타임이 아니라 저장소의 기존 Playwright E2E 하니스를 사용한다.
CI에는 결정적인 후보 page 수·fallback 호출 계약을 우선 고정하고, wall-clock p95는 로컬
성능 게이트로 기록해 환경 편차로 인한 flaky CI를 피한다.

## 9. Stage 2 판정

| 항목 | 판정 |
|------|------|
| same-page 최소 후보 | endpoint page 1장 |
| cross-page 최소 후보 | 두 endpoint 사이의 host pages |
| 인접 페이지 상시 추가 | 불필요 — endpoint가 다를 때만 범위에 포함 |
| 기존 positional API 변경 | 불필요 |
| options 확장 | 기존 `getSelectionRectsInCellEx` optional hints가 최소 범위 |
| cached tree만 적용 | 불충분 — page range 제한과 함께 적용 |
| split paragraph raw 기존 rect | oracle 부적합 — pointer page/layer fragment를 권위로 사용 |
| 별도 paginator/#2308 작업 | 불필요 |
