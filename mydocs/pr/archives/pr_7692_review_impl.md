---
kind: evidence
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-13
---

# PR #7692 체리픽·메인터너 보정 검토 계획

## 2026-10-15 최종 통합 검토 연결

최종 적용 범위와 판정은 [개별 review](pr_7692_review.md)와 [통합 review](planet6897_followup_20261015_review.md)를 따른다. 검증 code SHA는 `39a634e547621c32ae6f9ba8c1694d35c827078b`다. 최초 전체 실행·사용자 승인한 집중 재검증 합산, 전21문서181쪽 TSV와 최종 로컬 게이트를 구분해 기록했다. 아래 단계별 “현재”·미완료·C20 기록은 각 작성 당시의 이력이며 최종 상태를 대신하지 않는다. 원격 통합 PR의 CI·merge는 실제 확인 전 완료로 표시하지 않는다.


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

검토 진행 중이며 수용·승인 판정은 아직 하지 않았다. 범위는 **병합 칸 높이와 줄 상자**다.

- branch: `review/planet6897-followup-20261011`
- 초기 검토 base(과거 단계): `bfa28a77a2714482f647ee6a7b9aa7f7876d8404`
- 원 PR head: `135e2b075a8bee4aec792b003e9fa89618c32b5c`
- 기본 경로: `collaborator_external_pr` 9.1.1, 주 작업공간 체리픽 통합
- 보조 경로: intake, local validation, visual fixture, multi PR, rework

| 원 고유 commit | 적용 commit |
| --- | --- |
| `fa6cc7eab488a5b800f97b4df7c3df0f094616c9` | `62888424d8a2663af40dabea4ba3c83181017243` |
| `6b11130813d4c9e9ee8cdd4f5e9dc2e215a788d5` | `fc0ede395feedcb45b4834077f14e9aedc62ae6c` |
| `135e2b075a8bee4aec792b003e9fa89618c32b5c` | `338856f43010d0e2d0f08d67b700397949e5e927` |

원 fork history를 보존하고 중복 선행 commit·merge commit은 다시 적용하지 않았다.

## 구현 주장과 확인할 경로

선언 높이 constraint 생산 → 공용 rowspan solver → 측정 row heights와 실제 layout row heights 소비를 확인한다. 양쪽 constraint에서 내부 행 간격을 같은 방식으로 제외해 #7832의 행 간격 계약을 보존했다.

## 독립 기대값과 검증 계획

내부 행 간격이 있는 span, 이미 채운 행의 초과 높이와 빈 행, 일반 그림 칸·강제 줄바꿈을 대조한다. 80168 HWP/HWPX의 표 뒤 문단 악화와 1342000 전쪽을 확인한다.

최종 통합 source의 해당 lint·회귀·Native/fresh WASM 전쪽 TSV·대표 review 확대 판독이
완료되기 전에는 통과로 판정하지 않는다. 기여자 PNG·기여자 CI는 최종 통합 head의 증적을
대신하지 않는다. raw TSV·중간 JSON·로그는 ignored
`output/pr-review/planet6897-followup-20261011/`에 보존한다.

입력/Print SHA, 실제 source SHA·명령·관측값과 `충족 / 미충족 / 미검증 / 비해당`
판정은 실행 결과로 이 기록을 보완한다. 필요한 오늘할일은 최종 검토 묶음과 함께 통합 PR에
포함하며 별도 기록 PR을 만들지 않는다.


## 원 PR의 최신 동기화 확인 — 2026-10-12

최신 fork head는 `0c7bede4a0780b22e43f170c25b6c116c50b6835`다. 기존 검토 head와 고정 BFA를 제외한 새 commit은
upstream/devel 동기화 merge 하나뿐이며 새 작성자 고유 commit은 없다. 기존 적용 고유
commit과 출처 표는 유지한다. fork는 solver에 내부 행 간격 함수를 전달한다. 현재 후보는 측정·배치 양쪽에서 constraint를 만들 때 같은 내부 간격을 먼저 제외하는 보정으로 동등한 행 합 계약을 유지한다.

combined merge diff와 graph 근거는 ignored
`pr-7692-latest-merge-combined.diff`, `source-pr-refresh-merge-review.json`,
`nine-source-pr-live-20261012.json`에 있다. 원 PR 상태는 OPEN이며 원격 devel은 BFA다.


## 고정 base와 전체 시각 미달 대조 — 2026-10-12

정확한 글꼴 공급으로467 production 전쪽 TSV를 재산출했다. HWP는157/157쪽,
Native/fresh 최저35.10490%·미달31쪽이다. HWPX는 Native/fresh 모두156/156쪽,
최저1.49475%·미달110쪽으로 최종 종료를 확인했다.
HWPX90쪽 review에서 rhwp의 앞 표 끝 「계획」과 Print의 조문 대비표가 같은 쪽에 있어
내용 소속이 다르다. 이 차이를 글꼴·DPI 차이로 분류하지 않는다.

같은 입력·독립Print·글꼴로 고정upstream baseBFA를 실제 비교했다. HWP37·93쪽은
최저35.08606%, HWPX10·90쪽은 각각7.23121%·1.47757%였다. 이 선택 쪽의 미달은
이번 원고유 commit에서 새로 생긴 것이 아니다. base의 전쪽·fresh를 실행한 것으로
보고하지 않는다. 기존 전체 피델리티 후속은 열린#7783에 있고#7445 본문 표에도 있다.
현재 미달이나 신규 실물 회귀 추가 조건을 그 과거 이관으로 면제하지 않는다.

원문·TSV·로그는 `current-regulatory-results-467-correct-face-final.json`,
`current-regulatory-results-467-exact-base-bfa-selected.json`,
`regulatory-hwpx-exact-base-bfa-results.json` 및 대응score 디렉터리에 있다.


## 새 실물 검사 후보 등록 보류 — 2026-10-12

고정BFA에 없던 incoming 검사를 전쪽 시각 선행 조건에 맞춰 활성 등록에서 보류했다.
보류 대상은 #7672의4건과 #7692의3건, 합계7건이다. 원문은 바이트와 해시를 유지해
[보존 기록](../assets/issue7445/incoming-unqualified-regressions-20261012/README.md)에 있다.
이는 기존 devel 검사 이관이나 출력 결함 해결이 아니다. 이전10241PASS는 이 후보들을
포함한 실행 결과이며 등록 조건까지 충족한 것으로 승격하지 않는다. 정상 대조·전쪽 시각
수용은 계속 필요하고, 최종 파생 suite 준비·필수 lint·전체 정식 검사 수는 재확인한다.


## 그림 칸의 before/after와 Print 글꼴 프로그램 재확인 — 2026-10-12

원본14행 표를 복제한 작은2쪽 대조군에서, 정확한BFA의 칸(1,1)은152.8px이고
현재4ce는147.5px였다. 현재값은 입력의 칸 여백·그림 offset·그림 높이 합과 일치한다.
이 대조군은 원본4쪽의 개선이나 정상 회귀 등록으로 승격하지 않았다.

사용 글자를 보완한 HBATANG 서브셋에도 `『』` 표시 차이가 남아 독립 Print의 실제
TrueType 프로그램을 추출·대조했다. Print의HCR Batang은1000em·낫표500advance이고
`/usr/local/share/fonts/hwp-convert-mcp-hanbatang/HANBatang.ttf`의 동일 글리프 윤곽선·
전진폭과 일치한다. HBATANG.TTF(한컴바탕)는1024em·낫표1024advance로 **다른 face**다.
Unicode상Wide라는 분류로 이 글꼴의500advance를 전각으로 바꾸지 않았다.

올바른 HCR원 프로그램을 바이트 그대로 공급하고 current Native/fresh 전쪽을 다시
산출했다. 작은2쪽 대조군은 양쪽 최저94.56671%로90% 이상이다. 새review1쪽을 직접 판독해
`『』` 표시 복원을 확인했지만, 첫 표와 그림 경계가 Print보다 약3.8px 위에 있어
정식등록을 계속 보류한다. 원본2쪽review에서는 bullet 누락과 행 경계·줄바꿈 차이가
남는다. 이 차이를 글꼴 공급 완료나90% 점수로 면제하지 않는다. 원본4쪽은Native 최저39.21772%,
fresh 최저36.90483%, 각각1·2쪽이90% 미만이며 전체쪽수4/4다. 원본을 대신 새 입력만
통과시킨 것으로 보고하지 않는다.

ignored 증거: `owned-normal-picture-before-after-cell.json`,
`picture-corner-quote-independent-program-evidence.json`,
`validated-print-hcr-batang-program/provenance.json`,
`owned-picture-correct-hcr-{native,wasm}-{native,wasm}-results.json`.
실행 중인 미주 검증의 기존 글꼴 파일은 변경하지 않았고 올바른HCR공급 경로를 따로
고정했다. 작은 입력의 원본과 독립Print·원본4쪽의 기준은 각각 유지했다.


직접 판독한 새PNG는 ignored output의
`owned-picture-correct-hcr-native-native-scores/owned-normal-picture-table/owned-normal-picture-table/review/review_001.png`,
`owned-picture-correct-hcr-native-native-scores/picture-original-4-page/picture-original-4-page/review/review_002.png`다.


## 기본 글머리표와 빈 표 호스트 원점 진단 — 2026-10-12

기존 HCR 공급 실행에서 원본의 `굴림체`는 실제 Noto Sans KR ExtraLight였다.
별도 private GulimChe.ttf를 공급한 새 SVG에서는 실제 GulimChe(1024em)와
HCR Batang(1000em)을 확인했다. Print의 308문자 중 굴림체 윤곽은 304개,
전진폭은 307개가 같았다. 모든 글리프가 동일한 것으로 보고하지 않는다.
같은 공급의 4ce 수정 전 Native/fresh 전쪽은 모두 4/4쪽, 최저 36.90483%,
미달 1·2쪽이었다.

원본의 셀(20,3) 문단 0·11·13·14는 Bullet/참조 0이고 명시적 Bullet 레코드는
하이픈 하나다. Print 2쪽에는 기본 `●`가 4개 있으나 기존 출력에는 없었다.
기본 `●`와 본문 모양 상속·자동 내어쓰기·50% 거리를 반환하는 후보를 만들었다.
후보는 `c714968547bb2d59f5314d423643c6a3e51ee013` 위 미커밋 패치
SHA-256 `17e38f754c8ae4aa5ddf8de6a4608a2717a599097694bb19e2b6913ceefd7ad2`다.
`bullet_marker_text`가 만든 같은 MarkerHead를 줄 나눔의 `list_marker_geometry`와
실제 `apply_paragraph_numbering`이 소비한다. None/Number/Outline과 명시적 id1,
범위 밖 nonzero는 기존 계약을 유지한다.

독립 진단으로 원본에서 참조 0을 유지한 저장본과 명시적 `●` 정의를 추가한
저장본을 Hancom 2020 Print로 출력했다. 두 출력은 전체 4쪽 raster가 동일했다.
기존 원본 Print와 새 참조 0 Print는 1·2쪽이 동일하지만 3·4쪽은 다르므로,
roundtrip의 전쪽 동일성이나 새 입력의 원본 대체를 주장하지 않는다.
실제 0 참조와 명시적 기본 모양의 동일성은 이 Print 쌍으로 확인했다.

후보 Native 4쪽 TSV는 최저 36.98554%, 미달 1·2쪽, 전체 쪽수 4/4다.
2쪽 SVG에서 마커 4개 복원은 확인했으나 행 경계·줄바꿈·표 원점 차이가 남았다.
새 회귀는 등록하지 않았고 전체 수용·PR gate 통과로 판정하지 않는다.
직접 PNG는 `picture-bullet-after-direct-review/review/review_002.png`다.
글머리표 후보의 optimized fresh WASM 빌드는 23분 39초에 exit 0으로 끝났다.
루트 pkg/와 Studio/public의 JS·WASM SHA-256이 각각 같았다. fresh WASM 전쪽은
4/4쪽, 최저 36.98554%, 미달 1·2쪽으로 Native와 같았다. 두 backend의 측정 완료를
90% gate 통과로 보고하지 않는다. WASM 해시는
`ffcb20e8ab79236d29a3073c1b3ff97485da0d3fefcb676312e658a994bef60f`다.
Native Skia Print PNG 2쪽에서도 `●` 4개를 직접 확인했다. SVG raster와 별개의
실제 `export-png` 실행이며 exit 0이다. 표 원점·행 경계 차이는 같은 PNG에도 남았다.
증거는 `picture-bullet-after-native-skia-p2.log`와 대응 PNG다.

표 원점의 별도 진단에서는 빈 문단 0에 LineSeg가 없고 SectionDef·ColumnDef·Table이
함께 있는 것을 확인했다. 문서의 section/column 정보를 그대로 유지하고 호스트의
설정 컨트롤만 제외한 진단에서 첫 쪽 표 상단은 37.8→41.5733px로 이동했다.
독립 Print 긴 가로 괘선은 1·2쪽 모두 41.55467px다. 차이는 선언 위 여백 283HU와
일치하며, `reflow_empty_table_host`의 단일 컨트롤 판정이 설정 메타데이터까지
가시 컨트롤처럼 세는 경로를 확인했다. 2쪽은 그 진단에서도 37.8px로 남으므로
첫 조각과 이어받기 조각을 별도로 확인한다. 이 진단 시점에는 프레임 원점의 production 보정을 적용하지 않았다.

같은 후보의 Print SVG 경로에서 `RHWP_DIAG_SCAN=1`로 실제 분할 예약도 확인했다.
행 19의 컷은 `[3,4]`, 앞 행 합은 951.2px, 컷 내용은 85.333px, 패딩은 3.76px다.
첫 조각은 1040.293px를 예약하고 끝 행 override는 None이었다. 실제 RenderTree의
행 19 일반 셀은 81.1px였다. 같은 helper 이름만으로 예약·paint의 결과 공유를
단정하지 않으며, 원본/정규화 입력과 마지막 줄 간격의 재계산을 다음 진단 대상으로 남긴다.
이 차이는 구현으로 해결한 것으로 보고하지 않는다.

ignored 증거는 `picture-original-gulimche-print-program-evidence.json`,
`picture-original-correct-gulim-before-{native,wasm}-{native,wasm}-results.json`,
`picture-builtin-bullet-contract-before-implementation.json`,
`picture-bullet-zero-versus-explicit-print-raster-equality.json`,
`picture-bullet-original-versus-roundtrip-print-raster.json`,
`picture-original-builtin-bullet-after-native-native-results.json`,
`picture-frame-metadata-contract-before-implementation.json`,
`picture-original-bullet-svg-cut-debug.log`다.


### 표 원점 후보의 실제 적용

`reflow_empty_table_host`에서 SectionDef·ColumnDef를 가시 개체 수에서 제외했고,
`column_rowbreak_fragment_opens_outer_top`도 같은 재조판 빈 호스트 판정을 소비하도록
바꿨다. 비어 있지 않은 본문·공백 본문, 저장/합성 LineSeg, 다중 표·다른 컨트롤,
TAC·음수 오프셋·다른 wrapping의 기존 판정은 유지한다. 구역·단 정의 자체를 제거하지 않았다.
첫/통째 조각은 `from_empty_reflow_host → block/entry → block/prepare`에서 확정한
표 상단을 쓰고, 이어받기는 `fragment/budget`의 예약과 `table_partial`의 실제 배치가
같은 프레임 여백 판정을 쓴다. 확정 표 원점이 있으면 paint에서 다시 더하지 않는다.

포맷 첫 검사 실패는 신규 iterator 줄 배치 때문이었다. `cargo fmt --all` 후
포맷 재검사를 통과했고 Native 빌드는 7분 19초에 exit 0으로 끝났다. 실패 로그도 보존했다.
현재 두 source 파일의 후보 패치 SHA-256은
`0b3ffd0e4845e0dcfb7faf9910549f303f43614fc5768e5cc74271cccc1d6c76`다.
원본 Native 전쪽은 4/4쪽이며 1쪽 98.24368%, 2쪽 44.47623%,
3쪽 99.99019%, 4쪽 100%였다. 대조 문서 2/2쪽의 최저 94.56671%는 유지됐다.
새 원본 1·2쪽 PNG를 직접 열어 표 위 여백 복원과 남은 첫 행 높이 부족을 확인했다.
1쪽의 높은 점수도 끝 행의 box 불일치를 해결한 것으로 보고하지 않는다.
이 원점 후보의 fresh WASM과 최종 회귀는 아직 완료하지 않았다.
글머리표 전용 후보의 기존 점수를 이 새 원점 후보의 결과로 사용하지 않는다.
증거: `picture-frame-metadata-after-native-native-results.json`,
`picture-frame-metadata-direct-review/review/review_001.png`,
`picture-frame-metadata-direct-review/review/review_002.png`,
`picture-frame-metadata-cut-debug.log`, `picture-frame-metadata-native-build.log`,
`picture-frame-metadata-production-candidate.patch`, `picture-frame-metadata-fmt-first.log`.


### 행 19 물리 최소 높이의 독립 대조

독립 Print의 가로 괘선에서 행 19의 첫 조각은 86.62533px, 이어받기는
26.85066px, 합계는 113.47599px로 측정했다. 일반 셀 (19,2)와 (19,3)의
선언 높이 8531HU = 113.74667px와 차이가 0.27068px다. 현재 셀 box 합계
약 98.2px는 이 독립 물리 범위를 보존하지 않는다.
`row_cut_cell_order` 슬롯은 같은 행에서 시작하는 row_span1 셀만 소유하지만,
`row_uses_reflow_physical_frame`은 다른 열의 row_span11 셀까지 보고 물리
최소 높이 조회를 제외한다. 수용 높이·잔여를 걸침 셀 ledger와 대조할 필요가
있다. 이 높이의 production 수정은 아직 적용하지 않았다.
증거: `picture-row19-physical-minimum-before-implementation.json`.


### 행의 물리 최소 높이와 실제 수용 상자를 공유하는 후보

행 컷이 소유하는 일반 셀의 최소 높이를 다른 열의 걸침 셀이 지우지 않도록
`row_uses_reflow_physical_frame`을 행 컷의 셀 범위로 맞췄다. 걸침 셀의 내용·
물리 높이는 기존 block/straddle ledger가 계속 소유한다. 저장 프레임을 가진
일반 셀은 기존 저장 계약에 남고, 일반 셀이 없는 행에는 새 최소 높이를 만들지 않는다.

수용한 끝 행의 상자는 `scan_ordinary_row_step`이 실제 증가시킨 예약 높이에서
앞 행 합·행 간격을 제외해 생산한다. 물리 최소 높이가 내용보다 큰 컷에만 이 값을
`end_row_height_override`로 반환한다. `fragment/emit → PageItem::PartialTable →
table_partial`이 그 상자를 그대로 소비하고, 다음 조각의 잔여도 같은 상자에서 뺀다.
별도 helper로 수용 높이를 다시 추정하는 방식을 쓰지 않았다. 기존 명시적
override와 블록 컷은 이 후보가 덮어쓰지 않는다.

높이 후보의 포맷 검사는 통과했고 Native 빌드는 7분 48초에 exit 0으로 끝났다.
현재 네 source 파일 패치 SHA-256은
`f93e0c237715de49436703d72a6904c7408de8484767dcccd99d0e0276b4ce3d`다. 최종 원본 전쪽·독립 Print·
정상 대조·확대 판독 전에는 보정 완료나 회귀 등록으로 판정하지 않는다.
증거: `picture-row-minimum-native-build.log`, `picture-row-minimum-production-candidate.patch`.


### 높이 후보의 전쪽 결과와 최종 paint 덮어쓰기 확인

f93 후보의 원본 Native는 4/4쪽 최저 98.24368%, 미달 0쪽이었다. 대조 2쪽은
최저 94.56671%를 유지했다. 이 후보의 fresh WASM은 아직 실행하지 않았다.
실제 스캐너에서 행 19 끝 상자 89.09333px와 다음 조각 잔여 24.65333px를
확인했지만, 실제 첫 셀 box는 여전히 81.1px였다.
`table_partial`의 `fold_last_row`가 배치 후 8px를 다시 접는 분기를 찾았다.
이 분기의 `owns_declared_row_frame`은 저장 프레임만 인증하고 재조판 선언
최소 높이를 제외했다. 따라서 높은 TSV를 최종 결과 공유 완료로 판정하지 않았다.

재조판의 물리 최소 높이로 인증된 끝 행 override도 그리는 단계의 접기에서
보존하도록 후보를 보완했다. 내용만 소유한 일반 컷의 기존 접기는 유지한다.
새 후보의 포맷·Native 빌드는 순차 실행해 통과했으며, 결과는 f93과 구분한다.
정식 회귀 등록·PR 생성은 최종 Native/fresh 전쪽·확대 판독 뒤에만 진행한다.
증거: `picture-row-minimum-after-native-native-results.json`,
`picture-row-minimum-cut-debug.log`, `picture-row-frame-preserved-native-build.log`.

### 선언 행 상자 보존 후보의 전쪽 결과와 직접 판독

기준 HEAD `c714968547bb2d59f5314d423643c6a3e51ee013` 위 다섯 코드 파일의
패치 SHA-256은 `cbcc20124672fc67ec7f37f541c84b6f955c0304750c3815fcda910a775f3f43`다.
Native 빌드는 5분 12초, exit 0으로 끝났다. 원본 4/4쪽의 최저는
98.14494%, 미달 0쪽이며 대조 2/2쪽 최저 94.56671%도 유지했다.
실제 행 19 셀의 첫 상자 89.1px와 이어받기 24.7px 합계는 113.8px로,
선언 113.74667px의 물리 최소 높이를 보존한다. 이 결과는 내용 컷과 별도로
소유한 빈 물리 띠가 최종 paint에서 사라지지 않는 증거다.

최신 raster로 1·2쪽 review/overlay를 새로 만들고 같은 좌표·배율의 확대 패널을
직접 확인했다. 표 상단과 `●` 네 개는 복원됐지만 첫 조각 하단 괘선은 Print보다
약 3.5px 아래, 이어받기 첫 행 괘선은 약 2.2px 위에 남는다. 총 높이 보존과
90% 통과를 분할 경계의 시각 일치 완료로 판정하지 않는다. 이 범위는 미충족이며
정식 회귀 등록을 보류한다. raw NO_LS 컷과 정규화 synthetic LineSeg 컷의 마지막
줄간격 처리, 분할 예산과 `rowbreak_fragment_frame_bottom`의 실제 소비를 대조한다.
원본 총 높이나 독립 Print를 바꾸지 않고 배치 경로를 확인한다.

같은 cbcc 후보의 fresh web WASM 빌드는 9분 59초에 exit 0으로 끝났다.
최적화 없는 진단 빌드이며 source/patch 해시와 루트 pkg·Studio/public의 JS·WASM
해시 일치를 확인했다. 별도 frozen package로 측정한 원본 4/4쪽 최저
98.14494%, 대조 2/2쪽 최저 94.56671%, 미달 0쪽이다. 앞 단계 WASM 점수를
재사용하지 않았다. 점수 통과 이후 직접 확인한 분할 경계 차이는 여전히 미충족이다.
증거: `picture-row-frame-preserved-after-native-native-results.json`,
`picture-row-frame-preserved-actual-boxes.json`,
`picture-row-frame-preserved-direct-review/review/review_001.png`,
`picture-row-frame-preserved-direct-review/review/review_002.png`,
`picture-row-frame-preserved-direct-review/p1-last-row-2x.png`,
`picture-row-frame-preserved-direct-review/p2-continuation-head-2x.png`,
`picture-row-frame-preserved-fresh-web-build.log`,
`picture-row-frame-preserved-after-wasm-wasm-results.json`.

### 재조판 컷과 물리 하단의 공유 보정 후보

raw NO_LS와 synthetic LineSeg의 plain 문단은 모두 composed 줄에서 마지막 줄간격을
얻도록 맞췄다. 저장 LineSeg·컨트롤 문단·TAC·빈 유닛의 기존 계약은 유지한다.
선언 최소 높이로 실제 빈 물리 띠가 있는 RowBreak 표는 분할 예산에서도
`rowbreak_fragment_frame_bottom`을 소비한다. 일반 행 컷이 실제 내용을 수용한 뒤
그 행의 물리 최소 높이 중 가용 하단까지의 빈 띠를 함께 예약하고, 그 앞 상자를
끝 행 override와 이어받기 잔여가 공유한다. 이미 확정된 override와 블록 컷은
덮어쓰지 않으며 visible 내용이 예산을 넘을 때 사후 clamp하지 않는다.

구현 전 관측값과 적용·비적용 경로는
`picture-shared-reflow-frame-contract-before-implementation.json`에 보존했다.
이 후보는 여섯 코드 파일, 패치 SHA-256
`14385ddd475c6ea9b619c5b82a449427ab8f288c866a9de96b18f6f2709a1040`이다.
Native 빌드와 경계의 재출력을 먼저 진행하며 cbcc의 점수를 새 후보에 적용하지 않는다.
정식 회귀 등록·최종 lint·전체 검증·PR 생성은 아직 완료하지 않았다.
증거: `picture-shared-reflow-frame-native-build.log`,
`picture-shared-reflow-frame-production-candidate.patch`.

### 추가 후보에서 검출한 회귀와 소유 경계 수정

14385 후보의 Native 빌드는 5분 21초에 exit 0이었다. 전쪽 측정은
원본 4/4쪽 최저 74.45955%, 미달 2쪽, 대조 2/2쪽 최저 44.10867%,
미달 1·2쪽이었다. 현재 보정으로 만든 회귀이므로 기대값 갱신이나 이관으로
처리하지 않는다. 해당 후보의 fresh WASM 빌드는 시작하지 않았다.

대조 문서의 실제 스캐너는 종전 0..10행 대신 0..8행에서 끝났다.
보호 블록 8..10행 요구 276.3px에 가용 272.1px가 주어져, Print에서
첫 쪽에 있는 그림 묶음을 통째로 이월했다. 표에 선언 최소 높이를 가진
일반 행이 있다는 이유로 표 전체 예산을 줄인 잘못된 소유 가정을 확인했다.
원본은 확정 `fragment_placement`가 뒤에서 예산을 다시 계산해 앞선 하단
제한을 덮어쓰기도 했다. 실제 행 19 앞 상자가 91.9px로 늘어난 결과도 보존했다.

새 후보는 최종 원점·캡션·반복 머리 행을 반영한 물리 하단의 가용량을
`FragmentBudget → BlockRowScanVars → scan_ordinary_row_step`으로 전달한다.
실제 일반 행이 선언 최소 높이를 소유할 때만 그 값을 소비한다. 보호 블록은
별도 블록 ledger와 종전 가용량을 유지한다. 원점 덮어쓰기 이전에 계산한
값이나 표 전체의 다른 행 유무로 현재 컷의 소유를 판단하지 않는다.
두 원본 전쪽과 정상 대조를 재실행하기 전에는 회귀 해소로 판정하지 않는다.
증거: `picture-shared-reflow-frame-after-native-native-results.json`,
`picture-shared-reflow-frame-normal-cut-debug.log`,
`picture-row-frame-preserved-normal-cut-debug.log`,
`picture-shared-reflow-frame-actual-boxes.json`.

일반 행에 물리 하단을 전달하는 새 후보는 코드 파일 열 개이며 패치 SHA-256은
`c5e1232abadec02a180403af038697e4723e5a3030d1eea1283fa6c906f12a5c`다.
포맷 검사 → Native 빌드 → 원본·대조 전쪽 TSV → 실제 컷 추적을 순차 실행 중이다.
증거는 `picture-ordinary-reflow-frame-native-build.log`와
`picture-ordinary-reflow-frame-after-native-progress.log`에 새로 보존한다.

### 일반 행 소유 경계 후보의 Native/fresh 전쪽 검증

c5e123 후보의 Native 빌드는 4분 29초, fresh web WASM 빌드는 10분 12초에
각각 exit 0으로 끝났다. 원본 4/4쪽의 페이지별 일치율은 두 backend 모두
99.85950% / 99.89764% / 99.99019% / 100%이며 미달·누락·쪽수 불일치는 없다.
같은 올바른 글꼴 공급을 사용했고 frozen package와 루트 pkg·Studio/public의
JS·WASM 해시 일치를 확인했다. 최적화 없는 fresh 진단 package의 WASM 해시는
`01d3fdc265d8c532b571a7046e1b00b2f3c7109210007046c8d33a9c86ca171a`다.
대조 2/2쪽은 Native/fresh 모두 94.56671%로, 14385 후보가 만든 회귀를 해소했다.

원본 실제 컷은 `[3,4]`를 유지했다. raw 재조판 컷이 마지막 줄 뒤 간격을 제외한
77.33333px를 소비하고, 일반 행의 가용 86.8px까지 선언 물리 띠를 예약한다.
앞 상자 86.8px와 이어받기 상자 약 26.94667px는 원본 선언 8531HU를 보존한다.
보호 블록은 종전 0..10행 / 10..14행으로 분할되어 그림 묶음을 임의로 이월하지 않는다.
원본 첫·다음 조각의 Native/fresh review PNG와 같은 좌표의 확대 패널을 직접 열어
앞 조각 하단·이어받기 첫 행·기본 `●` 네 개·주변 글줄을 대조했다.
앞 후보의 큰 행 경계 차이는 해소됐다. 미세한 차이는 남으며 일치율을
픽셀 완전 일치로 보고하지 않는다. Native/fresh 원본 raster 자체는 4쪽 모두 같은 RGB다.

수정 전 renderer의 코드가 현재 커밋 base와 동일함을 `git diff 4ce..c714 -- src`로
확인했다. 같은 원본·글꼴의 수정 전 출력은 기본 마커 0개, 행 19 셀 상자 합계
98.2px, 첫·다음 표의 본문 상대 위 여백 0이었다. 수정 후는 마커 4개,
상자 합계 113.7px(직렬화 반올림), 상대 위 여백 3.8px다. 분할된 본문 문자열
순서는 유지됐다. 원본/PDF나 기대 좌표를 바꾸지 않았다.

전쪽 선행 조건과 직접 판독 후
`tests/cases/issue_7692_original_picture_rowbreak.rs`에 세 관계형 검사를 추가했다.
기본/명시 마커와 일반 문단 소유, 메타데이터 호스트의 선언 위 여백 한 번 적용,
선언 물리 최소 높이·셀 내부 포함·다음 행과의 비겹침·본문 순서/누락/중복을 검사한다.
전체 SVG hash나 실물의 절대 화면 좌표를 고정하지 않는다. 같은 검사 executable이
수정 전/후 frozen renderer를 각각 호출하는 검증을 진행 중이다. 파생 suite는
`regression_suite_016`이며 generated suite/manifest는 PR에 포함하지 않는다.

증거: `picture-ordinary-reflow-frame-after-native-native-results.json`,
`picture-ordinary-reflow-frame-after-wasm-wasm-results.json`,
`picture-ordinary-reflow-frame-direct-review/review/review_001.png`,
`picture-ordinary-reflow-frame-direct-review/review/review_002.png`,
`picture-ordinary-reflow-frame-wasm-direct-review/review/review_001.png`,
`picture-ordinary-reflow-frame-wasm-direct-review/review/review_002.png`,
`picture-relational-before-after-observations.json`, `picture-relational-test-build.log`.

세 관계형 검사 첫 실행은 수정 전3실패, 수정 후2통과·1실패였다. 마커 검사 초안이
일반 문단의 리터럴 `-`까지 마커로 집계한 검사 오류였다. 원본 머리 모양으로 마커
문단을 식별하고, 일반 문단은 전체 본문 보존을 검사하도록 수정했다. 기대 마커를
줄이거나 출력 결함을 면제하지 않았다. 동일 검사 executable 재실행 결과는
수정 전3실패 / 수정 후3통과다. 최초 실패 로그도 보존한다.
증거: `picture-relational-tests-after.log`, `picture-relational-v2-tests-before.log`,
`picture-relational-v2-tests-after.log`, `picture-relational-v2-before-after-test-results.json`.

### 기존 중첩 표 검사 실패와 독립 출력 대조

c5e123 후보의 기존 #7518 관계 검사는22건 중18통과·4실패였다. 같은 검사 executable,
같은 원본·글꼴로4ce renderer를 실행하면22건 모두 통과한다. 표 원점/최소 높이의
앞 단계 세 후보(0b3ffd / f93e0c / cbcc20)도 각각22건 모두 통과했다.
이 비교로 확인한 것은 후보 간 출력 차이와 기존 기대값에 대한 실패다. 4ce는
검증 기준인 `upstream/devel`이 아니므로 이 결과만으로 실제 출력 회귀나 이관 여부를
확정하지 않는다. 중첩 표 상단과 child UNIT018의 페이지 소속이 달라졌다.
사용자 지시에 따라 `upstream/devel`의 고정 SHA와 현재 후보를 같은 원본·독립
Print PDF·글꼴로 전쪽 비교하고, 2px 실루엣 일치율과 직접 판독으로 판단한다.
이 판정 전 기존 기대값을 바꾸지 않는다.

일반 본문 줄의 물리 끝 상자와 중첩 child-unit/viewport 전진 ledger를 같은 높이
도메인으로 취급한 가정을 제거한다. 실제 행의 소유 유닛이 일반 텍스트 줄일 때만
새 물리 하단·빈 띠 예약을 소비하며, 중첩 표와 객체 viewport는 별도 ledger를 유지한다.
중첩 표의 컷은 부모가 소비할 전진량을 소유하므로 끝 줄간격을 빼서 다음 유닛의
페이지 소속을 바꾸지 않는다. raw/synthetic 모두 같은 중첩 컨텍스트 판정을 쓴다.
문서 ID·픽셀 임계값·특정 속성 조합을 새 예외로 추가하지 않는다.

새 열 파일 후보 패치 SHA-256은
`1835419830780a3a8841b827324f738716c95e7d5186bbc983b273cb6f67a3c6`이다.
Native 빌드는 exit 0이며 기존22건과 새3건은 모두 통과했다. 원본4쪽 전쪽 Native
최저99.85950%, 정상 대조2쪽 최저94.56671%이고 쪽수 불일치는 없다.
기존 실패4건이 참조한6개 입력의 base/current 전쪽 TSV와 fresh 검증은 진행 중이다.
c5e123의 Native/fresh 통과를 새 후보의 증거로 재사용하지 않는다.
새 후보의 fresh·필수 lint·전체 회귀·PR 생성은 아직 완료하지 않았다.
증거: `picture-ordinary-existing-frame-tests.log`,
`picture-ordinary-existing-frame-tests-before-4ce.log`,
`picture-existing-frame-isolation-results.json`, `picture-unit-domain-frame-native-build.log`.

### 고정 upstream 기준 전쪽 비교에서 확인한 추가 출력 차이

기준은 `bfa28a77a2714482f647ee6a7b9aa7f7876d8404`, 후보는 위1835419 패치다.
같은 원본 그림 띠·기존 독립 Print PDF·글꼴·96dpi·compat2022로 비교했다.
기준 Native 전쪽 최저92.77089%와 달리 후보 Native/fresh WASM은2쪽88.45404%다.
22건 검사 통과로 면제하지 않는다. 기준과 후보의 p2 review를 직접 판독했다.
기준/4ce의 앞 조각 높이825.0과 이어받은 높이469.2가 후보822.9/471.3으로 바뀌고,
중첩 표 원점도128.9→129.9로 이동했다. 이 수치는 원인 진단이며 기대 좌표로 고정하지 않는다.

새 하단 예산·빈 띠 채움에만 일반 줄 소유 조건을 적용하고, 컷 끝 줄간격 제거와
수용 프레임의 전달/paint 보존에는 같은 조건을 빠뜨렸다. 중첩 내용을 포함한 행의
child-unit 원장까지 일반 줄의 물리 상자로 해석한 경로다. 변경한 일반 줄 계약의
생산(컷 간격/수용 높이)→예약·이어받기→paint 보존에 같은 소유 조건을 적용한다.
문서 ID나 허용 오차를 추가하지 않고 중첩/객체 원장을 이전 의미로 유지한다.

동시에 독립 PDF 텍스트에서 auto/mixed 네 입력의 UNIT 소속000–017 /018–036 /
037–055 /056–064를 재확인했다. 이 기존 기대값은 PDF와 일치한다.
`valid_generated/nested-split`은 기준 Native에서도4–6쪽 미달, 최저21.08402%다.
다른 입력의 미달과 혼합해 이관하지 않고 후보 비교와 원인 판독을 마저 수행한다.
증거: `failed-existing-7518-base-current-native-results.json`,
`failed-existing-7518-base-current-wasm-results.json`,
`failed-existing-7518-independent-print-unit-owners.json`,
`picture-band-base-before-current-tree-geometry.json`,
`failed-existing-7518-direct-review/upstream-bfa/picture-band-original/review/review_002.png`,
`failed-existing-7518-direct-review/current-unit-domain-wasm/picture-band-original/review/review_002.png`.

1cc7213 후보에서는 그림 띠의 Native 최저92.75255%와 이전128.9 원점이
회복됐지만 mixed8쪽은57.48558%였다. 원본 마지막 행의 선언6054HU(80.72px)는
14문단의 내용보다 작다. 독립 Print8쪽은 원본 문단9의 전문을 소유하지만 후보는
후반 줄만 소유했다. 일반 내용 행에도 끝 간격을 빼는 적용이 남아 있었으므로,
선언 최소 상자가 내용 외의 물리 공간을 소유할 때만 새 끝 간격 처리를 소비하도록
보정한다. 측정·예약·paint는 같은 `reflow_row_physical_minimum` 판정을 쓴다.
후보의 fresh 빌드는 Native에서 확인한 실패를 이유로 SIGTERM(exit143)으로
중단했다. 중단을 성공이나 fresh 검증 완료로 보고하지 않는다. 새 회귀를 추가하기
전에 보정 후 전쪽 Native/fresh90% 이상과 확대 판독을 다시 확인한다.

### 고정 devel과 보정 후보의 전쪽 판정

최종 보정 후보의 생산 코드 패치 SHA-256은
`1d37142e185235edfc1f2b3503220c02196f88a4f4c1c729cec8470d766ba43b`다.
기준은 위 `bfa28a77`이며 두 코드의 같은 원본·Print PDF·공급 글꼴을 비교했다.
각 칸은 해당 입력 **전쪽 최저**이며 Native/fresh WASM의 값이 같다.
전체 쪽수 불일치와 누락은 없다. 평균이나 일부 쪽의 높은 값으로 미달을 상쇄하지 않는다.

| 입력 | 쪽수 | 기준 Native/fresh | 후보 Native/fresh | 판정 |
| --- | --- | --- | --- | --- |
| 원본 그림 띠 | 2 | 92.77089% | 92.75255% | 90% 이상 유지; 앞/뒤 조각과 중첩 원점 복원, 미세 글꼴·테두리 차이 잔존 |
| valid_orientation/nested-auto-row | 8 | 96.81315% | 96.81295% | 소속·전쪽 기준 유지 |
| valid_orientation/nested-mixed-cell | 8 | 96.81315% | 96.81295% | 후보에서 생긴 문단 소속 회귀 보정 |
| valid_generated/nested-auto-row | 8 | 96.81315% | 96.81295% | 정상 저장 대조군 유지 |
| valid_generated/nested-mixed-cell | 8 | 96.81315% | 96.81295% | 정상 저장 대조군 유지 |
| valid_generated/nested-split | 6 | 21.08402% | 21.08609% | 양쪽4–6쪽 미달; 기존 #7788 문제이며 개선 완료로 판정하지 않음 |

정상 생성 nested-split의 전6쪽 표·셀 bbox와 개수는 기준과 후보에서 같다.
독립 Print4쪽은 세 번째 자식 행을 소유하지만 Native는 다음 쪽으로 넘기는 차이를
기준 review에서 직접 확인했다. 현재 보정의 회귀와 혼합하거나 기대값으로 숨기지 않는다.
#7788은 이미 #7445 본문 표에 등록돼 있어 새 이슈나 종료 표현을 추가하지 않는다.

mixed 마지막8쪽은57.48558%에서99.67347%로 회복했다. 기준/실패/보정의 같은
글꼴 `dump-pages --json`에서 p7 끝 컷과 p8 시작 컷은 `[2,10] / [2,11] / [2,10]`이다.
원본 문단9의 전문과 다음 두 문단이 마지막 셀의8쪽에 남는 것을 Native/fresh
review에서 직접 판독했다. 순수 내용 행은 끝 줄 전진량을 유지하고, 실제 선언 최소
상자의 빈 띠를 가진 행만 새 컷/예약/paint 계약을 소비한다.

4쪽 그림 원본은 새 후보 Native/fresh 최저99.85950%, 정상2쪽 대조군은94.56671%다.
새 후보의 원본1·2쪽 review와 그림 띠2쪽을 직접 판독했다. 25개 기존/신규 focused
검사는 통과했다. 그 검사만으로 놓쳤던 마지막 문단의 쪽 소속을 독립적으로 검사하는
`tests/cases/issue_7518_mixed_tail_source_ownership.rs`를, 해당 두 원문의 전쪽90% 이상과
직접 판독을 확인한 뒤 추가했다. 같은 compiled harness와 공급 글꼴에서 수정 전
1FAIL / 수정 후1PASS다. 실패는 원본 문단9의 일부가 다른 쪽으로 넘어간 실제 소속
오류이며 환경·빌드 실패가 아니다. 정상 저장 대조군도 같은 검사에서 통과한다.
이전25건과 합계26건의 focused 검사가 통과했다. 최종 fmt·Native/WASM/workspace
all-targets 세 Clippy·workspace build·manifest·unit-tiers는 모두 통과했다.
정책 기준은 다시 fetch한 `bfa28a77`이며 production 패치는 위1d37142와 같다.
fresh Node package 빌드와 Studio 단위1836건도 실패·skip 없이 통과했다.
전체 Rust 회귀와 Skia3종은 진행 중이며 이전 head의 전체 통과를 재사용하지 않는다.

증거: `failed-existing-7518-base-current-native-results.json`,
`failed-existing-7518-base-current-wasm-results.json`,
`failed-existing-7518-owned-physical-frame-native-results.json`,
`failed-existing-7518-owned-physical-frame-wasm-results.json`,
`nested-split-base-owned-physical-frame-comparison.json`,
`mixed-tail-{base,before,after}-same-font-pagination.json`,
`picture-owned-physical-frame-direct-review/{native,wasm}/`,
`picture-owned-physical-frame-regression_suite_{018,016}-tests.log`,
`mixed-tail-relational-test-build.log`, `mixed-tail-relational-before-after-results.json`,
`mixed-tail-relational-{before,after}.log`.

최종 검증 로그: `owned-frame-and-tail-final-lint-results.json`,
`owned-frame-and-tail-final-validation-results.json`,
`owned-frame-and-tail-final-{all-nextest,studio-unit}.log`.
대표 PNG7개는 `mydocs/pr/assets/planet6897_followup_20261011/picture-rowbreak-frame-1d37142/`에
원본 바이트로 보존했다. 원래 output과 SHA-256 연결은 `owned-physical-frame-stable-assets.json`에 있다.

### 2026-10-12 — 추가 실패 2건의 devel/현재 전체 페이지 대조

사용자 요청에 따라 테스트 기대값의 타당성을 먼저 판정했다. 고정 `upstream/devel`
`bfa28a77a2714482f647ee6a7b9aa7f7876d8404`와 현재 `c714968547bb2d59f5314d423643c6a3e51ee013`
+ 미커밋 제품 패치 `1d37142e185235edfc1f2b3503220c02196f88a4f4c1c729cec8470d766ba43b`를 비교했다.
두 입력에 대해 같은 원본·독립 Print PDF·글꼴 공급·2022 compat·96dpi·2px 반경으로
Native/fresh WASM 전체 페이지 TSV를 각각 산출했다. 총 8회 비교이며 쪽수 불일치·누락은 없다.

| 원본 | 전체 쪽수 | devel Native/fresh 최저 | 현재 Native/fresh 최저 | 현재 90% 미만 |
| --- | ---: | ---: | ---: | --- |
| `samples/task1749/saved_bounds_cumulative_page_break.hwpx` | 5/5 | 77.86946% / 77.86946% | 34.45569% / 34.45569% | 4·5쪽 |
| `samples/task2146/21761835_jeonjik_exemption_table.hwp` | 6/6 | 36.70489% / 36.70489% | 47.03926% / 47.03926% | 1–6쪽 |

기준 PDF는 각각 `pdf/task1749/saved_bounds_cumulative_page_break-hwpx-2020.pdf`
(원본 HWPX의 한컴 Print 5쪽, SHA-256 `798e06db6946edcbb05e7399d3aec08d847a57afc375d9d9e67b435e9e1f40d8`),
`pdf/task2146/21761835_jeonjik_exemption_table-hwp-2020.pdf`
(기존 출처가 확인된 한컴2022 Print 6쪽, SHA-256 `99d150b26805a29f3d3153528638f363e33f882d0e38ad11edb374baccdac127`)다.
각 입력 해시·실행 명령·동결 바이너리/패키지 해시·전체 페이지 결과는 아래 JSON에 연결했다.

- `issue_1811_hwpx_pi52_rowbreak_cut_matches_hwp_reference`: 독립 Print와 devel의 사회기여 표는
  4쪽 `[50,67,10]`, 5쪽 `[67,22,67,16,67,4]` 별표 글줄을 소유한다. 현재 4쪽은 `[50,67]`,
  5쪽은 `[10,67,22,67,16,67,4]`다. 현재 5쪽은 86.83787→34.45569%로 악화됐고,
  review PNG에서 표 뒤 본문·다음 표·제목까지 아래로 밀림을 확인했다. **기대값 유지 / 구현 보정 필요**.
- `cut_row_and_straddling_rowspan_do_not_gain_fragment_diagonals`: 원본 자체도 현재 r9c2 조각이
  1쪽과 2쪽에 나온다. devel은 1쪽에만 나오며, 현재 2쪽 머리행 아래에는 Print에 없는 빈 조각이
  생겨 다음 행 시작을 밀었다. 2쪽은 49.79156→48.80219%다. 최저값의 개선으로 이 경계 차이를
  면제하지 않는다. IR의 사선 속성 변형과 원본 전체 시각 비교를 구분한다.
  **기대값 유지 / 새 빈 이어받기 조각의 구현 보정 필요**.

두 입력 모두 devel에서 이미 미달하지만 현재 변경으로 생긴 위 차이는 기존 미달과 구분한다.
이번 판정에서는 실패 테스트를 삭제·이관하거나 기대값을 바꾸지 않았다. 제품 원인은 아직 확정하지
않았으며 줄 끝 간격과 선언 물리 높이의 컷·예약·이어받기·paint 소비 경로를 점검한다.
관련 전체 페이지 90% 및 확대 판독을 충족하기 전에는 회귀 기대값 변경을 보류한다.

증적 루트: `output/pr-review/planet6897-followup-20261011/`.

- 전체 비교: `additional-failures-base-current-results.json` 및 각 출력의 `silhouette.tsv`.
- 페이지별 전후·증감 22행: `additional-failures-base-current-paired.tsv`.
- 직접 판독: `additional-failures-direct-review/{upstream-bfa,current-c8}/native/savedbounds1749-hwpx/review/review_{004,005}.png`,
  같은 루트의 `jeonjik2146-hwp/review/review_{001,002}.png`.
- fresh WASM 대표 PNG는 같은 경로의 `wasm/`에 생성하며, 각 `provenance.json`으로 전체 TSV와
  원본 PNG 해시를 연결한다. Native 판독과 fresh PNG 판독 범위를 혼동하지 않는다.
  현재 fresh WASM `savedbounds1749-hwpx` 5쪽과 `jeonjik2146-hwp` 2쪽을 직접 열어
  Native에서 확인한 글줄 이월·뒤 내용 밀림과 머리행 아래 빈 조각을 각각 재확인했다.

전체 Rust 실행은 최종 **10,239 PASS / 2 FAIL / 45 skipped**, exit 100이다
(`owned-frame-and-tail-final-all-nextest.log`). Studio는 1,836 PASS, lint 9단계는 exit 0이며,
순차 실행기가 Rust 실패에서 종료하여 이후 Skia 단계는 미실행이다. 전체 통과로 보고하지 않는다.

### 2026-10-12 — 문단 완료 컷과 rowspan 공간 소유의 보정

줄 끝 간격의 수용 조건에서 선언 셀 최소 높이를 제거했다. 같은 composer 결과에서
`closing.vis_end == comp.lines.len()`일 때만 마지막 줄 뒤 간격·문단 뒤 간격을 조각 끝에서 제외한다.
문단 중간 컷은 다음 줄 전진량을 유지한다. 중첩 표 투영·부모의 혼합 유닛·개체는 기존 제외를 유지한다.
독립 Print의 사회기여 표는 완성 문단의 마지막 글줄을 4쪽에 두며, 8쪽 mixed-tail 원본은
중간 글줄 하나만 7쪽으로 떼지 않는다. 선언 셀 높이는 두 컷의 문단 소속을 구분하는 근거가 아니다.

이 후보 제품 패치는 `59f0aea9c0ea951a6af7628f93ed42170623e7c6dbe32f9ff4ed8758bb9fe7d9`,
동결 Native 바이너리는 `799510a80d2c36cc4365642919d588328210629dacb0ad0a2e03b0dc46f3361d`다.
새 바이너리로 원본을 다시 출력하여 사회기여 표의 4쪽 `[50,67,10]`, 5쪽
`[67,22,67,16,67,4]` 소속을 확인했다. 이미 컴파일된 관계형 검사에 새 CLI를 명시해
4쪽 그림 문서 3건과 mixed-tail 1건의 4 PASS를 확인했다. 이 검사들은 CLI로 실제 출력을
만들며 parser 변경은 없다. 직접 DocumentCore를 호출하는 실패 검사 2건의 새 source 실행과
전체 Native/fresh WASM 판정은 아직 이 결과로 대신하지 않는다.
증적: `completed-paragraph-cut-{native-candidate/provenance.json,focused-results.json,boundary/}`.

전직시험면제 원본의 컷 추적은 devel/현재 모두 r9의 전체 내용 컷 `[1,5,6]`을 소비함을 보였다.
devel은 기존 rowspan 행 밴드 계약으로 행을 끝내고, 현재는 새 독립 최소 높이 잔여를 만들어
2쪽에 빈 조각을 추가했다. `scan_ordinary_row_step`의 기존 `#2236` 밴드 계약은 이 원본의
독립 Print에서 내용이 끝난 행을 다음 쪽에 반복하지 않는 근거를 이미 갖고 있다.
앞 행에서 시작한 rowspan의 내부 행에도 독립적인 빈 최소 높이 잔여가 있다는 가정을 제거한다.
`reflow_row_physical_minimum`에서 기존 `row_has_prior_rowspan_cover`를 사용해 이 소유를 구분한다.
같은 행에서 새로 시작하는 rowspan은 앞 조각에서 공간을 이미 소유한 경우가 아니므로,
그림 문서 r19의 일반 셀 최소 높이는 보존한다.

공통 조회의 실제 소비 위치는 `scan_ordinary_row_step`의 가용 높이·완료 컷·수용 상자,
`fragment/emit.rs`의 `next_start_row_height_override`, `table_partial.rs`의 끝 행 상자 판정이다.
전체 행 측정의 기존 rowspan 높이 회계는 계속 사용한다. 후속 후보 C10의 제품 패치는
`44c5ae48c300043a80b8a99457e3b35e9f85222f14ff6d8ac3df0fd90b907f1d`이며 Native는
`e21a9b39d769b75b1142cd401fcb1f29148c099d01a4aa94231f1057c5feadd0`, fresh web WASM은
`2193cefcefc83a11680fcfec145731a937b63cb18b60802529e75f519a70a42e`다.
두 이전 실패 모듈과 기존 #7518 물리 프레임 모듈을 새 source로 다시 컴파일하여
**34 PASS / 0 FAIL**, exit 0을 확인했다. 기존 테스트의 기대값은 변경하지 않았다.
CLI 관계형 4건도 C10 실제 출력에서 통과했다. 전체 Rust·lint의 C8 결과를 C10 결과로 승계하지 않는다.

동일 원본·독립 Print·글꼴 공급으로 Native/fresh WASM 각각 10문서 전체 페이지를 측정했다.
두 backend의 페이지별 값과 쪽수는 일치했다.

| 대상 | 쪽수 | Native/fresh 최저 2px 일치율 | 판정 |
|---|---:|---:|---|
| 원본 그림 RowBreak | 4/4 | 99.85950% | 전쪽 90% 이상 |
| picture-band | 2/2 | 92.75255% | 전쪽 90% 이상 |
| 정상 그림 표 대조군 | 2/2 | 94.56671% | 전쪽 90% 이상 |
| orientation/generated auto/mixed 네 문서 | 각 8/8 | 96.81295% | 전쪽 90% 이상 |
| 사회기여 표 HWPX | 5/5 | 77.94686% | 4쪽 미달, 5쪽은 92.69644%로 복구 |
| 전직시험면제 HWP | 6/6 | 36.71894% | 전쪽 미달, 빈 r9 이어받기 제거 확인 |
| generated nested-split | 6/6 | 21.08609% | 4–6쪽 기존 미달, #7788 추적 유지 |

증적: `independent-row-frame-{native,wasm}-results.json`,
`independent-row-frame-existing-focused.log`, `independent-row-frame-boundary-results.json`.
현재 review PNG는 `independent-row-frame-direct-review/current-c10/{native,wasm}/`에 생성하며
각 provenance로 C10 입력·PDF·바이너리·TSV와 연결한다. 전체 미달 문서를 통과로 보고하지 않는다.
추적 원문: `jeonjik2146-{base,before}-rowcut-trace.log`, 같은 stem의 `pagination.json`.


### 사회기여 표 4쪽 후속 원인 추적

같은 4쪽의 별표 줄 수·문단 소속은 Print와 일치하지만, 첫 TAC 셀의 글줄 기준선은 약
3.8–4.2px 아래이고 뒤 non-TAC 조각은 약 7.7–7.9px 위다. Haansoft Dotum regular의
실제 font program 공급을 확인했으므로 글꼴 미공급으로 분류하지 않는다.
입력 HWPX의 PS11 `prev=500 HWPUNIT`(선택된 namespace case)은 6.667px 위 간격이다.
첫 셀의 5문단은 원래 저장 LineSeg가 없고 parser가 만든 synthetic 사다리다.
공통 `cell_paragraph_spacing_before`가 첫 합성 문단의 위 간격을 높이 측정에 포함하고,
`layout_composed_paragraph`도 그 문단 배치에서 사용한다. 이전
`hwpx_reset_first_lead`가 같은 위 간격을 가운데 정렬 높이와 실제 원점에 다시 더하므로
첫 문단 위 간격이 중복된다. 문서별 px 보정 없이 이 오래된 별도 계산을 제거하는 후보를
다음에 검증한다. 뒤 non-TAC 조각의 높이·정렬 차이는 별개로 추적한다.
진단: `savedbounds1749-p4-star-baseline-deltas.json`,
`savedbounds1749-p4-embedded-dotum-probe.json`, `savedbounds1749-source-para50.txt`.


C10의 backend 대조와 PNG 해시 검증은 `independent-row-frame-evidence-audit.json`에 남겼다.
Native/fresh WASM의 10문서 페이지별 값·쪽수·입력·PDF 해시가 동일하며,
대표 18개 review PNG의 10개 provenance와 입력 PNG 해시를 재검증했다.
이번 직접 판독은 Native 그림 원본 2쪽, fresh WASM 사회기여 4쪽·전직시험면제 2쪽·
generated mixed 8쪽이다. 같은 원본의 base/C8/C10 페이지별 증감 22행은
`additional-failures-base-current-c10-paired.tsv`에 보존한다.
C11 제품 패치 `cf22885fb509794d374d60d93297c77ad0fe5399b886f6824ca003864a94a880`에서
중복 first-lead 분기를 제거했으며 현재 Native 빌드 중이다. C10 증거를 C11 최종 증거로
승계하지 않는다. 사전 빌드도 `/home/tsjang/rhwp/target/pr-review`를 사용한다.


뒤 non-TAC 조각에서는 `cell_units_uncached`가 첫 NO_LS 문단의 위 간격을 첫 유닛에
예약하지만, `cell_cut_owns_reflow_paragraph_start`는 `start_unit>0 && para_idx>0`으로
이를 소유하지 않는다고 판정한다. partial의 내용 높이도 `pi>0`으로 첫 간격을 제외하며
실제 column-top 배치는 synthetic vpos=0을 사용해 같은 간격을 버린다.
첫 유닛이 해당 합성 문단의 `vis_start=0`을 소유하면 첫 문단에도 공통 위 간격을
한 번 적용하고, partial 높이 측정도 `cell_paragraph_spacing_before`를 소비하는 후보로
확인한다. 저장 LineSeg·문단 중간 컷·문단 앞의 별도 개체 유닛은 이 판정으로 대체하지 않는다.


C11 Native 빌드는 exit 0이고 바이너리 SHA-256은
`5ec51a88d57f2202ae9b1a4677862f2febbd61241f3b0915f69fd6521922df54`다.
사회기여 표 전쪽 최저는 **90.85559% / 5쪽**, 90% 미만 쪽이 없다.
첫 별표 기준선은 438.380→435.0467px로 이동했으며 독립 Print 434.5651px와의 차는
약 0.48px다. 이는 선언 앞 간격의 중복 제거로 생긴 결과이지 px 목표를 맞춘 조건이 아니다.
뒤 조각의 잘못 빠진 첫 간격은 C11에서 남아 있어 C12로 이어서 검증한다.
C11 CLI 관계형 4건은 통과했다. C12 제품 패치는
`9d965177b9e57b73b3ea9b8e6d05a1c29b2ef1038a1522f0dc3cfa25a8601cb2`이며
공용 target에서 사전 Native 빌드 중이다. Native/fresh 전쪽·기존 관계형 검사·대조군 결과를
받기 전 C12 완료로 보고하지 않는다.


C11 Native 13문서 비교는 종료됐다. 사회기여 표 및 3개 HWPX 대조군은 전쪽 90% 이상이며
대조군은 C10과 동일하다: broken-center 원본 100%, 저장 결재 원본 99.85464%,
위 정렬 중첩 원본 100%. 전직시험면제·generated nested-split의 기존 미달은 남는다.
`shared-synthetic-cell-lead-native-results.json`,
`shared-first-band-controls-before-results.json`으로 비교한다.
C11 4쪽 review를 직접 확인해 첫 표 글줄 개선과 뒤 분할 셀의 남은 위치 차이를 구분했다.

C12 Native 빌드는 exit 0이고 바이너리 SHA-256은
`8c526a19dd1f435be42c2dfb0f724df79dd886ad648cd8326c6921b1537a1d95`다.
현재 13문서 Native 비교와 기존 #1749/#7028/#7518/#7344/#6569 모듈의 새 source 검증을
진행 중이다. 실행 결과의 이름은 `shared-first-paragraph-band-{native,existing-focused}*`다.

디스크 부족을 예방하기 위해 종료된 C10 비교의 중간 SVG만 `.svg.gz`로 무손실 압축했다.
파일마다 압축 해제 바이트의 SHA-256을 원본과 대조한 뒤 압축본을 보존한다.
manifest는 `completed-c10-svg-lossless-compression.json`이며 원 경로에서 SVG가 필요하면
해당 `.svg.gz`를 해제해 원본 바이트를 복원한다. TSV·로그·PNG·동결 binary는 그대로다.
C12도 각 문서 sweep가 종료한 뒤 같은 검증으로 SVG만 압축하고 결과 JSON의
`archived_intermediate_svg`에 경로·원본 해시를 남긴다.
Cargo target과 shared cache는 삭제·이동·초기화하지 않았다.


C12 사회기여 표 Native 전쪽 TSV 산출 결과는 **5/5쪽**, 최저 **97.98406%**다.
페이지별 값은 99.91284 / 99.91587 / 98.15798 / 98.09619 / 97.98406%이며
누락·90% 미만 쪽이 없다. 원본 글줄 수·문단/쪽 소속은 보존됐다.
1차 관계 진단에서 첫 가운데 셀의 위·아래 간격 차는 C10 13.4px에서
C11/C12 6.7px로 바뀌었다. 이는 원본 앞 간격 500HU=6.667px와 같다.
뒤 조각은 첫 줄 위 간격이 C10/C11 1.9px에서 C12 8.6px로 바뀌어
원본 안 여백 141HU와 앞 간격 500HU를 한 번 포함한다. 셀 밖으로 보정·clamp하지 않았다.
진단 원문은 `savedbounds1749-c10-c11-c12-cell-gap-relations.json`이다.
이 수치는 진단 RenderTree의 반올림된 관계 값이며 절대 px 좌표를 회귀 기대값으로 삼지 않는다.
현재 결과는 Native 측정만이며 fresh WASM·최신 source 테스트·직접 PNG 판독 전에는
완료/승인 판정으로 승격하지 않는다.


C12의 기존 5개 모듈은 **39 PASS / 0 FAIL / 1,073 filtered**, nextest exit 0이다.
새 source 실행기의 `.exit-code`·결과 JSON·stdout은 모두 0/39 PASS이나,
도구 세션의 마지막 관측에서 143이 반환돼 종료 상태의 일관성을 확인하기 위해
이미 컴파일된 같은 검사만 다시 실행했다. 재확인도 **39 PASS**, exit 0이며 도구 종료도 0이다.
두 결과를 `shared-first-paragraph-band-existing-focused{,-confirm}*`에 보존한다.
CLI 관계형 4건은 동결 C12 binary를 명시한 실행에서 PASS다
(`shared-first-paragraph-band-cli-results.json`). 기존 기대값은 변경하지 않았다.
사회기여 4·5쪽 Native review를 직접 확인해 글줄 소속·표 외곽·뒤 문단 위치와
간격 개선을 확인했다. 약 1px 안팎의 Print 래스터/글꼴 기준선 차이를 남겼으며,
큰 배치·줄바꿈 차이는 관측되지 않았다. 수식은 이 입력의 변경 셀에 없다.
같은 source의 fresh web wrapper 빌드를 시작했다. 최종 전체 Rust·lint·Studio·Skia와
9개 PR 전체 시각 검증을 이 39건으로 대신하지 않는다.

종료된 C10 SVG 228개의 무손실 압축이 완료되어 **5,028,939,055바이트**를 확보했다.
각 파일은 압축 해제 SHA 검증을 통과했고 manifest로 복원 가능하다. 공유 Cargo cache는 보존했다.


C12 Native 13문서 **61쪽** 비교가 종료됐다. 11문서는 전쪽 90% 이상이다.
전직시험면제 6쪽 및 generated nested-split 6쪽의 기존 미달은 남으며 개선 완료로 분류하지 않는다.
사회기여 표의 고정 BFA 대비 전쪽 수치·증감은
`additional-failures-base-current-c12-native-paired.tsv`에 기록했다(두 원본 총 11행).
C12 Native 사회기여 4·5쪽 review PNG와 원본 PNG의 provenance 해시 검증도 통과했다.

fresh web 빌드의 초기 로그 헤더가 Native provenance의 exit 0·종료시각을 그대로
물려받은 기록 오류를 확인했다. 실제 WASM은 아직 실행 중이며 그 초기 필드를 완료 근거로
쓰지 않는다. 해당 실행의 `fresh-web-build.exit-code`와 최종 pkg provenance를 확인한다.
`shared-first-paragraph-band-fresh-web-build-status-note.json`에 혼동되는 필드의 출처를
명시했고 다음 실행의 wrapper 기록은 완료 필드를 null로 시작하도록 수정했다.
현재 열린 로그는 실행 중 덮어쓰지 않으며 종료 후 헤더에 Native 출처를 구분해 보완한다.


C12 fresh web 빌드는 exit 0으로 종료됐고 source 패치 9d965177…의 전후 해시가 같다.
WASM SHA-256은 `1a5c101ea32d3ae7346c2e6121a222a3ce964f513b35910f33b8a3332bf9392e`,
JS SHA-256은 `57805b611572e0d2bf9d3a871e801698450d9a8785e7bc524ebc422f7ac924a7`다.
루트 pkg와 Studio public의 두 파일 해시가 각각 일치했다.
실행 종료 후 초기 로그 헤더의 Native 완료 필드를 `native_build_*`로 구분해
완료시각 혼동을 보완했다. 실제 fresh 완료시각·exit 0·해시는 최종 pkg provenance에 있다.
같은 source·글꼴·입력·Print로 13문서 전쪽 fresh WASM 비교를 시작했으며
`shared-first-paragraph-band-wasm-progress.log` 및 대응 결과 JSON에 보존한다.

#7672의 기존 cross-engine 진단도 다시 확인했다. 동일 실제 편집본의 2024 Print에서도
최저 29.02132%였으며, 원본/편집본의 같은 2024 Print 15쪽 비교는 100%였다.
2020 기준 출력의 연도만 바꾸어 수식 캐시·미주 흐름 차이를 해소할 수 있다는 근거는 없다.
기존 진단은 `pr7672-engine2024-diagnostic-native-results.json` 및
`pr7672-original-vs-edited-same-engine2024-print-p15.json`에 남아 있다.
새 source와의 최종 증거로 승계하지 않으며 다음 보정의 원인 구분에만 쓴다.


### 사회기여 표의 관계형 회귀 검사 추가

같은 원본·독립 Print의 C12 Native/fresh WASM 전쪽 최저 **97.98406% / 5쪽**와
두 backend의 4·5쪽 review 직접 판독을 확인한 뒤
`tests/cases/issue_1749_synthetic_cell_spacing.rs`에 2건을 추가했다.
첫 검사는 내용 보존·줄 순서·셀 내부 포함 및 가운데 셀의 상하 간격 차가 원본
ParaShape 앞 간격 한 번인지 확인한다. 둘째 검사는 첫 source 문단의 소속과 줄 원점이
원본 안 여백·앞 간격을 한 번 포함하는지 확인한다. 원본의 두 배 저장 HWPUNIT을 해석하며
측정 helper의 합산값이나 실물의 고정 화면 좌표를 기대값으로 쓰지 않는다.
허용 오차도 고정 화면 위치 대신 그 줄 상자 높이의 1/10을 사용한다.

같은 새 harness에 동결 C10 CLI를 명시해 **2 FAIL / exit 101**을 재현했다.
실패는 중복 앞 간격(top-bottom=13.4, 원본 앞 간격=6.667)과 빠진 첫 간격
(top=1.9, 안 여백=1.88, 원본 앞 간격=6.667)이며 빌드·글꼴 환경 오류가 아니다.
C12 CLI 실제 출력은 **2 PASS / exit 0**이다. parser/원본 메타데이터 읽기는 변경되지
않았으며 전후 실제 배치는 각각 동결 CLI가 생성했다. 기존 정상 저장/합성 대조군은
별도 C12 기존 모듈 39 PASS와 전체 Native 13문서 결과에 연결한다.

증적: `savedbounds-synthetic-spacing-relational-{before-c10,after}{.log,-results.json}`.
새 검사는 최초 `regression_suite_028`에서 컴파일·전후 실행했다. 포맷 뒤 Source 줄 수가
바뀌어 파생 weight가 달라진 것이 manifest drift의 원인이었다. 포맷 후 prepare를 다시
실행했고 최종 배정은 `regression_suite_007`이다. 같은 테스트 source의 해시는 변경되지
않았으며 파생 suite/manifest는 stage하지 않는다.
Production 패치 9d965177…은 테스트 추가·포맷 후에도 동일하다. 전체 Rust·lints의 최종
결과를 이 2건으로 대신하지 않는다.


C12 Native/fresh WASM 13문서·61쪽 비교는 모두 종료됐고 문서별·페이지별 값이 동일하다.
`shared-first-paragraph-band-paired-audit.json`으로 쪽수·입력·Print·production 해시 및
페이지별 값의 일치를 확인했다. 11문서는 전쪽90% 이상이며 기존 미달2문서는 유지된다.
새 관계형 source의 fmt, 고정 BFA 대비 manifest와 unit-tier 정책 검사도 통과했다.
로그: `savedbounds-synthetic-spacing-{fmt-check,manifest-check-after-fmt,unit-tiers-check}.log`.
전체 Rust·3개 Clippy·Skia·Studio 최종 게이트는 다음 최종 source 단계에 남아 있다.
이 시점의 보정과 검사를 로컬 일반 커밋으로 고정하고 남은 #7672 원인 보정을 이어간다.


## C12 checkpoint 전체 검증 재실행 — 2026-10-13

제품10개 경로와 새 관계형 회귀3파일의 checkpoint는 `45a7a292ae5767069b4ee248daea87f4b631b94c`다.
이 commit의 제품 diff가 Native/fresh WASM 증거의9d965177…과 같다는 확인을
`shared-first-paragraph-band-commit-binding.json`에 보존했다.
새 case 추가 후 prepare·fmt·Native/WASM Clippy가 통과했고 workspace 빌드도 동시 작업2개로
재실행해 통과했다. 첫 빌드의 linker Bus error 및 같은 시점 errno28 기록을 보존했다.
현재 all-targets Clippy·정책 검사·전체 nextest·Skia를 같은 공용 target에서 순차 실행 중이다.
현재 checkpoint의 전체 Rust 결과가 나오기 전 이전 C8의10239PASS/2FAIL을 새 결과로 승계하지 않는다.
완료 단계별 exit code와 실제 명령/로그는 ignored
`shared-first-paragraph-band-validation-{progress.log,results.json}` 및
`shared-first-paragraph-band-validation-resume-jobs2-{progress.log,results.json}`에 남긴다.
Node/Studio와 optimized fresh web·출력 backend의 최종 증거까지 갱신한 후 전체 수용을 판정한다.


검증 후속 상태: 같은45a7에서 **Native/WASM/all-targets Clippy3경로 및 workspace
빌드·manifest/base 비교·unit-tier/base 비교까지 통과**했다. 전체nextest는
UTC2026-10-12 15:19:45에 시작했으며 최종summary/exit code는 아직 없다.
`shared-first-paragraph-band-validation-resume-jobs2-all-nextest.log`에 연결한다.

### C12 전체 회귀 종료 및 C13 구현 보정

C12의 전체 nextest는 run `cd00e92d-3f84-4b50-a37a-9d05179056f7`에서
**10,243건 실행, 10,241 PASS, 2 FAIL, 45 skipped / exit 100**으로 종료됐다.
실행 시간이 길거나 출력이 비었다는 이유로 중단하지 않았으며 terminal summary를 확인했다.
남은 실패는 `issue_7032_cell_empty_paragraph_flow::full_atom_window_preserves_uncut_alignment_and_spacing`와
`issue_3820_rowbreak_rowspan_band_keeps_pdf_page_35_36_boundary`다.
기존 #1749·#7028 실패는 이 checkpoint에서 통과했다. Skia 후속 단계는 전체 회귀 실패로
runner가 중지해 아직 실행하지 않았다. Native/WASM/all-targets Clippy와 두 정책 비교는 통과했지만,
이 결과를 새 C13 source의 완료 증거로 승계하지 않는다.

C13에서는 전체 unit window `(0, 전체 유닛 수)`와 무컷 셀의 앞 간격 계약을 일치시키는
`table_partial.rs` 보정을 시작했다. 첫 문단 소유만으로 부분 조각이라고 판단하지 않고,
실제로 시작·끝 유닛이 잘린 경우에만 조각의 앞 간격 보존을 적용한다. 기존 기대값은 변경하지 않았다.
전체 창 동등성과 #1749의 실제 부분 조각 검사를 같은 사전 nextest로 확인 중이다.
모든 사전 Cargo 검증도 `/home/tsjang/rhwp/target/pr-review` 한 곳을 재사용한다.
로그: `c13-complete-unit-window-focused.log`; 실행 입력·diff 해시:
`c13-complete-unit-window-provenance.json`.

#3820은 82쪽 원본 `samples/76076_regulatory_analysis.hwp`의 별도 관계형 검사다.
다른 규제 문서의 기존 낮은 점수를 근거로 이관하지 않는다. 같은 원본과 독립 한컴 Print의
BFA/C12 35·36쪽 tree/SVG를 `issue3820-base-c12-boundary/`에 산출해 빈 물리 밴드의
생산·예약·이어받기·배치 경로를 확인하며, 현재 보정에서 발생한 회귀는 구현을 수정한다.

C13 사전 nextest는 run `1e8f7d6b-1b53-4b10-9204-fd620708ce86`에서
**9 PASS, 448 skipped / exit 0**으로 종료됐다. 전체 창 동등성 실패를 구현 보정으로 해결했고,
#1749 부분 조각 앞 간격 2건과 #7032 기존 7건의 기대값을 유지했다.
이는 해당 source 보정의 focused 결과이며 전체 회귀·시각·lint 완료 판정은 아니다.
사용자 지시에 따라 최신 `upstream/devel`을 fetch했다. BFA 이후 새 base는
`552f5228d2dbfd6046c838f14241704f1bcb8a05`(#7861)이며, 현재 보정을 일반 커밋으로
고정한 뒤 이 base 위로 rebase한다. 새 base에는 dependency·GPU 제거·검증 정책 변경이 있다.
기존 C12 증거는 원 source/base의 기록으로 보존하고 새 통합 head의 검증과 구분한다.

### 최신 devel rebase와 rowspan 물리 밴드 보정 — C14

`552f5228d` 위로 44개 commit의 rebase를 완료했다. 새 head는
`d9007eccd53ef7b785d7f7d8ca760120f77ad3e8`이며 C13의 `table_partial.rs`는
rebase 전과 byte 단위로 동일하다. 미추적 15개 파일의 SHA도 모두 보존됐다.
충돌은 폰트 진단 JSON 두 곳과 renderer 모듈 선언 한 곳이었다. upstream의 시험지 진단
제외·GPU 폐기를 유지하고 이번 source의 공통 `reflow_float_host_line` 선언을 보존했다.
백업 ref는 `review/planet6897-followup-pre-552f5228d-3b249c29a`다.
`rebase-upstream-552f5228d-{audit.json,range-diff.log,log}`에 결과를 고정했다.

#3820의 실제 BFA/C12 p35는 2px 보조값 **97.57950/97.58945%**, p36은
**72.26471/33.42162%**다. BFA에도 미세 배치 차이가 있지만 현재 p36의 빈 r12c2·r12c5
누락과 뒤 행 전진은 새로운 회귀이며 이관 대상으로 처리하지 않는다. 두 variant의
`issue3820-base-c12-boundary/*/review/review_036.png`를 직접 판독했다.
이 증거는 rebase 전 source의 두 경계 쪽에 한정되며 최신 base의 전쪽 완료 증거가 아니다.

C14는 조각 경계를 가로지르는 rowspan이 소유한 물리 밴드를 일반 재조판 셀 최소와
구분한다. `scan_ordinary_row_step`의 기존 완료 컷/수용 밴드 → `fragment/emit.rs`의
crossing span/행 높이 잔여 → `continuation.start_row_height_override` → 다음 조각의
요구 높이·배치로 전달한다. 기존 일반 셀의 독립 최소 차단은 유지하고, 경계를 넘는
span 소유 밴드만 이전 fallback에 넘긴다. 정확한 관계형 기대값은 변경하지 않았다.
#3820과 함께 #7028의 불필요한 tail 재생성 여부, #7032 전체 창 동등성,
#1749 합성 셀 앞 간격을 같은 focused 검증으로 확인한다. 새 source의 전체 Native/fresh
WASM·lint·전체 회귀 결과는 아직 없다.

C14 focused nextest는 run `b4884e67-8554-4f67-bbdc-ff86c62b18d5`에서
**22 PASS, 854 skipped / exit 0**으로 종료됐다. #3820의 p35→p36 빈 꼬리 소유와
p18→p19 원 행 소유, #7028의 불필요한 조각 대각선/머리행, #7032 전체 창 동등성,
#1749 첫 문단 앞 간격을 기존 기대값으로 확인했다. 변경 source에는 rebase 충돌 해결 뒤
rustfmt의 모듈 선언 순서 정리도 포함된다. 제품 diff SHA는
`72aa1108e940ff9e7f2b8bf317573c9dd92c016519bf09ac2a6f5f5e9c96f4e9`다.
이 통과를 전쪽 Visual Sweep 또는 전체 Rust 통과로 확대하지 않는다.
`c14-crossing-span-results.json`과 `c14-crossing-span-focused.log`에 exact base/head,
공용 target, 실제 명령과 terminal 결과가 있다. Native Skia source의 실제 출력 확인을 이어간다.

C14 Native Skia 빌드도 exit0으로 종료됐다. 동결 바이너리는
`46f79d39fb3a96e6dc9971bd8a6440d53aae93586848baf6a60f4620ad552638`이며
`c14-crossing-span-native-candidate/provenance.json`에 base·diff·명령을 기록했다.
같은 C14 source의 default CLI를 focused Cargo가 실제로 빌드한 시각·해시로 동결해
먼저 Native SVG 경계를 대조했다. p35는97.58945%, p36은72.26366%다.
빈 이어받기 칸은 복원됐지만 p36의 미달은 남으며 whole82쪽/fresh WASM 완료로 보고하지 않는다.
`issue3820-c14-native-boundary/issue3820-c14-boundary/review/review_036.png`를 직접 확인했다.
원본의 p36 괘선 좌표와 source cell edge 비교를
`issue3820-c14-p36-vector-geometry.json`에 진단 자료로 보존했다. 이를 px 기대값으로 쓰지 않는다.

같은 C14 Native renderer의 font snapshot으로 #1749 p5의 charShape56·문단61의 별표45글자가
요청 `굴림체` → 선택 `Noto Sans KR`/bundled임을 관측했다. 올바른 private 공급 파일에는
`GulimChe`와 `굴림체` family naming record가 있지만 현재 custom typeface inventory는
Skia의 단일 primary family만 key로 삼는다. `c14-savedbounds-footer-font-probe/`의
trace·source·command·binary hash를 관측 근거로 사용하고 일반 SFNT 가족 이름 수용을 다음 단계에서
보정한다. 폰트 identity 계약과 실제 출력 폭을 함께 확인하며 문서별 예외나 임의 좌표 보정을 추가하지 않는다.

기준 파일 재검토에서 `76076-exact-input-hwp2020-20261004.pdf`는 다른 원본
`samples/issue1891/76076_regulatory_analysis.hwpx`(실제 HWP5, SHA49bbcc49…)에 대응함을
확인했다. 현재 `samples/76076_regulatory_analysis.hwp`의 SHA3308ba85…와 다르므로
이 PDF로 현재 입력의 gate를 갱신하지 않는다. 기존 독립 원본2024 Print와 실패 증거는 보존한다.
기준 출처는 tracked `76076_exact_reference_analysis.json`의 converter-input binding으로 확인했다.

## C15 — Native 공급 폰트의 SFNT 가족 이름 계약

Skia가 보고한 primary family와 같은 face의 SFNT Family(1)·Typographic Family(16)·WWS Family(21)
이름을 같은 custom/bundled Typeface에 등록하도록 보정했다. 로드한 face0의 실제 naming record만
읽으며 full-face(4)·PostScript(6) 이름은 가족 alias로 등록하지 않는다. 기존 디렉터리 공급 순서와
먼저 적재된 key의 우선순위를 유지한다. source는 `skia/font_lookup.rs`와 `skia/renderer.rs`다.
SFNT 이름 생산 → custom/bundled inventory → 기존 문자별 후보 사다리 → 실제 text paint와
renderer-owned native decision trace가 같은 supplied program을 소비하는 경로다.

정식 source `tests/cases/native_supplied_font_family_names.rs`는 폰트 identity 계약이다.
공개 Noto Sans KR에서 A 한 글자를 남기고 family를 새 이름으로 바꾼1716B의 합성 OFL fixture를
사용한다. 같은 SFNT의 한국어 family·typographic·WWS 이름이 primary와 같은 supplied program을
선택하고, full-face record는 family로 수용하지 않는지 검사한다. 문서는 명시적으로 합성 A이며
저장 LineSeg·px 위치·Hancom 레이아웃 oracle을 쓰지 않는다. real #1749의 출력 증거와 구분한다.
fixture 출처·원본/산출 SHA·이름과 OFL을 `tests/fixtures/native_font_family_names/`에 포함했다.

같은 새 계약 source를 보정 전 C14 Native library에 결합해 **2 PASS / 3 FAIL, exit101**을
확인했다. primary/비가족 대조군은 통과하고 한국어 family·typographic·WWS는 모두
실제로 `Noto Sans KR`를 선택해 supplied program 계약을 위반했다. 컴파일은 exit0이다.
`c15-native-family-contract-before-{build.log,results.json}`과 before 실행 로그에 source·fixture·
바이너리 SHA와 공용 target 명령을 고정했다. 포맷의 assertion 줄바꿈 변경은 별도 source 해시로 남긴다.
C15 after source의 Native 빌드·계약·원본 p5 실제 폰트/PNG 및 전체 필수 gate는 아직 검증 중이다.

#3820의 동일 원본·독립2024 Print는 C14 `--compat 2024`에서도36쪽72.26366%로 동일했다.
단순 호환 모드 변경으로 남은 배치 미달을 해소하지 못했다. 기존2022 실패 및 두 모드 증거를
보존하고, 다른 원본의2020 Print로 현재 입력을 대체하지 않는다.

C15 Native 빌드는 exit0, 바이너리
`8771726de2807d1ed3d754b1010a6060ab0a558933aead3e10bf7dcdfa6051ea`다.
같은 정식 identity case source를 C15 Native library에 결합해 **5 PASS / exit0**을 확인했다.
이 after 실행은 공용 target의 독립 harness이며, CI의 Cargo 파생 suite 실행을 대체하지 않는다.
`c15-native-family-verification-results.json`에 exact source·test·command·binary를 연결했다.
원본 #1749 p5의 문단61/charShape56 별표45개는 전부 **굴림체 → GulimChe/custom**을
선택한다. before의 Noto Sans KR/bundled와 actual renderer snapshot을 대조했다.

같은 C15 CLI의 직접 Native Skia print PNG를 독립 한컴2020 PDF와96dpi로 대조했다.
p4는98.81860%로 유지, p5는98.90970→99.23857%다. 두 review PNG를 직접 열어 p5 마지막
별표 행의 폭이 Print와 맞아지고, 표 조각·아래 본문·줄바꿈이 유지됨을 확인했다.
`c15-native-skia-savedbounds-review/provenance.json`에 input/Print/source·PNG·측정 결과를
고정했다. 이 두 쪽의 판독을 전5쪽 Native/fresh WASM 및 전체 corpus 통과로 확대하지 않는다.
fresh web build를 같은 source/공용 target에서 진행하며 최종 optimized 빌드와 lint·Skia·Studio·
전체 Rust·전쪽 시각 게이트는 아직 남아 있다.

C15 fresh diagnostic web build는 exit0으로 종료됐다. WASM SHA는
`88e188ad7972df00d26d5a479bdaafac93f47f3f12b1fc2162515a215d4b9586`, JS SHA는
`57805b611572e0d2bf9d3a871e801698450d9a8785e7bc524ebc422f7ac924a7`다.
루트 pkg와 Studio public의 두 파일도 byte/SHA가 같다. no-opt 진단 빌드이며 최종 optimized
검증으로 보고하지 않는다. C15 Native/fresh WASM은 같은14개 입력·독립 Print·fonts로
전쪽 TSV를 산출 중이다. frozen binary/package와 source diff의 바인딩은 보존하며,
완료된 문서의 SVG만 raw SHA 검증 뒤 무손실 압축한다.

#3820의3308 원본과5151 Print PDF는 기존 tracked
`mydocs/pr/assets/issue7445/76076-original-hwp-20261005/results.json`의 MCP job
`1a5ed915-52cc-4a77-8b36-96fb1199043b`로 동일 입력 대응을 재확인했다.
해당 원본의 전체 피델리티 문제는 #7445의 기존 #7784에 등록돼 있다. 다른 reference로
미달을 숨기지 않으며, 현재 발생시킨 회귀의 복원과 기존 미달 범위를 구분한다.
현 시점에서 C15 소스·폰트 identity 계약을 일반 commit으로 고정하고 남은 필수 lint와
Cargo suite·Skia·Studio·전쪽 시각 검증을 이어간다.

## 최신 고정 base의 원본 대조 — 2026-10-13

`valid_generated/nested-split.hwp`의 같은 원본·독립 2020 Print·글꼴·compat2022로
고정 devel `552f5228d`와 현재 `e2a525d36`의 Native 전6쪽을 확인했다.
최저값은 각각 **21.08402% / 21.08609%**, 두 출력 모두4–6쪽 미달이다.
Page/Body/Column/Table/Cell/Image/TextLine169개 노드의 bbox·소유 메타데이터와
쪽별 TextRun 내용·순서는 양쪽이 동일하다. TextRun의 글자 bbox까지 같다는 주장은 아니다.
현재4쪽 review에서 Print의 세 번째 자식 행이 뒤쪽으로 이동하고,6쪽은 Print의 빈
이어받기 띠에 Native 본문이 남는 것을 직접 확인했다. fresh WASM 비교는 대기 중이다.

#7788을 원격에서 다시 확인하면 기존 등록은 마지막 빈 문단의 셀 외곽선 함수와
당시5쪽89.52230%·6쪽100% 기록이다. 같은 파일명만으로 현재4–6쪽의 전체
쪽 소유 차이가 이미 등록됐다고 판정하지 않는다. 위 과거 단계의 #7788 연결과
현재 증상 범위를 구분하며, 이관 판정 전에 범위 연결을 명시적으로 보완한다.
정확한 기존 검사와 원본·Print는 유지했고 검사 제거·기대값 갱신은 하지 않았다.

ignored 근거: `c20-generated-split-pinned-base-current-native-comparison.json`,
`c20-generated-split-existing-issue-scope-audit.json`,
`c20-current-native-representative-review/valid_generated-nested-split/review/review_004.png`,
같은 경로의 `review_006.png`, `c20-pinned-upstream-generated-split-native-results.json`.
