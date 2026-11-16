# Bongseop-Kim의 #8982·#8983 검토·보완 계획

## 목표와 고정 범위

첫 기여 두 건의 유효한 수정을 보존해 수용 가능한 상태로 준비한다. 작업지시자가 2026-11-12 메인터너 보완·필수 검증 진행을 승인했다. 계획 작성 당시 승인 범위는 로컬 source 보정·검증이었다. 이후 별도 원격 승인을 받아 수용 PR의 CI·self-review·merge를 완료했다. #8982는 명시한 표 테두리, #8983은 첫 서식 앞 평문을 다룬다. 무테두리 표의 기본값 변경 제안 #7992는 유지한다.

## 선행 조사

- #8942·#8943의 원 API 재현과 논의를 읽었다. 최신 devel에도 원 버그가 남음을 실제 검사에서 확인했다.
- 이전 #7516 → 원 PR #7521 → 통합 #7601 → 실제 merge `e8cc27778011ce8b54e823fa7edb446b979304a6`·rebased 적용 `cb2bd912d5afc28e01015d79caf839664c942e63`·메인터너의 4,000자 경계 보정을 추적했다. 현재 8개 검사는 base에서 모두 통과했다.
- 원 PR마다 Rust 코드 약 9줄과 검사 파일 한 개다. 현재 base와의 원 PR별·누적 병합은 충돌이 없다. 원 fork와 기여 SHA를 보존했다.

## 승인된 보완안 — 한 단위로 수행

1. #8982에서 실제로 재현된 속성 경계 오류를 보정한다. `data-border="1"`과 `data-border="1" border="0"`가 현재 새 실선이 된다. HTML 속성의 이름·값 경계를 읽는 공통 규칙에서 보정하고, 다른 속성 값 안의 가짜 border와 기존 width/height/colspan/rowspan 등 공통 helper 소비자의 유효 입력도 확인한다. 문서 ID·수치 우회와 무관한 HTML 기능 확장은 추가하지 않는다. 실제 border의 양수·0·부재와 셀 CSS 우선순위를 함께 보호한다.
2. #8983의 앞·뒤 평문, 첫/둘째 문단, 목록, 서식이 있는 삽입점과 셀 API를 확인한다. 새 회귀가 재현될 때만 근거를 먼저 적고 같은 공통 경로에서 제한 보정한다. 원 기여의 의미를 바꾸는 미확인 조건을 추가하지 않는다.
3. 실제 API로 생성한 검증 문서를 HWP/HWPX로 저장하고, 같은 저장본의 독립 한컴 Print를 확보한다. 원본·버전·글꼴·Print 출처와 hash를 연결한다. Native/fresh 최적화 WASM 전쪽 TSV와 같은 쪽 review·overlay를 직접 판독한다. 기존 #7516 출력·내용 보존과 공존도 검증한다.
4. 편집 → 저장 → 재열기 → 재저장 → 재열기를 HWP/HWPX 각각 실행하고, 내용·글자 모양·칸/테두리·기존 개체 소유를 실제 공개 API의 정식 회귀로 보호한다. 두 기능을 함께 쓴 입력도 포함한다.
5. 전쪽 시각 선행 조건을 만족한 뒤 기존 새 테스트의 첫머리에 실제 유형·시각 측정·출처를 기록하고 필요한 경계를 보완한다. 새 source/test 보정 head에서 Rust lint·필수 회귀·handbook 및 역할별 보호 gate를 완료한다. 원 CI의 녹색을 새 보정 head의 완료로 대신하지 않는다.
6. 개별 review와 PR 본문·댓글 초안을 실제 결과로 완성한 뒤, 원격 push·PR·CI·merge·후속 처리는 준비가 끝난 후보로 따로 승인받는다. fork 권한과 최종 제출 경로도 실제 조회로 확정한다.

기여자 보완 경로를 선택하면 같은 사실과 필요한 최소 재현·시각·반복 저장 항목을 한국어 존댓말 댓글 초안으로 제공한다. 승인 전에는 게시하지 않는다.

## 사전 실행 결과

- 최신 base `80b1c354c29b`: 원 7개 중 원 버그 관련 4 FAIL, 대조 3 PASS; 기존 #7516 8 PASS.
- 누적 원 기여 코드 `cc6103f1a821`: 같은 15개 모두 PASS; 세 입력 × HWP/HWPX × 두 차례 저장의 12개 비교도 PASS.
- 별도 속성 반례: base는 정상 3/3, 후보는 data-* 관련 두 입력에서 실선이 새로 생긴다. 실제 border=0 대조는 유지된다. 브라우저의 원 DOM도 각 기대값과 일치했다.
- 사전 진단 당시 Native/fresh WASM 시각·독립 Print·정식 새 회귀 보완·최종 gate는 미실행이었다. 메인터너 보정 권한을 받은 뒤 수행할 범위와 현재 리뷰 실행을 구분한다.

## 구현 착수

공통 숫자 속성과 기존 이미지 style 추출이 같은 따옴표 속성 lexer를 소비하게 한다. 속성 이름·값을 순서대로 읽고 ASCII 이름만 대소문자 구분 없이 비교한다. 태그·다른 속성 값 안의 문자열과 data-* 접미부를 속성으로 취급하지 않는다. 기존 숫자 단위와 따옴표 값 지원 범위를 유지하며, 표의 여는 태그 경계도 같은 lexer가 확인한다. 승인 없는 공개 댓글·push·PR·merge는 수행하지 않는다.

## 시각 선행 조건 완료와 정식 회귀 보강

2026-11-12, 보정 source `0aa25a6634`의 Native와 최적화 완료 fresh WASM에서 12문서·12쪽씩 모두 raw 100.00%를 확인했다. 한컴 2020 Print의 실제 인쇄 필드와 전쪽/동일 영역 확대 판독을 연결했다. [정식 원장](../assets/bongseop-kim-20261112/README.md)을 만든 뒤 기존 두 회귀 파일의 첫머리에 실제 측정 범위·직전 파일 commit을 기록했다. 별칭 속성·span·그림 단위/실제 style 속성과 서식 있는 삽입점·복합/반복 저장 보호를 보강했다. 합성 API assertion을 별도 실물 입력의 픽셀 점수로 대신하지 않으며 px/쪽수 pin을 추가하지 않았다. 최소 글꼴 환경의 정식 suite·필수 lint·전체 회귀·Native 공식 검사·보호 전수와 실제 Studio 경로 검증을 완료한 뒤 최종 후보를 확정한다.

## 승인 범위 완료

최종 정식 회귀 head `39f0ac0477`에서 13개 로컬 게이트가 모두 통과했다. 전체 회귀 13,341개·집중 23개·Native 공식 검사 4,073개·모든 lint·workspace build, 주요 Native/fresh WASM 각 13쪽 raw 100.00%, 정본 보호 3,542행 공식 하락 0, 별도 필수 편람 383쪽 gate·Studio 6사례/반복 저장 24개 비교를 완료했다. 원 로그·실행 source·생산자 동일성·skip/ignored·세션 수집기 상태까지 정식 원장에 보존했다. 새 제품 수정·검사 기준 변경·예외 적용은 없다.

[최종 통합 검토](bongseop_kim_20261112_review.md)에 따라 수용을 권고했고, 별도 원격 승인을 받아 제출 후보의 Open PR·새 head CI·self-review·merge를 완료했다. 운영 기록 반영 뒤 승인된 댓글·동기화·종료 정리를 수행한다.

## 원격 승인·CI·self-review·병합 결과

작업지시자는 제출 후보 `62a3fe4e2982961f9356a890395dc056eca85570`의 원격 push·Open PR·CI·self-review·merge와 원 PR/이슈 후속 처리·운영 기록·정리를 승인했다. [통합 수용 PR #9017](https://github.com/edwardkim/rhwp/pull/9017)을 열어 같은 head를 검증하고 [self-review](https://github.com/edwardkim/rhwp/pull/9017#pullrequestreview-5875203068) 뒤 merge commit [`dbb0dfcdaf7aebe68e712807da3ca700774fd39b`](https://github.com/edwardkim/rhwp/commit/dbb0dfcdaf7aebe68e712807da3ca700774fd39b)로 통합했다.

- [CI](https://github.com/edwardkim/rhwp/actions/runs/43221597756): lint·Archive A–E·Build & Test 성공. 실제 기본 아카이브 13,151건과 이번/선행 관련 27건이 PASS이며, 로컬 전체 workspace 13,341건과 실행 범위를 구분한다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/43221598571): 변경 대상 Rust의 실제 분석 성공. Actions·JavaScript/TypeScript·Python은 경로 분류에 따라 분석을 생략했다. 종합 check의 `neutral`·3개 구성 누락 경고를 보존하며 전체 언어 분석 성공으로 쓰지 않는다. Adapter·Proptest와 최종 CI Impact Policy도 성공했다.
- CI의 Native·Frontend 생략과 미트리거 Render Diff는 위 로컬 Native/fresh WASM·Studio 실측과 구분한다. 새 CI 재실행·보호 설정 변경·보안 분류 변경은 없었다.
- 최신 devel `1a3ed4a7cd`의 #8956 표 조회 API 4건과 실제 병합 tree `721ccfdffb`의 27건을 추가 검증했다. 첫 실행의 공용 target 기준 라이브러리 재사용은 실패로 보존했고, 내용 변경 없이 현재 소스를 재컴파일해 27 PASS를 확인했다. GitHub의 실제 merge tree도 동일하다.
- 두 원 PR은 간접 MERGED다. #8942·#8943의 자동 CLOSED를 확인했고 #7992는 OPEN을 유지한다. 원 기여 SHA와 fork branch를 보존했다.
- duration 후처리: [duration workflow](https://github.com/edwardkim/rhwp/actions/runs/43227236387)는 success로 끝났지만 `ready:false, reason:no-verified-pr-duration-measurements`로 갱신을 보류했다. 제품 검증 CI를 재실행하지 않았다.

[실제 CI·병합 영수증](../assets/bongseop-kim-20261112/remote/merge-and-ci.json). workflow·판정 정책 변경이 없는 maintainer 일반 경로이므로 옵션 M의 mydocs 운영 기록만 devel에 반영한다. 기록 push와 devel 동기화 후 완료 댓글·감사 댓글을 게시하고 전용 branch/worktree·output 전체·유휴 debug를 정리한다.
