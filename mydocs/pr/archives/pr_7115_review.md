---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-14
---

# PR #7115 — U+318D 전각과 NO_LS 마지막 줄 메인터너 검토

**최종 판정: 메인터너 보정 후 수용 가능.** 보류 사유는 아래에 명시한 통합 보정 코드의 부분 개선 범위에서 해소했다. F1의 80168 108쪽 추가 줄바꿈·109쪽 이동과 75쪽에서 시작된 76~77쪽 한 줄 증가를 해소하고, 49쪽 다줄 대조군을 유지한다. 올바른 U+318D 전각 fallback을 반각으로 되돌리지 않았다.


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

원 PR head `fa94b52a07de08af9d4320ab911caefa19e323dd` → 현재 통합 이력의 cherry-pick `0fb0d9345963471fe5bbceee0944ef854c5008aa`. 원 저자와 `-x` 출처를 보존했다.


## 대상·rebase·보정 이력

| 항목 | 확인값 |
| --- | --- |
| 원 PR / 작성자 | [#7115](https://github.com/edwardkim/rhwp/pull/7115) / planet6897 |
| source SHA | `fa94b52a07de08af9d4320ab911caefa19e323dd` |
| rebase 후 cherry-pick SHA | `0fb0d9345963471fe5bbceee0944ef854c5008aa`; 원 저자와 `-x` 출처 보존 |
| 메인터너 보정 | `b80db1a21 + 2f59c8937 (초기 5d35b37cc의 전역 공백 보정 철회)` |
| 검증 Renderer 코드 | `2f59c89373f497068f9a0bcb2c22730ec7dc7e51` |
| WASM 캡처 도구 | `321f4cdb4`; Visual Sweep `--wasm-pkg`로 직접 캡처 |
| 작업 시작 시 동기화 기준 | `upstream/devel` 및 local `devel`: `037e4906a93e99896daa145a5ee5517824bfeaf4` |
| 작업 branch / target | `review/planet6897-20260914` / `target/planet6897-review-20260914` |
| rebase | 원 18 commit을 최신 기준 위에 재적용, 충돌 없음; 보정 전 head `51498859ef3d14a6a73f38ea3bfab4e9fb2ac9ca` |
| 원본·PDF 보존 commit | `ab3184254`; 기존 입력을 이름 변경해 중복 추가하지 않음 |
| 원 PR 상태 | 2026-09-14 재조회: OPEN, non-draft, source SHA 동일; mergeable / mergeState UNKNOWN |
| 관련 이슈 | [#7080](https://github.com/edwardkim/rhwp/issues/7080); 이번 로컬 작업에서 close하지 않음 |

이번 보정 전에 local devel을 fast-forward하고 review branch를 rebase했다.
rebase 전 이력은 `codex/planet6897-before-maintainer-rebase-20260914`에 보존했다.
누적 대상은 #7094, #7100, #7104, #7111, #7112, #7113, #7115, #7116, #7117, #7120, #7131, #7132이며
원 16 commit의 저자·출처를 유지했다. draft #7098은 제외했다. 원 reviewer는 jangster77이다.
원 PR의 CI와 이번 통합 코드 검증을 구분하며, 원격 push·통합 PR·merge·comment는 이 기록의 완료 항목이 아니다.

## 보류 사유와 메인터너 해결

**원인:** 이전 review의 유효 저장 LineSeg 가설을 정정한다. 실제 원본은 section 0, 표 host pi=936, cell=3, p=9/13의 LineSeg가 없는 NO_LS 문단이다. 들여쓴 다줄 셀의 반각 공백 채움 규칙이 양쪽정렬 분배가 없는 마지막 한 줄에도 적용돼 문장 끝을 밀었다.

**보정:** 동일한 frame 채움기로 현재 행 시작부터 글꼴 공백 후보를 계산하고, 남은 문단이 그 구간에서 끝나는 경우에만 후보 행·폭·높이를 함께 게시한다. 한 행 문단과 다줄 문단의 마지막 행에 공통 적용하며, 중간 행은 기존 반각 채움을 유지한다. 토큰 경계 재생은 이진 탐색으로 찾고, 커닝이 준비된 문단은 기존 폭 소유 경로를 유지한다. 원본 Justify와 NO_LS 들여쓰기 계약을 사용하며 파일명·쪽수·문자열 조건을 생산 코드에 넣지 않았다. 처음 시도한 전역 min(반각, 글꼴 공백) 보정은 49쪽을 과소 측정해 철회했다.

**직접 검증:** 독립 한컴 PDF p108의 9호와 마지막 3호가 각각 완전한 한 줄로 p108에 남는다. p75의 부대시설 조항은 한컴과 같은 3행 및 각 행의 전체 문자열을 검사한다. 그 마지막 다)가 별도 행으로 밀리지 않아 p76~77에 원 PR이 더한 한 줄을 제거한다. p49 대조군은 자산관리회사 조항의 줄끝 운 / 법 / 인과 총 3행을 유지한다. 전체 문서 페이지 수만으로 통과시키지 않고 Visual Sweep에서 해당 문장과 후속 내용을 직접 확인한다.

**범위·잔여:** 76~77쪽은 원 PR 이전 base의 흐름으로 복구한 것이며 한컴 전체 일치가 아니다. p77의 이전 조항 2행 잔존, p109 상단 여백, 기존 글꼴·좌표 차이 및 76076 p81의 사고/사고를 차이는 별도 잔여다. 이 차이를 숨기기 위해 문서별 분기나 baseline 갱신을 추가하지 않았다.

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
| 80168 21 | 10호 마지막 빗물처리계획이 한컴처럼 한 행으로 끝남. 종전 별도 획 행 제거. | 쪽 시작과 이후 조항의 기존 줄바꿈 차이 잔존 |
| 80168 49 | 자산관리회사 1호의 운 / 법 / 인 3행 보존. | 다른 조항의 기존 글꼴·줄끝 차이 잔존 |
| 80168 75 | 부대시설 2호가 한컴과 같은 전체 문자열의 3행. 별도 다) 행 제거. | 쪽 하단은 기존 base와 같은 한 줄 차이 유지 |
| 80168 76 | 원 PR의 추가 한 줄 밀림 제거. 본문 첫 줄이 base의 변경하지 아니하는 범위에서 건축으로 복구. | 한컴 PDF 첫 줄과는 기존 한 줄 차이 |
| 80168 77 | 원 PR 이전처럼 앞 조항 잔여 2행과 3호 3행. 원 PR이 더한 한 행 제거. | 한컴에는 앞 조항 잔여가 없는 기존 차이 |
| 80168 108 | 9호와 마지막 3호가 각각 한 줄로 같은 108쪽에 남음. | 그 밖의 기존 글꼴·들여쓰기·줄끝 차이 |
| 80168 109 | 이전 쪽 마지막 3호가 잘못 넘어오던 현상 제거. | 기존 상단 여백과 표 아래선 위치 차이 유지 |
| 76076 81 | 전체 82쪽 SVG가 보정 전 통합본과 동일. 이쪽의 사고/사고를 잔여도 재확인. | 한컴의 사고와 candidate 사고를 차이는 이번 해결 대상 아님 |
| form002 1 | 양식과 항목 구조 유지; 전체 10쪽 SVG 동일. | 기존 글꼴·위치 차이 유지 |

`_sweep.png`와 `_overlay.png`는 Visual Sweep 원출력이다. `_three_way.png`는 같은 Sweep의
PDF·최종 PNG에 보정 전 SVG를 동일 webfont rasterizer로 렌더한 결과를 나란히 놓은 보조 비교다.

- [pr7115_maintainer_form002_p001_sweep](../assets/pr7115_maintainer_form002_p001_sweep.png) — SHA-256 `c53df6df854216be752992bc32ae20293411802521b883b74a7e2b98c43061b2`
- [pr7115_maintainer_reg76076_p081_sweep](../assets/pr7115_maintainer_reg76076_p081_sweep.png) — SHA-256 `2773cf1b77de5b1e3bfa43d49c256d436c4a42989f2a41e9d7b3707db264c5ea`
- [pr7115_maintainer_reg80168_p021_sweep](../assets/pr7115_maintainer_reg80168_p021_sweep.png) — SHA-256 `45713b36c8b9b39790b5b90d6e81afaada436ed542b82ca073f4539f9ce16434`
- [pr7115_maintainer_reg80168_p021_three_way](../assets/pr7115_maintainer_reg80168_p021_three_way.png) — SHA-256 `6945d5575475f5ef286bfd8e3d4ffbdadd552bc087d48d75b1b82f5fec4887c1`
- [pr7115_maintainer_reg80168_p049_sweep](../assets/pr7115_maintainer_reg80168_p049_sweep.png) — SHA-256 `f6918f01c924be6d988934fe0dc61dc1ddfb30f405765579713a834cfd7bef1a`
- [pr7115_maintainer_reg80168_p049_three_way](../assets/pr7115_maintainer_reg80168_p049_three_way.png) — SHA-256 `5759b5bb39cdb85758d94f0a3487889b56dc87467e51be77e0e4315315ba9770`
- [pr7115_maintainer_reg80168_p075_sweep](../assets/pr7115_maintainer_reg80168_p075_sweep.png) — SHA-256 `d29a20bc9267a67fccf823ad2bfb12b4a335268d862303f10b2ac242c45fb05f`
- [pr7115_maintainer_reg80168_p075_three_way](../assets/pr7115_maintainer_reg80168_p075_three_way.png) — SHA-256 `37dc4b6f5d6853e4f6c6a4e01c1b0ea849baa1869c46dc200c5cbe372502d3f0`
- [pr7115_maintainer_reg80168_p076_sweep](../assets/pr7115_maintainer_reg80168_p076_sweep.png) — SHA-256 `f6cf63e94eefa4bb69b5e770dbaf8e1f8a726f6f3f2b1d71de0f21123975c31b`
- [pr7115_maintainer_reg80168_p077_sweep](../assets/pr7115_maintainer_reg80168_p077_sweep.png) — SHA-256 `c0e58869e74d56770aa09e4538c78b0822ec3c05cb6735ffb0e033f7189b5027`
- [pr7115_maintainer_reg80168_p108_sweep](../assets/pr7115_maintainer_reg80168_p108_sweep.png) — SHA-256 `2fa3e60101d30ac1d131e68cc78029bd61b18c9b18f658468c9240daceee5fbf`
- [pr7115_maintainer_reg80168_p108_three_way](../assets/pr7115_maintainer_reg80168_p108_three_way.png) — SHA-256 `301a1234c525d02f18eeab82dfd1717711f0634a2604170117fc8979e00659ac`
- [pr7115_maintainer_reg80168_p109_sweep](../assets/pr7115_maintainer_reg80168_p109_sweep.png) — SHA-256 `e357947b6d0bf5291bba1cf9d6d8ddedcca48c870219a17be0b09f83eb1c9e6d`
- [pr7115_maintainer_wasm_reg80168_p108](../assets/pr7115_maintainer_wasm_reg80168_p108.png) — SHA-256 `dce90823dd64e6a9261a51d2bdc52f8262c52301c6b089474c0495a59919b14c`
- [pr7115_maintainer_wasm_reg80168_p108_overlay](../assets/pr7115_maintainer_wasm_reg80168_p108_overlay.png) — SHA-256 `2eaf2a3d974cfff4bae07cf89f3b1094050cd952dd16f61d09381fd1025e8aa6`

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
| [samples/hwpx/form-002.hwpx](../../../samples/hwpx/form-002.hwpx) | 한컴 입력; form002 | `5ab8f7c368e02538f75f1cd2bd82bbd8de2f925a54ba7b38ec9395b2cdb804d4` |
| [pdf/hwpx/form-002-2022.pdf](../../../pdf/hwpx/form-002-2022.pdf) | 독립 한컴 PDF 10쪽; 직접 sweep 1쪽 | `629f1d93be234e4c4c551d319e247c1158d225cfe8a86bb179754a1b6cf2e077` |
| [samples/80168_regulatory_analysis.hwp](../../../samples/80168_regulatory_analysis.hwp) | 한컴 입력; reg80168 | `c8ad10fe9f07be5119cd804278017aefa46e555bbee4f05f0f5132fe4f591a22` |
| [pdf/80168_regulatory_analysis-2022.pdf](../../../pdf/80168_regulatory_analysis-2022.pdf) | 독립 한컴 PDF 157쪽; 직접 sweep 1,21,29,49,75-77,108-109쪽 | `7af457d9ec502132b1035582c16b1ba783e7faff71da0feef202690382bb1b95` |
| [samples/86712_regulatory_analysis.hwp](../../../samples/86712_regulatory_analysis.hwp) | 원 PR focused 계약에 사용한 기존 입력 | `32e2ed30e5d744ad747f04f090c022eca8270f9dd2d55e0613e2ad61058099e9` |
| [samples/issue6031/3249937_asset_management_rules.hwpx](../../../samples/issue6031/3249937_asset_management_rules.hwpx) | 원 PR focused 계약에 사용한 기존 입력 | `97b5d6c571a6b7626321c6a53d797e3511978447497bb73309bd23eaa8e7ea77` |

| 저장소 파일 | 역할 | SHA-256 |
| --- | --- | --- |
| [samples/76076_regulatory_analysis.hwp](../../../samples/76076_regulatory_analysis.hwp) | 76076 81쪽 잔여 대조 | `3308ba8505391bae2d0d62963e9399f4e48cdae574304cc0f89a311c6efbb6b5` |
| [samples/issue1891/76076_regulatory_analysis-2024.pdf](../../../samples/issue1891/76076_regulatory_analysis-2024.pdf) | 76076 81쪽 잔여 대조 | `06a389455d6b96e5f6580c9930fd8555256f9c712be85fb3cdaf31fc601a090d` |

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
[보정·후속 단계](pr_7115_review_impl.md)에 다음 절차를 기록한다.

## Merge 후 contributor PR comment 계획

[Visual Sweep GitHub comment 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 따른다.
최종 WASM Sweep의 아래 수치와 앞 절의 직접 시각 판정·남은 차이를 함께 게시한다.
flag 수는 해당 페이지의 자동 후보 유형 수이며 검출된 객체나 결함 개수가 아니다. 96dpi, 픽셀 차이 threshold 32의 결과다.

| reg80168 쪽 | 자동 flag 유형 수 | pixel_match | visual_accuracy_proxy_percent |
| --- | --- | --- | --- |
| 21 | 0 (없음) | 92.09600% | 7.36462% |
| 49 | 0 (없음) | 92.92198% | 8.31009% |
| 75 | 0 (없음) | 90.91012% | 9.46147% |
| 76 | 0 (없음) | 90.73416% | 8.19388% |
| 77 | 0 (없음) | 97.85255% | 5.57720% |
| 108 | 0 (없음) | 92.20938% | 8.42627% |
| 109 | 0 (없음) | 94.74094% | 7.30777% |

흰 여백을 포함한 pixel_match와 잉크 기반 proxy는 자동 비교 지표다. 기능 정확도 또는 전체 한컴 동등성 비율로 해석하지 않는다.
메인터너 판정은 위 직접 비교의 구체적 줄·그림·구문 계약에 한하며, 원 PR의 부분 개선 범위와 기존 잔여를 함께 설명한다.
대표 증적은 [비교 PNG](../assets/pr7115_maintainer_wasm_reg80168_p108.png)이며 게시할 raw URL 형식은
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7115_maintainer_wasm_reg80168_p108.png`다.

통합 merge SHA 및 asset의 devel 반영을 확인한 뒤, 승인된 후속 처리에서 원 source SHA·보정 SHA·통합 PR과 merge SHA를 포함한
UTF-8 Markdown 파일을 `--body-file`로 게시한다. API 재조회로 한글·본문·고정 이미지 링크를 확인한 뒤 원 PR을 close한다.
현재는 계획이며 게시·병합·close 완료를 뜻하지 않는다. 부분 해결 이슈는 자동 close하지 않는다.
