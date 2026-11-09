---
kind: investigation
status: active
canonical: mydocs/manual/github_operations.md
last_verified: 2026-11-10
---

# CodeQL 잔여 경고 정리 근거

기준은 `e00f9b7e26e581896fa17eaff3c8eb3f4889f9d4`, 로컬 보정은
`612b4053db0a09cd714aa85e9fefa045f7c4d193`이다. 작업지시자는 2026-11-09에
114건의 분류와 나머지 8건의 로컬 보정·검증을 승인했다. 원격 코드 반영과 새 원격 분석은
초기 승인에 포함되지 않았다. 이후 PR 제출·머지 승인은 아래 후속 단계로 구분한다.

## 선행 처리와 원격 분류

#8745와 PR #8750은 RHWP 인증·query·Debug 경계를 보정한 완료 작업이다.
[종료 댓글](https://github.com/edwardkim/rhwp/issues/8745#issuecomment-6722224756)은
vendor·테스트·legacy 경고를 별도 범위로 남겼다. 이번 목록은 그 분류가 재오픈된 결과가 아니다.

| 이번 분류 | 건수 | 근거 |
| --- | ---: | --- |
| `used in tests` | 67 | 공개 테스트 벡터·fixture 전용 입력 |
| `false positive` | 46 | 빈 암호·RNG 작업 배열·PDF zero IV 11건, PyO3의 수치·구조 진단 35건 |
| `won't fix` | 1 | legacy PDF RC4의 호환 한정 위험 수용 |
| 분류하지 않고 로컬 보정 | 8 | PDF 본문 로그 #597, qcms 테스트 포인터 #616–#622 |

각 원격 변경은 rule·파일·행·열·source SHA 확인 → PATCH → GET 대조로 검증했다.
직후 상태는 OPEN 8건, dismissed 183건, fixed 10건이며, 기존 dismissed 69건과 fixed
10건은 변경하지 않았다. main·devel 각각에서도 같은 8건의 OPEN 상태를 확인했다.
이는 취약점 114개를 구현으로 제거했다는 뜻이 아니다. RC4의 암호학적 약점도 남는다.

- [개별 판정과 실제 분류](alert-dispositions.json)
- [최초·현재 분석 및 main/devel source 동일성](source-analysis-identity.json)
- [수정 대상 source·sink의 도입 이력](source-sink-origins.json)

## 로컬 보정과 실제 실행

PDF backend의 두 진단에서 객체 전체와 복호화한 본문을 제거하고 종류·길이·쪽 번호를
유지했다. 공개 평문/빈 암호 PDF의 실제 로그를 캡처해 본문 비노출과 원문·캐시 반환값
보존을 검사했다. qcms 테스트의 일곱 nullable transform은 reference로 바꾸기 전에 검사한다.

`c_bindings`를 실제로 실행하면서 기존 LUT helper의 `Option` 반환형이 C adapter에 연결되지
않은 컴파일 결함도 발견했다. 예산 실패 시 sticky 오류를 보존하고 호출자의 출력 배열을
덮어쓰지 않도록 연결했다. 정상 LUT의 단조성과 양 끝값도 실제 C 경로로 확인했다.
이 선택 기능을 RHWP의 기본 경로에 활성화하거나 생산용 색 변환 계산을 바꾸지 않았다.

| 실행 | 결과 |
| --- | --- |
| PDF 로그 음성/정상 대조 | 기존 본문 출력에서 2건 FAIL → 보정 후 2건 PASS |
| qcms null 음성 대조 | 컴파일 어댑터만 연결한 기존 역참조에서 SIGSEGV 11 |
| qcms `c_bindings` 전체 | 30 PASS, 0 FAIL, 0 ignored; null 검사 2건과 예산 실패/정상 LUT 포함 |
| 기존 #8745 경계 / PDF frontend | 6 PASS / 17 PASS |
| 전체 release-test | 13,015 PASS, 0 FAIL, 45 skipped |
| 정식 Rust lint·workspace build·suite 정책 | 모두 통과; WASM32 검사 포함 |

전체·집중 회귀는 저장소의 공개 글꼴 5개만 보이는 격리 환경에서 실행했다. 새 회귀는
초기 자동 수집 경로인 `regression_suite_016`에 들어갔으며 CI 개별 호출은 추가하지 않았다.
조판·paint·글꼴 메트릭은 변경하지 않아 Native Skia 광범위 회귀·fresh WASM 렌더·Visual Sweep·
편람 gate를 이 작업의 증거로 주장하지 않는다.

정식 범위 외에 실행한 vendor 독립 Clippy는 기존 스타일 진단 125건으로 실패했다.
기준 소스에 컴파일 어댑터만 반영한 대조군과 메시지·상대 경로 multiset이 같고 새 진단은 0건이다.
이를 통과로 기록하거나 `allow`로 숨기지 않았다. 정식 root lint는 `-D warnings` 그대로 통과했다.
초기 환경·fixture·새 테스트 입력 오류는 수정했으며 실제 음성 대조와 구분한다.

[명령·시각·종료 코드·로그 해시·실행 환경](runtime-validation.json)을 함께 보존한다.
qcms의 upstream ICC 15개는 [고정 commit·Git blob·SHA-256 원장](qcms-reference-manifest.json)의
URL에서 받아 검증했다. 독립 테스트를 재현할 때 이 파일들을 manifest의 상대 경로로
`vendor/rhwp-qcms/`에 임시 연결해야 한다. 다운로드본·연결·별도 생성된 lock 파일은 커밋하지 않는다.

## CodeQL 확인 범위

CodeQL 2.27.4와 `codeql/rust-queries@0.1.46`의 경고 규칙 17개를 그대로 사용한다.
원격 SARIF의 rule 26개 중 나머지 9개는 summary/telemetry이며 보안 경고 규칙이 아니다.
로컬 실행은 이 17개 경고 규칙을 모두 보존하고 비경고 통계를 별도로 확인한다.

[입력·규칙 해시·추출 범위](local-codeql-inputs.json)에서 확인한 추출 파일은
4,717→4,718개다. 새 회귀 파일 하나만 추가됐고 기존 파일은 빠지지 않았다.
추출 경고 947건의 내용은 동일하며 양쪽 모두 209개 파일에 경고가 있다.
따라서 추출이 완벽하다고 주장하지 않는다. 수정한 vendor 파일 3개도 양쪽에 포함됐다.
[공개 합성 양성 대조](codeql-positive-controls.json)에서는 평문 로그와 고정 암호 경고를
실제로 탐지했다. 경고를 줄이기 위한 query/path/language/severity 제외는 추가하지 않았다.

수정 전 로컬 결과는 144건으로, 원격 analysis `2124551567`과 rule·sink 집합이 모두 같다.
보정 대상 8건의 모든 source·sink·열·flow도 그대로 재현했다. 143건은 전체 flow가 같고,
RNG 작업 배열 1건은 원격과 로컬이 각각 네 경로 중 다른 salt/key 사용 경로를 제시했다.
이는 같은 source의 대체 경로이며 실제 코드 차이가 아니다.
[원격 대조](codeql-baseline-remote-comparison.json)와 [8건의 수정 전 전체 flow](codeql-targets-before.json)를 보존한다.

보정 후보까지 같은 17개 경고 규칙을 실행했고 양쪽 모두 종료 코드 0이다.
[전후 비교](codeql-before-after.json)에서 원문 SARIF는 144→137건, 포인터 경고는 7→0건이다.
대상 이외 136건의 rule·주요 위치·메시지 의미는 유지됐다. RNG 작업 배열 1건의 경로 선택과
메시지 참조 순서 차이는 [원문 순서와 flow](codeql-witness-variations.json)로 보존했다.
분석 후에도 candidate snapshot과 작업 소스의 hash 차이는 0건이다.
[실제 분석 명령·시각·종료 코드·로그 해시](codeql-analysis-runs.json)도 함께 기록한다.

#597에 해당하는 경고 하나는 남았다. [수정 후 전체 4개 flow](codeql-targets-after.json)는
모두 PyO3 receiver에서 `page_index: usize`를 거쳐 수치 진단으로 이어진다.
본문 출력은 실제 로그 회귀에서 제거됐지만, 쪽 번호까지 credential로 추적하는 이 flow는
다른 35건과 같은 오탐으로 판단한다. 안전한 수치 진단을 없애거나 sanitizer를 추가하지 않았다.
[보정 후 판정 근거](post-fix-pdf-flow-review.json)는 수정 소스가 보호 브랜치에 반영된 뒤
추가 분류를 결정하기 위한 자료이며, 이번 승인에 포함되지 않은 #597의 dismiss는 수행하지 않았다.

SARIF 원문에는 이미 운영 분류한 결과도 포함된다. 따라서 137건은 GitHub의 OPEN 수가 아니다.
23:15 KST의 [최종 원격 재조회](remote-state-after-classification.json)는 main·devel 모두
OPEN 8건, 저장소 전체 dismissed 183건·fixed 10건이다. 기존 79건의 상태·분류 상세와
이번 114건의 분류도 다시 확인했으며 #8745는 completed 상태를 유지한다.

## 원격 완료 조건

로컬 보정과 GitHub의 `fixed` 상태는 구분한다. 보정 소스의 원격 반영과 동일 범위 분석,
main·devel 보호 브랜치에서의 실제 결과 확인이 필요하다. 포인터 7건의 `fixed`와
안전한 수치 진단만 남은 #597의 별도 분류를 확인해야 원격 정리가 끝난다.
초기 승인 단계에서는 PR·push·merge·새 workflow 실행을 수행하지 않았다.
이미 게시된 0.9.0 패키지와 배포물도 이번 로컬 보정으로 변경되지 않는다.

## 종료 정리와 보존 범위

필수 증적을 위 정식 경로에 반영한 뒤 실제 빌드·테스트·열린 파일 사용이 없음을 확인했다.
저장소 루트 `output/` 전체와 기본 작업공간의 `target/pr-review/debug`, 소유가 확인된 임시
snapshot 두 곳을 삭제했다. 추가 백업·압축·임시 내용 이관 없이 약 28.16 GiB를 확보했고,
삭제 경로의 부재를 확인했다. [정리 원장](cleanup.json)은 삭제 전후 공간과 유지한 profile을 기록한다.
원문 SARIF·로그와 내려받은 ICC의 임시 사본은 남아 있지 않다. Git source와 규칙 해시·
정규화 flow·실행 명령·시각으로 검증 범위를 재구성할 수 있다.

`task/8745-code-scanning-followup` worktree는 아직 원격에 제출하지 않은 보정 소스를 보존하므로
유지한다. 기본 devel 작업공간과 다른 소유자의 worktree·글꼴·Studio 배포용 JS/WASM,
공유 target의 release·release-test·WASM 등 다른 profile은 삭제하지 않았다.

## PR 제출·머지 승인 이후

2026-11-09 작업지시자가 “PR 로 머지 처리합니다”라고 지시해 devel 대상 PR 제출·CI·self-review·
머지 절차를 승인했다. main 승격이나 #597의 추가 보안 분류를 이 지시로 실행하지 않는다.

제출 전 최신 devel `edbd51067730098e830eb5d03aed625eac47b10f`의 #8857을 확인하고 충돌 없이
리베이스했다. 원래 보정 `612b4053db`는 `913713dbfd`로 변경됐으며 검증한 여섯 소스의 SHA-256은
그대로다. PDF backend·qcms·공통 codec 자원 crate·toolchain에는 새 base 변경이 없다.
위 CodeQL 전후 정량은 초기 base의 분석으로 보존하고, 최신 후보의 GitHub CI와 구분한다.
2026-11-10 새 base의 [필수 Rust 검증](pr-local-validation.json)을 모두 완료했다.
전체 release-test는 13,130 PASS·0 FAIL·45 skipped다. 로그 비노출 2건, 기존 인증 경계 6건,
새 base의 글꼴 프로그램 신원 4건과 번호 들여쓰기 2건도 통과했다.
현재 로그 검사의 자동 수집 경로는 `regression_suite_018`이며 실제 실행 로그의 경로를 보존했다.
최소 글꼴 목록은 동일한 공개 글꼴 5개다. qcms 선택 기능의 동일 소스·의존 경계 30건은
기존 증거를 재사용했다. [실행 입력의 commit·바이트·해시 대조](pr-input-fixtures.json)도 완료했다.
이 결과로 PR을 제출하고 해당 head의 실제 CI와 self-review를 진행한다.
