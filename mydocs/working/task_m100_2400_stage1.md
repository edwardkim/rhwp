# 단계별 완료 보고서 — Task M100 #2400 Stage 1

## 1. 결론

#2400은 최신 `upstream/devel@62bcae43`에서 HWP와 HWPX 모두 재현된다. `hitTest()`는
UI 114쪽의 텍스트 위치를 정확히 찾지만, 직후 표 경계 판정이 page hint 없는 첫 fragment
bbox를 사용한다. 첫 fragment 하단과 현재 클릭 y가 우연히 3.7px 차이여서 ±5px 표 외곽으로
오인되는 것이 직접 원인으로 확정됐다.

수정 범위는 page-scoped table bbox API와 Studio 클릭 판정으로 한정할 수 있다. line break,
pagination, 폰트 metric 또는 #2215 selection rect를 변경할 필요가 없다.

## 2. 환경

- 날짜: 2026-07-19
- 브랜치: `issue-2400-page-scoped-table-border-hit`
- worktree: `/private/tmp/rhwp-task2400`
- 기준: `upstream/devel@62bcae43`
- Node.js: v24.15.0
- wasm-pack: 0.15.0
- Studio: `http://127.0.0.1:7716`
- 픽스처:
  - `rhwp-studio/public/samples/issue1949_giant_cell_nested_tables_perf.hwp`
  - `samples/issue1949_giant_cell_nested_tables_perf.hwpx`

로컬 `wasm-pack build --target web --out-dir pkg`는 release WASM과 JS binding을 생성한 뒤
wasm-opt 설치 단계의 sandbox 권한 오류로 exit 1이었지만, 계측에 사용한 `pkg/rhwp_bg.wasm`과
binding은 현재 소스에서 정상 생성됐다. HWP/HWPX 로드와 API 호출이 모두 성공했다.

## 3. 재현 절차

저장소 headless Chrome 하니스로 실제 앱을 열고 다음을 수행했다.

1. HWP 또는 HWPX 115쪽 샘플을 로드한다.
2. 셀 문단 2499의 문자열에서 `어 있는 경우` 시작 offset을 API로 찾는다.
3. page hint 113의 selection rect와 `hitTest`를 이용해 첫 글자 앞의 실제 화면 좌표를 얻는다.
4. 같은 이벤트에서 legacy table bbox와 현재 page layout의 table fragment를 기록한다.
5. `page.mouse.click()`으로 실제 mousedown/up을 전달한다.
6. cursor 위치와 table object selection 상태를 확인한다.

일회성 probe와 스크린샷은 `/private/tmp/issue2400_probe.mjs`,
`/private/tmp/issue2400_before_fix.png`에 두었으며 저장소 변경에는 포함하지 않는다.

## 4. HWP/HWPX 결과

두 포맷의 결과는 동일했다.

| 항목 | 값 |
| --- | --- |
| page 수 | 115 |
| 대상 page | UI 114 / `pageIndex=113` |
| 대상 위치 | `cellParaIndex=2499`, `charOffset=77` |
| 클릭 page 좌표 | `(142.8, 1057.3)` |
| glyph rect | `(150.8, 1049.3, 16, 16)` |
| `hitTest` | sec 0 / parent 0 / control 2 / cell 2 / para 2499 / offset 77 |
| 실제 pointer 결과 | `tableObjectSelected=true` |

### bbox 대조

| 소스 | page | x | y | width | height | bottom | 클릭과 bottom 거리 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `getTableBBox()` | 0 | 75.6 | 75.6 | 597.2 | 985.4 | 1061.0 | 3.7px |
| `getPageControlLayout(113)` | 113 | 75.6 | 75.6 | 597.2 | 1000.3 | 1075.9 | 18.6px |

기존 `isTableBorderClick()`의 tolerance는 5px이다. 따라서 page 0 bbox로는 `nearBottom=true`,
현재 page 113 fragment로는 `nearBottom=false`다. 같은 입력을 실제 pointer로 전달했을 때
cursor 위치 자체는 offset 77로 계산됐지만 캐럿 표시 전에 table object selection 분기가
이를 가로챘다.

## 5. 코드 경로

```text
pointer mousedown
→ pageIdx=113, pageX/pageY 계산
→ wasm.hitTest(113, ...): 현재 셀/offset 정상
→ isTableBorderClick(pageX, pageY, sec, parentPara, control)
→ wasm.getTableBBox(sec, parentPara, control)
→ get_table_bbox_native(): page 0부터 순회, 첫 fragment 반환
→ page 113 좌표와 page 0 bbox 비교
→ enterTableObjectSelectionDirect()
```

같은 page 정보 손실은 이미 표 객체가 선택된 상태의 셀 재진입/이동 시작 판정과 hover move
cursor 판정에도 있다. 일반 클릭 한 곳만 고치면 뒤쪽 fragment의 객체 선택 상태에서 비슷한
오판이 남으므로 세 경로를 같은 page-scoped 계약으로 묶어야 한다.

## 6. 가설 판정

| 후보 | 판정 | 근거 |
| --- | --- | --- |
| `hitTest`가 다른 셀/문단을 반환 | 기각 | page 113, para 2499, offset 77 정확 |
| #2215 selection rect 오류 | 기각 | rect는 page 113 glyph 위치를 정확히 반환 |
| 첫 fragment bbox와 page-local 좌표 혼용 | 확정 | 기존 3.7px, 현재 fragment 18.6px |
| HWP parser 전용 문제 | 기각 | HWPX 결과와 수치까지 동일 |
| line break/pagination/폰트 metric | 직접 원인 아님 | 경계 판정 전 텍스트 위치가 정확 |

## 7. GitHub 반영

- 이슈 메타데이터: `bug`, assignee `postmelee`, milestone `v1.0.0`
- 확정 계측 코멘트:
  [#2400 issuecomment-5014875307](https://github.com/edwardkim/rhwp/issues/2400#issuecomment-5014875307)

## 8. 다음 단계

`mydocs/plans/task_m100_2400.md`의 승인 뒤 다음 순서로 진행한다.

1. legacy API를 보존한 page-scoped native/WASM bbox API와 RED 회귀 추가
2. Studio bridge와 일반 클릭·선택 표·hover 세 경로에 page 전달
3. pure border geometry 단위 테스트 GREEN
4. HWP/HWPX 실제 pointer와 실제 border 클릭 양방향 검증
5. focused 결과 공유 후 별도 승인으로 PR 전 전체 CI 실행
