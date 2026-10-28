---
kind: evidence
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-13
---

# PR #7814 체리픽·메인터너 보정 검토 계획

## 2026-10-15 최종 통합 검토 연결

최종 적용 범위와 판정은 [개별 review](pr_7814_review.md)와 [통합 review](planet6897_followup_20261015_review.md)를 따른다. 검증 code SHA는 `39a634e547621c32ae6f9ba8c1694d35c827078b`다. 최초 전체 실행·사용자 승인한 집중 재검증 합산, 전21문서181쪽 TSV와 최종 로컬 게이트를 구분해 기록했다. 아래 단계별 “현재”·미완료·C20 기록은 각 작성 당시의 이력이며 최종 상태를 대신하지 않는다. 원격 통합 PR의 CI·merge는 실제 확인 전 완료로 표시하지 않는다.


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

검토 진행 중이며 수용·승인 판정은 아직 하지 않았다. 범위는 **HWP3 변환 글꼴 이름**다.

- branch: `review/planet6897-followup-20261011`
- 초기 검토 base(과거 단계): `bfa28a77a2714482f647ee6a7b9aa7f7876d8404`
- 원 PR head: `03b415cde31ed9e7164cf1828f5e4811ef59ce6b`
- 기본 경로: `collaborator_external_pr` 9.1.1, 주 작업공간 체리픽 통합
- 보조 경로: intake, local validation, visual fixture, multi PR, rework

| 원 고유 commit | 적용 commit |
| --- | --- |
| `03b415cde31ed9e7164cf1828f5e4811ef59ce6b` | `66a62a74a339b8ea4410bc076184a4ccdd453d5f` |

원 fork history를 보존하고 중복 선행 commit·merge commit은 다시 적용하지 않았다.

## 구현 주장과 확인할 경로

HWP3 name mapping → document font IDs → renderer face/HFT 선택 및 HWPX 왕복을 확인한다. #7815 적층의 같은 font patch는 중복 적용하지 않았다.

## 독립 기대값과 검증 계획

원본 8개와 독립 한컴 변환본의 이름 계약, 대응 Print 전쪽, HWP3 왕복과 실제 진단 출력 hash 변경 내용을 확인한다. SO-SUEOP·sample16의 알려진 차이는 완료 증거가 아니다.

최종 통합 source의 해당 lint·회귀·Native/fresh WASM 전쪽 TSV·대표 review 확대 판독이
완료되기 전에는 통과로 판정하지 않는다. 기여자 PNG·기여자 CI는 최종 통합 head의 증적을
대신하지 않는다. raw TSV·중간 JSON·로그는 ignored
`output/pr-review/planet6897-followup-20261011/`에 보존한다.

입력/Print SHA, 실제 source SHA·명령·관측값과 `충족 / 미충족 / 미검증 / 비해당`
판정은 실행 결과로 이 기록을 보완한다. 필요한 오늘할일은 최종 검토 묶음과 함께 통합 PR에
포함하며 별도 기록 PR을 만들지 않는다.


## sample11 기존 독립 PDF 재사용 근거 — 2026-10-13

현재 HEAD `e2a525d36`의 원본 HWP3와 기존
`pdf/hwp3-sample11-hwp-2020.pdf`는 이전 #7091 및 #7078 공통 검토에 기록된
원본·독립 정본의 SHA와 정확히 같다. PDF는26,386,908bytes·151쪽이며 실제
Creator는Hwp2022다. #7091은 이 PDF를 독립 sample11 정본으로 명시했고,
한컴 변환 HWP5의 별도 출력과 기존 정본의 차이도 구분해 보존했다.

현행 기준 PDF 재사용 절차에 따라 이 기존 경로를 유지한다. 원래 호출 로그와
정확한 빌드는 회수하지 못했으며 새2022 출력으로 보고하지 않는다. 대표1·76·151쪽의
PDF 개방·raster 생성도 확인했다. 버전 표기만을 이유로 재변환하거나 Windows COM
등록을 변경할 필요는 없어졌다. 앞서 실패한2020 출력 시도·실패 증거는 보존한다.
이 재사용 판정은 현재 Native/fresh WASM 전151쪽·90%·확대 판독의 완료가 아니다.

입력/Print SHA·Git blob 일치·기존 출처와 미확인 항목은 ignored
`c20-hwp3-original11-existing-oracle-reuse-audit.json`에 기록했다.

## sample10 원본 Print PDF 크기 예외 승인 — 2026-10-13

사용자가 **“96MB 원본 PDF 커밋 예외 승인”**으로 50 MiB 상한의 예외를 승인했다.
전763쪽의 원본 PDF를 [`pdf/hwp3-sample10-original-2020.pdf`](../../../pdf/hwp3-sample10-original-2020.pdf)에
일반 Git blob으로 보존한다. 파일은 96,135,304바이트이며 분할·재인코딩·페이지 생략 없이
수신 원본과 SHA-256이 같다.

| 항목 | 확인값 |
| --- | --- |
| 원본 입력 | `samples/hwp3-sample10.hwp` |
| 입력 SHA-256 | `d9ceb35d8abfb73e9afbe349bccb2a986cf552d66ae7f3f34dcfe4d9d385cb48` |
| PDF SHA-256 | `f96b37df92a720389c342bbe84e5782790c80b44b8ee0e6311952ee8ed262fb9` |
| 출력 출처 | Windows Hancom Office 2020 `11, 0, 0, 9136`, COM `PrintToPDFEx`, Hancom PDF |
| 인쇄 계약 | `PrintMethod=0`, 전체 범위, 100%, 1부; 논리 쪽수와 실제 PDF 모두763쪽 |

출력 실행·수신 증거는 ignored `c20-hwp3-original10-com-print-provenance.json`,
사용자 승인·보존 파일 확인은 `c21-hwp3-original10-approved-full-pdf-preservation.json`에 기록했다.
source commit `5bd2620a3`의 전체 회귀 종료 후 이 증적을 별도 커밋한다.
그 실행은10,240건 통과·1건 실패·45건 제외였고, HML 성능 검사는533ms로500ms 상한을 넘었다.
동일 소스·상한의 단독 재검증은통과했다(테스트 전체 실행 시간88ms).
전체 실행의 실패 기록은 유지하며, 실행 격리 보정 후 전체 검증을 다시 확인한다.
이 단계의 Native Skia·정식 lint·fresh WASM·Canvas 재캡처는 실행되지 않았다.
상세 결과는 ignored `c21-validation-sequence-results.json`, `c21-isolated-hml-results.json`에 보존한다.
크기 예외는 전쪽 Native/fresh WASM TSV·쪽수 대조·90% 기준·직접 확대 판독을 면제하지 않는다.
이 문서의 시각 수용 판정은 아직 미검증이다.
