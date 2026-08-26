---
kind: report
status: active
canonical: mydocs/report/task_m100_6053_report.md
last_verified: 2026-08-26
---

# #6053 처리 결과 — B2-UI: 차트 행·열·라벨 구조 편집

- **Issue**: [#6053](https://github.com/edwardkim/rhwp/issues/6053) · 부모 [#3683](https://github.com/edwardkim/rhwp/issues/3683) Track B
- **계획서**: [task_m100_6053.md](../plans/task_m100_6053.md)
- **브랜치**: `task6053`, 기반 `upstream/devel = 70ebacc4c`
- **커밋**: S1(모델·페이로드) · S2(로컬 메뉴) · S3(다이얼로그) · S4(e2e) · S5(문서) 5건

## 결론 한 줄

**rhwp-studio 에서 차트 그리드 셀을 우클릭해 행·계열을 넣고 지우며 계열명과 카테고리 라벨을
고칠 수 있고, 그 편집이 화면과 저장본에 반영되며 Ctrl+Z 로 원복된다. Rust 변경 0줄.**

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

## 5. 검증 실측

```
npx tsc --noEmit                                   exit 0   (새 pkg/ 기준)
npx tsc --project tsconfig.ci-unit.json --noEmit   exit 0
npm test (Node 22.15.0)                            1168 tests / 1165 pass / 2 fail / 1 skip
python scripts/check_e2e_manifest.py               오류 3건 (전부 선행 부채 — 아래)
npm run e2e:issue-6053                             전 단계 통과
e2e issue-4694-chart-data-edit (B1 회귀)           전 단계 통과
cargo fmt --all -- --check                         exit 0
```

**`cargo fmt` 주석** — 이 파동은 `.rs` 변경 0건이다. 그런데 Windows `core.autocrlf=true`
체크아웃에서는 모든 `.rs` 가 CRLF 라 rustfmt(`newline_style = Unix`)가 전건 `Incorrect newline
style` 로 실패한다. 워킹트리의 `.rs` 1947개를 LF 로 정규화(저장 blob 이 LF 라 **내용 diff 0**)한
뒤 재실행해 **exit 0** 을 받았다. 또한 새 워크트리에는 gitignore 된 `tests/generated/*.rs` 가
없어 fmt 가 먼저 파일 부재로 죽는다 — `node scripts/rust-test-suite-manifest.mjs --generate` 로
32개를 만든 뒤 검사했다(생성물은 gitignore 대상이라 커밋하지 않았고, `Cargo.toml` 은 무변경).

### 선행 실패 — 착수 시점 기준선과 동일 (회귀 0)

착수 직후 clean `upstream/devel` 에서 기준선을 먼저 찍었다. 최종 실패 2건은 그 기준선과 같다.

| 실패 | 성격 |
|---|---|
| `style-undo-routing` — 스타일 WASM 반환 계약 | 선행. #4694 보고서의 pre-existing 목록에 있다 |
| `subsecond-runtime` — `Cargo.lock` 에서 `subsecond` 미검출 | 선행. 동일 |

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

## 7. 알려진 한계

- **중간 삽입·삭제는 원본 모양에 따라 코어가 거부할 수 있다.** 엔진이 위치 기반이라 아래·오른쪽
  칸이 제자리 치환 대상이 되고, 빈 값이나 `c:tx` 부재를 만나면 `valueNotPatchable`/
  `seriesNameNotPatchable` 이 선다. UI 는 막지 않고 코어의 한국어 `message` 를 그대로 보여준다.
  엔진 한계이며 이 파동에서 엔진을 고치지 않았다.
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
