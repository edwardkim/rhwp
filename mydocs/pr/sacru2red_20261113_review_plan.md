# sacru2red PR 2건 검토 계획

2026-11-13. 메인테이너 일반 검토에 다수 PR·시각 증적 절차를 적용한다.

| 대상 | 원 head | 검토 범위 |
| --- | --- | --- |
| #8951 | `22acc33b7e06647485bae503403ac6a465fb0dc8` | footer 문서의 body overflow 허용치 1행 제거 |
| #8975 | `286eb32e0877d2b1fccc0753b60d9604221c156f` | 비TAC 자리 차지 표의 CellBreak/RowBreak 생성 대조군·회귀 |

두 PR은 Ready이며 접수 시 각각 CI 32건 성공·정상 skip 5건이다. 리뷰어 `edwardkim` 지정을
2026-11-13 10:17 KST에 확인했다. 현재 승인은 접수·로컬 검토 범위이며 공개 댓글·push·PR 생성·승인 리뷰·merge는 별도다.

## 기준과 적용

- 최신 `upstream/devel`: `180a4ec97205b67c4dc4f90416063c01ecfdf396`.
- 검토 branch: `review/integration-7620-sacru2red-followup-tests`.
- #8951은 최신 base와 clean merge다. #8975는 `samples/issue7620/README.md` 한 곳에서
  최신 #8939 합성 LineSeg 설명과 신규 대조군 설명이 충돌한다. 양쪽 설명을 보존한 로컬
  누적 merge는 `d06e8861feb15146ad16dc5bd03d03ca85ff8d0a`다. 제품·검사 코드는 보정하지 않았다.
- 원 기여 SHA를 모두 조상으로 보존한다. 루트 devel과 다른 작업의 worktree는 변경하지 않는다.

## 선행 해결 이력

#7620 → #7644 → 수용 PR #7681(`4c7092397e6fb6d1d16e5521bb9049ef25814d5a`)에서
자리 차지 표의 여백·실제 빈 문단 점유를 보정했지만 footer 초과·CellBreak 독립 검증은 남았다.
#8945(`4ab860651b857bfe6d28b22b4565b2855bebb042`)는 첫 조각 아래 여백 예산 실패 시
표 이월을 부분 개선했다. #7620은 현재 OPEN이다. 과거 PR에만 적용한 시각 예외는 재사용하지 않는다.

## 필수 로컬 확인

1. 원 head와 최신 base의 diff, 정확한 CI·입력 hash·기준 PDF 출처, 기존 해결 이력과 현재 호출 경로를 확인한다.
2. #8951의 기존 Desktop Hancom 2020 Print를 재사용해 전 13쪽 Native/fresh WASM을 대조한다.
   body overflow ratchet의 소비 경로와 실제 footer 표·뒤 내용·쪽 번호를 확인한다.
3. #8975의 웹 인쇄 PDF는 현행 정본의 Desktop Print와 구분한다. 같은 입력을 Windows
   Hancom 2020 MCP의 Print 경로로 출력해 전 4쪽 Native/fresh WASM과 직접 대조한다.
   입력은 생성 대조군이며 한컴 저장본으로 표현하지 않는다.
4. 새 검사에서 글줄의 쪽 소속·순서·누락·중복, 표 조각·뒤 본문 관계와 두 나눔 모드의
   판별력을 확인한다. 최소 글꼴 환경의 집중 회귀, baseline 결정성, 변경 범위별 필수 게이트를 수행한다.
5. 전쪽 TSV와 변경 영역 확대 판독을 보존하고 PR별 판정·#7620의 남은 범위를 분리한다.
   새 기준·주석 또는 제품 보정이 필요하면 실제 근거와 제한 보정안을 제시한다.

공용 Cargo target은 `/home/edward/mygithub/rhwp/target/pr-review`다. 임시 로그·생성물은 루트
`output/pr-review/sacru2red-20261113`에만 둔다. source·입력·검사 변경이 없는 녹색 원 head의
광범위 CI 재사용과 새 보정 head의 로컬 필수 검증을 혼동하지 않는다.

## 승인된 추가 범위와 로컬 완료

위 충돌·보정 없음 설명은 접수 시점이다. 이후 실제 Desktop Print로 발견한 표 뒤 간격 차이에
사용자가 예외 대신 메인테이너 보정·재검증을 승인했다. 공통 제품 보정 `e5a6a1ff9f`와
회귀 보완 `b4b8c27260` 이후 필수 검사를 완료했고 원 기여 SHA 두 개를 그대로 보존했다.
[최종 전쪽 원장](assets/sacru2red-20261113/final/README.md)과
[원격 처리안](assets/sacru2red-20261113/remote-action-plan.md)을 준비했다.
원격 push·새 PR·댓글·merge는 이 로컬 승인에 포함하지 않는다.
