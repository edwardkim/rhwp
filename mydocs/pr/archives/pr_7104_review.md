---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-14
---

# PR #7104 — 다단 합성 LineSeg와 그림 감싸기 메인터너 검토

**최종 판정: 메인터너 보정 후 수용 가능.** 보류 사유는 아래에 명시한 통합 보정 코드의 부분 개선 범위에서 해소했다. F1의 추가 Square 그림·본문 교차를 메인터너 보정으로 해소했다. #6970 전체의 제목·본문 단/페이지 소속 차이는 기존 잔여로 남으며, 이 판정은 원 PR의 부분 개선 범위에 한한다.


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

원 PR head `add3a01b4a786da31663cc35746576495a96d357` → 현재 통합 이력의 cherry-pick `b9af4dc2377b27baa66e66d65f383a60dee6a2bd`. 원 저자와 `-x` 출처를 보존했다.


## 대상·rebase·보정 이력

| 항목 | 확인값 |
| --- | --- |
| 원 PR / 작성자 | [#7104](https://github.com/edwardkim/rhwp/pull/7104) / planet6897 |
| source SHA | `add3a01b4a786da31663cc35746576495a96d357` |
| rebase 후 cherry-pick SHA | `b9af4dc2377b27baa66e66d65f383a60dee6a2bd`; 원 저자와 `-x` 출처 보존 |
| 메인터너 보정 | `9f3b1897c + fb1b46e23` |
| 검증 Renderer 코드 | `2f59c89373f497068f9a0bcb2c22730ec7dc7e51` |
| WASM 캡처 도구 | `321f4cdb4`; Visual Sweep `--wasm-pkg`로 직접 캡처 |
| 작업 시작 시 동기화 기준 | `upstream/devel` 및 local `devel`: `037e4906a93e99896daa145a5ee5517824bfeaf4` |
| 작업 branch / target | `review/planet6897-20260914` / `target/planet6897-review-20260914` |
| rebase | 원 18 commit을 최신 기준 위에 재적용, 충돌 없음; 보정 전 head `51498859ef3d14a6a73f38ea3bfab4e9fb2ac9ca` |
| 원본·PDF 보존 commit | `ab3184254`; 기존 입력을 이름 변경해 중복 추가하지 않음 |
| 원 PR 상태 | 2026-09-14 재조회: OPEN, non-draft, source SHA 동일; mergeable / mergeState UNKNOWN |
| 관련 이슈 | [#6970](https://github.com/edwardkim/rhwp/issues/6970); 이번 로컬 작업에서 close하지 않음 |

이번 보정 전에 local devel을 fast-forward하고 review branch를 rebase했다.
rebase 전 이력은 `codex/planet6897-before-maintainer-rebase-20260914`에 보존했다.
누적 대상은 #7094, #7100, #7104, #7111, #7112, #7113, #7115, #7116, #7117, #7120, #7131, #7132이며
원 16 commit의 저자·출처를 유지했다. draft #7098은 제외했다. 원 reviewer는 jangster77이다.
원 PR의 CI와 이번 통합 코드 검증을 구분하며, 원격 push·통합 PR·merge·comment는 이 기록의 완료 항목이 아니다.

## 보류 사유와 메인터너 해결

**원인:** 앞 단의 문단에 연결된 그림이 원본 수평 오프셋에 따라 다음 단에 놓여도, 단 전환에서 배제 영역이 지워졌다. 또한 NO_LS 일반 본문은 그 영역을 줄 채움에 전달하지 않았다.

**보정:** 같은 물리 페이지에서는 그림 배제 영역을 단 사이에 보존하고 새 페이지에서는 비운다. 원본 NO_LS 그림의 문단 기준 좌표를 해석하고, 일반 본문의 frame이 만든 행과 높이를 fit와 paint가 함께 소비한다. 원본 IR의 LineSeg는 바꾸지 않는다. 빈 후속 lane은 다음 가시 글줄이 아니므로 마지막 제목을 양쪽 정렬로 벌리지 않는다.

**직접 검증:** 원본 첫 쪽 그림 pi=6/12/19의 bbox를 보존하면서 가로·세로 교집합이 각각 0.5px를 넘는 가시 TextLine 교차가 각각 4→0, 총 12→0이다. 종전 review의 1→3은 교차한 그림 개수이고 이 12는 교차한 글줄 개수다. 그림 3개와 본문을 유지했으며 BehindText 대조군에서는 겹침을 허용한다. 2쪽 제목 pi=88/93/99는 12pt 전각 자연 폭 64/32/48px를 유지한다.

**범위·잔여:** 문서는 한컴과 같은 3쪽이지만 전체 시각 동등성을 뜻하지 않는다. 기존 제목·문단 페이지 소속 차이는 남는다. #6970 종료 근거로 사용하지 않는다. 새 clipping·좌표 clamp·fixture 허용치 완화로 교차를 감추지 않았다. Sweep의 square_wrap_text_overlap flag 자체는 1~2쪽에 남는다. 1쪽 pi=6/12/19는 본문 관통에서 edge_clearance_loss로 바뀌었고, 그림 오른쪽과 글줄 시작의 반올림 좌표 차이는 0.0/0.0/-0.1px다. 독립 PDF의 첫 그림도 오른쪽 경계와 본문 시작이 x=672.2px로 맞닿는다. 이를 관통 0건과 구분하며 Sweep 전체 flag=0으로 보고하지 않는다. 2쪽 pi=78/ci=2 그림의 기존 physical_overlap은 보정 전 4행에서 최종 3행으로 줄었으나 남는다. 이번 F1의 1쪽 추가 교차 보정과 구분하며, #6970 전체 해결을 위해 후속 검토해야 한다.

| 조판 원칙 공통 항목 | 검토 결과 |
| --- | --- |
| 원인·일반성 | 원본 속성 및 독립 한컴 PDF/구문 계약을 사용. 문서 ID·문자열별 생산 분기 없음 |
| 측정·배치 일관성 | 조판 보정은 동일 frame의 행·폭·높이를 fit/paint에 공유. 수식 보정은 tokenizer 앞의 정규화 계층 |
| 줄 소속·점유 높이 | 원본 저장 LineSeg와 NO_LS를 구분하고 source IR에 렌더 전용 행을 덮어쓰지 않음 |
| 독립 증거와 반례 | 같은 원본의 한컴 PDF, 보정 전 코드, 최종 코드 및 반례를 분리해서 확인 |
| baseline·허용치 | 메인터너 보정으로 golden/ratchet/허용치를 완화하지 않음 |
| 주장·검증 경계 | 이 문서의 최종 code SHA와 실제 실행 결과를 기록. 기존 차이는 해결로 세지 않음 |
| 입력 보존 | 다음 표의 Git/LFS 원본과 PDF를 재사용; 다운로드 폴더만 가리키지 않음 |

## Visual Sweep 판정과 증적

렌더 비교는 [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md)의
`scripts/visual_sweep.py`로 수행하고 생성된 비교·overlay PNG를 직접 읽는다.
단순 페이지 수·픽셀 점수 또는 과거 candidate의 성공만으로 통과시키지 않는다.
21개 입력 363쪽의 전체 SVG·render tree·문자/레이아웃 ledger를 생성하고 보정 전 통합과 대조했다.
19개 문서는 SVG가 전부 동일하고 synth 1~2쪽과 80168 21/75/76/77/108/109쪽, 총 8쪽이 변경됐다.
실제로 변경된 페이지와 p49/p108 등의 대조 페이지는 최종 바이너리로 다시 Sweep했다.

| 입력·쪽 | 직접 확인한 변화/대조 | 남은 차이 |
| --- | --- | --- |
| synth 1 | 그림 pi=6/12/19의 물리 bbox 유지, 각 4개씩이던 TextLine 교차가 모두 0. 본문을 숨기지 않고 오른쪽 배제 영역을 따라 채움. | 기존 제목·본문 시작 높이와 단 소속 차이는 남음 |
| synth 2 | 빈 오른쪽 lane 때문에 마지막 제목을 벌리던 현상 없음. 자연 폭 64/32/48px 확인. | 한컴과의 단·쪽 소속 차이와 pi=78/ci=2 그림의 기존 physical_overlap 3행 잔존(보정 전 4행) |
| synth 3 | 보정 전 통합본과 SVG 동일; 전체 비교를 잔여 증적으로 보존. | 전체 시각 일치 아님 |

`_sweep.png`와 `_overlay.png`는 Visual Sweep 원출력이다. `_three_way.png`는 같은 Sweep의
PDF·최종 PNG에 보정 전 SVG를 동일 webfont rasterizer로 렌더한 결과를 나란히 놓은 보조 비교다.

- [pr7104_maintainer_synth-no-lineseg_p001_overlay](../assets/pr7104_maintainer_synth-no-lineseg_p001_overlay.png) — SHA-256 `62e51fb6fa78c230eab84305e95fae9fc139b03c70af6d672b4b565d06318de0`
- [pr7104_maintainer_synth-no-lineseg_p001_sweep](../assets/pr7104_maintainer_synth-no-lineseg_p001_sweep.png) — SHA-256 `af8669f6191ba1a858d4fd8eda7e424aa593ad21dce083eed3caf3471c5c4e9b`
- [pr7104_maintainer_synth-no-lineseg_p001_three_way](../assets/pr7104_maintainer_synth-no-lineseg_p001_three_way.png) — SHA-256 `bb765d85cb6fbca80b1fc772865a511bef159ebcf6c8e3e74dcf36559ae3c75f`
- [pr7104_maintainer_synth-no-lineseg_p002_sweep](../assets/pr7104_maintainer_synth-no-lineseg_p002_sweep.png) — SHA-256 `a55337d82c76c3ebe5ae31fa24a11cd338d78b03fcb52b15608b352ea6ac254a`
- [pr7104_maintainer_synth-no-lineseg_p003_sweep](../assets/pr7104_maintainer_synth-no-lineseg_p003_sweep.png) — SHA-256 `7b07895f9f7f990e470ac5c810a55fe277f83361f0a52b5d3f624a0d36bf8068`
- [pr7104_maintainer_wasm_synth_p001](../assets/pr7104_maintainer_wasm_synth_p001.png) — SHA-256 `559120170b2b512e331141902f616e891c7bcafc949c17305dea65a82d3ee947`
- [pr7104_maintainer_wasm_synth_p001_overlay](../assets/pr7104_maintainer_wasm_synth_p001_overlay.png) — SHA-256 `4852fbb0b679c5364a8a260799d56dc3804410ce981b7052acae3f620a51c725`

Chrome 152.0.7977.83에서 새 WASM의 SVG API로 5개 원본을 렌더했다. 텍스트 노드의 속성과 문자열을 Native와 직접 대조했다.

| 입력 | 확인 쪽 / 전체 쪽 | Native / WASM Text 노드 | 속성·문자열 |
| --- | --- | --- | --- |
| reg80168 | 108 / 157 | 319 / 319 | 일치 |
| soil | 1 / 6 | 752 / 752 | 일치 |
| transistor | 2 / 11 | 560 / 560 | 일치 |
| table-text | 1 / 1 | 140 / 140 | 일치 |
| synth | 1 / 3 | 883 / 883 | 일치 |

최종 브라우저 증적은 `321f4cdb4`의 Visual Sweep `--wasm-pkg` 경로로 다시 생성했다. 같은 Chrome WASM 문서에서 SVG와 render tree를 만들고, Sweep 내부에서 같은 원본의 CLI 글꼴 별칭과 공통 webfont 정책을 적용한다. 별도 HTML 캡처에서 발생한 휴먼명조 문제를 우회한 파일을 최종 증적으로 사용하지 않는다. `_wasm_*.png`도 이제 Sweep의 PDF 나란히 비교 또는 overlay 원출력이다. 원 SVG는 `wasm/raw_svg`, 실제 Chrome 버전·쪽 수·해시는 각 manifest에 남긴다. 5개 입력의 14쪽을 직접 Sweep했고, 전체 178쪽의 WASM SVG 텍스트 속성·문자열과 render tree는 Native 결과와 모두 동일했다. 새 도구의 Python 테스트 50개와 webfont 테스트 6개가 통과했다.

기존 before는 `48afe0f95`, 최초 base는 `93ffc3dd5`에서 만든 동결 바이너리다.
각각 rebased 보정 전 `51498859e`, 최신 base `037e4906a`와 Rust source·Cargo 입력의 diff가 없음을 확인했다.
최종 바이너리는 `2f59c89373f497068f9a0bcb2c22730ec7dc7e51`에서 빌드한 뒤 동결하고 변경 source의 SHA-256을 검증 종료 시 대조했다. 새 한컴 변환을 이번에 다시 했다고 주장하지 않는다.

## 입력·기준 출력 보존

확인 tree: `2f59c89373f497068f9a0bcb2c22730ec7dc7e51`. 다음 SHA-256은 LFS pointer 문자열이 아닌 실제 파일 바이트의 해시이며,
모두 Git에 포함된 원본이며 LFS 파일은 실제 바이트의 oid도 확인했다. full/OVR 공통 입력은 아래 재현 명령의 저장소 fixture 집합을 사용했다.

| 저장소 파일 | 출처·역할 | SHA-256 |
| --- | --- | --- |
| [tests/fixtures/issue_6970/synth_no_ls_square_wrap.hwp](../../../tests/fixtures/issue_6970/synth_no_ls_square_wrap.hwp) | 한컴 입력; synth-no-lineseg | `31a5b76148718d92e3fd150f4d68b84b5005b02d16863c4735a6178eb02f2799` |
| [pdf/pr-planet6897-20260914/synth-no-lineseg-hwp-2020.pdf](../../../pdf/pr-planet6897-20260914/synth-no-lineseg-hwp-2020.pdf) | 독립 한컴 PDF 3쪽; 직접 sweep 1-3쪽 | `70b63c11514d3d927418c1f116a0cc421614e7db5e9de312bd0d9f964b34e9a3` |

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
[익명화 다단 원본](../../../tests/fixtures/issue_6970/README.md)에 보존했다. 앞선 검토에서 확보한 MCP 변환 6건은 engine 2020,
Hancom 12.0.0.4605로 start → status(queued/running/succeeded) → download → SHA 확인까지 수행했다.
신고자 transistor PDF는 Hancom 2022의 기존 11쪽 출력이며 새 MCP 출력으로 오인하지 않는다.
인증 정보·임시 SVG/JSON·중간 로그는 커밋하지 않는다.


## 최종 코드 검증

| 검증 | 실제 결과 |
| --- | --- |
| focused nextest | 105 tests run: 105 passed, 9800 skipped |
| 전체 nextest | 9854 tests run: 9854 passed (2 slow), 51 skipped |
| Native Skia lib | ok. 3930 passed; 0 failed; 13 ignored; 0 measured; 0 filtered out; finished in 23.42s; ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s; ok. 165 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s; ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s |
| Native placeholder / direct PDF | 2 tests run: 2 passed, 196 skipped / 4 tests run: 4 passed, 189 skipped |
| Clippy native / wasm / workspace | 통과 / 통과 / 통과 |
| workspace build | 통과 |
| manifest / unit tiers | 통과 / 통과 |
| 새 WASM | 통과 |
| OVR 필수 5문서 | 5개 exit 0; 개체 회귀 0 |
| Visual Sweep | reg80168: 157쪽 중 6쪽 변경 / synth-no-lineseg: 3쪽 중 2쪽 변경 |

WASM은 `wasm-pack-locked.sh --no-opt`로 새로 빌드한다. Docker daemon 미가용으로 표준 Docker/wasm-opt
최적화 배포 빌드를 완료했다는 주장은 하지 않는다. OVR은 KTX/exam_math/언어기출/aift/biz_plan의
**devel 대비 개체 회귀 검사**이며 한컴 동등성을 대신하지 않는다.
첫 보정 head `9f3b1897c`에서도 9852/9852가 통과했지만 Sweep에서 추가 차이를 발견해 보정을 고쳤다.
다음 head `fb1b46e23`의 9853개 통과 후에도 p76~77 잔여를 확인해 다시 보정했다.
두 실행 결과를 현재 코드의 최종 검증으로 재사용하지 않는다.
Native placeholder의 첫 개별 호출은 생성 suite 배치가 오래되어 0 tests / exit 4였으며 통과로 세지 않았다.
`--prepare`로 생성물을 재정렬한 뒤 동일한 1304개 case 경로를 포함함을 확인하고 재실행했다.
이미 통과한 전체 테스트의 source·case 집합은 바뀌지 않았다. 생성 harness는 커밋하지 않는다.

```sh
cd /Users/tsjang/rhwp
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all -- --check
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
CARGO_TARGET_DIR=target/planet6897-review-20260914 scripts/wasm-pack-locked.sh --target web --out-dir <외부-pkg-폴더> --no-opt
venv/bin/python scripts/visual_sweep.py --wasm-pkg <새-WASM-web-package> --file-target <식별자> <Git-원본> <한컴-PDF> --rhwp-bin <검증-SHA-바이너리> --pages <검토-쪽> --dpi 96 --out <외부-산출-폴더>
```

생성 suite, 임시 SVG/JSON, 중간 로그, 인증 정보는 커밋하지 않는다.
[보정·후속 단계](pr_7104_review_impl.md)에 다음 절차를 기록한다.

## Merge 후 contributor PR comment 계획

[Visual Sweep GitHub comment 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 따른다.
최종 WASM Sweep의 아래 수치와 앞 절의 직접 시각 판정·남은 차이를 함께 게시한다.
flag 수는 해당 페이지의 자동 후보 유형 수이며 검출된 객체나 결함 개수가 아니다. 96dpi, 픽셀 차이 threshold 32의 결과다.

| synth-no-lineseg 쪽 | 자동 flag 유형 수 | pixel_match | visual_accuracy_proxy_percent |
| --- | --- | --- | --- |
| 1 | 2 (square_wrap_text_overlap, large_ink_region_drift) | 82.89834% | 11.48821% |
| 2 | 5 (square_wrap_text_overlap, column_text_flow_collapse, line_band_drift, column_line_band_drift, large_ink_region_drift) | 88.53523% | 6.03877% |
| 3 | 0 (없음) | 90.98380% | 7.21664% |

흰 여백을 포함한 pixel_match와 잉크 기반 proxy는 자동 비교 지표다. 기능 정확도 또는 전체 한컴 동등성 비율로 해석하지 않는다.
메인터너 판정은 위 직접 비교의 구체적 줄·그림·구문 계약에 한하며, 원 PR의 부분 개선 범위와 기존 잔여를 함께 설명한다.
대표 증적은 [비교 PNG](../assets/pr7104_maintainer_wasm_synth_p001.png)이며 게시할 raw URL 형식은
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7104_maintainer_wasm_synth_p001.png`다.

통합 merge SHA 및 asset의 devel 반영을 확인한 뒤, 승인된 후속 처리에서 원 source SHA·보정 SHA·통합 PR과 merge SHA를 포함한
UTF-8 Markdown 파일을 `--body-file`로 게시한다. API 재조회로 한글·본문·고정 이미지 링크를 확인한 뒤 원 PR을 close한다.
현재는 계획이며 게시·병합·close 완료를 뜻하지 않는다. 부분 해결 이슈는 자동 close하지 않는다.
