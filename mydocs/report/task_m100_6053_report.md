---
kind: report
status: active
canonical: mydocs/report/task_m100_6053_report.md
last_verified: 2026-08-26
---

# #6053 처리 결과 — B2-UI: 차트 행·열·라벨 구조 편집

- **Issue**: [#6053](https://github.com/edwardkim/rhwp/issues/6053) · 부모 [#3683](https://github.com/edwardkim/rhwp/issues/3683) Track B
- **계획서**: [task_m100_6053.md](../plans/archives/task_m100_6053.md)
- **브랜치**: `task6053`, 기반 `upstream/devel = 70ebacc4c`
- **커밋**: S1(모델·페이로드) · S2(로컬 메뉴) · S3(다이얼로그) · S4(e2e) · S5(문서) ·
  S6(주식형 렌더러 결함 — 수동 테스트 후속) 6건

## 결론 한 줄

**rhwp-studio 에서 차트 그리드 셀을 우클릭해 행·계열을 넣고 지우며 계열명과 카테고리 라벨을
고칠 수 있고, 그 편집이 화면과 저장본에 반영되며 Ctrl+Z 로 원복된다.** 편집 축은 Rust 변경
0줄이다. 다만 수동 테스트가 **주식형 렌더러의 오래된 결함**을 드러내 그것도 함께 닫았다(§6-2).

## 1. 무엇이 됐나 — 실브라우저 실측

```
GridModel(불변) → chart-data-target(페이로드) → wasm-bridge(JSON 패스스루) → 코어 structure
```

`npm run e2e:issue-6053` (headless) 전 단계 통과:

| 수용 기준 | 실측 |
|---|---|
| ① 우클릭 → 행 추가 → [확인] → 저장본 반영 | 메뉴 6종 노출 · 그리드 행 4→5 · 재조회 행 4→5, 라벨 4→5 · 손대지 않은 첫 값 보존 |
| ② 계열명·라벨 편집, `c:tx` 없는 계열·다층·비공유는 미개방 | 다이얼로그 렌더 실측(`.chart-data-series-locked` 분기) |
| ③ Ctrl+Z 원복 | 행 5→4, 계열 수 유지 |
| ④ 무편집 [확인] 무흔적 | 행·첫 값 불변 |
| ⑤ ESC 는 메뉴만 닫는다 | 메뉴 0항목 · 다이얼로그 유지 · 두 번째 ESC 에 다이얼로그 닫힘 |
| ⑥ 원형 안내 / 주식형 양끝 비활성 | `plot=pie` → 계열 추가 **활성 + 안내** · `plot=stock, hasUpDownBars=true` → 첫·끝 계열의 삭제와 바깥 삽입 비활성, **중간 삽입은 활성** |
| ⑦ studio 가드 + TS 검사 + B1 e2e 회귀 0 | 아래 §5 |

증적: `rhwp-studio/e2e/screenshots/6053-{1..6}-*.png`,
`output/e2e/chart-data-structure-issue6053-report.html`.

## 2. 착수 시점 실측이 이슈를 바로잡은 것 3건

이슈는 로컬 `devel` 이 110 커밋 뒤처진 시점에 쓰였다.

**① #6037 은 이미 devel 에 있었다.** PR #6052 자체는 unmerged close 지만 통합 PR #6072
(merge `ee7e8a6ed`)로 반영됐다. `chart.rs:237-239` 가 `plot`·`hasUpDownBars` 를 방출하고
`:580` 에 `candleAnchorBroken` 이 서 있으며 옛 가드 둘은 제거됐다. → 이슈가 S4 로 미뤄 둔
사전 판정을 **S3 에 합쳐 한 번에 붙였다.**

**② `structure: true` 는 '중간 삽입'을 모른다.** `plan_edits`(`chart.rs:794-932`)는 겹치는 칸
제자리 치환 + **꼬리 증감**만 한다. 중간 삽입·삭제는 아래·오른쪽 칸 전부를 제자리 치환
대상으로 만들고, 사고가 셋이다 — 빈 값(`valueNotPatchable`), 계열 이름 밀림
(`seriesNameNotPatchable`), 신설 계열 이름 필수(`seriesNameRequired`). 이슈의 R1 보다 넓다.

→ **띄우고 dryRun 이 판정하게 했다.** 사전 비활성은 UI 가 코어 규칙을 재현하게 만들고, 과잉
차단이 미달 차단보다 나쁘다. 다만 `seriesNameRequired` 만은 UI 가 예방한다 — 신설 계열에
템플릿 유무에 맞춰 기본 이름을 채우거나(있으면) 비운다(없으면).

**③ 좌표 둘이 어긋났다.** `chart-data-target.ts` 는 `src/ui/` 가 아니라 `src/core/`(줄은 정확).
`is_number` 는 `patch.rs:271` 이 아니라 `chart.rs:302-308` 의 비공개 `fn` — `is_safe_text` 만
`patch.rs:268-275` 에 `pub` 이다.

## 3. 이슈에 없던 실측 2건

**Enter 도 가로채야 한다.** `ModalDialog` 는 ESC 뿐 아니라 **비-입력 요소의 Enter 를
`.dialog-btn-primary` 클릭으로** 바꾼다(`dialog.ts:105-110`). 메뉴 항목은 `<div>` 라, 막지 않으면
메뉴에서 누른 Enter 가 다이얼로그를 확인해 버린다. `window` capture 에서 ESC·Enter 둘 다
전파를 끊는다.

**`pkg/` 스테일은 조용히 무력화된다.** `pkg/` 는 gitignore 된 빌드 산출물이고 봉투 JSON 은
`.wasm` 안 Rust 가 런타임에 조립한다 — 재빌드 전에는 `plot`/`hasUpDownBars` 가 `undefined` 다.
기존 `scripts/frontend-wasm-bindings.test.mjs` 는 export **이름만** 대조해 이 스테일을 잡지
못한다. 그래서 **e2e 가 두 필드의 부재를 즉시 실패시킨다.**

## 4. 단계별 산출

| Stage | 산출 | 검증 |
|---|---|---|
| S0 | devel FF(`385e93b2c→70ebacc4c`) · worktree · Node 22.15 · Docker WASM 재빌드 | 새 `rhwp_bg.wasm` 에 `hasUpDownBars`·`candleAnchorBroken`·`ofPie` 존재, `pieSeriesCountFixed` 부재 |
| S1 | `core/chart-grid-model.ts` 신설 + `core/chart-data-target.ts` additive 확장 | 기존 `chart-data-target.test.ts` **23건 무수정 통과** + 신규 21건 |
| S2 | `ui/local-context-menu.ts` 신설 | 소스 가드 4건 |
| S3 | 다이얼로그 재작성 + 사전 판정 + CSS 2종 | 배선 계약 12건 · 뮤테이션 원장 **3 유지** · `dialog-policy-ledger` green |
| S4 | e2e + MANIFEST + npm script + B1 e2e 환경 복구 | 실브라우저 전 단계 통과 |
| S5 | 계획서·보고서 | — |
| S6 | **(사후)** `render_stock` 역할 일반화 + 회귀 2종 | crate lib 165 · stock 통합 5 (기존 3종 무수정) · SVG 실측 |

## 5. 검증 실측

렌더러를 건드리므로 `local_validation.md` §4.3 의 **renderer/layout/typeset/WASM** 레인이다.

```
# Rust (렌더 레인)
cargo nextest run --tests --no-fail-fast        8358 run / 8357 passed / 43 skipped / 1 failed*
cargo clippy --all-targets -- -D warnings       exit 0
cargo fmt --all -- --check                      exit 0
git diff --check                                exit 0
node scripts/rust-test-suite-manifest.mjs --check [--base-ref]   exit 0 (양쪽)
node scripts/rust-unit-test-tiers.mjs --check [--base-ref]       exit 0 — 4221 tests, base 대비 불변
focused: issue_2277_stock                       5 passed (기존 3종 무수정 + 신규 2종)
focused: crate lib (rhwp-ooxml-chart)           165 passed

# Native Skia 3종
--features native-skia --lib                    passed
issue_2225_missing_picture_placeholder          2 passed
render_p37_direct_pdf_export                    4 passed

# WASM · studio
docker compose run --rm wasm                    성공 — pkg/ 갱신
npx tsc --noEmit / tsc(ci-unit)                 exit 0
npm test (Node 22.15.0)                         1168 / 1167 passed / 1 failed**
npm run e2e:issue-6053                          전 단계 통과
e2e issue-4694-chart-data-edit (B1 회귀)        전 단계 통과
python scripts/check_e2e_manifest.py            오류 3건 (선행 부채 — 아래)
```

**\* release-test 의 실패 1건은 이 변경과 무관한 Windows 줄끝 문제다.**
`wmf_emf_goldens_lock_current_engine` — `tests/fixtures/m09x_wmf_emf/*.golden` 의 저장 blob 은
LF 인데 `core.autocrlf=true` 가 워킹트리를 CRLF 로 바꾼다(`.gitattributes` 에 `*.golden` 핀이
없다). 골든 9개를 LF 로 정규화하니(내용 diff 0) **통과**했다. PR #6052 본문이 같은 실패를 같은
원인으로 기록했고, CI(Linux)는 영향받지 않는다. 이 PR 은 WMF/EMF 를 한 줄도 건드리지 않는다.

**\*\* studio 실패 1건**은 착수 기준선과 같은 선행 실패다(아래).

**`cargo fmt` 주석** — 워킹트리의 `.rs` 1947개가 CRLF 라 rustfmt(`newline_style = Unix`)가
전건 실패한다. LF 로 정규화(저장 blob 이 LF 라 **내용 diff 0**)한 뒤 검사했다. 또 새 워크트리에는
gitignore 된 `tests/generated/*.rs` 가 없어 fmt 가 파일 부재로 먼저 죽는다 —
`node scripts/rust-test-suite-manifest.mjs --prepare` 로 준비한 뒤 검사했다(파생 산출물과
`Cargo.toml`·`tests/suites/manifest.json` 은 커밋하지 않았다).

**시각 증적** — 수정 후 `rhwp export-svg` 실측:

| 문서 | 고저선 | 캔들 | 마커 |
|---|---|---|---|
| `issue6037/engine/시가고가저가종가-중간계열추가`(5계열) | 4 | 4 | 8 |

수정 전에는 이 문서가 꺾은선 5개였다(`hwp-stock-*` 0건). 산출을 144DPI 로 래스터해
`pdf/issue6037/engine/시가고가저가종가-중간계열추가-hwpx.pdf` 와 눈으로 대조했다 — 1월 검은
캔들(44→32 하락), 나머지 흰 캔들, 고저선이 추가계열 값 11까지, 추가계열 마커. **일치.**

### 선행 실패 — 착수 시점 기준선과 동일 (회귀 0)

착수 직후 clean `upstream/devel` 에서 기준선을 먼저 찍었다. 남은 실패는 그 기준선과 같다.

| 실패 | 성격 |
|---|---|
| studio `subsecond-runtime` — `Cargo.lock` 에서 `subsecond` 미검출 | 선행. #4694 보고서의 pre-existing 목록에 있다 |
| studio `style-undo-routing` — 스타일 WASM 반환 계약 | 선행 **간헐**. 단독 3/3 통과, 전체 스위트에서 초기 2회 실패 후 이후 통과 — 순서·부하 의존 플레이크 |
| Rust `wmf_emf_goldens_lock_current_engine` | 선행 **Windows 전용 줄끝**. §5 각주 참조 — LF 정규화 후 통과, 내용 diff 0 |

`check_e2e_manifest.py` 의 3건(`loading-busy-cursor`·`status-page-number`·`toolbox-visibility`
미등재)도 **clean devel 에서 동일하게 재현**했다. 이 변경과 무관한 선행 부채다.

### 기준선 측정 중 드러난 로컬 게이트 결함 2건

착수 기준선은 처음 **18건** 실패로 나왔는데, 그중 16건이 게이트 자체의 결함이었다.

**① CRLF 가 소스 가드를 조용히 무력화한다(4건).** 저장 blob 은 LF 인데 `autocrlf=true` 가
워킹트리를 CRLF 로 바꾸면 `dialog-policy-ledger` 의 블록 분해 정규식 `\n {2}\{\n` 이 전부
빗나가 **원장이 36건에서 파일당 1건씩 8건으로 무너진다**(임계값 30 미달 → 실패). embed·CanvasKit
가드도 같이 죽었다. CI(Linux/LF)에서는 통과하므로 로컬에서만 어긋난다. 워킹트리만 LF 로
정규화해 해소했다(설정 무변경, 내용 diff 0).

**② 새 워크트리의 `npm ci` 가 rolldown 네이티브 바인딩을 빠뜨린다(12건).** npm optional-deps
버그로 `@rolldown/binding-win32-x64-msvc` 가 설치되지 않아 시험 파일 8개가 통째로 죽었고
**36건이 아예 실행되지 않고 있었다**(1089 → 1125). `npm i …@1.2.5 --no-save` 로 채웠다
(잠금 파일 무변경).

남은 6건 중 4건(행위 드라이버)은 Node **v22.14.0** 에 `module.registerHooks`(22.15+)가 없어서였다.
22.15.0 설치로 해소해 최종 기준선이 2건이 됐다.

## 6. 계획서와의 이탈 1건

**B1 e2e(#4694)에 손을 댔다.** 계획서는 "B1 e2e 무수정 통과"를 수용 기준으로 뒀는데, 그 파일은
새 브라우저 프로필에서 **1단계부터 죽고 있었다** — 첫 실행 스킨 선택 대화상자가 편집 영역을
덮어 캔버스 클릭이 `.skin-onboarding-card` 에 먹혔다. 이 파일은 온보딩 도입 이전에 작성됐고
MANIFEST 배선이 `수동`이라 CI 가 잡지 못한 선행 부채다.

**단언은 한 줄도 바꾸지 않았다.** 최신 e2e(`undo-depth-issue5769`)와 같은
`dismissSkinOnboarding` 헬퍼만 넣었고, 넣자 6단계 전건 통과한다. 내 변경이 원인이 아님은
변경 파일 목록으로 확인된다 — B1 1단계 경로(`input-handler.ts`·`input-handler-mouse.ts`·
`ui/context-menu.ts`·`command/commands/insert.ts`·`main.ts`)는 **전부 무변경**이다.

## 6-2. 수동 테스트가 드러낸 렌더러 결함 — 주식형 계열 역할

작업지시자가 `시가고가저가종가`에 계열을 추가하니 **라인형으로 렌더링된다**고 보고했다.

**문서는 정상이었다.** 편집 후 봉투는 `plot=stock`·`hasUpDownBars=true`·양끝(시가·종가) 유지·
5계열로 의도한 그대로다. 어긋난 것은 rhwp 렌더러다.

### 틀린 술어

`crates/rhwp-ooxml-chart/src/renderer.rs` `render_stock` 이 계열 역할을 **위치로 고정**했다.

```rust
let (hi_i, lo_i, close_i, open_i) = match chart.series.len() {
    3 => (0usize, 1usize, 2usize, None),      // 고·저·종
    4 => (1, 2, 3, Some(0usize)),             // 시·고·저·종
    _ => return render_line(svg, chart, px, py, pw, ph),
};
```

### 반증 — 전부 저장소에 이미 있었다

| 자산 | 무엇 | 한컴 렌더 |
|---|---|---|
| `pdf/issue6037/engine/시가고가저가종가-중간계열추가-hwpx.pdf` | **rhwp 엔진**이 만든 5계열 | 캔들 유지 · 추가계열은 마커 |
| `samples/issue6037/MANIFEST.json` `finding.first_end` | 4→3 삭제, `upDownBars` 잔존 | *"HLC 구성인데도 upDownBars 가 남아 캔들이 그려진다"* |
| `samples/issue6037/고가저가종가-꼬리계열추가.hwpx` | 4계열 HLC | 옛 `(hi=1, lo=2)` 가 저가↔종가를 집던 사례 — **계열 수 3·4 안에서도 틀렸다** |

작업지시자가 회수한 한컴 편집기 산출(5계열, 새 계열 값 5)에서는 고저선이 **5까지** 내려와
「전 계열 최소」임을 직접 보인다. 그 파일은 커밋하지 않았다 — 위 세 자산이 같은 결론을
독립적으로 받쳐 주고, `samples/chart/**` 에 넣으면 전수 baseline 게이트 3종이 발동한다.

즉 **#6037 원장과 렌더러가 서로 모순인 상태**였고 아무도 잡고 있지 않았다.

### 참인 규칙과 수정

OOXML 본래 의미다 — **고저선 = 카테고리별 전 계열 최소↔최대**, **캔들 = 첫 계열↔끝 계열**
(`upDownBars` 가 있으면 계열 수와 무관). 위치 `match` 와 `render_line` 폴백을 걷어냈다.

변경면은 함수 머리 한 곳뿐이다. 마커 루프는 이미 전 계열을 돌고 있었고(`marker_symbol` 이
`Auto|Named` 인 계열마다), 축도 `raw_value_bounds(chart.series.iter())` 라 전 계열 기준이었다.

**기존 렌더는 바뀌지 않는다.** 코퍼스 두 픽스처의 전 카테고리에서 두 규칙이 같은 값을 낸다:

| 픽스처 | cat0 | cat1 | cat2 | cat3 |
|---|---|---|---|---|
| HLC `고가저가종가` | (11,55) | (12,57) | (13,57) | (21,59) |
| OHLC `시가고가저가종가` | (11,55) | (12,57) | (13,57) | (21,59) |

기존 stock 시험 6종이 **무수정 통과**한다. 폴백을 단언하던 1건
(`test_stock_unusual_series_count_line_fallback`)만 전제가 반증돼 같은 자리에서 다시 썼다 —
`rust-unit-test-tiers` 가 source-side 시험 **증가**를 금지하므로 개수는 그대로다.

### 회귀 시험 — 신규 픽스처 0건

`tests/issue_2277_stock.rs` 에 2종을 더했다(이미 커밋된 자산만 사용).

- `stock_five_series_keeps_candles_and_hilow` — 5계열이 고저선 4 · 캔들 4 · 1월 하락(`#404040`) ·
  마커 8(종가 4 + 추가계열 4)
- `stock_hilow_spans_all_series_not_positional_pair` — 4계열 HLC 에서 카테고리별 고저선 길이
  비율이 `[44,45,44,47]`(전 계열 최소↔최대)이지 `[21,23,21,14]`(옛 저가↔종가)가 아님

두 번째는 축·플롯 기하에 기대지 않는다 — 값축이 선형이므로 **길이의 비율**만으로 두 규칙이
갈린다.

### UI 는 아무것도 막지 않는다

처음에는 UI 에서 주식형 계열 수를 3·4로 묶어 막았다. **그 판단은 틀렸고 되돌렸다** — 한컴이
정상 처리하고 문서도 멀쩡한 편집을, 우리 렌더러가 부족하다는 이유로 막는 것은 과잉 차단이다.
렌더러를 고친 지금은 안내할 차이조차 없다. 주식형 사전 비활성은 **캔들 양끝 가드
(`candleAnchorBroken`, 편집 축)만** 남는다.

### 남은 차이 1건

5계열 산출에서 종가 마커 글리프가 한컴(×)과 다르다(rhwp ◆). 마커 글리프는 계열 인덱스
사이클(`si % 4`)의 근사이고 이번 변경이 건드리지 않은 기존 축이다 — 4계열 코퍼스의
`test_stock_close_marker_only`(×)는 그대로 통과한다.

## 6-3. 이어진 보고 — "추가한 계열에 선이 안 그려진다"

작업지시자가 §6-2 수정 뒤에도 **추가한 계열 자체에는 선이 없다**고 보고했다. 파고들어 보니
축이 둘로 갈렸고, 하나만 진짜 결함이었다.

**진짜 결함(고쳤다) — 렌더러가 계열 선을 아예 안 그렸다.** `render_stock` 은 고저선·캔들·마커만
그리고 계열 선을 한 줄도 그리지 않았다. 그래서 **한컴이 만든** 5계열
(`계열 5` = `c:spPr` 없음 = 기본 선 스타일)을 rhwp 로 렌더해도 선이 안 나왔다. 모델에 선 표시
필드가 없고 파서에 `a:ln > a:noFill` 처리도 없던 것이 원인이다.

수정: `OoxmlSeries.line_none` 신설 + 파서가 계열 최상위 `spPr > ln > noFill` 을 읽고 +
`render_stock` 이 표기 없는 계열만 선으로 그린다. 실측:

| 문서 | 계열 선 |
|---|---|
| 한컴이 만든 5계열 (`계열 5` = spPr 없음) | **1** (수정 전 0) |
| rhwp 가 만든 5계열 (전건 noFill) | 0 |
| OHLC·HLC 코퍼스 원본 | 0 (불변) |

**결함이 아닌 것 — rhwp 가 추가한 계열에 선이 없는 것.** 엔진이 마지막 계열을 통째로 복제하므로
새 계열이 템플릿의 `a:ln > a:noFill` 을 물려받는다. 한컴도 **그 문서를 같게 그린다** —
`pdf/issue6037/engine/시가고가저가종가-중간계열추가-hwpx.pdf` 가 추가계열을 마커로만 그리고,
원장이 "추가계열이 마커로 붙음"으로 판정해 두었다. 즉 파일과 그림이 서로 맞다.

**복제 시 `a:ln` 을 들어내는 수정을 시도했다가 되돌렸다.** `structure` 는 **위치 기반**이라
"새 계열"이 항상 꼬리다. 중간 삽입이면 꼬리에 붙는 것은 밀려난 **기존 종가**이고, 사용자가 넣은
계열은 중간 위치에서 옛 계열의 스타일을 물려받는다. 실측으로 확인했다 — 선을 얻은 계열이
`종가`였다. **엉뚱한 계열에 선을 주는 수정**이라 채택하지 않았다.

주식형은 캔들 양끝 가드 때문에 꼬리 삽입이 막혀 있어 **허용되는 추가가 전부 중간 삽입**이다.
따라서 이 경로로는 올바른 계열을 고를 수 없다.

올바르게 하려면 엔진이 **어느 계열이 새것인지** 알아야 하는데, `structure` 계약은 그것을 일부러
모델링하지 않는다(목표 행렬만 받는다). UI 는 알고 있으므로(`GridModel` 의 `source: null`)
페이로드에 그 표지를 더하는 확장이 필요하다 — 3면 계약(코어 JSON·CLI CSV·MCP)을 넓히는 일이고,
CSV 는 행렬뿐이라 표현할 수도 없다. **별건으로 접수한다**(§8-6). → **§6-4 에서 재정정 — 표지
없이 추론으로 이 파동에서 해결했다.**

## 6-4. 재정정 — 정체 추론이 명시 표지 없이 §6-3 을 닫았다

§6-3 은 "새 계열 표지는 3면 계약 확장이 필요하고 CSV 는 표현 불가"로 접수했다. 코드 전수
실측이 그 결론의 절반을 뒤집었다 — **명시 채널은 3면 동형을 깨므로 기각이 맞지만, 표지는
필요 없다.** 캔들 가드 `ends_kept` 가 이미 쓰던 이중 술어(비어 있지 않은 이름 일치 **또는**
값 벡터 일치)를 전 위치로 확장하면 목표 행렬만으로 원본↔목표 계열 대응이 서고, 대응 안 된
목표 자리가 곧 "새 계열"이다. 계약은 한 글자도 안 바꿨다.

**확정 스코프 — 비꼬리만.** 꼬리 삽입·꼬리 삭제·제자리 개명은 레거시 `appendSeries`/
`truncateSeries`/`renameSeries` 그대로다(바이트·op 불변 — CSV 계약 3시나리오·ooxml 구조
계약·원형 5종/막대 계열추가 원장 SHA 가 무수정 통과로 증명). 정체가 실제로 깨지는 것은
비꼬리 삽입·삭제뿐이고, 주식형 OHLC 는 캔들 가드 때문에 허용되는 삽입이 전부 중간이다 —
사용자가 본 결함이 정확히 이 경로였다.

구현 4축:

- **`data.rs`** — `ChartSeries.sp_pr_span`(계열 최상위 `c:spPr`)·`symbol_span`(`c:marker` 안
  `c:symbol`) 스캔. 깊이 카운터로 marker 하위 spPr 을 가르고, `dPt`/`dLbls` 는 기존
  SKIPPED_SUBTREES 가 이미 막는다.
- **`patch.rs`** — `InsertSeries{at,…}`(최종 문서 자리)·`RemoveSeries{at}`(원본 자리) 신설.
  복제 기계는 `append_series` 에서 추출한 `clone_series` 공유 — `strip_style` 이면 복제본의
  `sp_pr_span`·`symbol_span` 을 들어내 **기본 스타일**(Auto 마커·기본 선)로 되돌린다(한컴
  편집기가 더한 계열에는 둘 다 없다 — OHLC 실측). 앵커·재번호는 `finish()` 일괄 패스
  (복수 삽입/삭제 지원, `SeriesNotRenumberable` fail-closed). 레거시 꼬리 연산과는 상호
  배제(`OverlappingStructureEdits`).
- **`chart.rs`** — `infer_series_mapping`(단사 + 상대 순서 보존 + 삽입만/삭제만일 때만
  대응, 모호하면 전부 `None`) + `plan_identity_edits`(대응쌍은 원본 좌표 편집, 신설은
  `insertSeries`, 삭제는 `removeSeries`). `plan_edits` 머리에서 먼저 시도하고 실패하면
  현행 위치 기반으로 폴백 — R1(모호하면 무조건 폴백) 그대로.
- **봉투** — `changed[]` op 2종 추가: `insertSeries{at,name}`·`removeSeries{at,name}`.
  기존 op 단언은 전부 존재 단언이라 추가만으로는 깨지지 않았다(검증됨).

**판정 기준(§2 표) 실측 통과** — OHLC 위치 1 삽입 후 시가/고가/저가 `symbol none`+`a:ln
noFill`, 새 계열 spPr·symbol 없음(Auto·기본 선), 종가 Auto, idx/order 0..4. 한컴을 부르지
않고 `tests/cases/issue_6053_chart_series_identity_contract.rs` 4시험이 ①·② 양 표현에서
고정한다. `issue_4100` 의 중간 삽입·삭제 시험에도 스타일 단언을 추가했다.

**S5 동치의 반전** — 계열삭제 2종의 엔진 산출이 이제 스파이크 수술(`b2_remove_series`:
요소째 삭제 + 재번호)과 **바이트 동일**하다. "위치 기반이라 바이트가 다르다"던
`engine_documents_match_spike_documents_except_positional_series_delete` 는 전제가 사라져
`…_byte_for_byte`(24/24 동일)로 다시 썼다. 스파이크 산출은 #5447 이 이미 한컴 판정을 받은
모양이라, 정체 경로는 "판정받은 그 바이트"로 수렴한 것이다.

**원장 파급 — 정밀히 8파일.** `samples/issue6037/engine/시가고가저가종가-중간계열{추가,삭제}`
·`samples/issue5652/{묶은,누적}세로막대형-계열삭제` × 2포맷을 재생성해 반영하고 MANIFEST
2종의 `original_sha256` 8건을 갱신했다(판정 단위 13/8·자산 32 불변). 재생성 대조에서 그 외
hwpx 10건이 걸렸으나 전부 zip `version made by` 호스트 바이트(UNIX 03↔DOS 00, 엔트리당
1B)뿐 — 내용 동일이라 반영하지 않았다. issue5652 원장은 UNIX(03) 관례라 반영분 hwpx 2건은
03 으로 정규화했고, issue6037/engine 원장은 DOS(00) 관례라 그대로 뒀다.

**한컴 재판정 완료(R6 해소).** `output/issue6053_rejudgment/`(8파일 + PANJEONG.md) 핸드오프로
작업지시자가 한컴 변환 PDF 8건을 회수했고, 144DPI 래스터 판정으로 **전건 반영**을 확정했다:

| 단위 | 새 회차 vs 이전 회차 | 판정 근거 |
|---|---|---|
| {묶은,누적}세로막대형-계열삭제 (4) | 래스터 **완전 동일** | 이 코퍼스는 계열 색이 자동 배색(idx 사이클)이라 재번호 산출과 위치 기반 산출이 같은 그림 — 기존 "반영" 그대로 유효 |
| 시가고가저가종가-중간계열추가 (2) | 0.1512% | 추가계열이 **선+마커(기본 스타일)**로 그려짐 — rhwp 렌더와 동형. 이전 회차는 마커만 |
| 시가고가저가종가-중간계열삭제 (2) | 0.0210% | **종가가 자기 Auto 마커를 지킴** — 이전 회차는 저가 요소 상속으로 마커 소실 |

hwpx·hwp 변환 짝 4쌍의 래스터가 전부 동일해 포맷 간 판정 무모순이고, 전건 1190×1682 1쪽
(기하 불변)이다. 새 PDF 8건을 원장 경로에 반영하고 MANIFEST 2종의 `hancom_pdf_sha256` 8건·
OHLC 래스터 해시 4건·`editor_observation` 을 갱신했다(판정 단위 13/8·집계 불변).
`tools/hancom_chart_judgment_verify.py` 가 두 원장 전건(220건/156건) 통과를 확인했다.
issue5652 의 pdftoppm 보조 해시는 pymupdf 래스터가 바이트 동일하므로 유지했다(로컬에
pdftoppm 부재). issue5652 판정표의 "잔여 계열 색이 밀려도 정상" 문구는 이 결함을 정상으로
선언한 것이었으므로 "자기 색을 지킨다 — 밀리면 결함"으로 바로잡았다.

**검증 실측** — 최신 upstream/devel(6b5c4f871) 리베이스 후: release-test nextest 8373 전건
(재생성 자산의 렌더 계약 갱신 포함) · chart 크레이트 lib 165 · Native Skia(lib 3946+165 등
4패키지, issue_2225 2, render_p37 4) · clippy -D warnings · fmt · tier/suite-manifest 게이트 ·
CLI CSV 왕복 실측이 §2 표와 전칸 일치 · WASM Docker 재빌드 · studio 단위 1168 전건(선행
실패 1 도 해소) · e2e 2종(issue-6053·issue-4694) 전 단계 통과. 시각 실측 — 재생성
`중간계열추가` 의 rhwp 렌더에서 추가계열이 선+마커로 서고 캔들·고저선·종가 마커가 유지된다.

**검증 중 잡은 환경 함정(커밋 없음)** — 이 워크트리 체크아웃이 CRLF 로 재번짐(smudge)되어
소스·픽스처를 텍스트로 스캔하는 게이트들이 거짓 실패했다: wmf/emf 골든 9건, studio 원장·
소스 스캔 시험 6건(`\n` 정규식), `subsecond` Cargo.lock 스캔, Docker 가 실행하는
`wasm-pack-locked.sh` 의 shebang(`sh\r`). 전부 스캔 대상 LF 정규화로 해소했다 — 내용
불변이라 커밋할 것이 없고, §8-2(.gitattributes `eol=lf`) 제안이 유효함을 재확인한다.

## 7. 알려진 한계

- **중간 행 삽입·삭제는 원본 모양에 따라 코어가 거부할 수 있다.** 행 축은 여전히 위치
  기반이라 아래 칸이 제자리 치환 대상이 되고, 빈 값을 만나면 `valueNotPatchable` 이 선다.
  UI 는 막지 않고 코어의 한국어 `message` 를 그대로 보여준다. **계열 축은 §6-4 정체 경로가
  닫았다** — 다만 대응이 모호한 입력(중복 이름·값, 삽입·삭제 동시, 순서 역전)은 위치 기반으로
  폴백하므로 그 경로의 스타일 상속은 남는다(설계 — 모호하면 무조건 폴백).
- **`name === '' ⟺ c:tx 부재` 추론에 기대지 않는다.** 봉투의 `name === null` 만 "이름 칸 없음"으로
  읽고, 그 계열은 입력을 열지 않는다. 빈 문자열 이름은 코어 판정에 맡긴다.
- **한컴 재판정은 하지 않았다.** 쓰기 경로가 엔진 `set_chart_data_*` 와 동일 바이트이고 #5652 S5
  가 판정 단위를 닫았다. #4694 도 같은 논리였다.
- **차트 시각 회귀 게이트는 여전히 없다** — #3938 종료 코멘트가 지목한 렌더 축의 기존 공백.

## 8. 후속 제안

1. `check_e2e_manifest.py` 의 미등재 3건 등록 (별건, 선행 부채)
2. `.gitattributes` 에 소스 가드가 읽는 경로의 `eol=lf` 고정 — Windows 체크아웃에서 게이트가
   조용히 무력화되는 것을 구조적으로 막는다
3. 차트 e2e 2종을 CI 에 배선 — 지금은 둘 다 `수동`이라 이번 같은 부패를 아무도 못 잡는다
4. **차트를 렌더 회귀 게이트에 넣기** — 이번 결함이 오래 살아남은 이유가 여기 있다.
   `tests/golden_svg/**` 에 차트가 **0건**이고, renderer baseline 의 차트 4건은 `extended` 티어라
   `--scope=full` 없이는 안 돌며(게다가 등록된 것은 HLC 뿐, 캔들이 있는 OHLC 는 없다),
   `render-diff` 계열은 `rhwp-studio/public/samples` 한정이라 차트를 아예 안 덮는다.
   추가로 `.github/workflows/render-diff.yml` 의 paths 에 `crates/**` 가 없어 크레이트만 바꾼
   PR 은 Render Diff 가 뜨지도 않는다
5. 마커 글리프 사이클을 한컴 정답지에 맞추기 — 5계열에서 종가가 ×(한컴) vs ◆(rhwp)로 갈린다 (§6-2)
6. ~~**새 계열의 스타일** — 페이로드에 "이 위치는 신설" 표지가 필요하다~~ → **해결 (§6-4)** —
   표지 없이 정체 추론(이름·값 대응)으로 이 파동에서 닫았다. 계약은 한 글자도 안 바꿨고,
   재생성 8파일의 한컴 재판정도 전건 반영으로 회수·원장 반영까지 끝났다.
