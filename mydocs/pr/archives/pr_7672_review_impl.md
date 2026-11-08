---
kind: evidence
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-13
---

# PR #7672 체리픽·메인터너 보정 검토 계획

## 2026-10-15 최종 통합 검토 연결

최종 적용 범위와 판정은 [개별 review](pr_7672_review.md)와 [통합 review](planet6897_followup_20261015_review.md)를 따른다. 검증 code SHA는 `39a634e547621c32ae6f9ba8c1694d35c827078b`다. 최초 전체 실행·사용자 승인한 집중 재검증 합산, 전21문서181쪽 TSV와 최종 로컬 게이트를 구분해 기록했다. 아래 단계별 “현재”·미완료·C20 기록은 각 작성 당시의 이력이며 최종 상태를 대신하지 않는다. 원격 통합 PR의 CI·merge는 실제 확인 전 완료로 표시하지 않는다.


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

검토 진행 중이며 수용·승인 판정은 아직 하지 않았다. 범위는 **미주 제목 소유 경계**다.

- branch: `review/planet6897-followup-20261011`
- 초기 검토 base(과거 단계): `bfa28a77a2714482f647ee6a7b9aa7f7876d8404`
- 원 PR head: `b387d1358256c0ae02f68b8c2a497026c3f0d1e9`
- 기본 경로: `collaborator_external_pr` 9.1.1, 주 작업공간 체리픽 통합
- 보조 경로: intake, local validation, visual fixture, multi PR, rework

| 원 고유 commit | 적용 commit |
| --- | --- |
| `d9976901da859be26367b32f5220efa01618de5c` | `72fb611e6cfbbac2526447639d0c8396f0030d54` |

원 fork history를 보존하고 중복 선행 commit·merge commit은 다시 적용하지 않았다.

## 구현 주장과 확인할 경로

미주 등록 범위 → 첫 문단 소유 조회 → 미주 측정·배치의 제목 여부를 추적한다. #7832의 미주 범위 검사와 새 첫 문단 검사를 함께 보존했다.

## 독립 기대값과 검증 계획

같은 미주 본문의 문 접두사·다른 접두사 편집 대조군을 저장하고 같은 편집본의 독립 Print를 확보한다. 원본 출력 무변화로 편집 경로를 비해당 처리하지 않는다.

최종 통합 source의 해당 lint·회귀·Native/fresh WASM 전쪽 TSV·대표 review 확대 판독이
완료되기 전에는 통과로 판정하지 않는다. 기여자 PNG·기여자 CI는 최종 통합 head의 증적을
대신하지 않는다. raw TSV·중간 JSON·로그는 ignored
`output/pr-review/planet6897-followup-20261011/`에 보존한다.

입력/Print SHA, 실제 source SHA·명령·관측값과 `충족 / 미충족 / 미검증 / 비해당`
판정은 실행 결과로 이 기록을 보완한다. 필요한 오늘할일은 최종 검토 묶음과 함께 통합 PR에
포함하며 별도 기록 PR을 만들지 않는다.


## 편집본 출처 재확인과 시각 실패 — 2026-10-12

첫 편집본을 current Native CLI `467bfd1d8`로 같은 원본·같은 명령으로 다시 생성했다.
`2023-note348-mun.hwp`는 이전 `f083228cd` 생성본과 바이트 및 SHA-256
`29317054c045cdb4d4fbbd92a9a86ac1cd8c1bf81dda8873166c752de8b1516b`가 동일하다.
따라서 이전 CLI가 만든 잘못된 편집본이라는 가정을 시각 실패의 원인으로 사용하지 않는다.
기존 편집본에 대응하는 독립 Print는 계속 동일 입력의 기준이다.

AB production의 기존 전쪽 raster에서 1·15·20쪽 review를 만들어 직접 판독했다.
15쪽(37.26871%)에서 같은 미주 문항의 세로 간격 및 뒤 칼럼 시작 내용이 독립 Print와
다르다. 글꼴만의 차이·낮은 평균·과거 입력이라는 분류로 면제하지 않았다.
같은 입력의 current production Native 전쪽 재출력은 진행 중이다.

ignored output의 `pr7672-current-edit-input-byte-comparison.json`,
`pr7672-edited-native-scores-valid-subsets/pr7672-2023-note348-mun/pr7672-2023-note348-mun/review/review_015.png`,
`pr7672-current-467-native-sweep.log`에 연결한다. 기존 raster 판독과 current 재출력을
구분하며 미주 제목 소유 보정의 전체 시각 수용은 아직 미충족이다.


## 미주 높이 진단의 독립 대조 — 2026-10-12

동일 편집본의 2020 Print와 새 2024 Print 모두 전쪽 90% gate를 충족하지 않는다.
원본을 새 2024 엔진에서 그대로 인쇄한 출력과 편집본의 15쪽은 100% 일치하여,
문 접두사 편집·rhwp 저장 자체가 그 쪽 차이의 원인이라는 가정은 배제했다.
기존 원본 PDF와 새 Print는 별도로 보존했으며 어느 쪽도 버전 표기만으로 무효로
판정하지 않는다. 원본/새 Print 공통 HyhwpEQ 128글자와 Haansoft Dotum 245글자의
전진폭·분해 윤곽선은 동일하다.

frozen production `467bfd1d8`의 실제 미주 debug에서 22번 미주 p12·p26·p35의
formatter 높이는 저장 LineSeg와 일치한다. 빈 줄에 기본 12pt를 잘못 가산한다는
가정은 배제했다. 같은 미주 제목은 Print와 맞지만 이후 알짜 풀이 제목은 약15px,
따라서 문단과 다음 미주 제목은 약22.5px 아래로 밀린다.

독립 한컴 2024 재저장은 원본의 미주 LineSeg를 유지하면서 1,591수식 중60개의
common 높이를 변경했다(미주 수식54개). 22번 미주의 단순 prime 수식은1125→900HU,
prime+위첨자는1163→1050HU로 바뀌었고, 이 차이는 관측한 누적 전진 차이와 대응한다.
이는 원본·재저장본 모두 실제 한컴 생성본임을 유지한 대조다. 현재 AST intrinsic 높이가
새 한컴 높이와 같은 경우는54개 중33개뿐이므로 모든 수식을 AST 값으로 치환하는
구현은 채택하지 않았다. cache 수용·제어 크기·미주 누적과 실제 paint의 공유 경로를
더 확인해야 하며 전체 시각 수용은 계속 미충족이다.

ignored 증거: `pr7672-note22-height-debug.log`,
`pr7672-original-vs-edited-same-engine2024-print-p15.json`,
`pr7672-old-fresh-print-equation-font-probe.json`,
`pr7672-original-resaved-equation-common-comparison.json`,
`pr7672-equation-intrinsic-original.json`, `pr7672-equation-intrinsic-resaved.json`.


## 원 PR의 최신 동기화 확인 — 2026-10-12

최신 fork head는 `c70ad95b65653acd51a63a05d3dbbfbac883421b`다. 기존 검토 head와 고정 BFA를 제외한 새 commit은
upstream/devel 동기화 merge 하나뿐이며 새 작성자 고유 commit은 없다. 기존 적용 고유
commit과 출처 표는 유지한다. 첫 미주 문단 소유 조회와 BFA의 등록 원본 범위 판정을 함께 유지한 병합이다. 현재 후보도 두 판정을 구분한다.

combined merge diff와 graph 근거는 ignored
`pr-7672-latest-merge-combined.diff`, `source-pr-refresh-merge-review.json`,
`nine-source-pr-live-20261012.json`에 있다. 원 PR 상태는 OPEN이며 원격 devel은 BFA다.


## 저장 캐시 변경의 진단 한계 — 2026-10-12

독립 한컴 재저장본의 새 수식 크기를 쓰고, 변경된 수식 소유 줄의 저장 캐시를 그 차이만큼
조정한 진단clone을 실행했다(46줄,20쪽). 원본·독립 재저장 파일은 바꾸지 않았다.
같은 fresh 원본2024Print에 대한 진단14쪽89.36291%,15쪽37.20063%이며,
14쪽의 다음 문항 첫 내용과15쪽의 칼럼 시작 소속이 여전히 다르다. 정상 텍스트 줄로의
누적 차이 전파를 추가한 진단도14쪽 점수를 바꾸지 않았다.
수식 common 크기 차이는 근거 있는 원인 단서지만 이를 캐시 덮어쓰기만으로 해결했다고
판정하지 않는다. 이 진단clone을 원본 일치·새golden·정식fixture 근거로 사용하지 않는다.

증거는 `pr7672-resaved-equation-cache-diagnostic.rs`,
`pr7672-resaved-equation-cache-raster/metrics_014.json`, `metrics_015.json`,
대응review와 `pr7672-resaved-equation-cache-propagation-raster/`다.


## 새 실물 검사 후보 등록 보류 — 2026-10-12

고정BFA에 없던 incoming 검사를 전쪽 시각 선행 조건에 맞춰 활성 등록에서 보류했다.
보류 대상은 #7672의4건과 #7692의3건, 합계7건이다. 원문은 바이트와 해시를 유지해
[보존 기록](../assets/issue7445/incoming-unqualified-regressions-20261012/README.md)에 있다.
이는 기존 devel 검사 이관이나 출력 결함 해결이 아니다. 이전10241PASS는 이 후보들을
포함한 실행 결과이며 등록 조건까지 충족한 것으로 승격하지 않는다. 정상 대조·전쪽 시각
수용은 계속 필요하고, 최종 파생 suite 준비·필수 lint·전체 정식 검사 수는 재확인한다.


## 단순 대조군과 경계 뒤 줄간격 변경 확인 — 2026-10-12

공개 편집 API로 작성한 수식 없는 한 미주 문서를 한컴2020에서 저장한 뒤, 같은 본문에
`문서 `와 `가나 `를 각각 삽입하고 독립 Print를 확보했다. Native/fresh 전쪽1쪽은
각각92.25034%와92.19368%지만, 정확한BFA의 소유 문단 줄 배치도 현재4ce production과
동일했다. 따라서 이 입력은 원 결함의 수정 전FAIL 증거가 아니다. 직접 review 판독에서도
미주 번호 위치 및 후속 번호 중복이 Print와 달라 정식fixture/회귀에 등록하지 않았다.
원본20쪽이나 다른 원본의 수용 증거로 사용하지 않는다.

현재4ce production에서 실제 원본 편집본14쪽을 debug 재출력했다. 문단758의 진행량은
처음36.7px이며, 다음 미주23 경계 준비가 앞 문단의 마지막 줄간격을452→1984HU로
바꾸고 나면 같은 단 재측정·실제 출력에서57.1px가 된다. 차1532HU=20.42667px는
`prepare.rs`의 `apply_endnote_render_tail_spacing` 호출과 state command의 저장 줄 변경으로
설명된다. 서로 다른 줄간격 상태를 같은 입력으로 간주한 비교가 아니며, 이를 helper의
인덱스 차이만으로 설명하지 않는다. 이 간격 제거·누계 변경이 독립 Print와 맞는지는 아직
입증되지 않았으므로 근거 없는 clamp/간격 삭제를 구현하지 않았다.

ignored 증거: `plain-endnote-owner-before-after-lines.json`,
`plain-endnote-owner-{native,wasm}-{native,wasm}-results.json`, 대응review1쪽,
`pr7672-current-tail-spacing-debug.log`, `pr7672-control-and-tail-spacing-audit.json`.

같은14쪽의 독립 Print와 현재4ce raster를 같은 좌표·배율로 확대 대조했다.
미주23 제목뿐 아니라 앞 적분 수식도 함께 내려가 있어, 마지막 render tail 간격만
삭제하면 앞 수식의 오차를 남기면서 실제 미주 사이 간격을 줄이게 된다.
단순 tail 간격 제거를 보정으로 적용하지 않았다.

미주22 내부의 같은 높이12px 본문 줄 사이 거리를 대조했다. `양변을 미분하면`부터
첫 `양변의 계수 비교`까지 Native365.2px / Print350.4px, 차14.8px다.
독립 재저장본에서 이 범위의 수식 소유 줄5개가 각각225HU 줄어든 합계는15px다.
같은 기준 줄부터 마지막 `따라서`까지는 Native692.2px / Print669.76px,
차22.44px이며 소유 줄8개의 캐시 높이 차 합계1688HU는22.50667px다.
누적 차이가 미주 내부의 수식/줄 높이 변화와 대응하는 원인 증거다.
이 결과를 원본 수식 캐시의 무조건 덮어쓰기나 전체 문서 개선 완료 근거로 쓰지 않는다.
독립 재저장본의 script/AST 변화·저장/재조판 계약을 구분한 보정이 아직 필요하다.
증거: `pr7672-note22-23-boundary-direct-review/review/review_014.png`,
`pr7672-note22-23-boundary-direct-review/note22-23-p14-2x.png`,
`pr7672-note22-relative-cache-drift-observations.json`,
`pr7672-print-notes22-24-fragment-owner-observation.json`.
원본 편집8개·독립Print 총180쪽의 현재4ce Native/fresh 전쪽 검증 결과는
아래 전쪽 검증 절에 기록했다. 최초browser 시작 오류1건도 재실행·측정을 완료했다.


## 실제 편집본 전쪽 검증 범위 확장 — 2026-10-12

각각 `문서 ` / `가나 `를 삽입한 실제 편집본8개·독립 Print 총180쪽을 현재4ce
Native와 fresh web WASM에서 전쪽 산출했다. 입력·PDF16개 SHA를 고정 corpus와 대조했다.

| 편집 대상 | 문서당 쪽수 | Native 최저 / 미달 쪽수 | fresh WASM 최저 / 미달 쪽수 |
| --- | ---: | --- | --- |
| 2023 note348 두 편집본 | 20 | 각각29.12455% /12 | 각각29.12455% /12; 가나 최초browser launch 오류 후 해당 건 재실행·측정 완료 |
| 미주사이20 note134 두 편집본 | 24 | 각각35.57167% /16 | 각각35.57167% /16 |
| 미주사이20 note254 두 편집본 | 24 | 각각35.57167% /16 | 각각35.57167% /16 |
| 구분선위0·사이20·아래2 note229 두 편집본 | 22 | 각각40.44552% /13 | 각각40.44552% /13 |

측정된 범위는 Print와 전체 쪽수가 같지만 전쪽90% gate를 통과하지 않았다.
프로세스 runner가 결과를 수집하고 정상 종료한 것과 각 visual gate의 exit1을 구분한다.
브라우저 시작 오류를 시각 미달 또는0%로 세지 않는다. 원본이 미달인 동안 후보4건의
등록 보류를 유지하며 기대값을 낮추지 않는다.

ignored 결과: `pr7672-all-edits-current-4ce-native-results.json`,
`pr7672-all-edits-current-4ce-wasm-final-results.json`와 대응 TSV. 최초browser 오류 원문은
`pr7672-all-edits-current-4ce-wasm-results.json` 및 raw log에 별도로 유지한다.
각 결과의 raw log를 code source 이름에 맞는 `*-current-4ce-{native,wasm}-sweep.log`로
보존했다. 예전467 접미사를 가진 재실행 log도 현재 header는4ce이므로 과거467 증거로
다시 인용하지 않는다. 이전467의 score·manifest·source provenance는 별도로 보존한다.


## 고정 devel과 현재 후보의 같은 조건 비교 — 진행 중

C12에서 수식이 없는 사회기여 표를 보정한 뒤 미주 편집 문서의 원인을 다시 분리한다.
앞선4ce 전쪽 결과의 font path에는 현재 검증된 `validated-print-hcr-batang-program`과
complete unicode 공급 경로가 포함되지 않았으므로 같은 실행 조건의 현재 후보 결과로
승계하지 않는다. 다른 앞선 결과의 조건을 바꾸거나 통과 판정을 덮어쓰지 않는다.
실제 편집 `2023-note348-mun.hwp`와 대응20쪽 Print를 같은 검증된 글꼴로 현재 C12와
고정 BFA Native에서 순차 측정하기 시작했다.
입력·PDF 해시는 기존 paired corpus의 값과 재대조했으며 새 대체 원본을 만들지 않았다.
결과는 `pr7672-note348-base-c12-native-{progress.log,results.json}`에 보존한다.
현재 최소값·전체 페이지 비교가 종료하기 전 원인을 기존 미달/새 회귀로 단정하지 않는다.


같은 실제 편집본·Print·현재 검증 글꼴의 Native 비교가 완료됐다.
고정 BFA는 **20/20쪽·최저28.91644%**, C12는 **20/20쪽·최저29.07705%**다.
양쪽 모두 1·2·3·11·13·14·15·16·17·18·19·20쪽이90% 미만이다.
현재17쪽은2.92567%p 개선됐지만 최소값·나머지 배치의 미달을 상쇄하지 않는다.
기존 cache/미주 흐름 차이를 이번 API 변경의 새 회귀로 단정하지 않으며, 이 비교만으로
이관·기대값 변경·수용 판정을 하지 않는다. 이 원본의 실제 출력 보정은 남아 있다.
기존4ce 결과를 새 글꼴의 결과로 승계하지 않고 이 종료된 비교를 기준 진단에 연결한다.


## 미주22의 실제 저장 줄과 수식 상자 직접 확인

동결 C12 CLI `dump-endnote-lines`로 실제 편집본과 독립 한컴 재저장본의
`s0:p218:ci0:note4`를 직접 대조했다. 양쪽 저장 줄은 동일한
`vpos=6806, lh=1125, bl=878, gap=452`다. 원본 수식2개의 상자는 높이1125·기준선78%이고,
재저장본은 높이900·기준선86%로 바뀌었다. script·BaseUnit900은 같은 사례다.
재저장이 저장 LineSeg 자체를900으로 갱신했다는 해석은 근거가 없다.
기존 metadata의 line_seg_diff 목록이 비었다는 관측을 실제 덤프에서 재확인했다.

`dump-endnote-lines`의 기본 compose도 두 입력 모두 lh1125를 소비하므로,
수식 common 상자만 바꾸는 것으로 저장 줄·format·실제 paint의 공통 결과가 확보됐다고
판정하지 않는다. 다음 보정 전에 저장 줄 재사용과 재조판의 소비 경로를 구분하고
최종 줄 전진량·기준선을 추적한다. 현재 원본/Print의22px 누적 차이는 남아 있으며
픽셀 목표 clamp 또는 원본 metadata의 무조건 덮어쓰기를 적용하지 않았다.
증거: `pr7672-c12-note22-p4-stored-line-and-equation.log`,
`pr7672-resaved-note22-p4-stored-line-and-equation.log`.
C12 제품/회귀 단계는 로컬45a7a292a에 고정했으며, 이번 원인 추적은 다음 단계 기록이다.


## 미주 수식 높이의 실제 소비 경로 — 2026-10-13

제품 코드는 `45a7a292ae5767069b4ee248daea87f4b631b94c`에 고정한 채 조사했다.
현재20쪽 Native raster로14·15쪽 compare/overlay/review를 구성했고, 기존4ce PNG를
현재 증거로 재사용하지 않았다. overlay helper가0부터 시작하는 page 인자를 받으므로
첫 합성의 쪽 라벨을 바로잡고 PNG 해시도 다시 기록했다. 원 raster는 바꾸지 않았다.
14쪽은 미주22 제목이 대응하지만 이후 수식/본문 간격이 누적되어 미주23 첫 본문이
Print와 다른 쪽에 놓인다.15쪽은 양쪽 단의 시작/뒤 문항 위치가 다르다.
이 차이는 글꼴 예외나 전체 쪽수 일치로 면제할 수 없다.
증거: ignored `pr7672-note348-base-c12-direct-review/provenance.json` 및
`current-c12/review/review_014.png`, `review_015.png`.

| 단계 | 실제 소비 경로와 관측 |
| --- | --- |
| 수식 흐름 상자 | `equation::flow_metrics_hwp`는 일반 영문 수식의 저장common 폭·높이·백분율 기준선을 유지한다. 한글 대체 글꼴 경로만 AST 메트릭을 공유한다. |
| 저장 줄 수용 | `composer::line_breaking::remeasured_equation_start_row`는 현재 폭 차이만 검사한다. 높이만 달라진 수식이 같은 부분 재조판 경로에 자동 진입한다고 볼 수 없다. |
| 미주 측정 | `endnotes::paragraph::typeset_endnote_paragraphs`는 원본 미주 문단을 compose/format한다. `resolve_line_metrics`는 합성/글꼴 보정 조건을 제외하면 저장 줄 높이를 소비한다. |
| paint 입력 | 같은 함수가 별도clone에 미주 시작vpos와 첫 문단 번호를 적용해 `append_endnote_paragraph`로 등록한다. 한쪽 복사본만 바꾸면 측정과paint의 결과 공유가 깨진다. |
| 실제 수식 원점 | `paragraph_layout.rs`의 TAC-only와 run 혼재 분기는 모두 흐름 상자 높이와 AST 표시 기준선을 소비한다. TAC-only는행 y·줄 기준선에서, 혼재는 공백 여부와 셀 조건에 따라 최종 y를 정한다. common 높이 교체만으로 줄 전진/최종 원점 보정을 입증하지 않는다. |
| 다음 문단 | `HeightCursor`의 수식 재조판 재앵커도 위 부분 재조판 판정을 소비한다. 저장 사다리가 후속 문단에서 옛 높이를 복원하는지 같이 확인해야 한다. |

실제 한컴 원본/재저장본의 같은 script 미주 수식 1,119개 중54개의 높이가 달랐다.
33개는1125→900,5개는1163→1050,16개는1305→1080HU이며 모두 BaseUnit900이다.
53개에 prime/작은따옴표가 있고 나머지는 `rm triangle BPQ times cos theta`다.
prime 문자만 조건으로 삼는 보정은 마지막 사례를 설명하지 못한다.
정상 대조군의 x제곱은1050, 제곱근 포함 첨자는1142, 분수는2172HU를 원본/재저장본이
같이 유지한다. 현재 AST가 각각900/900/2196HU를 반환하므로 전체 저장 상자의 AST 덮어쓰기는
독립 기준으로 정당화되지 않는다. 높이 차이가 있는 수식에 별도 수치를 맞추는 보정을 추가하지 않았다.
증거: `pr7672-original-resaved-same-script-height-changes.json`,
`pr7672-resaved-vertical-structure-negative-controls.json`.

현재 C12와 고정 BFA의 fresh web WASM도 같은 편집본·Print·전체20쪽·검증된 글꼴로
비교를 시작했다. 두 package의 실제 WASM/JS 해시와 C12 production diff의 checkpoint 일치를
확인했다. Native CLI는 note/font 정책 조회에 쓰고 렌더는 각 fresh WASM을 사용한다.
진행/종료 결과는 `pr7672-note348-base-c12-wasm-progress.log`, `*-results.json`에 보존한다.
완료 전에는 fresh WASM 전쪽 검증 완료로 기록하지 않는다.


### 현재 checkpoint 실제 formatter 재확인

`RHWP_ENDNOTE_LINE_DEBUG=1`과 동결 C12 Native로 `dump-pages --compat 2022 --json`을
실제 편집본/독립 재저장본 각각 실행했고 두 건 모두exit0·20쪽이다.
미주22의 첫 문항·뒤 본문을 포함한75진단줄을 각 입력별로 보존했다.
para4는 실제 편집본의 수식 상자1125HU→재저장본900HU인데 두 formatter 모두 높이15px·
advance21.0267px다. para30도 상자1163→1050HU이나 formatter 높이15.5067px·
advance21.5333px를 같이 유지한다. 이는 기본 compose 덤프뿐 아니라 최종 조판 format 호출에서
확인한 저장 줄 소비다. 독립 재저장본을 실제 편집본의 대체 기준으로 사용하지 않았다.
증거: `pr7672-c12-actual-format-results.json`,
`pr7672-c12-{edited,genuine-resaved}-actual-format-note22.log`.


### 현재 Native/fresh WASM 전쪽 결과와 빌드 게이트

현재 C12의 같은 편집본20쪽 fresh web WASM 비교가 종료됐다.
Native와 모든 페이지 점수·쪽수·입력/Print SHA·글꼴 조건이 동일하며,
최저29.07705%, 미달1·2·3·11·13·14·15·16·17·18·19·20쪽이다.
`pr7672-note348-c12-current-native-wasm-paired-audit.json`으로 재대조했다.
BFA fresh WASM 비교는 별도 실행 중이며 아직 완료 비교에 포함하지 않는다.
Print / 고정BFA / 현재 C12의 같은14쪽 미주22 영역을96dpi 원본PNG에서2배 확대해
`pr7672-note348-base-c12-direct-review/review.png`로 구성했다. crop은 시각 판독용이며
회귀 검사의 좌표 선택/기대값에 사용하지 않는다. 원 raster/확대 PNG 해시를 별도로 보존했다.

code checkpoint45a7에서 prepare·fmt·Native Clippy·WASM Clippy가 통과했다.
최초workspace 빌드는 병렬 링크의 signal7 Bus error로exit101이었다. 같은 시점
대표17쪽PNG 합성에도errno28(디스크 공간 부족)이 발생했다. 공유 target을 삭제하지 않고
`CARGO_BUILD_JOBS=2`로 동일 workspace 빌드를 재실행하여 exit0을 확인했다.
partial PNG를 같은 완료 raster로 다시 합성하고 현재/BFA14·15·17쪽 PNG 해시/완전성을 확인했다.
통과한 Clippy 두 경로를 반복하지 않고 이어서 all-targets Clippy·base 정책·전체 Rust·Skia를
동일 `/home/tsjang/rhwp/target/pr-review`에서 순차 실행하도록 했다.
진행을 전체 검증 완료나 PR 승인으로 보고하지 않는다.
로그: `shared-first-paragraph-band-validation-{progress.log,results.json}`,
`shared-first-paragraph-band-validation-resume-jobs2-{progress.log,results.json}`,
`shared-first-paragraph-band-validation-linker-retry-provenance.json`.


검증 후속 상태: 같은45a7에서 **Native/WASM/all-targets Clippy3경로 및 workspace
빌드·manifest/base 비교·unit-tier/base 비교까지 통과**했다. 전체nextest는
UTC2026-10-12 15:19:45에 시작했으며 최종summary/exit code는 아직 없다.
`shared-first-paragraph-band-validation-resume-jobs2-all-nextest.log`에 연결한다.

## 검토 범위 밖 수식 실험 철회 — 2026-10-13

추가 진단에서 `x!=0`의 ASCII 관계 기호 문제를 발견해 수식 parser 실험
`64ba01b32`와 포맷 `062373e1b`를 작성했다. 독립 문법 진단은 3실패·1통과에서
4통과로 바뀌었으나 원본 API 생성 문서는 Native/fresh WASM 모두 86.87259%였다.
한컴 재저장 대조군은 별도 입력이며 Native 91.01124%와 확대 판독에서 남은 글꼴 차이를
확인했다. 대조군 통과를 원본의 통과로 보고하지 않는다.

실제 fresh WASM의 새 `insertEquation` 출력 폭도 확인했다. 기존 생성본의 2673 HWPUNIT와
독립 한컴 재저장본의 2811 HWPUNIT에 비해 실험 생성본은 2193 HWPUNIT로 작아졌다.
기호 의미 변경만 수용하고 생성 메트릭 오류를 남길 수 없어 이 추가 실험의 코드 변경을
철회한다. 실험 전 통합 C16 `a72718552`와 `src`, `crates`, `tests`, `scripts`,
Cargo 설정의 byte-identical 복원을 `git diff --exit-code`로 확인했다.

원래 9개 PR·19개 고유 source commit의 검토 범위는 유지한다. 고정 upstream/devel
`552f5228d`와 C16 사이의 equation renderer 및 equation insertion source diff는 없다.
새 글꼴/수식 메트릭 기능을 이번 통합에 추가하지 않으며, 원래 미주 편집본과 나머지 PR의
검증 완료 여부는 각 원본·독립 Print의 기존 범위로 계속 판정한다.

실험은 `review/planet6897-equation-alias-experiment-20261013`의 `062373e1b`에 보존한다.
입력·한컴 재저장본·Print·문법 진단·TSV·확대 이미지·생성 폭 관측은 ignored
`output/pr-review/planet6897-followup-20261011/equation-ascii-relations-diagnostic/`와
`equation-experiment-withdrawal-plan.json`에 남긴다. 활성 회귀 검사를 추가하거나
기존 기대값을 완화하지 않았다.

## 현재 전쪽 Native 재검증과 실제 Print 세대 — 2026-10-13

현재 `e2a525d36`의 같은 편집 입력·독립 Print로 `note348-mun`과 대조군은
각각20쪽 최저 **29.07841%**다. 두 입력은 실제2020 Print에 맞춰 compat2022로
검증했다. `gap20-note134-mun`은 실제2024 Print에 맞춰 compat2024로24쪽을
검증했고 최저10쪽 **35.58943%**다. 이전에 다른 세대로 산출한 진단 값과 구분한다.

현재 note348 마지막20쪽은 앞에서 넘어온 본문으로 시작하고 두 원 도형과 뒤 문30의
세로 위치·칼럼 소속이 Print와 다르다. gap20의10쪽은 첫 수식 줄부터 대응 내용이
다르고 문8/9·그래프와 오른쪽 문10/11/12의 누적 전진이 어긋난다. 실제 배치 차이를
글꼴 획 차이나 테스트 통과로 면제하지 않는다. 같은 입력과 실제 Print 세대를 사용한
고정 devel `552f5228d` 비교를 진행 중이며, 기존 미달·현재 회귀 판정은 아직 보류한다.

ignored 근거: `c20-native-whole-corpus-results.json`,
`c20-current-edited-note-direct-review/provenance.json`,
같은 폴더 `pr7672-2023-note348-mun/review/review_020.png`와
`pr7672-gap20-note134-mun/review/review_010.png`,
`c20-pinned-upstream-note348-mun-native-results.json`,
`c20-pinned-upstream-gap20-mun-native-results.json`.

## note348 최신 기준 비교 — 2026-10-13

고정 devel `552f5228d`의 같은 편집본·2020 Print·compat2022 Native 전20쪽도
미달이며 최저 **28.91722%**다. 현재 `e2a525d36`은 **29.07841%**다.
쪽별 TextRun 내용·순서는 전20쪽에서 같지만,17쪽의 배치 노드52개가 바뀌었다.
현재17쪽 오른쪽 칼럼은 앞에서 넘어온 수식 뒤 `x≥0`, `g(x)`로 시작한다.
기준 devel은 `x≥0`부터, Print는 `g′(x)`부터 시작하며 양쪽의 누적 세로 위치도
Print와 다르다. 과거 미달이라는 사실만으로 현재 출력이 무변화이거나 회귀가 없다고
판정하지 않는다. 두 출력 모두 시각 수용은 미충족이고 원인·fresh WASM 검토가 남았다.

ignored 근거: `c20-note348-pinned-base-current-native-comparison.json`,
`c20-note348-page17-base-current-direct-review/base/review/review_017.png`,
같은 폴더의 `current/review/review_017.png`.

## 17쪽 변경과 미주 소유 판정 경로 — 2026-10-13

현재 dump-pages에서 렌더 문단846은 원본 본문348의 미주 내부
`noteParaIndex=6`이며 문자열은 `문서…`로 시작한다. 번호·제목 소유자인0번
문단이 아니다. 기준의 접두사 판정은 이 본문을 제목으로 읽지만 현재 소유 판정은
제목에서 제외한다. 입력의 소유 정보로 이 분기 의미를 확인했으며 과거 픽셀로
되돌리기 위해 잘못된 제목 압축을 복원하지 않는다. 이 확인은 전체 Print 배치가
정확하다는 증거가 아니며 전20쪽 시각 수용은 계속 미충족이다.

출처 생산은 `typeset/notes/endnotes/paragraph.rs`의 원본 `ep_idx` 등록, 측정은
`typeset.rs::endnote_first_local_paras`와 `notes/endnotes/measure.rs`의 cursor 설정,
렌더는 `layout.rs::set_endnote_para_sources`·`endnote_para_starts_note`와
`height_cursor.rs::starts_endnote`다. 같은 소유 기준 뒤의 실제 배치 차이는 별도로
유지한다. 근거는 ignored `c20-note348-page17-title-ownership-path-audit.json`과
`c20-note348-current-dump-pages.json`이다.


## 최신 devel의 fresh WASM 확인과 추가 직접 판독 — 2026-10-13

동일 note348 편집본·2020 Print·compat2022에서 고정 devel `552f5228d`의
fresh WASM **전20쪽**을 완료했고 최저 **28.91722%**로 Native와 같다.
현재 Native 전20쪽 최저는 **29.07841%**이며 현재 fresh WASM은 대기 중이다.
기준도 미달이라는 사실만으로 현재17쪽 배치 변경을 무회귀로 판정하지 않는다.
ignored 근거는 `c20-pinned-upstream-note348-mun-wasm-results.json`과 대응 TSV다.

현재15쪽 확대 전 비교에서도 왼쪽은 Print에 없는 이월 확률 문단으로, 오른쪽은
앞의 분수로 시작하며 문24~30의 누적 위치와 마지막 내용의 쪽 소속이 다르다.
gap20의 마지막24쪽은 그래프·후속 증명·빈 오른쪽 꼬리가 대응하지만 수식·본문
모양과 간격의 확대 판독이 남았다. 이 한 쪽의90% 통과로 앞쪽 미달을 상쇄하지 않는다.
`c20-current-edited-note-additional-direct-reading.json`에 판독과 이미지 출처를 연결했다.


gap20-note134의 고정 devel fresh WASM도 실제2024 Print·compat2024로 **전24쪽**
완료했고 최저 **35.58337%**다. 기준 Native와 동일하다. 기준 note34820쪽은
모든 Native/fresh WASM PNG가 바이트 단위로 같고 쪽별 TSV 점수도 같다.
gap2024쪽은 쪽별 TSV 점수가 같지만17쪽의2픽셀이 다르므로 이미지 전체의
동일성을 단정하지 않는다. 두 backend의 점수 일치를 Print와의 시각 수용으로 읽지 않는다.
현재의 대응 fresh WASM 완료와 변경 영역 판독은 계속 남아 있다.
근거: `c20-pinned-upstream-gap20-mun-wasm-results.json`,
`c20-pinned-upstream-edited-note-native-fresh-wasm-parity.json`.
