# #8201 결과 보고 — 서명·날인 기호 U+F012B 를 1em 타원 안 '인'으로

Issue: [#8201](https://github.com/edwardkim/rhwp/issues/8201) ·
계획: [task_m100_8201.md](../plans/task_m100_8201.md), [구현 계획](../plans/task_m100_8201_impl.md)

PR base `bc9dfdd05`(upstream/devel), code head `75fadf315`. 게이트·Visual Sweep 은 그 위 문서 커밋
`ceaffc236` 에서 실행했다(코드 동일).
PR #8296 은 upstream 이 앞서 나가며 두 번 충돌했고, 두 번 모두 최신 devel 위로 rebase 했다.

- 1차(+402커밋, base `db5cd5bcc`): refactor 커밋에서 upstream 의 글자 위치 헬퍼 `cluster_x`·Canvas2D 글꼴
  체인 `canvas_text_font::family_chain`·Skia 바깥 `char_x` 와 겹친 5개 파일, 공개 face 커밋에서 Canvas2D
  글꼴 목록 1곳.
- 2차(+543커밋, base `bc9dfdd05`): refactor 커밋에서 `text_measurement.rs`(upstream 이 이 함수의
  `areaDotFallback` 분기를 없앰 — upstream 구조를 따르고 네모 숫자 분기만 `enclosed_pua` 판정으로 바꿈)와
  Skia `text_replay.rs`(upstream 이 글꼴 생성 뒤 `fonts::preserve_geometry` 를 추가 — 안쪽 글자 글꼴에도
  같은 호출 적용), fix 커밋에서 `pua_surface.inc.rs`(upstream 이 문자 추가를 한도 검사가 있는
  `storage.push_char` 로 바꿈 — 합성 기호면 원 키를 유지하는 조건에 upstream 의 새 호출을 씀).

두 번 모두 upstream 쪽 새 API·구조에 우리 판정을 얹는 방식으로 풀었다. 이전 회차의 결과는 이번 head 에
그대로 쓰지 않고 아래를 모두 다시 쟀다.
커밋: `1969d09aa` 계획·입력 보존 → `63235319f` 회귀 가드(RED) → `c3da64085` refactor →
`4b82b5300` fix → `c636956cd` 합성 fixture 렌더링 검사 제외 → `55b490420` 증적 →
`75fadf315` 안쪽 '인'의 공개 대체 face → 증적 커밋.

## 원인과 정답

`hancom_pua.rs`의 `(0xF012B, "(인)")`가 기호 1글자를 3글자로 투영했고, 측정과 paint가 모두 이를
소비했다. 서로 일관되지만 한컴과 다르다. 한컴은 이 기호를 문서 글꼴과 무관하게 Haansoft Batang
GID 3 한 글리프(`/W` 1000, 타원 안 '인')로 그린다. 같은 글리프를 세 독립 한컴 출력에서 확인했다.

| 근거 | 관측 |
| --- | --- |
| `samples/issue8201/seal-delegation-form-2020.pdf` (Hwp 2020) | GID 3, 전진 57 device unit ≈ 같은 줄 한글 58 (10pt·자간 −31%) |
| `pdf/복학원서-2022.pdf` (Hwp 2022) | 서명란과 접수증 날인선 가운데 모두 GID 3 |
| `pdf/issue4680/20117321_gijang_handover_bond_ledger_form-2020.pdf` (Hwp 2020) | GID 3, 같은 윤곽 경계 |
| GID 3 윤곽(UPM 1024) | 바깥 타원 (50,−76)–(979,842), 안쪽 (77,−50)–(952,817), '인' (181,34)–(801,757) |
| 6개 한컴 PDF의 일반 '인' | (116,−69)–(912,842), 전진 1024 |

`#937`은 같은 복학원서 PDF를 `(인)`으로 읽어 기대값을 고정했다. 그 기대값을 이 근거로 교정했다.

## 변경

- refactor: 네모 숫자 판정·기하를 `EnclosedPua` 하나로 모았다(Rust 약 12곳, TS 4곳). rebase 전
  refactor 커밋에서 네모 숫자 40개 문서의 SVG·Native PNG 2,908쪽이 당시 base `9418ed0c7` 과 바이트 단위로
  같았고, rebase 후에는 아래 영향 범위의 samples 전수 SVG 비교로 seal 문서 밖 무변화를 다시 확인했다.
- fix: `EnclosedPua::CircledSeal` 과 `enclosed-pua-metrics.json` `circledSeal`. 측정 1em(자간·장평은
  일반 경로), paint 는 SVG `<ellipse>`·Canvas2D `ellipse`·Skia `draw_oval`·CanvasKit `drawOval` 과
  변환한 안쪽 '인'. 표시 투영은 "실제 공급 원 glyph 우선" 분기에 합쳐 원 키를 유지한다.
  `(인)`은 텍스트 표면·글자겹침의 읽기 문자열로 남는다. 접수증 날인선 합성(#2020/#4384)도 원 기호를 쓴다.
- 안쪽 '인'의 대체 face: Haansoft Batang 다음에 동봉 OFL `RHWP Public Batang`(일반 '인' 경계가
  Haansoft Batang 과 거의 같음, 중심 차이 +0.02pt)을 요청한다(`circledSeal.publicFace`). SVG·Canvas2D·
  Skia 가 같은 face 목록을 쓰고, Studio 가 웹폰트를 적재하도록 문서 정보 `fontsUsed` 에 싣는다
  (`content_fonts`). 이 보정 전 macOS Chrome SVG 는 고딕 대체로 원 안 '인'이 한컴보다 0.45pt 왼쪽이었다.
  보정 후 원 중심 대비 '인' 중심은 한컴 −0.44pt, SVG(Chrome) −0.47pt, Native −0.44pt 다
  (576dpi 래스터의 잉크 경계로 측정, `seal-delegation-form-p001-seal-centering-closeup.png`).
  네모 숫자는 face 목록이 `[innerFace]` 하나라 출력이 그대로다.

## 영향 범위

samples 1,406 문서의 전 쪽 SVG(14,287쪽)를 base `bc9dfdd05` 와 head `ceaffc236` 로 내보내 비교했다. 바뀐 문서는
8개이며 모두 seal 문서다(이슈 서식, 복학원서 HWP·HWPX, issue4680, issue5800, issue5870, hwp3-table-caption,
issue1842). 사라진 `(인)` 묶음 16곳과 새 "타원 + 안쪽 '인'" 16곳이 1:1로 대응하고, 그 밖의 텍스트 변화와
쪽수 변화는 없다. 복학원서(HWP·HWPX) 접수증 날인선은 seal 이 좁아진 만큼 대시 글자와 그 선분이 2개씩
늘었다. 나머지 1,398 문서는 바이트 단위로 같고, 쪽수가 달라진 문서는 없다(암호 문서 3건은 양쪽 모두 0쪽).

## 검증

### Rust·Studio (검증 head `ceaffc236` = code `75fadf315`, 정책 비교 base `bc9dfdd05`)

CONTRIBUTING 절차로 같은 SHA 의 별도 review worktree 에서 실행했다. 공용 target `target/pr-review`.

| 검사 | 결과 |
| --- | --- |
| `git diff --check base...head`, `cargo fmt --all -- --check` | 통과 |
| Clippy native / `wasm32-unknown-unknown` lib / workspace all-targets (`-D warnings`) | 통과 |
| `cargo build --workspace`, suite manifest 테스트·`--check --base-ref`, unit tier `--check --base-ref` | 통과 |
| `cargo nextest run --cargo-profile release-test --tests` | 12,229 통과 / 0 실패 / 45 skip (코퍼스 래칫 포함) |
| Native Skia: `--features native-skia --lib`, `issue_2225…`, `render_p37…` | `rhwp` lib 3,857 통과·12 ignored(workspace 다른 lib 182 통과) / 2 / 22 통과, 실패 0 |
| 새 sample 보안 검사 (`RHWP_SECURITY_SWEEP_SAMPLES_JSON`) | 6 통과 |
| RED ①: 최종 `issue_8201_seal_pua` 를 refactor 만 있는 `c3da64085` 에 넣어 실행 | 4 FAIL(상자 안 타원 개수 `:240`, seal 1em 전진 `:264`, 복학원서 seal 기호 위치 `:282`, 공개 face 요청 `:323` 의 단언) / 반례 1 PASS |
| RED ②: 같은 테스트를 공개 face 보정 전 `c636956cd` 에 넣어 실행 | 공개 face 검사(`:323`) 1 FAIL / 나머지 4 PASS |
| 수정 후 같은 테스트 | 5 통과 |
| Studio `tsc --noEmit`(fresh WASM 포함) | 통과. base 의 `package-lock.json` 으로 `npm ci` 한 뒤 실행 |
| Studio 단위 `npm test` (Node 22.23.4) | 2,702건 중 2,700 통과·실패 0 — base `bc9dfdd05`+base WASM 도 같은 결과. Node 22.12.0 으로는 head·base 모두 같은 27건이 실패했다(2,439건 실행, 목록 동일): 그 환경의 타입 제거 지원 문제 |
| `e2e/renderer-contract.test.mjs`(seal replay drawOval + '인' 추가) | 통과 |
| `e2e/issue-536…`, `e2e/issue-4158…` 브라우저 e2e 원본 | 이전 회차에서 base·head 모두 `renderPageToCanvas` 가 #7084 의 export 보호("supplemental metrics are active")에 걸려 판정 전 중단 — 테스트가 그 이전 작성분. 이번 head 에서는 아래 로컬 변형만 실행 |
| 위 두 e2e 의 로컬 변형(Canvas2D 호출만 앱 export 경로와 같은 `withPortableMetrics` 로 감쌈, 커밋 안 함) | 4158: base·head 모두 전체 통과. 536: base·head 모두 상자 높이 37px 로 기대 30.7px(#7642 이전 0.72em 기준) 실패 — 같은 결과. 두 테스트의 Canvas2D PNG 는 base·head 바이트 동일 |
| seal Canvas2D 로컬 e2e(이슈 서식 1쪽, `withPortableMetrics`) | head: seal 전진 9.174 = 한글 9.174, 잉크가 전진 상자 안 — 통과. base: 3.833 vs 8.420 — 실패(결함 재현) |

산출물: Native CLI `rhwp-ceaffc236` SHA-256 `893a9c32…`, fresh WASM `rhwp_bg.wasm` `d5ee4be6…`,
`rhwp.js` `261594f3…`. base CLI `81a77403…`, base WASM `rhwp_bg.wasm` `14b937f7…`(`rhwp.js` 는 head 와 같음).

### Visual Sweep (96dpi, print 프로필, 2px 관용 실루엣)

base CLI `bc9dfdd05`(SHA-256 `81a77403…`)·base WASM(`14b937f7…`)과 head CLI `ceaffc236`·head fresh
WASM(위 산출물)을 같은 입력·기준 PDF로 쟀다. 이번 base 에서 upstream 의 Sweep 도구가 바뀌었다(10-28 DPI·글꼴
공급 개선, #8397 등). TSV 는 소수 둘째 자리까지 쓰므로 아래 값은 `summary.json` 의 다섯째 자리 값이다.
WASM 출력 스크립트가 `node:module` 의 `stripTypeScriptTypes`(Node 22.13+)를 쓰게 되어 WASM Sweep 은
Node 22.23.4 로 실행했다(22.12.0 에서는 import 단계에서 실패해 측정 불가였다). 이 PR 은 Sweep 도구와
그 스크립트가 읽는 Studio 모듈을 바꾸지 않으므로 base·head 는 같은 측정 도구로 쟀다.
글꼴 공급은 아래 "검증 환경 주의"의 `RHWP_FONT_PATH` 로 base·head·Native·WASM 모두 같다. 값은 gate
판정값인 실루엣 raw 일치율(%)이다.

| 입력 · 기준 PDF | 쪽 | base Native | head Native | base WASM | head WASM |
| --- | --- | --- | --- | --- | --- |
| 이슈 서식 · `samples/issue8201/seal-delegation-form-2020.pdf` | 1 | 99.91003 | 99.91367 | 99.90870 | 99.91235 |
| 〃 | 2 | 99.97820 | 99.97820 | 99.97824 | 99.97824 |
| 복학원서 · `pdf/복학원서-2022.pdf` | 1 | 97.22964 | 97.26337 | 97.24647 | **97.22414** |
| issue4680 · `pdf/issue4680/…bond_ledger_form-2020.pdf` | 1 | 98.91692 | 99.09794 | 98.92897 | 99.10997 |

복학원서만 색상 경계 대조 픽셀이 있다(Native base·head 186개, WASM base·head 195개; 보정값 Native
97.41354 → 97.44752, WASM 97.43899 → 97.41677). 나머지는 대조 픽셀 0개로 raw 와 보정값이 같다. 방법은
`threshold_boundary_color_support_v1`, 2px 이웃, 색상 관용 32.

전체 쪽수는 세 문서 모두 PDF 와 같다(2·1·1). 90% 미만·누락·측정 불가 쪽은 없고, 대표 review 의
`pr_review_gate` 는 head Native·WASM 모두 `passed`(쪽수 불일치 없음)다.

head 가 base 보다 낮은 곳은 복학원서 WASM 하나다(raw −0.02233%p, 불일치 2,789 → 2,810px, +21px).
같은 판정식으로 불일치 픽셀을 영역별로 나눴다(96dpi 좌표).

| 영역 | base WASM | head WASM | base Native | head Native |
| --- | --- | --- | --- | --- |
| 서명란 `(인) (Signature)` (x 360–465, y 585–612) | 15 | 25 | 52 | 0 |
| 접수증 날인선 (x 60–740, y 770–794) | 29 | 40 | 29 | 45 |
| 그 밖 | 2,745 | 2,745 | 2,721 | 2,719 |

- 서명란: head WASM 의 원과 안쪽 '인'에는 불일치 픽셀이 없다. 늘어난 10px 은 모두 x 450–454, 즉 뒤따르는
  `(Signature)` 의 닫는 괄호 자리다. WASM 은 이 라틴 문자열을 Native 보다 4–5px 좁게 그린다(base 도 같음).
  base 는 `(인)` 이 두 글자 더 길어 문자열 끝이 PDF 를 넘었고(Native 48px·WASM 12px 초과),
  head 는 기호가 1em 이 되어 WASM 에서 PDF 끝보다 몇 px 모자란다.
- 접수증 날인선: 줄 전체가 PDF 보다 2px 아래다(PDF 선 y=780, rhwp 대시 y=782 — base·head·Native·WASM 모두
  같음). head 의 원이 `(인)` 보다 아래로 더 크게 나와 이 기존 오프셋이 몇 px 더 드러난다. Native 도 같은
  경향이다(29 → 45).
- 안쪽 '인' 글꼴: WASM Sweep 카탈로그(`RHWP_FONT_PATH`)에는 NanumMyeongjo.ttc 만 있어, Chrome 은 시스템에
  설치된 `RHWP Public Batang`(`~/Library/Fonts`, 저장소 `ttfs/opensource/RHWPPublicBatang-Regular.ttf` 와
  같은 SHA-256 `7e3aee5e…`)을 썼다. 같은 파일을 카탈로그로 명시적으로 공급해 다시 내보낸 head WASM PNG 는
  이슈 서식 1·2쪽, 복학원서 1쪽 모두 바이트 단위로 같았다(요청·적재 목록에 `rhwp public batang` 확인).

점수와 별개로 review·300dpi 확대본을 직접 판독했다. 표 외곽·앞뒤 문단·쪽 구성은 base 와 같고, seal 이
한컴처럼 타원 안 '인' 한 글자가 되며, 이슈 서식 `명` 이 셀 안에 들어오고, 복학원서 서명란·접수증 날인선
가운데가 한컴과 같은 기호다(`mydocs/pr/assets/issue_8201_seal_pua/*-closeup.png`, Studio Canvas2D 는
`…-canvas2d-closeup.png`). 남은 차이(제목 굵기, 복학원서 로고·`(Signature)` 라틴 글꼴·자간)는 base 에도 같다.

## 검증 환경 주의

이 Mac 의 `~/Library/Fonts/NanumMyeongjo.ttf` 와 시스템 자산 `NanumMyeongjo.ttc` 가 PostScript 이름을
공유해 Native Skia 가 cmap 과 윤곽을 서로 다른 파일에서 가져온다. 명조 대체 체인의 한글이 두 글리프씩
밀린다(복학원서 base Native "본 인"→"볶 읶"). 코드와 무관한 환경 결함이라 Native·WASM 측정은 수정
전·후 모두 시스템 ttc 사본(SHA-256 `423cf9c2…460e`)을 `RHWP_FONT_PATH` 로 공급해 다시 쟀다.
이 공급 없이 잰 첫 base 값(복학원서 95.40706%)은 오염값으로 폐기했다.

## 남은 범위와 비범위

- HWP5 원시 `0xA12B` 경로: 평면 15 정규화 계약은 `issue_5800` 이 지킨다. 그 합성 fixture 의 기준 PDF
  (`pdf/pr_5815/...`)는 도장란을 이(Yi) 음절 글리프로 그려 seal 근거가 되지 않아 시각 일치는 미검증이다.
- `samples/issue5870/empty_host_float_flow_advance.hwp`·`hwp3-table-caption.hwp`·
  `issue1842_cell_tac_group_lineheight.hwp` 의 seal: 저장소에 한컴 PDF 가 없어 한컴 대조는 미검증이다.
- 글자겹침(tcps) 안의 F012B 는 근거가 없어 종전 `(인)` 을 유지했다.
- HTML DOM 렌더러는 네모 숫자와 같은 원문 유지 계약이다(합성 없음).
- 레이어 JSON 소비자는 F012B 를 네모 숫자처럼 원 코드포인트로 받아 합성해야 한다(`displayText` 없음).
- 접수증 날인선의 점선/대시 모양과 2px 세로 오프셋, `(Signature)` 라틴 글꼴·WASM 의 라틴 폭 차이는
  종전부터 있던 차이로 이번 범위가 아니다.
- 안쪽 '인'은 Haansoft Batang 다음에 `RHWP Public Batang` 을 쓴다. Studio 는 이 웹폰트를 적재하지만,
  두 face 가 모두 없는 환경(예: 글꼴을 설치하지 않은 외부 SVG 뷰어)에서는 문서 글꼴 체인으로 그려
  '인'의 여백이 한컴과 조금 다를 수 있다.
- Studio CanvasKit 의 실제 화면 캡처는 하지 않았다. seal replay 계약(`drawOval` + 안쪽 '인')은
  `e2e/renderer-contract.test.mjs` 로만 확인했다.
