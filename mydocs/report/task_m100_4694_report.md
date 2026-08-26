---
kind: report
status: active
canonical: mydocs/report/task_m100_4694_report.md
last_verified: 2026-08-13
---

# #4694 처리 결과 — 차트 숫자 데이터 편집 UI (B1-UI)

- **Issue**: [#4694](https://github.com/edwardkim/rhwp/issues/4694) ·
  부모 [#3683](https://github.com/edwardkim/rhwp/issues/3683) Track B
- **계획서**: [task_m100_4694.md](../plans/task_m100_4694.md) — 승인 후 단계대로 진행, 이탈 1건(§6)
- **브랜치**: `task4694`, 기반 `upstream/devel = f7a98ce04` (S5 시점 rebase, 충돌 0)
- **커밋**: 계획서 + S1(코어/WASM) + S2(브리지/매처) + S3(다이얼로그) + S4(배선) + S4(e2e·보정) 6건

## 1. 무엇이 됐나

**rhwp-studio 안에서 기존 차트를 선택해 숫자 값을 바꾸고, 그 편집이 화면과 저장본에
반영되며, Ctrl+Z 로 바이트 단위 원복된다.** B1 엔진(#4100)의 4층 배선이 닫혔다.

```
wasm_api export 5종 → wasm-bridge 위임 → ChartDataDialog → 커맨드/컨텍스트 메뉴/더블클릭
```

e2e 실브라우저 판정(전 단계 통과):

| 단계 | 결과 |
|---|---|
| 컨텍스트 메뉴 "차트 데이터 편집..." | 대조 성공 선택에만 노출 |
| 차트 더블클릭 | 다이얼로그 개방(한컴 UX 동형) |
| 값 수정 4.3 → 91.7 (#4055 sentinel) | 재조회 반영 + **캔버스 재렌더에서 첫 막대가 솟음** (스크린샷) |
| Ctrl+Z | **91.7 → 4.3 원복** — 스냅샷 undo 의 bin 바이트 복원 실증 (§3 R1) |
| 무편집 [확인] | 쓰기·undo 기록 없이 닫힘 (무흔적) |

증적: `rhwp-studio/e2e/screenshots/4694-{1..4}-*.png` (로컬) ·
`output/e2e/issue-4694-chart-data-edit-report.html`

## 2. 단계별 산출

| Stage | 산출 | 검증 |
|---|---|---|
| S1 | `list_charts_native()`(`object_ops/chart.rs`) + `wasm_api.rs` export 5종 (`listCharts`/`getChartData`/`setChartData`/±`ByIndex`) | `tests/issue_4694_chart_list.rs` 4건 — 열거=직렬화 동일성·wire 필드명·표 셀 container 경로(러스트 합성)·R1 핀. TDD RED→GREEN |
| S2 | 브리지 5메서드 + `chart-data-target.ts`(matchChartRef·타입) + `mutation-method-registry` 등재 | vitest 15건(본문 직속/cellPath 두 철자/headerFooter/중첩 표/오매칭 방지) + mutation-routing-guard |
| S3 | `ChartDataDialog` + `chart-data-` CSS(테마 변수) + UI 규칙 문서 표 갱신 | 배선 핀 5건(무변경 관문→dryRun→snapshot 라우팅→클로저 내 재열거→폴백) + dialog-policy-ledger |
| S4 | `insert:chart-data-edit` 커맨드·컨텍스트 메뉴·더블클릭·`chartTargetFromSelection` 정규화 | vitest 6건 + e2e 본선(§1) + MANIFEST 등재(체커 이상 없음) |

studio 스위트 총계: **873 pass / 9 fail** — 실패 9건은 stash 기준 대조로 **본 변경 없이도
동일하게 실패**함을 확정(환경 의존: cell-selection-caret-sync·embed 2건·원장·CanvasKit
replay·focused-cursor-geometry·nested-cell-backspace-merge·selection-ordering·dioxus 버전).

## 3. 착수 전 실측 — R1(스냅샷 undo)이 성립한다

계획서 §8 R1 은 "스냅샷 undo 가 `bin_data_content` 를 복원하는가"를 유일한 조기 확인
항목으로 뒀다. 코드 추적 결과 **복원한다**:

- `save_snapshot_native`(`document.rs:1702`)는 직렬화가 아니라 **`Document` 통째 clone** —
  `bin_data_content` 는 `Document` 의 소유 필드라 함께 복제된다
- 차트 편집은 슬롯 **대입**이라(`chart.rs`) 앞서 뜬 clone 이 옛 바이트를 그대로 든다
- `restore_snapshot_native` 가 `bump_bin_data_epoch` + `rebuild_derived_state`
  (page/layer 캐시 전면 clear)를 수행 — undo 후 재렌더까지 닫혀 있다

이를 `snapshot_restore_rolls_back_a_chart_edit_byte_for_byte`(코어 계약 테스트)로 고정했고
— **이런 바이트 단위 undo 복원 판정은 이번이 저장소 최초**다(기존 테스트는 epoch 변화만
봤다) — e2e Ctrl+Z 단계가 브라우저에서 재확인했다.

## 4. 구현이 지킨 계약

- **주소는 정본(by_index) 단일 경로** — studio 는 `listCharts()` 열거 → `matchChartRef`
  대조 → `ByIndex`. 매칭 실패 = 메뉴 미노출(오매칭으로 다른 차트를 고치는 것이 최악).
  index 드리프트는 쓰기 직전 operation 클로저 안 재열거·주소 재대조로 차단
- **코어 검증기가 단일 진실** — UI 검증은 UX 용 선제일 뿐, [확인] 은 dryRun 선검증을
  거치고 거부되면 닫지 않는다. 페이로드는 미변경 셀 문자열을 **원본 그대로**(4.30 보존),
  `name` 미전송(c:tx 부재 대조 함정 회피), labels 는 분산형 실변경 시에만
- **무변경 무흔적** — `changedCount 0` 이면 쓰기도 undo 스텝도 없다
- **편집 로직·캐시 무효화는 코어 재사용** — 이 작업의 재포장·무효화 코드 추가 0줄.
  `issue_4100`·`issue_2724` 계열 계약 테스트 전부 무수정 green

## 5. 과정에서 잡힌 결함·실측 3건

1. **다이얼로그 닫힘 후 Ctrl+Z 가 문서에 닿지 않았다** — e2e 가 잡았다. 포커스가
   textarea 로 복귀하지 않아서다. `afterClose → requestAnimationFrame(ih.focus())`
   (field:edit 의 onClose 복원 선례와 동형)로 보정. **e2e 없이 머지했으면 실사용
   첫 undo 가 침묵하는 결함이었다**
2. **이 CDP 환경에서 `click({clickCount:2})` 는 dblclick 을 합성하지 못한다** —
   (연쇄로도) 이벤트 자체가 발생하지 않음을 관찰 리스너로 실측. down/up 4연타
   제스처만 발생시킨다. e2e 헬퍼 주석으로 고정
3. **하드코딩 밝은 색이 다크 테마에서 겉돌았다** — 스크린샷 검수에서 발견,
   `--ui-*` 테마 변수로 전환

## 6. 계획서와의 이탈 — e2e 의 표 셀 케이스

계획서 T5 는 "표 셀 안 차트 케이스 1건 포함"을 적었다. **넣지 않았다.** 코퍼스에 셀 내
차트 픽스처가 없고, 신규 HWP/HWPX 픽스처 커밋은 `local_validation.md` §4.3.1 의 IR
sweep·overflow-cell baseline 절차를 유발한다 — #4099 §7-2 가 지목한 "검증할 실문서 없이
가지를 늘리는" 방식이기도 하다. 대신 그 경로는 **코어 열거 테스트(러스트로 표 합성)와
매처 단위테스트(cellPath 두 철자·중첩 2단)** 가 커버하고, e2e 는 본문 직속 경로를 닫았다.
셀 내 차트 실문서가 생기면 e2e 확장은 케이스 추가 한 줄이다.

## 6-1. 사전 보정 점검 — #4603 보정 패턴 기준 적대적 재검토 (작업지시자 요청)

머지 전, #4603 메인테이너 보정의 패턴(조용한 오매칭·캐시 잔존·무편집 왕복 훼손)을
기준으로 이 작업을 4갈래 실측했다. 결과와 조치:

| # | 의심 | 판정 | 조치 |
|---|---|---|---|
| R1 | 컨테이너 안 ole 선택이 맨 3좌표로 떨어져 본문 차트와 오매칭 | **실결함(중대)** — 보고서 초판의 "대조 실패 → 안전 축소" 전제가 틀렸다. ole 레이아웃 노드가 Image(#1151/#1161)와 달리 컨테이너 문맥을 방출하지 않아, 셀/글상자 차트 선택이 본문 직속과 구분 불가했고 같은 문단 앵커의 다른 차트를 조용히 열 수 있었다 | **정공법 반영**(72b7e1482) — ole RawSvg/Placeholder 방출에 cellPath 배선(표 셀 #1138 + 글상자 sentinel #1171 합류), 매처 textbox 규칙, 맨 좌표 모호성 거부. 레이아웃 방출 핀 테스트 추가. **부수 효과: 표 셀·글상자 차트 편집이 실제로 열린다** |
| R2 | snapshot no-op 시 스냅샷 예산 누수 | 기각 — null 경로가 before 즉시 반납·after 미생성·push 생략 3중 방어(`undo-noop-skip` 가드 존재). 오히려 picture-props 가 무변경에도 phantom undo 를 만드는 기존 결함 부수 발견(후속 이슈 후보) | 불요 |
| R3 | 신규 커맨드/이벤트가 걸릴 원장·가드 누락 | 갱신 필요 원장 없음(뮤테이션 원장·MANIFEST 는 기등재, 커맨드/메뉴/다이얼로그 가드는 하한·표본 방식). `insert:equation-edit` 와 구조 동형 확인 | automation 주석 개수 41→42 갱신, 비-차트 OLE(한셀) 더블클릭 미개방 음성 계약을 e2e 에 추가 |
| R4 | 실쓰기 무기록 사유 뭉갬 → "쓰기 시점 이미 같은 값"에 오안내 | 실결함(경미) | 사유 3종(재대조 실패/거부/no-op) 구분 — no-op 은 무흔적 닫힘 |

## 7. 알려진 한계 (안전 축소)

- **머리말/꼬리말 안 ole 의 레이아웃 문맥은 아직 미배선** — 그 안의 차트 선택은 맨
  3좌표로 오며, 본문 직속 차트와 좌표가 겹치면 매처가 **모호로 보고 거부**한다(오매칭
  대신 메뉴 미노출). 겹치는 비-차트 hf ole 의 이론적 잔여만 남는다 — hf 문맥 배선이
  정공법이고 `insert:picture-props` 의 같은 구멍(글상자 ole 속성이 본문 개체로 감)과
  함께 후속 이슈 후보다
- **각주/미주(noteRef) 선택**: 대상 아님(`chartTargetFromSelection` 이 null) — 동일 축소
- **3인자 wasm export**(`getChartData(sec,para,ci)` 등)는 studio 가 쓰지 않는다 —
  코어·CLI 와의 표면 대칭용(embed 소비자)
- **한컴 전면 재판정은 하지 않았다** — 쓰기 경로가 엔진 `set_chart_data_*` 와 동일
  바이트이고 #4603 에서 7종 전건 판정 완료. 대신 **선택 스팟체크 1건을 수행해 양성**:
  작업지시자가 `output/issue_4694/` 번들(e2e 동일 편집, 4.3→91.7)을 한컴 변환 PDF 로
  판정 — 편집본에서 Y축 0~100 재조정 + 첫 막대만 솟음(대조군은 원형), 오류 대화상자
  없음. studio 캔버스 재렌더와 동일 형상 (판정 기록: `output/issue_4694/README.md`)
- studio 스위트의 기존 실패 9건은 본 변경과 무관(§2) — 별도 이슈 후보

## 8. 검증

```text
cargo test --profile release-test --test issue_4694_chart_list   4 passed
cargo test --profile release-test --tests                        (전체 — §8-1 갱신)
Native Skia 3종                                                  (§8-1 갱신)
rustfmt --check (변경 .rs 3파일, LF 정규화 후)                   diff 0
cargo clippy --all-targets -- -D warnings                        (§8-1 갱신)
wasm-pack build (docker compose run --rm wasm)                   성공 — .d.ts 에 5종 반영
studio: tsc(ci-unit) 통과 · 스위트 873 pass / 9 pre-existing fail
studio e2e: issue-4694-chart-data-edit --mode=headless           전 단계 통과
e2e MANIFEST 체커                                                이상 없음
```

전체 로그: `$TMPDIR/task4694_full_test.log`

### 8-1. 최종 head 게이트 (`upstream/devel = f7a98ce04` rebase + §6-1 보정 반영 후)

```text
cargo test --profile release-test --tests     542 바이너리 / 5912 passed / 0 failed / 31 ignored
Native Skia 3종                               58 + 2 + 4 passed / 0 failed
rustfmt --check (변경 .rs, LF 정규화 후)      diff 0
cargo clippy --all-targets -- -D warnings     통과 (exit 0)
git diff --check                              통과
wasm-pack build (Docker, 최종 head 재빌드)    성공 — pkg/ 갱신
studio 스위트 (최종 head)                     875 pass / 9 pre-existing fail · tsc(ci-unit) 통과
studio e2e (새 pkg 기준, 6단계)               전 단계 통과 — 메뉴 노출·더블클릭·편집 반영·
                                              Ctrl+Z 원복·무편집 무흔적·비-차트 OLE 미개방
한컴 스팟체크                                  양성 — §4-5 판정 기록
```

- 게이트는 두 라운드 돌았다 — rebase 직후 1차(5911 passed), §6-1 R1 정공법 반영 후
  2차(5912 passed, +1 은 신규 레이아웃 방출 핀). 렌더러 방출 확장에도 기존 시각·렌더
  계열 전건 초록 — 본문 직속 ole 의 JSON 은 바이트 그대로다(context None → 빈 문자열)
- rebase 가 파일을 CRLF 로 재체크아웃해 fmt 개행 검사가 한 번 끊겼다 — 변경 .rs 를 LF
  재정규화 후 통과(메모리의 CRLF 함정 그대로. git 저장 내용은 무변)
- studio 기존 실패 9건은 stash 기준 대조로 본 변경과 무관 확정(§2)
