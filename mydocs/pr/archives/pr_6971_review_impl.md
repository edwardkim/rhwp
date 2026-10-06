# PR #6971 메인터너 보정

## 제출 및 전체 회귀 완료 (2026-09-10)

보정 commit `874a709fef806b6f470bcf7932d365c5328eed5d`를 upstream 작업 branch에 push하고 [통합 PR #6995](https://github.com/edwardkim/rhwp/pull/6995)를 생성했다. 리베이스 후 전체 회귀는 9,417 passed/46 skipped/0 failed, Native Skia lib는 4,112 passed/13 ignored/0 failed, 관련 Skia 통합 6개도 통과했다.

동일 code candidate의 GitHub Full CI, CodeQL 분석, Canvas visual diff, Adapter, Proptest 성공과 MERGEABLE/CLEAN을 확인했다. 이 archive review 두 개와 오늘할일은 같은 PR의 문서-only trailing commit이며 코드나 증적을 다시 바꾸지 않는다.

최신 tail CI 확인 후 사용자가 승인한 일반 merge, devel sync, 원 PR supersede close, issue 상태 확인·comment, 이번 branch 정리를 수행한다. [검토 기록](pr_6971_review.md#제출-및-전체-회귀-재검증-2026-09-10)에 실제 run URL과 사전 comment 계획을 고정했다. 아래 실행 전 상태는 과거 단계의 기록이다.


## 리베이스 후 보류 사유 해소 (2026-09-10)

최신 `upstream/devel` `2a780e0d296846df577866eba6ac8f388527551b`에 autostash rebase를
완료했다. 충돌이 없었고 로컬 cherry-pick은 `a6d48e1c7ff3a2e7425666935651119d1fac2abf`로
변경됐다. 기존 미커밋 메인터너 보정은 유지했으며 제품에 추가 예외나 경고 억제를 넣지 않았다.

기존의 control-loop 뒤 host text 방출과 일반 표의 text 소유권 분리가 새 기준에서도 동작했다.
기존 focused 회귀 2개, 혼재 구성 프로브 4개, 유효한 decoration 하단 높이 6개의 표 1/2개
조합 12개, bbox 비교, 빌드 및 Clippy 3종이 통과했다. 60/70px 용지를 decoration 경로로
잘못 간주한 프로브는 사용자 승인 후 범위를 보정했으며 원 실패 로그도 남겼다.

13.3px overflow는 비가시 공백 host 줄의 진단으로 확인했다. 본문 표와 비공백 글자는
본문 하단 안에 있고, 리베이스 후 대표 PNG를 직접 열어 하단 잘림이 없음을 확인했다.
두 보류 사유를 해제하고 [최종 PR 판정](pr_6971_review.md#최종-판정)을
`메인터너 보정 후 수용 가능`으로 갱신했다. 세부 수치, 현재 source/바이너리 SHA,
범위 밖 oversized 관찰 및 실제 comment 계획은 review 문서를 따른다.

전체 회귀 및 Native Skia 전체 통과는 리베이스 전 결과이며 이번 기준에서 재실행하지 않았다.
보정 commit 고정, push 전 필수 전체 검증, 최신 head Full CI와 사용자 merge 승인은 별도
게이트다. 이번 작업에서 commit, push, PR 생성, comment 또는 merge는 수행하지 않았다.

## 리베이스 전 검증 기록 (2026-09-10 20:41 KST)

빌드, fmt, Clippy 3종, 전체 nextest 회귀, native-skia lib 및 Skia 통합 테스트를 모두
종료 코드 0으로 마쳤다. 기존 host heading 회귀 2개와 임시 프로브의 baseline / flow-first /
decoration-first / two-decorations 4개 구성에서 제목 1회 렌더를 확인했다.

임시 프로브의 `Control` import 오류는 사용자 승인 후 `rhwp::model::control::Control`로
수정했다. 해당 오류는 제품 코드 오류가 아니며, 수정된 프로브의 컴파일과 실행은 모두 통과했다.

한컴 기준 PDF와 실제 비교 PNG를 만들어 직접 열어 제목 복원을 확인했다. 전체 검증 결과,
source/바이너리/입력/PDF SHA-256, 실행 명령, 증적 보관 경로와 comment 계획은
[리베이스 전 PR 검토 기록](pr_6971_review.md#리베이스-전-검증-기록-2026-09-10-2041-kst)을 따른다.

**남은 한계:** 페이지 하단의 여러 decoration 표 분할 조건은 직접 검증하지 않았다.
baseline의 overflow 13.3px 진단과 결재표의 미세한 위치 및 선/글꼴 차이가 남는다.
따라서 제목 중복 프로브 통과를 전체 배치 정상 또는 최종 수용으로 확대하지 않으며, 현재
머지 보류 사유는 위 잔여 확인이다. 아래 `미검증` 기록은 최초 보정 시점의 이력이다.

이번 검증에서는 commit, push, GitHub 게시, PR 생성 및 merge를 수행하지 않았다.

## 최초 보정 기록

## 기준과 범위

- 원 PR: [#6971](https://github.com/edwardkim/rhwp/pull/6971), 관련 이슈 [#6969](https://github.com/edwardkim/rhwp/issues/6969).
- base: `4e0ce9283086ddd33e3660cf3927db8175384a43`.
- 체리픽: `557a4f2e43c9c72cce7d7f3af6e1db5901c72d47`.
- 변경 대상은 `src/renderer/typeset.rs`의 host 텍스트 방출 소유권과 방출 시점이다.
- 기여자 원 변경은 보존하고 메인터너 보정을 작업트리에 추가했다. 별도 소스 commit은 아직 없다.

## 적용 내용

1. 데코레이션 분기에서 즉시 텍스트를 방출하는 대신 pending 상태를 기록한다.
2. 일반/TAC 표 분기로 진입하면 기존 표 경로가 host 텍스트를 소유하도록 표시한다.
3. control 루프가 끝나면 일반 표 소유자가 없는 데코레이션 host의 전체 텍스트 줄을 한 번 방출한다.
4. 방출은 기존 문단 높이 조정 전에 수행하며, 형제 표의 앵커 계산 도중에는 제목 높이를 추가하지 않는다.

기존 일반/TAC 표의 pre/post 텍스트 방출, 지연 표 처리 및 layout fallback 구현 자체는 변경하지 않았다.
원 PR의 fixture와 테스트 원본은 수정하지 않았다.

## 남은 검증

- 일반 표 -> 데코레이션 표 및 역순에서 host 텍스트의 발행 횟수와 실제 렌더 결과.
- 같은 앵커 줄의 복수 데코레이션 표가 페이지 하단에 있을 때 행 잘림/연속 쪽 처리.
- 단일 데코레이션 host 제목 복원, 기존 지연 이월 제목 회귀 및 mixed TAC/그림 문단.
- 변경 범위별 빌드, 회귀, Clippy와 한컴 기준 PDF/대표 PNG 직접 시각 비교.

현재는 보정 구현만 적용한 미검증 상태이며 [review](pr_6971_review.md)의 머지 보류를 유지한다.
검증 및 후속 commit/push는 별도 진행 단계다. 원시 검증 로그와 중간 산출물은 실행 시 `output/`에 둔다.
