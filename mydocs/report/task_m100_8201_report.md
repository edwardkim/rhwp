# #8201 결과 보고 — 서명·날인 기호 U+F012B 를 1em 타원 안 '인'으로

Issue: [#8201](https://github.com/edwardkim/rhwp/issues/8201) ·
계획: [task_m100_8201.md](../plans/task_m100_8201.md), [구현 계획](../plans/task_m100_8201_impl.md)

PR base `0bb36e6f9`(upstream/devel), code head `90aba0886`.
게이트·Visual Sweep 은 base `f770ef502` 위의 `529ee86d1`(code `f8f32af6c`)에서 실행했다. 그 뒤 upstream 이
문서만 4커밋(`AGENTS.md`·`CLAUDE.md`·`mydocs/`) 바꿔 그 위로 다시 rebase 했고, 코드·테스트·자원 트리는
`529ee86d1` 과 같다(`git diff 529ee86d1 HEAD` 는 upstream 문서 7개뿐). 새 base 로 `git diff --check`,
manifest·unit tier `--check --base-ref 0bb36e6f9` 를 다시 실행해 통과했다.
커밋: `6069ecad7` 계획·입력 보존 → `076ad6451` 회귀 가드(RED) → `2299c2f7a` refactor →
`fb0e636a1` fix → `9dcd1fdd1` 합성 fixture 렌더링 검사 제외 → `a202d6474` 증적 →
`90aba0886` 안쪽 '인'의 공개 대체 face → 증적 커밋.

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

samples 1,353 문서의 전 쪽 SVG(14,033쪽)를 base `f770ef502` 와 head `529ee86d1` 로 내보내 비교했다. 바뀐 문서는
8개이며 모두 seal 문서다(이슈 서식, 복학원서 HWP·HWPX, issue4680, issue5800, issue5870, hwp3-table-caption,
issue1842). 사라진 `(인)` 묶음 16곳과 새 "타원 + 안쪽 '인'" 16곳이 1:1로 대응하고, 그 밖의 텍스트 변화와
쪽수 변화는 없다. 복학원서(HWP·HWPX) 접수증 날인선은 seal 이 좁아진 만큼 대시 글자와 그 선분이 2개씩
늘었다. 나머지 1,345 문서는 바이트 단위로 같다(암호 문서 3건은 양쪽 모두 0쪽).

## 검증

### Rust·Studio (검증 head `529ee86d1` = code `f8f32af6c`, 정책 비교 base `f770ef502`)

CONTRIBUTING 절차로 같은 SHA 의 별도 review worktree 에서 실행했다. 공용 target `target/pr-review`.

| 검사 | 결과 |
| --- | --- |
| `git diff --check base...head`, `cargo fmt --all -- --check` | 통과 |
| Clippy native / `wasm32-unknown-unknown` lib / workspace all-targets (`-D warnings`) | 통과 |
| `cargo build --workspace`, suite manifest 테스트·`--check --base-ref`, unit tier `--check --base-ref` | 통과 |
| `cargo nextest run --cargo-profile release-test --tests` | 11,577 통과 / 0 실패 / 45 skip (코퍼스 래칫 포함) |
| Native Skia: `--features native-skia --lib`, `issue_2225…`, `render_p37…` | `rhwp` lib 3,858 통과·12 ignored(workspace 다른 lib 182 통과) / 2 / 14 통과, 실패 0 |
| 새 sample 보안 검사 (`RHWP_SECURITY_SWEEP_SAMPLES_JSON`) | 6 통과 |
| RED ①: 최종 `issue_8201_seal_pua` 를 refactor 만 있는 `969721c95`(현 `2299c2f7a`) 에 넣어 실행 | 4 FAIL(seal 4.242 vs 한글 8.829, 상자 안 타원 0개, 복학원서 seal 기호 수, 공개 face 미요청) / 반례 1 PASS |
| RED ②: 같은 테스트를 공개 face 보정 전 `e28bd0d21`(현 `9dcd1fdd1`) 에 넣어 실행 | 공개 face 검사 1 FAIL / 나머지 4 PASS |
| 수정 후 같은 테스트 | 5 통과 |
| Studio `tsc --noEmit`(fresh WASM 포함) | 통과. base 의 `package-lock.json` 으로 `npm ci` 한 뒤 실행 |
| Studio 단위 `npm test` | 2,311건 중 27건 실패 — base `f770ef502`+base WASM 에서도 같은 27건(목록 동일). 이 환경(Node 22.12 타입 제거 실험 옵션)의 기존 실패 |
| `e2e/renderer-contract.test.mjs`(seal replay drawOval + '인' 추가) | 통과 |
| `e2e/issue-536…`, `e2e/issue-4158…` 브라우저 e2e 원본 | base·head 모두 `renderPageToCanvas` 가 #7084 의 export 보호("supplemental metrics are active")에 걸려 판정 전 중단 — 테스트가 그 이전 작성분 |
| 위 두 e2e 의 로컬 변형(Canvas2D 호출만 앱 export 경로와 같은 `withPortableMetrics` 로 감쌈, 커밋 안 함) | 4158: base·head 모두 전체 통과. 536: base·head 모두 상자 높이 37px 로 기대 30.7px(#7642 이전 0.72em 기준) 실패 — 같은 결과. 두 테스트의 Canvas2D PNG 는 base·head 바이트 동일 |
| seal Canvas2D 로컬 e2e(이슈 서식 1쪽, `withPortableMetrics`) | head: seal 전진 9.174 = 한글 9.174, 잉크가 전진 상자 안 — 통과. base: 4.242 vs 8.829 — 실패(결함 재현) |

산출물: Native CLI `rhwp-529ee86d1` SHA-256 `71321e87…`, fresh WASM `rhwp_bg.wasm` `b21782ca…`,
`rhwp.js` `c1c8d182…`. base CLI `91e442e4…`, base WASM `rhwp_bg.wasm` `f51d1389…`(`rhwp.js` 는 head 와 같음).

### Visual Sweep (96dpi, print 프로필, 2px 관용 실루엣)

base CLI `f770ef502`(SHA-256 `91e442e4…`)·base WASM(`f51d1389…`)과 head CLI `529ee86d1`·head fresh
WASM(위 산출물)을 같은 입력·기준 PDF로 쟀다. 출력은 직전 base `238de6de4` 회차와 픽셀 단위로 같다.
글꼴 공급은 아래 "검증 환경 주의"의 `RHWP_FONT_PATH` 로 base·head·Native·WASM 모두 같다. 값은 gate
판정값인 실루엣 raw 일치율(%)이다.

| 입력 · 기준 PDF | 쪽 | base Native | head Native | base WASM | head WASM |
| --- | --- | --- | --- | --- | --- |
| 이슈 서식 · `samples/issue8201/seal-delegation-form-2020.pdf` | 1 | 99.65126 | 99.67278 | 99.63982 | 99.66136 |
| 〃 | 2 | 99.91336 | 99.91336 | 99.91352 | 99.91352 |
| 복학원서 · `pdf/복학원서-2022.pdf` | 1 | 97.28575 | 97.32742 | 97.27006 | 97.31709 |
| issue4680 · `pdf/issue4680/…bond_ledger_form-2020.pdf` | 1 | 98.64195 | 98.79450 | 98.65395 | 98.80607 |

복학원서만 색상 경계 대조 픽셀이 base·head 모두 185개다(보정값 Native 97.46709 → 97.50893,
WASM 97.45127 → 97.49843). 나머지는 대조 픽셀 0개로 raw 와 보정값이 같다. 방법은
`threshold_boundary_color_support_v1`, 2px 이웃, 색상 관용 32.

전체 쪽수는 세 문서 모두 PDF 와 같다(2·1·1). 90% 미만·누락·측정 불가 쪽은 없고, 대표 review 의
`pr_review_gate` 는 head Native·WASM 모두 `passed`(쪽수 불일치 없음)다. head 는 모든 쪽에서 base 이상이다.
점수와 별개로 review·300dpi 확대본을 직접 판독했다. 표 외곽·앞뒤 문단·쪽 구성은 base 와 같고, seal 이
한컴처럼 타원 안 '인' 한 글자가 되며, 이슈 서식 `명` 이 셀 안에 들어오고, 복학원서 서명란·접수증 날인선
가운데가 한컴과 같은 기호다(`mydocs/pr/assets/issue_8201_seal_pua/*-closeup.png`, Studio Canvas2D 는
`…-canvas2d-closeup.png`). 남은 차이(제목 굵기, 복학원서 로고·`(Signature)` 라틴 글꼴)는 base 에도 같다.

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
- 접수증 날인선의 점선/대시 모양, `(Signature)` 라틴 글꼴 차이는 종전부터 있던 차이로 이번 범위가 아니다.
- 안쪽 '인'은 Haansoft Batang 다음에 `RHWP Public Batang` 을 쓴다. Studio 는 이 웹폰트를 적재하지만,
  두 face 가 모두 없는 환경(예: 글꼴을 설치하지 않은 외부 SVG 뷰어)에서는 문서 글꼴 체인으로 그려
  '인'의 여백이 한컴과 조금 다를 수 있다.
- Studio CanvasKit 의 실제 화면 캡처는 하지 않았다. seal replay 계약(`drawOval` + 안쪽 '인')은
  `e2e/renderer-contract.test.mjs` 로만 확인했다.
