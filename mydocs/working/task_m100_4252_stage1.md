# Stage 1 — task_m100_4252 구현·집중 검증

- **이슈**: [#4252](https://github.com/edwardkim/rhwp/issues/4252)
- **계획서**: [`mydocs/plans/task_m100_4252.md`](../plans/task_m100_4252.md)
- **브랜치**: `fix/issue-4252-nested-table-selection-path`
- **분기 기준**: `upstream/devel` `fcc3b2135`
- **작업 시각**: 2026-08-08 KST

> 이 문서는 최초 기준 `fcc3b2135`의 Stage 1 결과다. 최신 `upstream/devel` 재배치와 최종 후보
> 검증은 [Stage 2 보고서](task_m100_4252_stage2.md)를 따른다.

## 1. 원인과 RED

`layout_partial_table()`이 페이지 경계를 넘는 자식 표를 다시 조판할 때 자식 표 하나만 담은 합성
`Paragraph`를 `(para=0, control=0)`으로 만들었다. 자식 표 데이터 조회에는 필요한 좌표지만, 실제
외부 셀의 `CellContext`를 전달하지 않아 같은 합성 좌표가 PageRenderTree의 선택 메타데이터까지
노출됐다.

실제 fixture 17쪽의 중첩 `TextRun.cell_context.path`를 원본 IR에 재적용하는 래칫은 수정 전
245개 경로가 resolve되지 않아 RED였다. 물리 5쪽 `구 분` hit-test는 래퍼 표 엔트리를 잃은
다음 경로를 반환했다.

```text
parentPara=7
(control=1, cell=0, cellPara=0)
(control=0, cell=0, cellPara=0)
```

따라서 Studio의 `getTableCellBboxesByPath()`가 경로 1의 외부 셀 문단
`controls[0]`을 조회하면서 `표가 아닙니다` 오류를 냈다.

첫 경로 출처 수정 뒤 작업지시자가 다시 검증하면서 두 번째 결함층이 드러났다. RenderTree의 유효한
`TextRun.cell_context`와 깊이가 같은 traversal context가 hit-test에서 우선되어 래퍼 경로를 다시
잃었고, 전 17쪽의 TextRun/TableCell 중심 hit-test 표본 181건이 원본 IR에서 resolve되지 않았다.
또한 표만 포함하고 TextRun이 없는 부모 셀 문단은 두 번째 `Esc`의 caret anchor를 찾지 못했으며,
동일 키 이벤트에서 표 선택 렌더러가 두 번 호출됐다.

## 2. 구현

- `layout_partial_table()`과 셀 조판 함수에 `Option<&CellContext>`를 추가했다.
- 최상위 부분 표는 기존처럼 실제 본문 `para_index/control_index`로 경로를 만든다.
- 재귀 부분 표는 이미 계산된 enclosing context를 빌려 전달하고, 현재 셀의
  `cell_index/cell_para_index/text_direction`만 마지막 path entry에 반영한다.
- 캡션과 세로쓰기 셀도 동일한 실제 경로와 방향 메타데이터를 보존한다.
- hit-test는 traversal context가 원본 run context보다 **더 깊을 때만** 보완 경로로 사용한다.
  재귀 부분 표의 `TableNode`에도 합성 좌표가 아닌 실제 포함 셀 문단·현재 표 control을 기록한다.
- 표만 포함한 부모 문단의 caret는 기존 RenderTree fallback 순회 안에서 현재 표의 좌상단 또는 유효한
  자손 TextRun을 anchor로 사용한다. 사용자 입력 시점의 새 전 페이지 탐색은 추가하지 않았다.
- `Esc` 처리부의 직접 `renderTableObjectSelection()` 호출을 제거하고 기존 동기
  `table-object-selection-changed` 구독자가 한 번만 렌더링하게 했다.
- 표 셀 bbox 조회는 이미 만들어진 page tree를 `build_page_tree_cached()`로 재사용한다. 조회 범위와
  결과는 유지하면서 선택 때마다 전 페이지 레이아웃을 다시 만들던 비용을 제거했다.

수정 후 물리 5쪽의 실제 경로는 다음과 같다.

```text
section[0].paragraph[7]
  controls[1] outer table / cell[0] / paragraph[0]
  controls[2] wrapper table / cell[0] / paragraph[12]
  controls[0] child table / selected child cell
```

## 3. 래칫과 집중 검증

| 검증 | 결과 |
| --- | --- |
| 수정 전 #4252 실제 fixture 래칫 | RED, 잘못된 고유 중첩 경로 245건 |
| 수정 중 전 페이지 hit-test 래칫 | RED 181건 → GREEN 0건 |
| `issue_4252_nested_partial_table_cell_path` | PASS, 5 passed; raw·hit-test 경로 0건·부모 caret·물리 2쪽 controls[5]·55개 cell bbox |
| `issue_2007_nested_cell_pagination` | PASS, 6 passed; 17쪽·#4159 계약 포함 |
| `issue_2212_nested_cell_path_bbox` | PASS, 1 passed |
| `cargo clippy --all-targets -- -D warnings` | PASS |
| `cargo fmt --all -- --check`, `git diff --check` | PASS |
| 표준 Docker release WASM build | PASS, wasm-pack 0.15.0·wasm-opt 완료 |
| Studio TypeScript·단위 테스트 | PASS, `tsc --noEmit`·802 passed |
| `npm run e2e:issue-4252` | PASS, 실제 Esc 선택·전체 경로·외곽선 1·핸들 8·렌더 1회·부모 caret·경고 0 |
| `npm run e2e:issue-4159` | PASS, 17쪽·물리 3쪽 bottom 선 1,196/1,203 픽셀 |
| #3137 focused input performance probe | PASS, full repaint 0·long task 0·sync flush 0 |

기하를 계산하는 좌표·크기·페이지 컷 분기는 변경하지 않았으며, #2007 구조 핀과 #4159 실제
Canvas2D 픽셀 판정이 그대로 통과했다.

## 4. 브라우저 성능 판정

동일 환경의 release WASM에서 5회 로드와 9회 bbox lookup 중앙값을 비교했다. 원래 실패하던 물리
5쪽은 경로 정정 직후의 비캐시 상태도 함께 남겨 선택 hot path의 변화를 분리했다.

| 항목 | 기준 | 최종 | 판정 |
| --- | ---: | ---: | --- |
| fixture 17쪽 load | 41.7ms | 45.6ms | idle prefetch 영향을 받는 변동 지표로 단독 회귀 판정하지 않음 |
| 기존 물리 2쪽 valid bbox lookup | 5.5ms | 0.1ms | 캐시 재사용으로 단축 |
| 물리 5쪽 자식 표 bbox lookup | 원 수정 전 오류; 경로 정정 직후 26.6ms | 0.4ms, 55 cells | 정상화·캐시 재사용 |
| 표 선택 renderer | 유효 경로 정정 직후 2회·35.7ms | 1회·1.2ms | 중복 호출·재조판 제거 |

#3137 대형 셀 HWP 입력 스모크는 operation p95 1.90ms, cursor update p95 0.10ms,
render p95 2.70ms, 2rAF p95 27.40ms, full-page render 0,
long task 0, 동기 pagination flush 0으로 통과했다. 이번 변경은 문서 조판 중 경로 metadata 복제만
추가하며 입력·선택 hot path의 작업량은 줄였다.

콘솔의 `canvas-view.ts` `requestIdleCallback`과 `viewport-manager.ts`
`requestAnimationFrame` 장시간 경고는 기존 페이지 prefetch·새 visible page 렌더 경로에서 발생한다.
#4252 선택 경로에는 작업을 추가하지 않았고 해당 두 파일도 변경하지 않았다. 따라서 이 경고를
#4252가 해소했다고 주장하지 않으며, 계속 재현되면 별도 성능 이슈로 추적한다.

## 5. WASM과 시각 증적

- 최종 `pkg/rhwp_bg.wasm`: 7,643,370 bytes
- SHA-256: `21663c57767b3bca3a5ac53598568fa7f12184b7f20df1adb86114d700c25225`
- dev 서버는 이 저장소의 `pkg`를 제공한다. 이미 열린 탭은 새로고침하고 문서를 다시 열어야 새
  WASM 인스턴스를 사용한다.

`output/4252/`:

- `page5-child-table-object-selection.png` — 물리 5쪽 자식 표 Esc 선택 외곽선·핸들
- `perf-before.json`, `perf-after.json` — 수정 전과 최종 경로·반복 시간
- `perf-after-run2.json`, `perf-after-run3.json` — 최종 캐시 보정 전 중간 측정
- `issue3137-after/summary.json` — 입력 성능 probe 결과
- `../e2e/issue-4252-nested-partial-table-object-selection-report.html` — E2E 보고서

작업지시자가 최종 WASM을 적용한 rhwp-studio에서 동일 자식 표 선택·커서 동작의 시각 판정을
통과시켰다. 원격 push·PR 생성·이슈 comment·close는 수행하지 않았으며 다음 단계 승인을 기다린다.
