---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-14
---

# PR #7111 — 검증된 글꼴의 가운뎃점 전진폭 검토

**최종 판정: 메인터너 보정 후 수용 가능.** C1 [해결]: 최신 devel #7084의 supplemental_metrics와 충돌했다. resolved_to_text_style에서 supplemental_metrics 복제와 font_metric_trusted 전달을 모두 보존해 로컬 bdee95748로 적용했고 #7084 계약도 focused 검증에 포함했다. 최초 조회의 원 fork head는 GitHub DIRTY였고 최종 상태 snapshot은 위 표에 별도 기록했다.


## 통합 PR #7138 최종 CI 확인 (2026-09-14)

[통합 PR #7138](https://github.com/edwardkim/rhwp/pull/7138)의 code candidate
`a7898ff72a0b22e1e4071b682616a26dc82fd801`에서 원 PR 12건과 메인터너 보정을 함께 검증했다.
아래 기존 조사 이력의 최초 누적 SHA와 현재 제출 SHA를 구분한다.

- [Full CI, attempt 1](https://github.com/edwardkim/rhwp/actions/runs/34829125405): Build & Test, Rust lint, Native Skia, archive A/B/C/D, frontend package 성공.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34829125417): Rust/Python/JavaScript 분석 성공.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/34829124987), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/34829125311), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/34829125263) 성공.
- CI Impact Policy와 GHAS CodeQL 성공. 조회 시점 `MERGEABLE/CLEAN`; 병합 전에는 trailing head를 다시 확인한다.
- CI의 base는 `042b02badf862ac7f8582b7615bcf9a3ac0b65df`다. 의존성 갱신이 포함된 최신 base와의 검증도 위 PR CI에서 성공했다.
- 로컬 최종 renderer `2f59c89373f497068f9a0bcb2c22730ec7dc7e51`: 전체 nextest 9,854 통과/51 skipped, Native Skia·세 Clippy·WASM·OVR5 통과. Visual Sweep 도구 `321f4cdb4`: Python 50개/webfont 6개 통과.
- WASM Sweep 5입력 14쪽 재캡처 및 178쪽 Native/WASM 텍스트·render tree 일치. 독립 한컴 PDF 전체 일치를 뜻하지 않는다. 로컬 Docker 최적화 WASM 빌드는 미검증이고 CI `WASM Build`도 skipped다.
- #7104·#7113·#7115의 원 head blocker는 기록된 보정으로 해소했다. #6970의 기존 p2 그림 겹침 3행·단/쪽 소속, #7105의 eqalign/OLE 실제 편집 등 명시한 부분 이슈 잔여는 유지한다.
- 이번 trailing commit은 review·오늘할일만 갱신한다. code candidate를 merge/rebase하지 않는다. 새 head의 CI 재사용과 required aggregate 통과, 작업지시자의 merge 승인 후 후속 처리한다.

원 PR head `cd772e0bd5ae9c7032085f4a300267da86341933` → 현재 통합 이력의 cherry-pick `10f2c727f562c7135282076ad6097500d0bd1dcf`. 원 저자와 `-x` 출처를 보존했다.


## 대상과 체리픽

| 항목 | 확인값 |
| --- | --- |
| 원 PR | [#7111](https://github.com/edwardkim/rhwp/pull/7111) — 수정(renderer): 믿을 수 있는 글꼴 표에서는 가운뎃점을 적힌 폭대로 잰다 (#7092) |
| 작성자 / reviewer | planet6897 / jangster77 사전 지정 |
| 원 base / 규모 | `devel`; 10 files, +374/-27 |
| source head | `cd772e0bd5ae9c7032085f4a300267da86341933` |
| 최초 기준 devel | `93ffc3dd59c120bd54df4c2ac6d1ddbe630f8a2d` |
| 검증한 누적 code head | `48afe0f95abc4cd76b002dfc314f3cc044f10b31` — 12 PR / 16 commit |
| 원본·기준 PDF 보존 commit | `6933852a11b7e5998429eeb15708fdaeed7db626` |
| 최신 devel 정렬 | `037e4906a93e99896daa145a5ee5517824bfeaf4`; 정렬 후 로컬 head `28d702d8f84bacb7ecec4f2a6dafd049e09bfcfa` |
| 검토 branch / target | `review/planet6897-20260914` / `target/planet6897-review-20260914` |
| 원 PR 상태 snapshot | 2026-09-14T16:10:01.589135+09:00; OPEN, draft=False, merge=UNKNOWN |
| 관련 이슈 | [#7092](https://github.com/edwardkim/rhwp/issues/7092); 부분 반영 범위만 판단, 이번 작업에서 종료하지 않음 |

대상은 조사 시점 open·non-draft 12개다. draft #7098은 제외했다. 최종 재조회에서도 대상과 source SHA는 같았다.
원 PR의 성공/skip CI는 확인했지만 최신 통합 candidate의 GitHub Actions 결과로 재사용하지 않는다.
검토·체리픽은 로컬 작업이며 원격 PR 생성·push·merge·close는 아직 하지 않았다.

| 원 commit | 로컬 적용 commit | 보정 |
| --- | --- | --- |
| `effdf7cd3f8b781cbc00affc145d3189e509dc0a` | `bdee95748cb8dd43e25a692119a37434159c2bf6` | supplemental_metrics와 font_metric_trusted 보존 |
| `91e94a7a8a085131fabcaf8e924a3075a79eea05` | `7a58f41a98dd5f6482ae26723ab809dcde7575c8` | 충돌 없음; -x·저자 유지 |
| `cd772e0bd5ae9c7032085f4a300267da86341933` | `b4c4b717bee9534c20cbebfcdd2ef02f8072b9d3` | 충돌 없음; -x·저자 유지 |

### 실제 변경 경로

- `mydocs/report/middle-dot-advance-7092/README.md` (+28/-0)
- `mydocs/report/middle-dot-advance-7092/d23-p1-before-after-oracle.png` (+0/-0)
- `mydocs/report/middle-dot-advance-7092/jubo-p2-before-after-oracle.png` (+0/-0)
- `src/renderer/font_metrics_overlays.rs` (+20/-1)
- `src/renderer/layout/text_measurement.rs` (+86/-25)
- `src/renderer/mod.rs` (+7/-0)
- `src/renderer/style_resolver.rs` (+59/-1)
- `tests/cases/issue_7092_middle_dot_advance.rs` (+161/-0)
- `tests/fixtures/issue_7092/README.md` (+13/-0)
- `tests/fixtures/issue_7092/soil-quarry-permit-standard.hwp` (+0/-0)

## 조판 원칙과 원인 계층

전각 메트릭을 일괄 .notdef로 보지 않는다. 치환되지 않은 TTF 선언과 독립 실측을 확보한 HY신명조 face만 신뢰하고, 결측 Latin1·HFT 치환은 기존 fallback을 유지한다. 측정·배치가 TextStyle의 동일 신뢰 플래그를 사용한다.

주요 검토 위치: [src/renderer/layout/text_measurement.rs](../../../src/renderer/layout/text_measurement.rs).

| 공통 항목 | 판정 | 근거·제한 |
| --- | --- | --- |
| 구현 근거와 일반성 | 충족 | 전각 메트릭을 일괄 .notdef로 보지 않는다. 치환되지 않은 TTF 선언과 독립 실측을 확보한 HY신명조 face만 신뢰하고, 결측 Latin1·HFT 치환은 기존 fallback을 유지한다. 측정·배치가 TextStyle의 동일 신뢰 플래그를 사용한다. |
| 측정·배치 일관성 | 충족 | 공통 측정·배치 값과 한컴의 실제 위치를 다음 절에서 대조한다. #7104와 #7115의 미해결 줄/그림 경계는 통과로 보지 않는다. |
| 줄 소속과 점유 높이 | 비해당 | 이 PR 자체는 줄 소속/점유 높이 계산을 바꾸지 않는다. 글자폭·출력에 따른 누적 줄바꿈 차이는 별도 실제 sweep 결과에서 다뤘다. |
| 사례와 증거의 독립성 | 충족 | 합성 계약과 실제 원본·한컴 PDF/직렬화/API·진단 증거를 분리했다. 실행 결함이 발견된 경우에도 정상 샘플 통과로 상쇄하지 않았다. |
| 기준값 변경 | 비해당 | 이 PR은 rendering baseline·golden·허용치를 변경하지 않는다. |
| 주장과 검증 범위 | 충족 | source SHA·실행 코드·실제 원본·명령·결과를 아래에 기록했다. 실행 회귀, 기존 잔여, 미검증을 구분하고 원 PR CI를 통합 CI로 재사용하지 않았다. |
| 실제 입력 커밋 | 충족 | 개인 다운로드 경로만 남기지 않고 Git object 또는 LFS oid와 실제 SHA-256을 대조했다. 기존 동일 파일은 재추가하지 않았다. |

## 직접 실행·시각 판정

soil-quarry 1쪽 한컴 PDF·SVG의 B7 전진폭과 jubo 2쪽의 좁은 점선 leader를 대조했다. 76076 18–19쪽 RowBreak 경계도 전후 비교했다. 원 PR의 최초 넓은 신뢰 조건이 아닌 최종 allowlist commit cd772e0bd를 적용했다. soil-quarry의 B7 31개는 PDF 1.0003em, candidate 0.9992em, baseline 0.2992em이다. jubo는 PDF의 기존 글꼴 매핑 차이가 커서 좁은 leader 유지만 판정했다.

C1 [해결]: 최신 devel #7084의 supplemental_metrics와 충돌했다. resolved_to_text_style에서 supplemental_metrics 복제와 font_metric_trusted 전달을 모두 보존해 로컬 bdee95748로 적용했고 #7084 계약도 focused 검증에 포함했다. 최초 조회의 원 fork head는 GitHub DIRTY였고 최종 상태 snapshot은 위 표에 별도 기록했다.

**잔여·미검증:** 휴먼명조 등 모든 face의 B7이 해결된 것은 아니다. #7092 전체 종료 근거로 쓰지 않는다. 76076의 기존 표 높이·글꼴 차이는 개선 주장 밖이다.

시각 검증 필요: **예**. 전체 문서의 SVG·render tree·문자/레이아웃 ledger를 생성하고, 아래 페이지를 96dpi로 한컴 PDF / base / 통합 세 방향으로 직접 읽었다. 페이지 수·픽셀 점수만으로 통과시키지 않았다.

- soil-quarry-p001: [한컴 / 변경 전 / 통합 비교](../assets/pr7111_soil-quarry-p001_3way.png), [한컴·통합 overlay](../assets/pr7111_soil-quarry-p001_ovl.png)
- jubo-p002: [한컴 / 변경 전 / 통합 비교](../assets/pr7111_jubo-p002_3way.png), [한컴·통합 overlay](../assets/pr7111_jubo-p002_ovl.png)
- reg76076-p018: [한컴 / 변경 전 / 통합 비교](../assets/pr7111_reg76076-p018_3way.png), [한컴·통합 overlay](../assets/pr7111_reg76076-p018_ovl.png)
- reg76076-p019: [한컴 / 변경 전 / 통합 비교](../assets/pr7111_reg76076-p019_3way.png), [한컴·통합 overlay](../assets/pr7111_reg76076-p019_ovl.png)

원본 PDF가 내장하지 않은 글꼴의 기존 매핑, editor-only placeholder, 선 굵기 차이는 전체 일치로 판정하지 않았다.
단일 쪽 SVG가 `_001` 없이 저장되면 fidelity helper의 파일명 기반 쪽수가 0으로 표시되는 경우가 있었다.
실제 SVG·render-tree 파일과 한컴 PDF 1쪽을 확인해 계수 오류와 렌더 실패를 구분했다.

## 입력·기준 출력 보존

확인 tree: `28d702d8f84bacb7ecec4f2a6dafd049e09bfcfa`. 다음 SHA-256은 LFS pointer 문자열이 아닌 실제 파일 바이트의 해시이며,
Git object 또는 LFS oid와 일치한다. full/OVR 공통 입력은 아래 재현 명령의 저장소 fixture 집합을 사용했다.

| 저장소 파일 | 출처·역할 | SHA-256 |
| --- | --- | --- |
| [samples/issues/2809/jubo_20260104.hwp](../../../samples/issues/2809/jubo_20260104.hwp) | 한컴 입력; jubo | `3fb2e25a8bd57ec8f2c8b2754613e3e69ace831d441a63f601f6dbea8e79d3ac` |
| [pdf/issue-2809-jubo_20260104-2020.pdf](../../../pdf/issue-2809-jubo_20260104-2020.pdf) | 독립 한컴 PDF 6쪽; 직접 sweep 2쪽 | `a73d50620bf8fe96beaff72ba0e40cd34f396ec75de9798ac1fd0402e28f8e2b` |
| [samples/76076_regulatory_analysis.hwp](../../../samples/76076_regulatory_analysis.hwp) | 한컴 입력; reg76076 | `3308ba8505391bae2d0d62963e9399f4e48cdae574304cc0f89a311c6efbb6b5` |
| [samples/issue1891/76076_regulatory_analysis-2024.pdf](../../../samples/issue1891/76076_regulatory_analysis-2024.pdf) | 독립 한컴 PDF 82쪽; 직접 sweep 18-19쪽 | `06a389455d6b96e5f6580c9930fd8555256f9c712be85fb3cdaf31fc601a090d` |
| [tests/fixtures/issue_7092/soil-quarry-permit-standard.hwp](../../../tests/fixtures/issue_7092/soil-quarry-permit-standard.hwp) | 한컴 입력; soil-quarry | `60433ba0f761ff9be54c8a17baeb26a3d24a50082afdea103adaa61226987a2b` |
| [pdf/pr-planet6897-20260914/soil-quarry-hwp-2020.pdf](../../../pdf/pr-planet6897-20260914/soil-quarry-hwp-2020.pdf) | 독립 한컴 PDF 6쪽; 직접 sweep 1쪽 | `5d2fdb6c82503d2f4908f0781d381490302d5a207333c260b45869090e193b5b` |

OVR 공통 입력도 같은 확인 tree의 파일을 사용했다.

| 저장소 파일 | 역할 | SHA-256 |
| --- | --- | --- |
| [samples/KTX.hwp](../../../samples/KTX.hwp) | OVR 5 공통 입력 | `b6c1492152f53e8dd7d4bbbb4faca88866bb8458e9018c70c936cd469ea6fab3` |
| [samples/exam_math.hwp](../../../samples/exam_math.hwp) | OVR 5 공통 입력 | `e40e3d675373c8efb3a844fc71f209600d3b0db987a04b3808b8e74a6b1671fe` |
| [samples/21_언어_기출_편집가능본.hwp](../../../samples/21_언어_기출_편집가능본.hwp) | OVR 5 공통 입력 | `905454045ca2e236839a7cab59750678116d08af3db31dbf846819af355b8d15` |
| [samples/aift.hwp](../../../samples/aift.hwp) | OVR 5 공통 입력 | `a3e94e613a7d3dad0ee11e2df8f9572a5b7c2d704602960c2075b5fd22df995c` |
| [samples/biz_plan.hwp](../../../samples/biz_plan.hwp) | OVR 5 공통 입력 | `8b786d6824622afae2220b203beeef6e5592157e1896fea055ebc602817113c1` |

신규 자료 출처·변환 영수증은 [새 한컴 PDF manifest](../../../pdf/pr-planet6897-20260914/README.md),
[transistor 원본·신고자 PDF](../../../tests/fixtures/issue_7105/README.md),
[익명화 다단 원본](../../../tests/fixtures/issue_6970/README.md)에 보존했다. 신규 MCP 변환 6건은 engine 2020,
Hancom 12.0.0.4605로 start → status(queued/running/succeeded) → download → SHA 확인까지 수행했다.
신고자 transistor PDF는 Hancom 2022의 기존 11쪽 출력이며 새 MCP 출력으로 오인하지 않는다.
인증 정보·임시 SVG/JSON·중간 로그는 커밋하지 않는다.

## 검증 결과와 재현

검증 코드 `48afe0f95abc4cd76b002dfc314f3cc044f10b31`와 최신 정렬 head의 Rust source·Cargo 입력은 동일하다. 추가 fixture commit은 원본/PDF 보존이며,
나중에 들어온 upstream은 Studio Vite/@types/chrome 의존성과 기존 검토 문서만 변경했다.
Rust 검증을 이 upstream 변경 후 재실행했다고 주장하지 않는다.

| 검증 | 실제 결과 |
| --- | --- |
| fmt / generated manifest / unit tiers | 통과; 생성 suite는 stage하지 않음 |
| focused nextest | 97 passed, 9800 skipped |
| 전체 nextest | 9846 passed, 51 skipped; 실행 528.167초 |
| Native Skia lib | 3930 + 15 + 165 + 2 passed, 13 ignored |
| Native placeholder / direct PDF export | 2 / 4 passed |
| Clippy native / wasm / workspace all-targets | 3종 통과, workspace build 통과 |
| OVR 필수 5문서 | KTX 27, exam_math 20, 언어 기출 15, aift 74, biz_plan 6쪽; 개체 회귀 0 |
| 새 WASM / 실제 Chrome | 빌드 성공; Chrome 152.0.7977.83에서 soil 1쪽, transistor 2쪽, table-text 1쪽, synth 1쪽을 열고 렌더. Native와 752/560/140/883 Text element 속성·텍스트 모두 동일 |
| WASM 배포 빌드 제한 | Docker daemon 미가용으로 native wasm-pack `--no-opt` 사용. 표준 Docker/wasm-opt 배포 빌드 완료를 주장하지 않음 |
| 원 PR CI | 위 source head의 Actions에 실패·대기 없음(성공/skip); 최신 통합 PR CI는 미실행 |

```sh
cd /Users/tsjang/rhwp
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all -- --check
cargo build --locked --target-dir target/planet6897-review-20260914 --profile release-test --bin rhwp
cargo nextest run --locked --target-dir target/planet6897-review-20260914 --cargo-profile release-test --tests --test-threads 6 --no-fail-fast -E 'test(/issue_4680|issue_7097|issue_6970|issue_7092|issue_7105|issue_7080|issue_7081|issue_7059|issue_7051|issue_7130|issue_3820_rowbreak_rowspan_band|issue_6590|issue_7084|issue_1285|svg_snapshot/)'
cargo nextest run --locked --target-dir target/planet6897-review-20260914 --cargo-profile release-test --tests --test-threads 6 --no-fail-fast
cargo test --locked --target-dir target/planet6897-review-20260914 --profile release-test --features native-skia --lib -- --test-threads 6
node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --locked --target-dir target/planet6897-review-20260914 --features native-skia
node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --locked --target-dir target/planet6897-review-20260914 --features native-skia
cargo clippy --locked --target-dir target/planet6897-review-20260914 -- -D warnings
cargo clippy --locked --target-dir target/planet6897-review-20260914 -p rhwp --lib --target wasm32-unknown-unknown -- -D warnings
cargo build --locked --target-dir target/planet6897-review-20260914 --workspace
cargo clippy --locked --target-dir target/planet6897-review-20260914 --workspace --all-targets -- -D warnings
node scripts/rust-test-suite-manifest.mjs --check
node scripts/rust-unit-test-tiers.mjs --check
scripts/wasm-pack-locked.sh --target web --out-dir /private/tmp/rhwp-planet6897-review-20260914/pkg --no-opt
```

시각 재현 예시(실제 입력·PDF·페이지는 위 표):

```sh
venv/bin/python scripts/visual_sweep.py --file-target <식별자> <입력> <한컴-PDF> \
  --rhwp-bin <해당-SHA에서-빌드한-rhwp> --pages <검토-쪽> --dpi 96 --out <외부-산출-폴더>
```

### 새 WASM browser 증적과 공통 시각 재현

[soil 1쪽](../assets/pr7111_browser_wasm_soil.png),
[transistor 2쪽](../assets/pr7113_browser_wasm_transistor.png),
[table-text 1쪽](../assets/pr7117_browser_wasm_table-text.png),
[synth 1쪽](../assets/pr7104_browser_wasm_synth.png)을 실제 Chrome에서 캡처했다.
`HwpDocument(bytes).renderPageSvg(pageIndex)`와 `pageCount()`를 호출하고
저장소 webfont projection으로 렌더했다. 정적 기존 pkg를 재사용하지 않았다.

전수 문자/레이아웃 검사는 각 base/candidate 바이너리를 `RHWP_BIN`으로 지정해 아래 명령으로 실행했다.

```sh
RHWP_BIN=<해당-SHA-바이너리> venv/bin/python tools/fidelity_compare/fidelity_compare.py \
  0 <마지막-0-based-쪽> --source <입력> --reference-pdf <한컴-PDF> \
  --label <문서명> --reference-grade 'Hancom PDF' --text-only --export-all-svg \
  --layout-ledger --out-dir <외부-산출-폴더>
```

OVR은 `tools/object_visual_regression.py`의 `PRESETS['ovr5']` 다섯 입력을 차례로 실행했다.
module의 `RHWP`를 새 base/candidate 바이너리로, `git_head()`를 해당 검증 code SHA로 지정했다.
각 입력은 base에서 `--no-hwp --save-baseline -o <base-folder>`, candidate에서
`--no-hwp --baseline <base-folder>/baseline.json -o <candidate-folder>`로 호출했고 전부 exit 0이었다.
이 실행은 **devel 대비 개체 회귀 검사**이며 한컴 PDF 동등성 검사를 대신하지 않는다.

## 다음 조건과 merge 후 contributor PR comment 계획

세 PR의 보류 사유를 메인터너 보정으로 해소했고 통합 PR #7138의 code candidate CI가 성공했다. 이 판정은 원 PR의 부분 개선 범위에 한한다.

이번 review·오늘할일 trailing 기록 뒤 최종 head Actions/mergeable 재확인과 작업지시자 merge 승인이 필요하다.
원 source PR을 지금 close하거나 승인을 원격 게시하지 않는다.

시각 근거를 사용한 PR은 [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md) direct link와
위 실제 페이지·지표·사람 판정, `mydocs/pr/assets/`의 대표 PNG를 merge 후 comment에 포함한다.
raw image 링크 형식은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/<위-PNG>`다.
merge SHA·devel asset 반영을 확인한 다음 승인된 범위에서 UTF-8 Markdown `--body-file`로 게시하고
API로 본문·한글·고정 이미지 링크를 재확인한다. 이 문단은 게시 계획이며 게시 완료가 아니다.

[적용·후속 단계](pr_7111_review_impl.md)

### 후속 댓글용 최종 Sweep 지표 보완

최종 renderer `2f59c8937`의 Native Visual Sweep(96dpi, 픽셀 차이 threshold 32) 결과다.
위 원 PR의 직접 시각 판정·대표 비교 PNG와 함께 다음 수치를 게시한다. 자동 flag 수는 페이지별 후보 유형 수이며 객체나 실제 결함 개수가 아니다.

| 입력·쪽 | 자동 flag 유형 수 | pixel_match | visual_accuracy_proxy_percent |
| --- | --- | --- | --- |
| soil-quarry 1쪽 | 0 (없음) | 89.00480% | 7.04378% |
| jubo 2쪽 | 0 (없음) | 86.72624% | 11.85028% |
| reg76076 18쪽 | 0 (없음) | 88.42353% | 20.00573% |
| reg76076 19쪽 | 0 (없음) | 92.08815% | 36.23274% |

기존 PNG는 위 본문에서 설명한 한컴/보정 전/통합 비교이며, 이 표는 최종 코드의 재실행 지표다. #7132를 포함한 이 표의 입력은 후속 메인터너 보정 전후 SVG가 동일했다.
objection-form/cancel-request의 단일 쪽은 overlay JSON에서 원본 파일 숫자(3030681/3079571)가 page 키로 들어간 기존 계수 문제를 실제 1쪽과 대응했다. 수치 자체는 변경하지 않았다.
픽셀 일치율은 여백·글꼴·기존 위치 차이의 영향을 받는다. 잉크 기반 proxy는 사람의 정확도 판정이 아니며, 전 문서 한컴 동등성으로 주장하지 않는다.
기존 차이와 부분 개선의 범위는 위 직접 판정을 유지한다.
대표 asset은 [비교 PNG](../assets/pr7111_soil-quarry-p001_3way.png)이며, 실제 merge 뒤
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7111_soil-quarry-p001_3way.png`로 고정한다.
[Visual Sweep GitHub comment 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결하고,
merge SHA·devel asset을 확인한 뒤 `--body-file` 게시 및 API 재조회 조건을 따른다.
