# Task M100 #2428 Stage 1 — 종료 검증 기록

## 1. 결론

2026-07-23 현재 `upstream/devel@29b5547e`에서 #2428의 완료 조건을 다시 검증했고 모두
통과했다. 각주가 없는 115쪽 거대 표의 UI 114쪽에서 HWP/HWPX 각각 12회 실제 pointer 클릭을
수행한 결과 네이티브 `hitTestFootnote` 호출은 두 포맷 모두 0회였다. 캐럿은 매번 셀 문단
2499의 기대 offset 77/78로 이동했고 표 객체 선택이나 각주 모드 오진입은 없었다.

실제 각주 문서에서는 두 포맷 모두 본문 각주 마커 클릭과 각주 영역 클릭으로 각주 편집 모드에
들어갔으며, 본문 클릭으로 다시 빠져나왔다. 따라서 fast-reject가 실제 각주 동작을 차단하지 않는다.

## 2. 통합 상태

| 항목 | 확인값 |
| --- | --- |
| 원 기여 PR | [#2471](https://github.com/edwardkim/rhwp/pull/2471), closed/unmerged |
| 누적 통합 PR | [#2521](https://github.com/edwardkim/rhwp/pull/2521), merged 2026-07-20 |
| 통합 merge commit | `625e23a3d59ebe1002ef96a6d52a99c54e4b0f73` |
| #2428 구현 commit | `0564f976c4c5d513aa52270d0408267e14bba682` |
| focused 회귀 commit | `2c785d9bb3f65116e6fda81b4b12f89ede3e8e01` |
| 최초 종료 검증 기준 | `12f8a820c82e34cbc61042df4b613532b8459a37` |
| PR 전 재검증 기준 | `29b5547e256a3d6a1f8c94c9434c14a351b5543a` |

`git branch -r --contains 0564f976...`로 `upstream/devel` 포함을 확인했다. 구현 commit은
기여자의 원 commit `58ae9c2e...`를 collaborator가 cherry-pick한 것으로 저자 정보와 원 commit
참조를 보존한다.

## 3. 환경과 재현 입력

최신 `devel` 재검증 시각은 2026-07-23 12:08 KST 전후다.

| 항목 | 값 |
| --- | --- |
| OS / 아키텍처 | macOS 26.5.2 (25F84), arm64 |
| Rust / Cargo | 1.93.1 / 1.93.1 |
| wasm-pack | 0.15.0 |
| Node.js / npm | 24.15.0 / 11.12.1 |
| Chrome | 150.0.7871.130, headless new |
| viewport | 1600×1000, DPR 1 |
| browser reported memory / concurrency | 16 GiB / 12 |

| 픽스처 | SHA-256 |
| --- | --- |
| `samples/issue1949_giant_cell_nested_tables_perf.hwp` | `ef10261cd29325116028e4f4f3e6be1a72c675eb771bddfd8484e7fe5aa94b4e` |
| `samples/issue1949_giant_cell_nested_tables_perf.hwpx` | `fc6e5f156de470dfbb14aab392389491720ee7fb1bf6f03fe9a018e93b420c65` |
| `samples/footnote-01.hwp` | `5bbc8a8fd23415aad59dd91d1eb261050946d9c64635933fcd49609ec2cc94e5` |
| `samples/hwpx/footnote-01.hwpx` | `2b59b7248af275a5fa6e108f95997ceacbec9f3bb4fc9b1a30502373a8f672cb` |
| 최초 기준 production `pkg/rhwp_bg.wasm` | `6a467a115af0170481d96aeac5b58d59627877b1210627540ca127689991fdb2` |
| 최신 기준 production `pkg/rhwp_bg.wasm` | `625a270400ffeaa1e3f2e1ae5d6f792525879ad67be3ca8321941fc23811ade9` |

## 4. 빌드와 focused 회귀

다음 검증이 통과했다.

```text
env CARGO_BUILD_JOBS=1 wasm-pack build --target web --out-dir pkg
npm --prefix rhwp-studio run build
env CARGO_TARGET_DIR=/Users/melee/Documents/projects/forks/rhwp/target \
  CARGO_BUILD_JOBS=1 cargo test --test issue_2428_footnote_fast_reject -- --nocapture
```

- production WASM build: 통과
- Studio `tsc` + Vite/PWA production build: 169 modules, 통과
- `issue_2428_footnote_fast_reject_matches_page_metadata`: 1 passed, 0 failed, 0.02s

최신 `devel`의 svg2pdf 공급원 이관도 반영해 새 의존성을 받은 뒤 같은 production build를
완료했다. `pkg/rhwp_bg.wasm` 해시가 달라진 것은 이 46커밋 누적분을 포함한 새 산출물이기
때문이며, 아래 브라우저 검증은 새 해시를 대상으로 했다.

기본 병렬 WASM build는 같은 로컬 머신의 동시 작업으로 메모리 압박을 받아 exit 137이 발생했다.
소스를 바꾸지 않고 job 수만 1로 제한한 clean worktree production build는 통과했다. 이는 코드
실패가 아니라 로컬 자원 조건이며, 최종 판정은 성공한 단일 job production 산출물로 수행했다.

## 5. 거대 표 실제 클릭 계측

### 5.1 방법

production WASM과 Studio build를 로드한 깨끗한 Chrome profile에서 실제 `page.mouse.click()`으로
`mousedown` 처리기를 통과시켰다. UI 114쪽(`pageIndex=113`) 마지막 줄의 hit range를 현재
`hitTest`로 찾고 다음 두 점을 교대했다.

| 기대 offset | page point | hit x range | 기대 cursor rect |
| ---: | --- | --- | --- |
| 77 | `(124.25, 1057.3)` | 탐색 범위 내 `90.0..158.5` | page 113, `(150.8, 1049.3, h=16)` |
| 78 | `(165.0, 1057.3)` | `159.0..171.0` | page 113, `(166.8, 1049.3, h=16)` |

각 클릭에서 전체 handler와 `pageHasFootnoteFootholds`, `hitTestFootnote`, 본문 각주 마커 hit,
주 body hit, `moveToHit`, cursor path lookup을 계측했다. 브라우저 client 좌표의 정수 양자화가
글자 경계를 넘지 않도록 range 중앙을 사용했다.

### 5.2 결과

| 포맷 | page 수 | load | 전체 handler 첫 클릭 | p50 | p95 | native `hitTestFootnote` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| HWP | 115 | 1024.6ms | 13.9ms | 2.8ms | 9.225ms | 0/12 |
| HWPX | 115 | 1051.0ms | 4.1ms | 3.25ms | 5.16ms | 0/12 |

HWP의 이슈 baseline과 같은 지표를 비교하면 다음과 같다.

| HWP 지표 | 2026-07-19 baseline | 2026-07-23 재검증 | 감소 |
| --- | ---: | ---: | ---: |
| 전체 `mousedown` p50 | 258.1ms | 2.8ms | 98.9% |
| 전체 `mousedown` p95 | 268.6ms | 9.225ms | 96.6% |
| `hitTestFootnote` 호출 | 12/12 | 0/12 | 100% 생략 |

HWPX는 이슈 본문에 독립된 변경 전 baseline이 없으므로 HWP baseline을 전용해 감소율을 만들지
않았다. 현재 절대값과 호출 생략, 정확성만 별도로 판정했다.

하위 경로 요약은 다음과 같다.

| 포맷 | page metadata | 본문 각주 마커 hit | 주 body `hitTest` | `moveToHit` | cursor path lookup |
| --- | --- | --- | --- | --- | --- |
| HWP | 12회, p50 0ms | 12회, p50 0.1ms | 24회, p50 0.8ms | 12회, p50 0ms | 0회 |
| HWPX | 12회, p50 0ms | 12회, p50 0.1ms | 24회, p50 0.9ms | 12회, p50 0ms | 0회 |

두 포맷의 24회 클릭 모두 다음 단언을 만족했다.

- `pageHasFootnoteFootholds(pageIndex=113) == false`
- cursor cell paragraph = 2499, char offset = 기대 77/78
- cursor rect page = 113
- `tableObjectSelected == false`
- `cursor.isInFootnote() == false`

비차단 관찰로, 두 거대 문서의 마지막 상태 갱신에서 `getCursorRectInCell` 경로 오류 뒤
`hitTest`의 `cursorRect`를 쓰는 기존 fallback 경고가 각각 1회 집계됐다. 최초 검증과 최신
재검증에서 동일했고 최종 rect는 위 기대값과 일치했다. `hitTestFootnote` fast-reject나 캐럿
정확성 실패는 아니므로 #2428 종료를 막지 않으며, fallback 경로 자체를 정리할 경우 #2400 계열의
별도 진단 범위로 다룬다.

## 6. 실제 각주 회귀

각주 샘플 첫 페이지는 `pageHasFootnoteFootholds=true`였다. render tree에서 찾은 본문 marker와
`FootnoteArea` 내부를 실제 pointer로 클릭하고 상태를 확인했다.

| 포맷 | 본문 marker hit | marker 클릭 후 | 각주 영역 클릭 후 | 본문 클릭 후 | 판정 |
| --- | --- | --- | --- | --- | --- |
| HWP | footnote 1, source index 0 | 각주 모드 진입 | 각주 모드 유지 | 각주 모드 해제 | PASS |
| HWPX | footnote 1, source index 0 | 각주 모드 진입 | 각주 모드 유지 | 각주 모드 해제 | PASS |

두 포맷 모두 각주 cursor rect가 page 0의 `(92.7, 1000.8, h=12)`로 확인됐다. marker click과
각주 영역 click이 기존 `hitTestFootnote`/`hitTestInFootnote` 경로를 계속 사용하므로 fast-reject의
양성 페이지 동작도 보존됐다.

## 7. PR 전 최신 devel 추가 검토

최초 검증 기준 `12f8a820` 뒤 `devel`이 46커밋 전진한 것을 확인하고
`29b5547e`로 rebase했다.

- `cursor_rect.rs`, `wasm_api.rs`, `wasm-bridge.ts`, `input-handler-mouse.ts`,
  `issue_2428_footnote_fast_reject.rs`와 4개 검증 픽스처에는 변경이 없었다.
- 다만 render/layout 통합 `88730063f`, per-page 각주 조사 `530c41cce`, svg2pdf 공급원 이관
  `0ecf68610`이 포함되어 간접 영향 가능성을 배제하지 않았다.
- focused Rust test, production WASM, Studio production build와 HWP/HWPX pointer matrix를
  새 기준에서 모두 다시 실행했다.
- 이슈 #2428은 여전히 OPEN이고 코멘트 0건, assignee 0명이며, 새 종료 검증 PR은 없었다.

따라서 46커밋 누적분은 #2428 구현 계약을 변경하지 않았고 실제 동작도 회귀하지 않았다.

## 8. 판정

#2428의 성능, 캐럿 정확성, 실제 각주 동작, HWP/HWPX, focused 회귀 조건을 모두 만족한다.
추가 코드 변경은 필요하지 않다. 종료 검증 문서 PR을 `devel`에 merge한 뒤 그 PR과 이 기록을
이슈에 연결하고 수동 close하는 것이 적절하다.
