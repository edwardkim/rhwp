---
kind: evidence
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-13
---

# PR #7805 체리픽·메인터너 보정 검토 계획

## 2026-10-15 최종 통합 검토 연결

최종 적용 범위와 판정은 [개별 review](pr_7805_review.md)와 [통합 review](planet6897_followup_20261015_review.md)를 따른다. 검증 code SHA는 `39a634e547621c32ae6f9ba8c1694d35c827078b`다. 최초 전체 실행·사용자 승인한 집중 재검증 합산, 전21문서181쪽 TSV와 최종 로컬 게이트를 구분해 기록했다. 아래 단계별 “현재”·미완료·C20 기록은 각 작성 당시의 이력이며 최종 상태를 대신하지 않는다. 원격 통합 PR의 CI·merge는 실제 확인 전 완료로 표시하지 않는다.


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

검토 진행 중이며 수용·승인 판정은 아직 하지 않았다. 범위는 **장치 격자와 상대 크기 자간**다.

- branch: `review/planet6897-followup-20261011`
- 초기 검토 base(과거 단계): `bfa28a77a2714482f647ee6a7b9aa7f7876d8404`
- 원 PR head: `b35f6eb56024033ce0ebf4e8488e6affef4a0067`
- 기본 경로: `collaborator_external_pr` 9.1.1, 주 작업공간 체리픽 통합
- 보조 경로: intake, local validation, visual fixture, multi PR, rework

| 원 고유 commit | 적용 commit |
| --- | --- |
| `b35f6eb56024033ce0ebf4e8488e6affef4a0067` | `af25ec3999a7bbacfef19969fefb5d68c9960b41` |

원 fork history를 보존하고 중복 선행 commit·merge commit은 다시 적용하지 않았다.

## 구현 주장과 확인할 경로

언어 슬롯의 실제 크기 → char_width_decision 장치 전진폭 → HU 줄 나눔 → 배치·paint를 추적한다. 보충 canvas 메트릭은 적용 여부를 별도로 확인한다.

## 독립 기대값과 검증 계획

검증용 합성 입력 4개와 대응 Print의 전쪽을 먼저 비교한다. 78494의 쪽수·누락 줄, 부동소수 HU 경계, ratio·relative size·space tie를 독립 PDF 간격과 대조한다.

최종 통합 source의 해당 lint·회귀·Native/fresh WASM 전쪽 TSV·대표 review 확대 판독이
완료되기 전에는 통과로 판정하지 않는다. 기여자 PNG·기여자 CI는 최종 통합 head의 증적을
대신하지 않는다. raw TSV·중간 JSON·로그는 ignored
`output/pr-review/planet6897-followup-20261011/`에 보존한다.

입력/Print SHA, 실제 source SHA·명령·관측값과 `충족 / 미충족 / 미검증 / 비해당`
판정은 실행 결과로 이 기록을 보완한다. 필요한 오늘할일은 최종 검토 묶음과 함께 통합 PR에
포함하며 별도 기록 PR을 만들지 않는다.

## 합성 검증 입력의 정상 저장과 검출력 확인

원래 4개 HWP는 한 `BodyText/Section0`의 처음/끝에 구역 정의를 중복 포함했다.
Native는 1쪽, 독립 Print는 2쪽이었다. 실제 한컴 Office 2020으로 정상 저장한 대조군은
두 BodyText 스트림 각각의 첫 문단에 구역 정의를 갖는다. 실패 원본과 양쪽 SHA를
`pdf/issue7702/input-evidence/`에 보존하고 활성 samples를 정상 저장본으로 바꿨다.
이 작업은 검증 입력의 생성 오류를 보정하며 원본 입력의 렌더링 문제 해결로 보고하지 않는다.

정상 저장 후 독립 Print(method0)한 8쪽은 기존 기준 PDF 8쪽과 RGB 픽셀이 전부 동일하다.
따라서 기존 Print와 회귀 기대값은 그대로 사용했다. 문단 수·공백 제외 본문도 보존했다.
source `ab5112`의 Native/fresh WASM 전체 페이지 최저는 99.56697% 이상이다.
fresh WASM의 상대 크기·자간 대표 PNG와 확대본에서 크기·간격·행 순서를 직접 확인했다.
획 raster 차이는 남지만 누락·중복·줄 이동은 관찰하지 않았다. 최종 통합 head에서는 시각
출력과 필수 게이트를 새로 고정한다.

같은 정식 회귀 원본과 정상 저장 입력을 #7805 전의 source
`f22cdb0359cd0c0c400e1b0348c7f8c63f53be0c`에 연결하면 4건 모두 기대한 전진폭·자간 원인으로
FAIL하고, 보정한 production source에서는 4건 모두 PASS했다. 검사를 삭제하거나 허용치를
바꾸지 않았다. 로그는 `pr7805-normalized-formal-tests-before.log`와
`pr7805-normalized-formal-tests.log`이다.

ignored 검토 output의 `pr7805-original-normalized-print-raster-equality.json`,
`pr7805-normalized-native-results.json`, `pr7805-normalized-fresh-wasm-results.json`,
`pr7805-normalized-fixture-review.png`, `pr7805-normalized-fixture-review-enlarged.png`에 연결한다.
