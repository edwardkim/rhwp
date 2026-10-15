---
kind: evidence
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-13
---

# PR #7815 체리픽·메인터너 보정 검토 계획

## 2026-10-15 최종 통합 검토 연결

최종 적용 범위와 판정은 [개별 review](pr_7815_review.md)와 [통합 review](planet6897_followup_20261015_review.md)를 따른다. 검증 code SHA는 `39a634e547621c32ae6f9ba8c1694d35c827078b`다. 최초 전체 실행·사용자 승인한 집중 재검증 합산, 전21문서181쪽 TSV와 최종 로컬 게이트를 구분해 기록했다. 아래 단계별 “현재”·미완료·C20 기록은 각 작성 당시의 이력이며 최종 상태를 대신하지 않는다. 원격 통합 PR의 CI·merge는 실제 확인 전 완료로 표시하지 않는다.


## 변경 전 C20 통합 검증 — 2026-10-13

변경 전 C20 검증 source HEAD는 `e2a525d36f2923918185f62610869e7c3068b9d0`, 검증 base는
`552f5228d2dbfd6046c838f14241704f1bcb8a05`다. 오늘 `upstream/devel` fetch와 rebase가 모두 성공했으며
SHA 변경·충돌은 없었다. 아래 과거 단계의 SHA·관측값은 당시 기록으로 유지한다.

이 C20 source에서 Rust 전체 회귀 **10,241건 통과·45건 제외**, Studio **1,836건 통과·실패 0건**,
Native Skia library/workspace **4,046건 통과·12건 ignored**를 확인했다.
Studio typecheck/build, Native·WASM·workspace all-target Clippy와 두 정책의 base 비교,
최적화 fresh WASM 및 Native 빌드도 exit 0이다. 모든 Cargo 빌드는 공용
`/home/tsjang/rhwp/target/pr-review`를 사용했다. 실행 명령·시간·SHA·exit code는
ignored `output/pr-review/planet6897-followup-20261011/c20-validation-sequence-results.json`,
상세 lint 결과는 `c20-lint-results.json`에 보존한다.

9개 PR의 전체 Native/fresh WASM 시각 검증은 **진행 중**이다. 이 테스트 통과를
시각 수용·PR 승인으로 판정하지 않는다. 문서별 실제 Print 세대, 전체 TSV, 대표 review와
직접 확대 판독을 대조한 뒤 이 PR의 적용 범위를 최종 판정한다. 현재 head의 결과는
`c20-native-whole-corpus-results.json`과 `c20-wasm-whole-corpus-results.json`에 기록한다.
완료되지 않은 결과 파일의 부재·일부 레코드는 통과가 아니다. 원격 통합 PR·CI·merge와
후속 처리는 아직 진행하지 않았다.


### C21 Canvas 점 페인트 보정 후보

현재 소스에는 C20 이후의 Canvas 가운뎃점 보정이 있다. 일반/효과 페인트가
SVG·Skia와 같은 전진 상자 중앙 및 점 기하 상수를 소비한다. 근거·실제 호출
차이는 [#7815 구현 검토](pr_7815_review_impl.md)의 최신 문장부호 항목을 따른다.
C20 Native 주요24문서·325쪽은 완료했지만 미달 문서가 남았다. C20 전체 WASM과
뒤 추가 corpus는 새 소스와 섞이지 않도록 중단됐다. C20 완료 증거는 그대로 보존한다.

보정 후보의 prepare·fmt·Native/WASM/all-target Clippy·workspace build와
고정 base 대비 manifest/unit-tier **8단계 preflight는 모두exit0**다.
새 code commit의 정식 회귀·fresh WASM 실제 재생·관련 전쪽 시각 검증은 아직
필요하며, 이전 C20 결과를 보정 후보의 최종 통과로 옮겨 적지 않는다.
원격 PR·CI·merge와 후속 처리는 미실행이다. 상세 후보와 명령은 ignored
`c21-canvas-dot-source-candidate.json`, `c21-canvas-lint-preflight-results.json`,
`c21-c20-source-supersession.json`에 보존한다.


## 상태와 검토 범위

검토 진행 중이며 수용·승인 판정은 아직 하지 않았다. 범위는 **문장부호 언어 슬롯**다.

- branch: `review/planet6897-followup-20261011`
- 초기 검토 base(과거 단계): `bfa28a77a2714482f647ee6a7b9aa7f7876d8404`
- 원 PR head: `f2a334bc4e90fbedd021e59f4cf2420cbcfbdb21`
- 기본 경로: `collaborator_external_pr` 9.1.1, 주 작업공간 체리픽 통합
- 보조 경로: intake, local validation, visual fixture, multi PR, rework

| 원 고유 commit | 적용 commit |
| --- | --- |
| `27fed2286618a993ca961692a119f914f22db8c1` | `800fcc433100f6772680b97da8870685e2cbaa2a` |
| `f2a334bc4e90fbedd021e59f4cf2420cbcfbdb21` | `5895c485bc470c61527ebc5df2447099338fa256` |

원 fork history를 보존하고 중복 선행 commit·merge commit은 다시 적용하지 않았다.

## 구현 주장과 확인할 경로

슬롯 감지·neutral carry → char advance → run boundary·bbox → Native/Canvas/Skia paint를 추적한다. 저장 탭 wrapper와 글꼴 환경의 명시적 대체/결측 glyph chain 구분을 보존했다.

## 독립 기대값과 검증 계획

curly quote 원본·Print 전쪽과 배치 순서를 확인한다. TAC 각주 flush의 이미 누적한 폭을 다시 inline_x에 더하던 경로를 제거했다. marker 앞뒤 텍스트와 저장 탭을 관계로 검사한다. coverage hash와 baseline 증가는 최종 head에서 재측정한다.

최종 통합 source의 해당 lint·회귀·Native/fresh WASM 전쪽 TSV·대표 review 확대 판독이
완료되기 전에는 통과로 판정하지 않는다. 기여자 PNG·기여자 CI는 최종 통합 head의 증적을
대신하지 않는다. raw TSV·중간 JSON·로그는 ignored
`output/pr-review/planet6897-followup-20261011/`에 보존한다.

입력/Print SHA, 실제 source SHA·명령·관측값과 `충족 / 미충족 / 미검증 / 비해당`
판정은 실행 결과로 이 기록을 보완한다. 필요한 오늘할일은 최종 검토 묶음과 함께 통합 PR에
포함하며 별도 기록 PR을 만들지 않는다.

## 전체 회귀에서 확인한 분류 원장 변경

`ab5112e29a2a23535525e4fe1dc96936defbe4e1` 전체 회귀에서 #4962의 공개 분류 원장
검사가 실패했다. 별도 aggregate 실행으로 3개 원본을 재측정했다. HWP/HWPX format parity
2개는 counts·categories·두 hash가 기존 원장과 같았다. 수출입 현황 원본은 문장부호의
언어 슬롯 경계가 바뀌어 source run 합이 4,573→4,585였고, HFT 원본 메트릭을 사용하는
글리프 대체 경로에서 metricFace 보고도 달라졌다. #7701까지의 source
`f22cdb0359cd0c0c400e1b0348c7f8c63f53be0c`와 비교한 원문 aggregate를 보존했다.

전후 coverage 16,685·layout 16,872·paragraph 2,565 및 모든 category 합·width source별
문자 수는 동일하다. 바뀐 분류 키·run 소유로 canonical aggregate/legacy hash가 달라져
원장의 run 합과 두 hash만 재측정값으로 갱신한다. 배치 좌표·실루엣 허용치·겹침 baseline은
이 작업으로 바꾸지 않는다. 자료는 ignored 검토 output의
`coverage-current-*.json`, `coverage-through-7701-overlay-and-policy-heuristic-hwp.json`,
`coverage-language-slot-run-boundary-differences.json`이다. 강제 재컴파일한 주 작업공간의
production 라이브러리에서 기존 정식 분류 검사 원본을 실행해 PASS했다
(`coverage-formal-test-after-verified.log`). 최종 source의 전체 회귀·lint와
curly 원본의 전쪽 Native/fresh WASM 시각 미달 보정은 여전히 필요하다.

기여자 commit이 갱신한 `tests/golden_svg/issue-147/aift-page3.svg`는 현재 어떤 활성
snapshot 검사에서도 사용하지 않는다. 전쪽 시각 증거 없이 과거의 절대 SVG 배치를 다시
고정하지 않도록 고정 base의 자료를 보존해 해당 갱신을 최종 diff에서 제거했다.


## 잉크 없는 자리차지 표 호스트의 독립 대조

현재 curly 원본은 전체 Native/fresh WASM 82.12343%로 미충족이다. 문장부호 폭과 별도로
원본 pi9의 양수 들여쓰기를 가진 빈 호스트 줄이 사라져 뒤 pi10 제목이 앞당겨진다.
동일 표를 쓴 정상 연속 paraPr의 한컴 Print 대조에서 양수 첫줄 들여쓰기와 명시적 공백은
호스트 줄을 유지했고, 정렬만 변경·내어쓰기·왼쪽 여백만 변경한 대조군은 유지하지 않았다.
글자의 잉크 유무로 줄 공간을 판정하던 가정을 재검토한다. 세로 offset 대조에서 줄 높이가
표 위에 들어가는 경우에는 호스트 줄이 표 뒤 꼬리가 되지 않는다.

공통 쿼리 `reflow_float_host_line::trailing_advance`의 입력은 원본 문단·종단 글꼴·
문단 줄간격·자리차지 밴드 시작이다. 표 `host_spacing`의 측정/fit과 실제 표 배치 후 간격이
같은 결과를 소비한다. 수용된 `ParagraphFloatPlacement`가 있으면 그 `occupied_bottom`이
최종 흐름 원점이므로 측정 after에 포함한 줄을 paint에서 다시 가산하지 않는다.
저장 줄·여러 개체·명시적 개행은 각자의 기존 소유 경로를 유지한다.

자료는 ignored output의 `curly-float-flow-independent-contract.json`,
`curly-float-flow-vertical-probe-print-observations.json`,
`curly-float-flow-native-before-observations.json`이다. 드문 style200으로 만든 최초 진단은
무효 입력이므로 제외했다. 구현 후보의 원본/대조군 재출력·전쪽 90%·확대 판독과
최종 통합 검증은 진행 중이다. 양수 offset의 큰 빈 공간에서 뒤 본문을 표 위에 배치하는
대조군은 기존 출력도 잘못되어 별도 실제 호출 경로를 계속 확인한다.


첫 후보의 전체 Native 원본은 82.12343→98.50387%이고 기존 quote 전진폭 검사는 PASS다.
`curly-host-line-candidate-native/curly_quote_latin_slot/review/`의 원본 전체와
수정 전/후/Print 동일 영역 확대에서 표 뒤 제목·본문의 줄 소속과 간격 복원을 확인했다.
15개 독립 대조군에서도 점수만으로 완료하지 않았다. 공백 대조군의 실제 문단 조각과 표
꼬리의 중복 계상을 직접 찾아 후보를 수정하고 있다. 공백 등 실제 텍스트가 있는 호스트는
기존 `PartialParagraph`가 줄을 소유하며, 새 쿼리는 텍스트가 없는 호스트의 들여쓰기 줄만
표 꼬리에 게시한다. 첫 후보의 공백 결과는 수용 근거에서 제외한다.

Native Clippy 첫 후보는 통과했지만 수정 후보의 다시 빌드·재검사와 fresh WASM은 아직
필요하다. 큰 양수 offset의 앞 빈 공간에 다음 문단을 넣어야 하는 대조군과 음수 offset
대조군의 작은 흐름 차이는 점수가 90%를 넘더라도 정확한 배치로 판정하지 않는다.
최종 통합 검증과 나머지 #7672·#7701 전쪽 미달은 여전히 남아 있다.


중복 소유를 제거한 두 번째 후보는 production 라이브러리 실제 재빌드(6분51초)와
Native Clippy(1분20초)를 통과했다. 16개 한쪽 입력 전체 Native TSV를 다시 산출했으며,
원본 98.50387%가 유지되고 공백 대조군은 기존 문단 조각의 흐름으로 복원됐다.
이 재실행의 원문은 `curly-host-line-candidate-2-native-results.json`이며,
정확한 dirty source 해시는 `curly-host-line-candidate-2/provenance.json`에 고정했다.
15개 대조군의 최저 94.50988%는 큰 양수 offset의 잘못된 뒤 본문 소속을 면제하지 않는다.
공백/음수 offset 대조군도 기존 작은 간격 차이가 남아 독립 Print와 완전 일치라고
기록하지 않는다. fresh WASM·정식 관계형 회귀 추가의 선행 조건은 아직 진행 중이다.


두 번째 후보의 기존 quote 정식 검사도 실제 production 라이브러리에서 다시 실행해
1건 PASS했다(`curly-host-line-formal-test-after-2.log`). 포맷·Native Clippy·WASM lib
Clippy와 고정 base `bfa28a77` 대비 unit tier 검사를 통과했다. 소스 변경 뒤 낡은 파생
harness로 수행한 첫 manifest 검사는 drift로 실패했고, `--prepare`로 파생 suite를 다시
준비한 뒤 manifest와 포맷 check 모두 통과했다. 파생 파일은 PR에 stage하지 않는다.
워크스페이스 build/all-targets Clippy·전체 회귀·Skia·Studio 브라우저와 fresh WASM은
최종 source의 추가 필수 게이트로 남아 있다. 최신 upstream fetch도 동일 base였다.


## 새 production source의 fresh WASM·브라우저와 관계형 회귀

`467bfd1d87a8801632a253c595065d58edb63138`의 루트 기본 release web package를 고정
`target/pr-review`에서 새로 빌드했다(18분01초, exit0). 루트 pkg와 Studio public의
JS/WASM SHA-256이 일치하며 새 package는 ignored `curly-host-line-fresh-wasm-candidate/`에
동결했다. WASM SHA-256은
`f9fd949a405793a9cf667821c264b9f9e4b2448f3d51577a831eb50393f26a02`다.
원본 전체 한쪽 fresh WASM TSV98.50387%가 Native98.50387%와 같았다.
WASM 실행의 font policy·note metadata 조회는 동결한 AB CLI(동일 font 코드)를 사용했고,
실제 WASM 기하는 새467 package에서 산출했다. 최신 Native CLI도467 production으로
별도 빌드·동결했다. 마지막 통합 증적에서는 이 최신 CLI와 새 package를 사용한다.

실제 Chrome·Vite의 HWPX 로드 완료를 확인한 뒤 WASM Canvas2D API로 출력했다.
Canvas 출력1쪽, 실제 잉크17,727픽셀과 독립 Print98.25768%를 확인했고 review에서
표 뒤 제목·후속 본문의 같은 소속과 간격을 직접 판독했다. 최초 진단은 초기 빈 HWP를
성급히 캡처해 실패했으므로 제외했으며 SourceFormat=hwpx 로드 완료를 기다리도록
진단을 고친 재실행은 exit0이다. 로그·해시·Canvas TSV·review는
`curly-host-line-browser-canvas/`와 `curly-host-line-browser-canvas-2.log`에 있다.

이 선행 증거 뒤 `tests/cases/issue_reflow_empty_float_host_line.rs`에 정식 회귀를 추가한다.
뒤 제목의 원본 소유를 찾아 그 앞 표와 제목 줄 사이에 선언 종단 글꼴의 줄 높이가 들어가는지
확인한다. 바깥 아래 여백은 원본에서 가져오고 고정 페이지 좌표·고정 px 영역·SVG 해시는
쓰지 않는다. 실제 수정 전 AB production library에서 gap1.88 < declared em13.333으로
의도한 assertion FAIL, 수정 후 PASS다. 같은 원본의 들여쓰기 없는 표 뒤 독립 빈 문단이
가짜 호스트 줄에 의해 밀리지 않는 정상 대조도 PASS했다. 실제 두 검사 로그는
`curly-host-line-relational-before-two.log`와 `curly-host-line-relational-after-two.log`다.
정식 suite 재준비·통합 전체 실행은 아직 남아 있다. 큰 offset 등 별도 대조의 남은 차이를
이 원본·정상 대조의 통과로 면제하지 않는다.


파생 suite 재준비 뒤 실제 `run-rust-test.mjs`가 새 모듈을
`regression_suite_010::issue_reflow_empty_float_host_line`로 해석했고,
nextest가 **2건 실행·2PASS·205SKIP, exit0**으로 끝났다. 0건 실행을 통과로 세지 않는다.
두 검사 최종 본문은 수정 전 library에서 1FAIL/정상대조1PASS, 수정 후2PASS다.
같은 production source의 Studio 재실행은1,834PASS/0FAIL/2SKIP이고 TypeScript exit0이다.
Node 전용 package가 필요한 기존2개 skip은 web WASM·브라우저 검증으로 대체 실행했다고
기록하지 않는다. 전체 Rust/Skia·나머지 미달 문서·통합 PR/CI/merge는 아직 남아 있다.


## Node WASM·Studio 누락 검사 완료 — 2026-10-12

같은 production `467bfd1d8`의 캐시된 release Rust WASM 입력에서
`CARGO_TARGET_DIR=/home/tsjang/rhwp/target/pr-review scripts/wasm-pack-locked.sh --target nodejs --out-dir pkg-node --no-opt`
를 실제 실행했다(exit 0, 28.08초). Node용 바인딩이 필요한 2건까지 포함해
Studio `npm test`는 **1,836 통과 / 실패 0 / 건너뜀 0**이다. 기존 web fresh package와
Node package를 구분하며 Node no-opt 빌드를 web SVG sweep 증거로 바꾸어 사용하지 않았다.
브라우저 Canvas legacy/layer 비교는 KTX·사업계획서·TAC 대조군 3건 모두 통과했다.

원문 로그: ignored output의 `curly-host-line-fresh-node-wasm-build.log`,
`curly-host-line-studio-unit-with-node-wasm.log`, `curly-host-line-studio-render-diff.log`.
전체 통합 Rust·Skia와 나머지 문서의 시각 게이트는 별도로 완료해야 한다.


## 원 PR의 최신 동기화 확인 — 2026-10-12

최신 fork head는 `e932486256b8b141f56834d4ea8d7304416bcc48`다. 기존 검토 head와 고정 BFA를 제외한 새 commit은
upstream/devel 동기화 merge 하나뿐이며 새 작성자 고유 commit은 없다. 기존 적용 고유
commit과 출처 표는 유지한다. fork의 합쳐진 슬롯 run 분할·저장 탭·말미 공백 조건을 대조했다. 명시적 환경의 휴먼명조 프로그램 신뢰는 BFA의 검증된 원 hmtx 근거를 현재 후보에 유지한다.

combined merge diff와 graph 근거는 ignored
`pr-7815-latest-merge-combined.diff`, `source-pr-refresh-merge-review.json`,
`nine-source-pr-live-20261012.json`에 있다. 원 PR 상태는 OPEN이며 원격 devel은 BFA다.


## Canvas·Skia 따옴표 추가 보정 — 2026-10-12

production `467bfd1d8`에서 동일 입력 `curly_quote_latin_slot.hwpx`의 교정된 실제 글꼴을
공급해 Native/fresh WASM 전쪽 TSV를 재산출했다. 17개 소형 문서 양쪽 최저는
99.56697%이고, 이 따옴표 문서 Native/fresh WASM은 99.65006%다. 실제 Canvas는
Studio 등록 글꼴 목록에도 같은 프로그램을 등록해야 정책상 HY신명조로 치환되지 않는다.
등록 후 전체 쪽은 99.09501%, Skia Print/명시적 font-path 출력은 98.98308%이나,
4개 probe 문단 확대 판독에서 원 글리프가 다시 좁아지는 차이를 검출했다. 점수만으로
수용하지 않았다.

원인은 `text_measurement.rs`의 `metricTrueTypeGlyph` 생산 → 문자 전진폭 →
`forces_halfwidth_punct_paint` → Canvas/Skia의 0.5배 paint 분기다. 판정이
`embeddedMetric`이면서 1em 이상인 경우만 원 글리프로 인정하여, 검증된 휴먼명조
프로그램과 원래 좁은 Times 글리프 모두 측정 결과와 달리 다시 축소됐다.
판정이 두 원 프로그램 폭 경로를 그대로 소비하도록 보정한다. 저장 IR·문자 원점·
기존 HFT 호환 overlay와 다른 구두점의 적용 조건은 바꾸지 않는다.

before actual 출력과 font program 해시는 ignored
`correct-face-curly-browser-canvas-registered/`, `correct-face-curly-skia-print/`,
`correct-face-curly-registered-owned-lines-enlarged-review.png`에 보존했다.
정식 관계형 검사 후보는 ignored `issue_7702_curly_quote_paint_advance.pending.rs`에
보관하며, 수정 후 Native/fresh WASM 전쪽 TSV 및 실제 Canvas/Skia 확대 판독이
충족되면 `tests/cases/`에 등록한다. before/after 함수 검출은 unchanged predicate의
공개 범위만 넓힌 고정 e72 worktree와 보정 source를 대조할 계획이며 아직 미실행이다.


### 실제 before 검출과 정상 대조군

고정 e72 production에서 paint 판정의 공개 범위만 넓힌 library로 같은 후보 검사를
컴파일했다. 실제 실행은 의도한 휴먼명조 TrueType 따옴표의 재축소 때문에 FAIL했고,
기존 호환 글꼴의 좁은 paint 정상 대조군은 PASS했다(1 PASS·1 FAIL, exit101).
컴파일 실패가 결함 재현으로 섞이지 않았다. `curly-paint-before-public-only.log`와
library/case 해시 `curly-paint-before-public-only-provenance.json`을 보존했다.
보정은 신뢰된 원 프로그램 폭과 기존 전각 메트릭에 한정하며 미확인 호환 폭을 보존한다.
수정 후 실행·시각 gate는 진행 중이고 후보는 아직 정식 source에 등록하지 않았다.


## 실제 출력의 Times 신뢰 누락 재검토 — 2026-10-12

`167c6a4fc`에서 완성한 새 web WASM 해시는 `ab0451e9e1f96514a5dc582075af20cc7f83a0e8f8be9d98b2d7bfacba6ea47d`이며
루트 pkg와 Studio public이 같다. 같은 production의 작은 문서17종 전쪽 Native/fresh WASM은
모두90% 이상이다. 따옴표 원본 전체1쪽은 Native/fresh99.65006%, Skia99.40060%,
Canvas99.51564%다. Native CLI는 copy 당시HEAD가cddd이고 rustfmt 변경이 함께 있었으며
그 실제 source는aa46 커밋과 같다. WASM의 실제 build head는167c6a4fc다.

**이 수치를 확대 판독의 완료로 바꾸지 않는다.** 새 Canvas/Skia/Print의 원본 따옴표4줄을
같은4배율로 읽어 Times가 아직 눌린 것을 확인했다. 실제 Canvas fillText trace는 휴먼명조
scaleX=1, Times scaleX=0.5였다. 독립 Print의 Times CID TrueType 프로그램과 공급한 실제
Times 프로그램의 네 곡선 따옴표는 전진폭·분해 윤곽선이 전부 동일하다(작은따옴표682/2048,
큰따옴표909/2048). Times는 원래 좁은 글리프이므로 다시 절반으로 줄일 근거가 없다.

원인은 실제 TTF 선언 → `style_resolver::metric_widths_verified_face` 누락 → 실제 TextRun의
`font_metric_trusted=false` → 공용 paint predicate → Canvas/Skia0.5배 축소다.
Times를 독립 프로그램·Print로 확인한 face로 인정하는 보정을 추가했다. 기존 TTF 선언·
치환 여부 조건은 그대로 소비하며 HFT와 확인되지 않은 호환 선택의 신뢰를 높이지 않는다.
helper를 직접 trusted=true로 만든 검사는 통과했지만, 원본을 실제 DocumentCore로 읽는
추가 후보는 기존compiled library에서 Times의 실제 run 신뢰 assertion으로 FAIL했다
(2PASS/1FAIL,exit101). 후보는 아직ignored output에 보존하고 새 source의 시각 게이트 뒤
정식 `tests/cases/`에 등록한다. 원본의 쪽·TextRun 소유·각 face의 네 글자 보존까지 검사한다.

증거는 `curly-paint-fixed-small-{native,wasm}-results.json`,
`curly-paint-fixed-browser-skia-owned-lines-enlarged-review.png`,
`curly-paint-fixed-browser-canvas-quote-trace/result.json`,
`curly-times-print-vs-supplied-program.json`, `curly-times-actual-document-before.log`다.
이 Times 보정 뒤의 재빌드·관련 시각 재검증·lint·정식 suite는 미검증이며 이전 산출물의
통과를 새 source의 통과로 쓰지 않는다.


## Times 보정 검증과 정식 검사 — 2026-10-12

production `4cebc8d095b64ce2786bc5071d31ad5a76a09bb1`의 새 web wrapper는24분05초,
exit0으로 끝났다. 동결한 `curly-times-fixed-web-pkg`와 루트pkg/Studio public의 WASM은
SHA-256 `588805a853cc34215e12994c0bf1609cc1dc23bf7bba601f16392e4937c5422c`로 같다.
같은 production의17문서 전쪽 Native/fresh는 모두PASS·최저99.56697%다.
따옴표1쪽은 두backend99.65006%, Skia99.40018%, Canvas99.51551%다.
실제 브라우저fillText trace에서 두face의 곡선 따옴표가 모두scaleX=1이었다.
Skia의 보정 전/후/Print 네줄을4배율로 직접 읽어 Times의 원 곡선 폭이 복원된 것을 확인했다.
단순 점수는99.40060→99.40018로 소폭 낮아졌어도 직접 판독과 실제scale가 개선을 입증한다.

[정식 검사](../../../tests/cases/issue_7702_curly_quote_paint_advance.rs)는 원본의 실제
TextRun 소유·face별 네글자 보존·프로그램 폭 관계·paint 선택과 호환 대조군을 검사한다.
원래paint library에서1PASS/2FAIL, Human paint 보정 후 Times 신뢰 보정 전2PASS/1FAIL,
현재default library3PASS다. 정식generated suite025에서는 실제3건 실행·3PASS·211SKIP,
exit0이었다. 프레임 절대px·SVG해시·현재구현의 계산값을 배치 정답으로 쓰지 않는다.

Native/WASM lib/all-targets 세Clippy와workspace build·fmt check가PASS다.
첫prepare 뒤fmt가새모듈을 포맷하면서 전체generated header해시가달라져 manifest가
drift로FAIL했으며 원문로그를 보존했다. fmt종료후다시prepare한manifest pinnedBFA,
fmt check, unit tier pinnedBFA는 모두PASS다. 최종inventory는1454sources/6173staticattrs다.
code source가같은이후head의차이는 신규검사·보존·문서이고 기존생산산출물을 다른source로
보고하지 않는다. 전체Rust 최종실행·다른PR의시각수용·통합CI/merge는아직남아있다.

증거는 `curly-times-validation-results.json`, `curly-times-fixed-small-{native,wasm}-results.json`,
`curly-times-fixed-browser-canvas/`, `curly-times-fixed-skia-owned-lines-enlarged-review.png`,
`curly-all-actual-document-original-before.log`, `curly-times-actual-document-before.log`,
`curly-times-formal-nextest.log`, `curly-times-formal-lint-results.json`,
`curly-times-formal-reprepared-policy-results.json`이다.


## 最新 production 보정 후 Skia·Node·Studio 검증 — 2026-10-12

production source는 `4cebc8d095b64ce2786bc5071d31ad5a76a09bb1`이며 이후 commit은
검사·검토 기록이다. 현재source의 `cargo test --locked --profile release-test
--target-dir /home/tsjang/rhwp/target/pr-review --features native-skia --lib -- --test-threads=12`는
3864+15+165+2=**4046PASS /0FAIL /12IGNORED**, exit0으로 종료했다.
`issue_2225_missing_picture_placeholder`와 `render_p37_direct_pdf_export`의
공식 추가 Skia 범위는 현재source로 각각 **2PASS /0FAIL**, **4PASS /0FAIL**,
exit0으로 종료했다. library4046건과 추가6건을 구분한다.

root wrapper의 `--target nodejs --out-dir pkg-node --no-opt`가 현재source의 release
WASM 입력에서 Node 바인딩을 생성했다(exit0). 이 Node package까지 반영한 Studio
`npm test`는 **1836PASS /0FAIL /0SKIP**, exit0으로 종료했다. 최적화한 web package의
브라우저·시각 증거와 Node no-opt package의 검증을 구분한다.

ignored 로그: `current-4ce-native-skia-lib.log`, `current-4ce-fresh-node-wasm-build.log`,
`current-4ce-studio-unit-with-node-wasm.log`, `current-4ce-native-skia-focused-results.json`.
이 결과를 최종 통합 전체 Rust 검사 또는 남은 원본의 시각 통과로 보고하지 않는다.


## 현재 fresh WASM의 실제 문장부호 페인트 확인 — 2026-10-13

현재 `e2a525d36`의 fresh package와 실제 Studio 초기화 응답의 JS/WASM 해시를
대조했다. 기존 curly 원본 전1쪽의 실제 Canvas API와 독립2020 Print 비교는
**99.51551%**다. 원본의 따옴표32개는 휴먼명조·Times 프로그램을 사용하며
모두 수평 배율1로 그렸다. 같은 영역4배 확대에서 전각 휴먼명조/좁은 Times의
모양과 앞뒤 `가`의 순서가 유지되고 추가 반폭 축소·겹침은 보이지 않는다.
작은 raster 획 차이는 남아 있으며 나머지 범위의 완료로 확대하지 않는다.
원본에는 실제 U+00B7이 없어 가운뎃점 검출 증거로 재사용하지 않았다.

별도 기존 원본 `k-water-rfp.hwp`의 실제27쪽 문서를 로드한 뒤 첫 점을 소유한
3쪽을 선택했다. 가운뎃점7개는 현재 Canvas에서 Palatino 선언 글리프를
수평0.5배로 그리지만 같은 fresh WASM SVG는 원 도형7개를 그린다.
독립 Print3쪽의 Type3 `/T5`·`/HFT43` 원 프로그램은 전진0.384em,
가로 중심0.192em, 세로 중심0.3515em, 반지름0.057/0.0565em의 원형 윤곽이다.
SVG/Skia의 전진 상자 중앙·공통 점 기하와 Canvas의 글리프 축소 소비 경로가
다르므로 Canvas의 해당 경로 보정과 실제 재생 재검증이 필요하다.
효과 페인트도 별도 글리프 경로여서 적용 조건을 함께 대조해야 한다.

이 진단에서 명시 공급한 집합에 Palatino 프로그램은 없어 실제 대체 글리프의
모양을 정본 Palatino로 판정하지 않는다. 실행으로 확인한 소비 경로 차이와
독립 Print 기하를 기록한 것이며, 원본27쪽의 전쪽 시각 수용이나 현재 변경이
새 회귀를 만들었다는 판정은 아니다. 첫 진단의 스크립트 범위 오류와 빈 초기
HWP를 조회한 두 번째 오류는 보존했고, 실제 다쪽 원본의 로드 완료를 기다린
세 번째 실행은exit0이다. 제품 실패와 진단 실패를 구분한다.

근거는 ignored `c20-current-curly-canvas-print-review/provenance.json` 및
`review/review_001.png`, `quote-enlargement/compare_001.png`,
`c20-middle-dot-paint-consumer-audit.json`,
`c20-middle-dot-independent-print-glyph-records.json`,
`c20-current-middle-dot-canvas-trace-attempt3-command.json`에 연결한다.

## C22 fresh WASM 실제 Canvas 재생 — 2026-10-13

실제 source `c9f0f521cbd2bdc9391e3e5931068a3315e372b5`를 root wrapper로
optimized web package 빌드했다(exit0,18분40초). 고정 공용 target을 사용했으며
`pkg`/Studio public/브라우저 최초 WASM 응답의 SHA 일치를 확인했다. Vite의 JS URL
치환은 기록한 단일 치환으로 검증했다. source·정상4개 face의hash/OS2 descriptor와
브라우저실제font loading을 보존한 진단2개는 모두exit0이고warning은없다.

기존 curly_quote 원본1쪽을 같은 독립2020 Print로 현재 Native/fresh WASM 각각
전1쪽 TSV 재산출했다. 두 경로는99.65006%,1/1쪽이다. 실제 Canvas API PNG의
전1쪽은99.51551%이며,따옴표32개의fillText 가로배율은 모두1이다. 같은4배 확대
영역을 직접 열어 따옴표·한글 및 앞뒤 간격,이어지는숫자/본문 기준선을 대조했다.
큰 장평·기준선·간격 차이는 보이지 않으며 작은 raster 경계 차이는 남는다.
이1쪽은가운뎃점을 포함하지 않아 B7 보정 증거로 대신하지 않는다.

B7는 k-water 실제 원본27쪽의3쪽에서7개run을 추적했다. 이전 C20의
fillText7/arc0 경로 대신 현재는fillText0/arc7이며 같은전진값을 보존한원형 경로다.
직접 Canvas4-face 진단에는Palatino 프로그램이 포함되지 않아 문서 전체의 글리프
일치를 주장하지 않는다. 이 실제 원본의전27쪽 Native/fresh WASM은별도 진행 중이며,
전쪽·독립 Print·직접확대 판독 이후에만 관련회귀검사의 추가/변경과완료를 판정한다.
#7702의넓은저장LineSeg/grid 축도 이가운뎃점진단만으로 종료하지 않는다.

증적은 ignored `c22-fresh-web-pkg/provenance.json`,
`c22-canvas-runtime-observations.json`, `c22-curly-quote-paired-results.json`,
`c22-curly-quote-native-wasm-canvas-results.json`과각실행로그에 있다.
현재 Native/fresh WASM은각 `c22-curly-quote-paired-{native,wasm}/curly-quote/`의
TSV/manifest이고,실제 Canvas대표는 `c22-curly-quote-print-direct-canvas/review_001.png`,
확대판독은 같은 경로의 `compare_001.png` 및 `enlargement-provenance.json`이다.
기존C20점수와새C22실행을구분하며,9PR전체의시각게이트통과로 승격하지 않는다.

## 가운뎃점 실물 전쪽 검증과 브라우저 회귀

C22의 동일 원본 `samples/k-water-rfp.hwp`(SHA-256
`1b4d1169c8e622b7365e9029ced513b13206f5939cf12024438c38d30253c2b4`)을
독립 `pdf/k-water-rfp-hwp-2020.pdf`와 대조했다. 현재 Native와 fresh WASM은
각각 전27쪽 산출·누락 없음·Print27쪽 일치, 최저24쪽94.12523%다.
실제 점이 있는3쪽은 Native/fresh WASM96.92201%, 직접 Canvas96.94814%다.
전쪽 TSV만의 `not_evaluated` 상태를 자동 승인으로 승격하지 않는다.

실제 점7개의 소유 run에서 같은 영역을 골라 Canvas/Print를8배로 직접 열었다.
원형 점의 중심·앞뒤 글자 간격은 같은 위치에 있고, 원의 raster 경계·글꼴 잉크에는
작은 차이가 남는다.24쪽 대표도 직접 열었으며 마지막 표의 회색 바탕·외곽 하단이
Print보다 짧은 차이는 별도 배치 검토로 남긴다. 가운뎃점 paint 수정은 이 표 상자의
해결을 주장하지 않는다.9PR 전체 통합 판정과 #7702의 다른 저장 조판 축도 그대로 보류한다.

[정식 브라우저 검사](../../../rhwp-studio/e2e/middle-dot-canvas-paint.test.mjs)는
실제 WASM `renderPageToCanvas`를 호출한다. 원문 내용으로 대상 쪽을 찾고, 실제 점의
전진·글줄 안에 원형 점 하나가 있는지, 대체 글리프 호출이 없는지, 앞뒤 내용·문단 소유·
순서·쪽 소유가 보존되는지 검사한다. 절대 쪽 번호·px 위치·SVG 해시를 고정하지 않는다.
Canvas 전용 실제 호출을 Native Rust helper 검사로 대체하지 않고 기존 Studio E2E
경로에 두었다. `npm run e2e:middle-dot-canvas-paint`로 실행한다.

동결 C20 `e2a525d36f2923918185f62610869e7c3068b9d0`의 패키지에서 exit1로
fillText7/arc0을 검출했고, C22 `c9f0f521cbd2bdc9391e3e5931068a3315e372b5`의
패키지에서는 exit0으로 fillText0/arc7과 내용 소유 보존을 확인했다. 첫 baseline 시도의
Vite403은 환경 실패로 분리했다. 동일 패키지를 허용된 pkg 하위로 바이트 그대로 임시
복사한 두 번째 실행만 수정 전 결함 증거이며 종료 확인 후 복사본만 정리했다.
원 동결 패키지와 현재 root pkg·public 파일은 보존했다.

원 로그·원본/패키지 SHA·전후 관측은 ignored `c23-middle-dot-formal-before-after-audit.json`,
`c23-middle-dot-formal-before-r2.log`, `c23-middle-dot-formal-after-npm.log`,
`c22-kwater-paired-{native,wasm}/`와 `c22-kwater-dot-three-backend-results.json`에 있다.
최종 대표 증적은 다음 경로에 보존했다.

- [Native review](../assets/planet6897_followup_20261011/c23-middle-dot-canvas/native-review-p003.png)
- [fresh WASM review](../assets/planet6897_followup_20261011/c23-middle-dot-canvas/wasm-review-p003.png)
- [Canvas review](../assets/planet6897_followup_20261011/c23-middle-dot-canvas/canvas-review-p003.png)
- [독립 Print와 점7개 확대 비교](../assets/planet6897_followup_20261011/c23-middle-dot-canvas/canvas-print-owned-dots-enlarged-review.png)

이 단계의 새 변경은 브라우저 검사·실행 alias·증적·검토 기록이다. Native/Rust·Studio
제품 코드와 기존 회귀 기대값은 변경하지 않았다. 위 source SHA의 실제 결과를 새 Rust
빌드로 보고하지 않으며, 최종 production 수정 뒤 영향을 받는 검증은 다시 진행한다.

## 2026-10-14 원 PR head 변동 감사

GitHub에서 이 PR은 OPEN이며 최신 source head는 `6a59bad9e9d762db336aba7e0614085ee39ffb54`다.
통합 브랜치의 기존 적용 범위에 새 체리픽을 추가하지 않았다. 원격 head에는 다른 PR의
merge와 후속 보정도 포함돼 있으므로 두 tree의 전체 차이를 이 PR 고유 변경으로 보지 않는다.
기존 통합 source의 테스트·시각 결과를 최신 원 PR head의 검증으로 보고하지 않는다.
읽기 전용 fetch ref와 commit 목록은 ignored
`output/pr-review/planet6897-followup-20261011/c37-source-pr-remote-head-drift.json`에 보존한다.
현재 통합 branch의 수용 판정·PR·CI·merge는 미완료다.
