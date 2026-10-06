# 최근 종료 PR 100건과 Claude 작업 지침 품질 감사

작성일: 2026-09-10 (KST)
성격: 기록 기반 회고 감사 및 지침 보정 제안. 개별 PR 재승인이나 제품 재검증이 아니다.

## 1. 결론

**CLAUDE.md의 분량 부족보다, 자동 로딩 진입점과 contributor 스킬에 남은 낡은 규칙이 현재 정본과
충돌하는 것이 확인 가능한 가장 직접적인 문제다.** 여기에 단일 재현 사례 중심의 조건 설계,
잘못된 기대값을 고정하는 회귀, 수치 검증을 사용자-visible 결과로 확대하는 습관이 겹친다.

CLAUDE.md만 길게 보강하면 아래 스킬을 다시 읽을 때 같은 모순이 생긴다.
권장 조치는 **짧은 자동 로딩 진입점 + 정본으로 연결되는 역할별 실행 절차 +
원인/반례/최종 출력의 완료 조건**을 함께 정비하는 것이다.
최초 감사 단계에서는 제안 문서만 작성했다. 이후 사용자 지시에 따라 PR #6997에서
실제 CLAUDE.md와 contributor 스킬 및 활성 자식 8개에 보정을 반영했다.
제품 코드와 테스트는 변경하지 않았으며 이번 지침 보정의 검증은 미실행이다.
현재 반영 범위와 남은 조건은 [PR #6997 self-review](../pr/archives/pr_6997_review.md)에 기록한다.

## 2. 모집단과 집계 방법

- 수집 기준: 2026-09-10T13:15:14.620Z.
- 종료 시각 범위: 2026-09-07 08:45:31 UTC부터 2026-09-10 12:58:03 UTC까지.
- GitHub repository GraphQL의 CLOSED/MERGED PR을 UPDATED_AT 내림차순으로 200개 조회했다.
  이들의 closedAt으로 다시 정렬하여 최신 100개를 선택했다. 마지막 조회 updatedAt은
  2026-09-05 09:07:37 UTC로 100번째 종료 시각보다 오래되어 이후 페이지가 순위를 바꿀 수 없다.
- 검색 결과의 PR 번호순/갱신순이나 미지원 closed 정렬을 최신 종료순으로 간주하지 않았다.
- 본문 100개, 일반 코멘트 140개, review 이벤트 7개, inline thread 4개/댓글 7개,
  PR별 commit 항목 480개와 변경 파일 목록을 수집했다.
  480개는 통합 PR의 중복 이력을 포함하며 고유 commit 수가 아니다.
- comments/reviews/commits의 추가 페이지는 없었다. 파일이 100개를 초과한 #6939(135개),
  #6847(105개)는 다음 페이지도 수집했다. inline thread/comment 페이지 누락도 없었다.
- 100개 전체는 기록 기반으로 분류하고 반복 보정 사례와 지침 충돌을 심층 대조했다.
  **100개 전체 source diff를 줄 단위 재검토하거나, 모든 CI 로그/비공개 원문을 재검증한 것은 아니다.**
  표의 미분류는 결함 없음의 판정이 아니다.

| 구분 | 수 |
| --- | ---: |
| GitHub MERGED | 42 |
| GitHub CLOSED, 직접 merge 아님 | 58 |
| 비통합 기능·테스트 PR | 70 |
| 통합 PR | 11 |
| 문서 전용 PR | 9 |
| CI 운영 PR | 6 |
| Dependabot PR | 4 |

GitHub 상태 42/58과 목적별 70/11/9/6/4는 서로 다른 분류다.
CLOSED 58개에는 체리픽 수용, 중복 종료, 의존성 통합이 포함된다. 거절/불량 58건을 뜻하지 않는다.

### 보정 확인 하한과 Claude 귀속 한계

- 서로 다른 원 기능·테스트 PR **25개**에서 명시적 메인터너 보정 판정, 구체적인 보정 코멘트
  또는 분리된 보정 commit을 확인했다. 동작·안전성·시험/fixture 보완을 함께 포함한 보수적 하한이다.
- 별도로 문서 PR #6843에도 보정이 있지만 제품 코드 결함으로 세지 않았다.
- 25개: #6985, #6983, #6980, #6971, #6967, #6952, #6949, #6938, #6914, #6897, #6885, #6881, #6880, #6863, #6853, #6811, #6808, #6804, #6801, #6798, #6796, #6792, #6789, #6784, #6773.
- 전체 100개 중 63개에 본문 또는 commit 메시지의 Claude/Anthropic 표기가 있다.
  통합 PR의 상속 표기를 제외한 기능·테스트 PR에서는 51/70개다. 보정 확인 25개 중에는 21개다.
- 이는 **자기신고/Co-Authored-By 표기의 존재**이지 실제 모델 버전, 전체 작성량,
  각 보정의 책임, 해당 스킬을 실행한 사실의 증명이 아니다.
  Claude 표기가 없다고 인간 단독 작업으로 분류하지 않는다.
- 원 PR + 통합 PR + 후속 문서 PR을 세 번의 Claude 실패로 세지 않았다.
  source 원인 수정과 검토 도구/테스트 입력 오류도 구분했다.

## 3. 지침에서 발견한 문제

### P1. 스킬이 필수 전체 회귀와 세 Clippy를 선택 사항으로 낮춘다

[clippy-and-tests.md:9](../../.claude/skills/rhwp-contributor/references/clippy-and-tests.md#L9)는
native Clippy 한 줄을 기본으로 한다. 같은 문서
[45행](../../.claude/skills/rhwp-contributor/references/clippy-and-tests.md#L45)과
[49행](../../.claude/skills/rhwp-contributor/references/clippy-and-tests.md#L49)은 전체 회귀를
선택적으로 실행하거나 시간이 없으면 관련 시험만 기록하도록 안내한다.

반면 [CONTRIBUTING.md:152](../../CONTRIBUTING.md#L152)의 현행 범위 표는 Rust source에
세 Clippy와 전체 integration, renderer에는 Native Skia/WASM/시각 검증을 요구한다.
[PR 템플릿](../../.github/pull_request_template.md)도 해당 범위와 검증 SHA를 요구한다.

이 충돌은 추상적인 우려만이 아니다. [#6773 본문과 검토 코멘트](https://github.com/edwardkim/rhwp/pull/6773#issuecomment-5551786944)는
generated suite 부재와 기여 규칙을 이유로 all-targets 검증을 건너뛴 해석을 명시한다.
메인터너는 별도 review worktree에서 생성하는 것과 source 제출물에 포함하지 않는 것을 구별했다.

**보정 방향:** 스킬의 독자적인 최소 검증 표/명령을 제거하고 현행 CONTRIBUTING의 범위 표와
worktree 절차를 그대로 참조한다. source 원본 commit과 검증 checkout/산출물의 동일성을 기록한다.
필수 검증을 실행하지 못했으면 미완료로 남기며, 기록만으로 면제하지 않는다.
검토자의 검증된 CI 재사용 예외를 기여자의 제출 전 검증 면제로 확대하지 않는다.

### P1. 시각 검증 지침이 정확한 oracle와 실제 출력 확인을 약화한다

[visual-evidence.md:18](../../.claude/skills/rhwp-contributor/references/visual-evidence.md#L18)는
전후 SVG/PDF를 페이지 수·텍스트 해시로 대체할 수 있게 하고,
[28행](../../.claude/skills/rhwp-contributor/references/visual-evidence.md#L28)은
한컴 PDF를 정답지로 보지 않는다고 단정한다.
앞부분의 숫자 시험만으로 끝내지 말라는 지시와도 맞지 않는다.

현행 [시각·fixture 증적 정본](../manual/pr_review/visual_fixture_evidence.md)은
출처와 대응 원문이 확인된 한컴 직접 출력/MCP PDF를 기준으로 인정하고 대표 PNG 직접 판독을 요구한다.
한컴 설치/비공개 자료를 모든 외부 기여자에게 요구한다는 뜻은 아니며,
기준 접근이 없을 때 수치 결과를 기준 일치로 승격하지 말아야 한다.

[#6794 최초 리뷰](https://github.com/edwardkim/rhwp/pull/6794#issuecomment-5557491197)는
자동 검사 성공에도 차례가 기준과 다른 쪽에 있고, 새 테스트가 잘못된 1쪽 배치를
요구했다고 지적했다. [#6938 리뷰](../pr/archives/pr_6938_review.md#L191)는
placeholder 제거와 복합 차트 의미 복원을 구별했다.
flagged=0인 그림도 이중축 선/막대를 단일축 막대로 왜곡할 수 있었다.

**보정 방향:** 코드 작성 전에 기대 출력의 페이지 소유·대상 존재·축/계열/라벨 등 의미를 정하고,
대응 내용을 맞춘 실제 패널로 확인한다. 페이지 수/문자 수/해시/전체 pixel match는 보조 신호다.
기준의 글꼴·래스터 차이까지 무조건 blocker로 만들거나 parser의 구조 보존 PR에
전체 픽셀 동일성을 요구하는 반대 방향의 과잉 검증도 피한다.

### P1. CLAUDE.md는 AGENTS.md의 자동 로딩을 보장하지 않고 역할 라우팅도 약하다

[CLAUDE.md:7](../../CLAUDE.md#L7)은 AGENTS.md 등을 일반 Markdown 링크로 읽으라고 안내하지만
자동 import는 없다. 기여자용 CONTRIBUTING 직접 경로도 부트스트랩의 핵심 라우팅에 없다.
동시에 contributor 스킬은 기여/PR 작업을 maintainer 검토 문서로 보내고,
CONTRIBUTING은 외부 기여자에게 내부 운영 기록을 제출하지 말라고 한다.

Claude Code의 [공식 메모리 문서](https://code.claude.com/docs/en/memory#agentsmd)는
AGENTS.md가 자동 로딩 대상이 아니며 CLAUDE.md의 import로 연결할 수 있다고 설명한다.
일반 링크를 읽으라는 요청은 유효하지만, 실제 읽혔다는 증거는 아니다.

**보정 방향:** CLAUDE.md에 AGENTS.md import와 짧은 역할 분기, 지침 충돌 처리,
품질 완료 조건을 둔다. 이미 로딩된 파일을 순환해서 다시 읽지 않는다.
수백 줄의 모든 manual을 자동 import하지 않고 작업별 정본만 선택한다.
보정된 세션에서 실제 로딩 여부는 별도 확인해야 한다.

### P2. 생성물 정책과 PR 체크리스트를 스킬이 오래된 형태로 복제한다

[clippy-and-tests.md:29](../../.claude/skills/rhwp-contributor/references/clippy-and-tests.md#L29)는
integration 준비에 --generate, [37행](../../.claude/skills/rhwp-contributor/references/clippy-and-tests.md#L37)은
source unit inventory 생성에 --generate를 안내한다.
현행은 review worktree의 --prepare와 source-tier의 무생성 --check를 구별한다.

[pr-template-checkboxes.md:3](../../.claude/skills/rhwp-contributor/references/pr-template-checkboxes.md#L3)는
실제 템플릿 첫 체크박스를 fmt로 해석하라고 하며, 템플릿이 다르더라도 그 해석을 강제한다.
현재 첫 항목은 변경 범위별 검증과 제출 SHA 일치다.
SKILL.md에는 고정 regression_suite_015와 모든 스킬 게이트 3회 반복도 남아 있다.

**보정 방향:** 최신 템플릿을 읽어 해당 항목만 사용한다. 생성/커밋/실행의 경계를 명확히 하며,
suite 번호와 특정 과거 스킬 개발 작업의 규칙을 일반 기여 절차로 재사용하지 않는다.
지침을 검사하는 테스트도 오래된 명령 문자열을 정답으로 고정하고 있는지 후속 보정 때 점검한다.

### P2. 스킬 개발 당시의 작업 제한이 일반 기여용 지침에 남아 있다

[implement-scope.md:6](../../.claude/skills/rhwp-contributor/references/implement-scope.md#L6)와
[exceptions.md:39](../../.claude/skills/rhwp-contributor/references/exceptions.md#L39)는
과거 스킬 고도화 파동과 #5322의 제한을 일반 기여 경로에 포함한다.
DocumentCore/CLI를 일괄 금지하거나 항상 closes를 요구하는 문구도 현재 작업의 범위를
잘못 제한하거나 부분 해결 이슈를 닫도록 유도할 수 있다.

**보정 방향:** 과거 파동의 비범위는 역사 예제로 내리고, 현재 작업의 비범위는 해당 이슈에서 결정한다.
부분 해결은 Refs와 잔여 추적을 사용하고, 전체 종료는 실제 수용 범위가 충족될 때만 제안한다.

### 지침의 당시 존재와 실제 사용 여부

#6971, #6938, #6794, #6773의 수집된 최종 head에서 GitHub blob을 조회했다.
네 head 모두 CLAUDE.md는 c372755347921b478626461eab9e8bfb6e73cd5c,
clippy-and-tests.md는 73d02400da2ef8b16ced82a449e2913a358a37a2였다.
후자에 선택적 전체 회귀와 --generate 안내가 그대로 있었다.

이는 해당 head에 낡은 문서가 존재했다는 증거다.
최초 제출 시점의 모든 파일 이력이나 실제 Claude 세션이 그 스킬을 호출한 사실까지
확인한 것은 아니다. 개인 CLAUDE.md/auto memory/실행 transcript는 이번 감사에서 열지 않았다.

## 4. 반복된 구현·검증 실패 유형

| 실패 유형 | 기록으로 확인한 사례 | 필요한 작성자 단계 방어 |
| --- | --- | --- |
| 한 사례의 조건을 일반화 | #6792의 120px, #6798의 64px/저장 좌표 조건, #6914의 간격 생략 | 구조적 근거, 진짜 양성/음성, zero/누락/범위 밖 및 기존 문서 대조 |
| 측정과 실제 paint의 조건 불일치 | #6789의 무리 사전 판정, #6980의 clip/flow, #6881의 공통/Native 크기 상한 | 결정 결과를 공유하고 paint 뒤 bbox만 덮어 맞추지 않기 |
| 단일 개체만 확인 | #6971의 decoration/일반/TAC 혼재, #6863의 여러 인라인 도형 | 0/1/복수 및 순서 교체, 기존 소유자의 중복 방출 방지 |
| 구조 성공을 의미 성공으로 확대 | #6938의 차트 종류/축/역참조, #6967의 UTF-16/끝 marker, #6952의 빈 접미 | 빠진 값과 0/빈 값 구별, 형식별 실제 왕복, 의미 필드 직접 단언 |
| 원인 수정 위치가 잘못됨 | #6811의 글꼴 명령 후처리, #6794의 paint 쪽 이동 | lexer/parser/IR/typeset/layout/paint 중 책임 계층에서 해결 |
| 테스트가 의도한 경로를 실행하지 않음 | #6796/#6804의 입력 부재 시 return, #6773의 메모리 삭제만 검사 | 필수 fixture 누락은 실패, 대상 존재·개수 먼저 확인, 저장/재열기까지 |
| 안전 가드가 너무 늦거나 정상값을 거부 | #6938의 기반 재귀/역참조, #6885의 손상 CFB, #6881의 준비 후 예산 | 파싱/할당 전에 예산, 타입/순환/깊이 검사, 합법적 null/empty 음성 대조 |
| 문자열 계약을 동작 검증으로 오인 | #6908이 지적한 #6905 문자열 검사 | 실제 브라우저 resize/스크롤 양성·음성 시나리오 |
| 검토 도구/메인터너 자체 실수 | #6893/통합 #6939 폰트, #6938 보정 파서의 VtPicture/VtString, #6853 중간 왕복 회귀 | 작성자와 메인터너에 같은 gate; 원 PR 결함과 보정 중 새 결함 분리 |

근거 링크:
[#6792](https://github.com/edwardkim/rhwp/pull/6792#issuecomment-5557416953),
[#6798](https://github.com/edwardkim/rhwp/pull/6798#issuecomment-5570880545),
[#6914](https://github.com/edwardkim/rhwp/pull/6914#issuecomment-5598959894),
[#6789](https://github.com/edwardkim/rhwp/pull/6789#issuecomment-5557273648),
[#6980](https://github.com/edwardkim/rhwp/pull/6980#issuecomment-5618089229),
[#6881](https://github.com/edwardkim/rhwp/pull/6881#discussion_r3955552191),
[#6967](https://github.com/edwardkim/rhwp/pull/6967#issuecomment-5618079729),
[#6908](https://github.com/edwardkim/rhwp/pull/6908#issuecomment-5589274715).

잘못된 기대값을 둔 시험은 수정 전 실패/수정 후 성공해도 올바른 oracle가 아니다.
#6794가 직접적인 반례다. 따라서 red/green과 외부 기대 결과의 독립성은 모두 필요하다.

### 모든 보정을 Claude 원 오류로 세면 안 되는 이유

- #6942/#6933은 메인터너 제품 코드 보정 없이 수용했다는 코멘트가 있다.
- Claude 표기가 있는 #6851/#6849는 각각 공통 gradient 축, BinData 참조 보존 범위로 승인됐다.
- #6883/#6960/#6794는 contributor가 후속 수정한 이력도 있어 최종 보정 commit만으로 초기 오류율을 계산할 수 없다.
- #6843은 기존 문서의 fixture gate 목록 자체가 빠져 있었음을 보고했다. 지침을 따른 작성자에게
  누락된 gate 실패를 전부 돌릴 수 없다.
- #6938의 메인터너 추가 구현에서 생긴 타입/null/예약 바이트 오류, #6853 중간 보정 회귀,
  #6893의 시각 도구 글꼴 문제는 원 기여자 오류와 별도다.
- #6796/#6804처럼 제품 방향을 수용하면서 공개 회귀/증적을 보강한 경우도 있다.
  25건은 모두 제품이 잘못 구현됐다는 통계가 아니다.

## 5. CLAUDE.md 보정 초안

다음은 감사 시 작성한 설계 초안이다. 실제 적용 지침은 루트 CLAUDE.md와 contributor 스킬을 따른다.
기존 프로젝트 개요, parser/IR 경계, Studio 규칙 링크를 유지하면서 공통 import와 완료 조건을 반영했다.
아래 초안 자체를 실행 명령의 정본이나 검증 통과 결과로 사용하지 않는다.

~~~markdown
@AGENTS.md

## 역할과 정본 선택

- 이미 로딩한 CLAUDE.md/AGENTS.md를 순환해서 다시 읽지 않는다.
- 일반 코드 기여/버그 수정/PR 제출은 CONTRIBUTING.md의 변경 범위별 체크리스트와
  검증 worktree 절차를 먼저 따른다. 외부 기여자에게 maintainer 운영 기록을 요구하지 않는다.
- maintainer 검토/보정/merge는 pr_review_workflow.md 및 역할별 자식 문서를 따른다.
- 스킬/예제/auto memory의 요약이 현재 정본과 충돌하면 정본을 따른다.
  낡은 요약을 근거로 필수 검사를 생략하거나 실제 템플릿 항목을 재해석하지 않는다.
- 승인되지 않은 원격 조치나 테스트는 문서의 절차만을 근거로 임의 수행하지 않는다.
  수행할 수 없는 필수 게이트는 완료로 표시하지 않는다.

## 버그 수정의 완료 조건

- 수정 전에 입력, 실제 증상, 독립적인 기대 결과와 비범위를 정한다.
  테스트 기대값을 현재 구현의 좌표/페이지 수에서 역으로 만들지 않는다.
- 원인 계층을 특정한다. parser/IR/typeset/layout/paint/편집 상태 중 책임 위치를 고치며,
  bbox clamp, 표시 숨김, 개수 감소만으로 원인 해결을 주장하지 않는다.
- 조건에 쓰는 단위/좌표계/소유권/출처를 설명한다. 샘플 이름, 관측값에 맞춘 임의 문턱,
  raw 값 부재만으로 편집 이력이나 페이지 소유권을 추정하지 않는다.
- focused 회귀는 양성뿐 아니라 해당 변경의 음성/경계/혼재를 다룬다.
  필수 입력과 대상 개체의 존재·개수를 먼저 단언한다. 0개 순회나 입력 부재 return을 통과로 세지 않는다.
- 측정과 배치, 공통 선택기와 실제 backend는 같은 조건과 결정 결과를 사용한다.
  복수 컨트롤의 순서·텍스트 방출·흐름 전진은 각 소유자가 한 번만 처리한다.
- parser/serializer는 HWP/HWPX의 변경된 왕복 계약을 확인한다.
  미지원, 누락, null, 빈 문자열, 숫자 0을 구별하고 잘린 입력·역참조·자원 경계를 검토한다.
- 편집은 필요한 경우 실제 변경, Undo/Redo, 저장·재열기, 기존 개체 불변까지 확인한다.
  동일값 setter 성공이나 소스 문자열 존재를 제품 동작 검증으로 대체하지 않는다.
- 렌더 변경은 실제 전후 출력에서 같은 내용을 대응해 확인한다.
  신뢰 가능한 첨부 한컴 PDF는 재사용한다. 기준이 없으면 미확인 범위를 남긴다.
  페이지 수/텍스트 해시/flagged=0/전체 pixel match만으로 시각 통과를 선언하지 않는다.
- 검증 범위와 명령은 현행 CONTRIBUTING/로컬 검증 정본에서 선택한다.
  generated 제출 금지는 별도 검증 worktree의 --prepare나 필수 회귀 생략을 뜻하지 않는다.
  검증 SHA와 실제 결과를 기록하고, 보정 전 통과를 보정 후 통과로 재사용하지 않는다.
- 기준선 갱신은 원인과 새 입력/기존 회귀를 구별한다. 실패를 숨기기 위한 허용치 상향,
  반례 삭제, 시험 skip으로 끝내지 않는다.
- 완료 보고에는 해결 범위, 남은 차이, 미실행 검증을 분리한다.
  일부 개선을 이슈 전체 해결로 확대하지 않는다. 메인터너 보정도 같은 조건으로 검증한다.
~~~

이 초안은 기존 상위 지시나 사용자 승인 범위를 대체하지 않는다.
메인터너가 기준 자료 접근 권한을 갖지 않는 외부 기여자에게 인증 정보·비공개 자료를 강요하지 않는다.
공개 fixture/동등한 동작 근거와 미확인 범위를 남긴 뒤 역할별 검토로 넘긴다.

## 6. 함께 보정할 파일과 실행 순서

| 우선 | 파일 | 최소 조치 |
| --- | --- | --- |
| 1 | CLAUDE.md | 자동 import, 역할 라우팅, 짧고 확인 가능한 품질 완료 조건 |
| 1 | .claude/skills/rhwp-contributor/SKILL.md | fmt 단독 강조/독자 최소 gate/고정 suite/3회 반복을 현행 정본 포인터로 치환 |
| 1 | references/clippy-and-tests.md | 전체 회귀 선택화 제거, 세 Clippy/prepare/check/실제 SHA 경로로 일원화 |
| 1 | references/visual-evidence.md | 수치 대체/한컴 불신 단정 제거, 기준 출처와 실제 출력 판독/범위 구분 |
| 2 | references/pr-template-checkboxes.md | 첫 칸 강제 재해석 제거, 실제 PR 템플릿을 따름 |
| 2 | references/analyze-canonical.md 및 procedure-order.md | contributor/maintainer 역할 구별, 기대 결과·반례 계획 연결 |
| 2 | references/implement-scope.md 및 exceptions.md | 과거 #5322 작업 제한과 현재 일반 기여를 분리 |
| 2 | references/pitfalls.md 및 관련 examples/계약 시험 | 오래된 절차의 재유입 방지; 문자열 일치만으로 품질 보장하지 않음 |

references 경로는 모두 .claude/skills/rhwp-contributor/ 아래다.
현재 CONTRIBUTING과 PR 템플릿은 이미 개선된 부분이 많아 우선 규칙을 더 복제하지 않는다.
예제/계약 시험은 이번에 전수 읽거나 고치지 않았으므로 실제 보정 시 필요한 파일을 먼저 확정한다.
기존 자동 로딩 rules/개인 memory와 중복되거나 충돌하는지도 별도 적용 단계에서 확인한다.

### 완료 판정 제안

1. 문서 정합성: 전체 회귀 선택화, --generate 기본 경로, fmt 첫 칸 강제, PDF 불신 문장이
   활성 진입점/연결 예제에서 재등장하지 않고 역할별 정본이 하나로 연결된다.
2. 새 Claude 세션에서 실제 로딩 문서를 확인한다. 이 감사에서는 Claude를 실행하지 않았다.
3. 과거 실패를 작은 블라인드 과제로 재사용한다. #6773(검증 worktree),
   #6794(올바른 페이지 oracle), #6971(혼재 소유권), #6938(차트 의미/잘린 입력),
   #6808(실제 Undo 상태)를 고정 입력으로 사용하되 수정 정답 코드는 주지 않는다.
4. 생성된 테스트가 실제로 이전 결함을 잡고, 올바른 기준과 양립하며, 정상 반례를 보존하는지
   독립 검토한다. 지침 문구 암기나 기존 패치 복사로 통과시키지 않는다.
5. 이후 기능 PR 20개에서 최초 검토 통과, 메인터너 제품 보정, 시험/증적 보완, 문서/통합 비용을
   별도 추적한다. 비교 가능한 변경 범위끼리 평가하고 임의 목표 개선율을 확정 사실로 쓰지 않는다.

CLAUDE.md는 실행을 강제하는 보안 설정이 아니다. 공식 문서는 강제가 필요하면 hook을 사용하라고
구분한다. 추후 자동 gate를 만들더라도 명령 문자열 탐지만으로 검사 실행/성공을 증명하지 말고
실제 대상 SHA·입력·종료 결과와 연결해야 한다.
이번 감사는 hook을 설치하거나 정책을 강화하지 않았다.
[Claude Code 공식 설명](https://code.claude.com/docs/en/memory#claudemd-vs-auto-memory)

## 7. 100개 PR 분류표

M은 명시적 메인터너 보정 확인 하한 25개다. 제품 보정과 시험/증적 보완이 섞여 있으므로
각 설명을 함께 본다. I=통합, D=문서, O=CI 운영, B=의존성.
M이 없다는 것은 보정/결함이 없다는 보증이 아니다.
Claude 표기는 본문/commit의 자기신고 유무이며 모델 책임 판정이 아니다.
각 링크는 수집 당시 기록의 출처다. PR 본문/코멘트는 추후 수정될 수 있으므로 head도 고정했다.

| 순번 | PR | 종료 UTC | 상태 | 종류 | Claude 표기 | 수집 head | 검토 메모/근거 |
| ---: | --- | --- | --- | --- | --- | --- | --- |
| 1 | [#6994](https://github.com/edwardkim/rhwp/pull/6994) | 2026-09-10 12:58:03 | MERGED | 원제품PR | 미확인 | `e573ec0de7894a3f06c4438d2754977f1ecbff4c` | 자체 기능 개발; 여러 조판 보정 단계와 검증 한계 기록 [기록](https://github.com/edwardkim/rhwp/pull/6994#issuecomment-5619103168) |
| 2 | [#6971](https://github.com/edwardkim/rhwp/pull/6971) | 2026-09-10 12:54:59 | CLOSED | 원제품PR | 있음 | `5c68bbd909be51a059121096bda1ae7617efcdd6` | M: decoration/일반/TAC 혼재 시 host 텍스트 소유권과 앵커 보존 [기록](https://github.com/edwardkim/rhwp/pull/6971#issuecomment-5619007668) |
| 3 | [#6995](https://github.com/edwardkim/rhwp/pull/6995) | 2026-09-10 12:50:52 | MERGED | 통합 | 있음 | `e528b433b271a4a2c15c866f9ed903d5a679471b` | I: #6971 보정 통합; 원 PR과 중복 계산 제외 [기록](https://github.com/edwardkim/rhwp/pull/6995#issuecomment-5619008318) |
| 4 | [#6993](https://github.com/edwardkim/rhwp/pull/6993) | 2026-09-10 12:25:05 | MERGED | 문서 | 미확인 | `44f04c4a7e9ab48fa44315f3dc122584fd584e0f` | D: #6991 후속 기록 |
| 5 | [#6991](https://github.com/edwardkim/rhwp/pull/6991) | 2026-09-10 12:04:28 | MERGED | CI | 미확인 | `ffbaf8301ad84ffedafc68922e1ba5446cbe2a06` | O: 검증된 merge candidate/CI 재사용 증거 [기록](https://github.com/edwardkim/rhwp/pull/6991#issuecomment-5618670867) |
| 6 | [#6983](https://github.com/edwardkim/rhwp/pull/6983) | 2026-09-10 11:40:56 | CLOSED | 원제품PR | 있음 | `c445d5043a61e667e215e570411681ac4721c9d2` | M: 회귀 gate 누락 key/렌더 오류 fail-closed 및 16분할 [기록](https://github.com/edwardkim/rhwp/pull/6983#issuecomment-5618091166) |
| 7 | [#6980](https://github.com/edwardkim/rhwp/pull/6980) | 2026-09-10 11:40:47 | CLOSED | 원제품PR | 있음 | `889133a40f6f5a3405bc5f26b43d0e489509f5f1` | M: paint clip 확장과 기존 flow 경계 분리 [기록](https://github.com/edwardkim/rhwp/pull/6980#issuecomment-5618089229) |
| 8 | [#6978](https://github.com/edwardkim/rhwp/pull/6978) | 2026-09-10 11:40:37 | CLOSED | 원제품PR | 미확인 | `607e18a61a3ff9c06256b5ab05c441ecca74749f` | 수용: full-page TAC 줄/페이지 소유권; 시험 위치 이동 이력 [기록](https://github.com/edwardkim/rhwp/pull/6978#issuecomment-5618087209) |
| 9 | [#6977](https://github.com/edwardkim/rhwp/pull/6977) | 2026-09-10 11:40:27 | CLOSED | 원제품PR | 있음 | `776058d0c5c5d8a808976ff0dcbe71b4d31e267b` | 기여자 후속: bright/contrast 공통 순서와 워터마크 판정 [기록](https://github.com/edwardkim/rhwp/pull/6977#issuecomment-5618084559) |
| 10 | [#6968](https://github.com/edwardkim/rhwp/pull/6968) | 2026-09-10 11:40:18 | CLOSED | 원제품PR | 있음 | `b2fc8ed57f4355d3573ece8cc78705b9f5ea193d` | 수용: 구역 경계 fieldEnd 연결 [기록](https://github.com/edwardkim/rhwp/pull/6968#issuecomment-5618082094) |
| 11 | [#6967](https://github.com/edwardkim/rhwp/pull/6967) | 2026-09-10 11:40:08 | CLOSED | 원제품PR | 있음 | `e0565095b66e045a4ee3dc8215f71f2bb50e0571` | M: UTF-16 위치/exclusive end/COLORREF/marker 왕복 [기록](https://github.com/edwardkim/rhwp/pull/6967#issuecomment-5618079729) |
| 12 | [#6960](https://github.com/edwardkim/rhwp/pull/6960) | 2026-09-10 11:39:59 | CLOSED | 원제품PR | 있음 | `dfc69fab6a8538d788cb51d363c6ce9baefdaec0` | 기여자 후속: CanvasKit/Canvas2D 공유 crop 조건; 예산 변경은 별도 감사 대상 [기록](https://github.com/edwardkim/rhwp/pull/6960#issuecomment-5618077871) |
| 13 | [#6959](https://github.com/edwardkim/rhwp/pull/6959) | 2026-09-10 11:39:49 | CLOSED | 원제품PR | 있음 | `9ed3a34b90bddc1b6fd9f0327c546b9aa9b03b5b` | 수용: bbox와 Line/Path 좌표; golden/fixture 등록 후속 [기록](https://github.com/edwardkim/rhwp/pull/6959#issuecomment-5618075969) |
| 14 | [#6990](https://github.com/edwardkim/rhwp/pull/6990) | 2026-09-10 11:34:24 | MERGED | 통합 | 있음 | `bc5ab3bd80c8db2b98e1ba2129d99827d80854d6` | I: 8개 원 PR 통합; #6967/#6980/#6983 보정 |
| 15 | [#6985](https://github.com/edwardkim/rhwp/pull/6985) | 2026-09-10 10:48:54 | CLOSED | 원제품PR | 있음 | `375df9bb7c2574bf174d15e757735c0d7de862a2` | M: 글상자 크기 확대만으로 남는 본문 겹침/표 앵커 [기록](https://github.com/edwardkim/rhwp/pull/6985#issuecomment-5617447513) |
| 16 | [#6989](https://github.com/edwardkim/rhwp/pull/6989) | 2026-09-10 10:45:32 | MERGED | 통합 | 있음 | `509ae0d3a7443c89a420e4ce49fb06c9f92f7565` | I: #6985 보정 통합 [기록](https://github.com/edwardkim/rhwp/pull/6989#issuecomment-5617448386) |
| 17 | [#6987](https://github.com/edwardkim/rhwp/pull/6987) | 2026-09-10 08:41:11 | MERGED | 원제품PR | 미확인 | `fc076d53782f8950ab52713d3cfa3cfa6e8a8418` | 자체 기능 개발: 줌 안정화; 비선점 등 한계 구분 [기록](https://github.com/edwardkim/rhwp/pull/6987#issuecomment-5615821053) |
| 18 | [#6982](https://github.com/edwardkim/rhwp/pull/6982) | 2026-09-10 05:31:57 | MERGED | 문서 | 미확인 | `4949403c44ccc9f09b1456cab5bbcd5d58b228e0` | D: #6979 후속 기록 |
| 19 | [#6979](https://github.com/edwardkim/rhwp/pull/6979) | 2026-09-10 05:28:59 | MERGED | 원제품PR | 미확인 | `9197ae2106f97fba8763af09836419f125fc3747` | 자체 기능 개발: 수평 여백 범위; 페이지 내용 대응 오류 정정 [기록](https://github.com/edwardkim/rhwp/pull/6979#issuecomment-5613702264) |
| 20 | [#6962](https://github.com/edwardkim/rhwp/pull/6962) | 2026-09-09 16:46:38 | MERGED | 원제품PR | 미확인 | `7c4c041b82affb3b9ed1d07e995802f5b6778c5f` | 자체 기능 개발: HWP3 정렬 호환성; 전체 fidelity와 구분 [기록](https://github.com/edwardkim/rhwp/pull/6962#issuecomment-5605543474) |
| 21 | [#6952](https://github.com/edwardkim/rhwp/pull/6952) | 2026-09-09 14:47:56 | CLOSED | 원제품PR | 있음 | `4529c2a0c0104a2783b44a5b798a7badce628042` | M: 선행 #6940의 숫자 형식 빈 접미 보존; 보정 테스트 입력 오류도 존재 [기록](https://github.com/edwardkim/rhwp/pull/6952#issuecomment-5603830609) |
| 22 | [#6949](https://github.com/edwardkim/rhwp/pull/6949) | 2026-09-09 14:47:50 | CLOSED | 원제품PR | 있음 | `8fbfd865b1190fdf95b5a98d80467a53c944ce59` | M: ROP 완성 시퀀스/clip/팔레트 조건; 원 이슈 잔여 [기록](https://github.com/edwardkim/rhwp/pull/6949#issuecomment-5603829248) |
| 23 | [#6958](https://github.com/edwardkim/rhwp/pull/6958) | 2026-09-09 14:45:35 | MERGED | 문서 | 미확인 | `1a20f0e6a5360dbca4b03e61adf9c37612b48579` | D: #6957 후속 기록 |
| 24 | [#6957](https://github.com/edwardkim/rhwp/pull/6957) | 2026-09-09 14:41:08 | MERGED | 통합 | 있음 | `4e422a57d6775eb2f11dffb70b37632823659829` | I: #6949/#6952 보정 통합 [기록](https://github.com/edwardkim/rhwp/pull/6957#issuecomment-5603864613) |
| 25 | [#6773](https://github.com/edwardkim/rhwp/pull/6773) | 2026-09-09 13:51:01 | MERGED | 원제품PR | 미확인 | `24d27959db697abb3102a7ecadeabe5ae0cb4b73` | M: 위임 guard/표 삭제 이벤트/저장 재열기; 검증 절차 오해 명시 [기록](https://github.com/edwardkim/rhwp/pull/6773#issuecomment-5551786944) |
| 26 | [#6955](https://github.com/edwardkim/rhwp/pull/6955) | 2026-09-09 12:17:28 | MERGED | 문서 | 미확인 | `d6b7b3a83f70db48856819ecb77166a33fe61327` | D: #6942 후속 기록 [기록](https://github.com/edwardkim/rhwp/pull/6955#issuecomment-5601713782) |
| 27 | [#6940](https://github.com/edwardkim/rhwp/pull/6940) | 2026-09-09 11:58:28 | CLOSED | 원제품PR | 있음 | `52660ccb2c322b17b520103af06e3b5760899542` | 수용: 번호 토큰/빈 장식 문자; 추가 필드 잔여 분리 [기록](https://github.com/edwardkim/rhwp/pull/6940#issuecomment-5601454786) |
| 28 | [#6938](https://github.com/edwardkim/rhwp/pull/6938) | 2026-09-09 11:58:24 | CLOSED | 원제품PR | 있음 | `a343125084db800bdd7bbb8b719ee8b2158665cd` | M: 복합 차트 의미/재귀 안전성/역참조; 보정 자체의 파서 오류도 존재 [기록](https://github.com/edwardkim/rhwp/pull/6938#issuecomment-5601454147) |
| 29 | [#6953](https://github.com/edwardkim/rhwp/pull/6953) | 2026-09-09 11:57:19 | MERGED | 문서 | 미확인 | `053075a8a9ec2a5782fbe26adac966e49cf23705` | D: #6951 후속 기록 |
| 30 | [#6942](https://github.com/edwardkim/rhwp/pull/6942) | 2026-09-09 11:51:25 | MERGED | 원제품PR | 미확인 | `8f468c849e0b9fa3266830becb3b8a92323e5e99` | 추가 메인터너 제품 코드 보정 없음 명시 [기록](https://github.com/edwardkim/rhwp/pull/6942#issuecomment-5601713463) |
| 31 | [#6951](https://github.com/edwardkim/rhwp/pull/6951) | 2026-09-09 11:49:59 | MERGED | 통합 | 있음 | `459cba08d1d18adb64f55a2998948881cf2bb774` | I: #6938/#6940 통합; 복합 차트 보정 [기록](https://github.com/edwardkim/rhwp/pull/6951#issuecomment-5601484781) |
| 32 | [#6811](https://github.com/edwardkim/rhwp/pull/6811) | 2026-09-09 11:10:57 | MERGED | 원제품PR | 있음 | `09bfc393fb56dfa98c232162b8714678e7cf28dd` | M: 글꼴 접두사를 렉서에서 처리; 숫자/중첩 명령 보존 [기록](https://github.com/edwardkim/rhwp/pull/6811#issuecomment-5600922988) |
| 33 | [#6948](https://github.com/edwardkim/rhwp/pull/6948) | 2026-09-09 09:59:26 | MERGED | CI | 미확인 | `d7677c43de4df9d79851ab92eefa95cfb5f080f8` | O: 문서 base 전진/post-merge CodeQL 재사용 [기록](https://github.com/edwardkim/rhwp/pull/6948#issuecomment-5600284038) |
| 34 | [#6933](https://github.com/edwardkim/rhwp/pull/6933) | 2026-09-09 09:14:52 | MERGED | 원제품PR | 미확인 | `dfb208e8516da920d172872fbd16e9dd3771db26` | 추가 메인터너 제품 코드 보정 없음 명시; 최신 함수 인자는 기여자 후속 [기록](https://github.com/edwardkim/rhwp/pull/6933#issuecomment-5599720565) |
| 35 | [#6909](https://github.com/edwardkim/rhwp/pull/6909) | 2026-09-09 09:03:41 | CLOSED | 원제품PR | 미확인 | `b0ab0ea8ddbe7b8226ecb908df45cbae36a81e20` | 수용: setter original 크기 보존; 전체 resize 해결 아님 [기록](https://github.com/edwardkim/rhwp/pull/6909#issuecomment-5599272353) |
| 36 | [#6883](https://github.com/edwardkim/rhwp/pull/6883) | 2026-09-09 09:03:39 | CLOSED | 원제품PR | 있음 | `0e862edb0418668bc51b283565b2af3343881922` | 기여자 후속: raw_stream 부재를 편집 이력으로 오인한 gate 축소 [기록](https://github.com/edwardkim/rhwp/pull/6883#issuecomment-5599272070) |
| 37 | [#6945](https://github.com/edwardkim/rhwp/pull/6945) | 2026-09-09 09:02:22 | MERGED | 문서 | 미확인 | `95056d2252346e60f88fb4cf20b9c20f94494f63` | D: #6944 후속 기록 |
| 38 | [#6944](https://github.com/edwardkim/rhwp/pull/6944) | 2026-09-09 08:54:54 | MERGED | 통합 | 있음 | `44b2c1def8dc43ff4bd4dfcce7264dfa725eb8a5` | I: #6883/#6909 통합; 통합만으로 보정 건수 증가시키지 않음 |
| 39 | [#6937](https://github.com/edwardkim/rhwp/pull/6937) | 2026-09-09 08:40:52 | CLOSED | 원제품PR | 있음 | `038a3b828cf2e82c4ebfb98a1cdb468476538aa0` | 수용: 중첩 fieldEnd; 구역 간 잔여는 #6968과 분리 [기록](https://github.com/edwardkim/rhwp/pull/6937#issuecomment-5598963681) |
| 40 | [#6934](https://github.com/edwardkim/rhwp/pull/6934) | 2026-09-09 08:40:47 | CLOSED | 원제품PR | 있음 | `893be62716f05378251e93264d6805bd67dd0d06` | 수용: 차트 인식 가드; 전체 복원은 #6938과 분리 [기록](https://github.com/edwardkim/rhwp/pull/6934#issuecomment-5598962318) |
| 41 | [#6918](https://github.com/edwardkim/rhwp/pull/6918) | 2026-09-09 08:40:43 | CLOSED | 원제품PR | 있음 | `8db9e0019b31af7a920a70a41fb9d08ae92db6bb` | 수용: 아래로 떨어진 float의 흐름 영향; 시험 모듈 위치 수정 [기록](https://github.com/edwardkim/rhwp/pull/6918#issuecomment-5598961112) |
| 42 | [#6914](https://github.com/edwardkim/rhwp/pull/6914) | 2026-09-09 08:40:39 | CLOSED | 원제품PR | 있음 | `fd2bfdb38e35921b46033c0fea4a88f0794d47d8` | M: 간격 생략을 저장 위치와 현재 흐름 일치로 제한 [기록](https://github.com/edwardkim/rhwp/pull/6914#issuecomment-5598959894) |
| 43 | [#6943](https://github.com/edwardkim/rhwp/pull/6943) | 2026-09-09 08:35:13 | MERGED | 통합 | 있음 | `daf888f5eda3ca9e3ddae207c3ba986c2a7ddfc1` | I: 4개 PR 통합; #6914 보정 [기록](https://github.com/edwardkim/rhwp/pull/6943#issuecomment-5598964817) |
| 44 | [#6932](https://github.com/edwardkim/rhwp/pull/6932) | 2026-09-09 07:06:20 | CLOSED | 원제품PR | 있음 | `82632a01c2e009e74c50d0a8692e1e1b2997e08c` | 수용 기록: float 띠와 저장 흐름 신뢰 [기록](https://github.com/edwardkim/rhwp/pull/6932#issuecomment-5597678400) |
| 45 | [#6931](https://github.com/edwardkim/rhwp/pull/6931) | 2026-09-09 07:06:16 | CLOSED | 원제품PR | 있음 | `116a1e743ed77402d467c15bab3ac4e19676f7af` | 수용 기록: WMF 기본 TA_TOP [기록](https://github.com/edwardkim/rhwp/pull/6931#issuecomment-5597677232) |
| 46 | [#6926](https://github.com/edwardkim/rhwp/pull/6926) | 2026-09-09 07:06:12 | CLOSED | 원제품PR | 있음 | `137edf196b5c62bda9bf01a2bd4b6ef3d57fef64` | 통합 중 focused 회귀 보완; 제품 보정 여부를 단정하지 않음 [기록](https://github.com/edwardkim/rhwp/pull/6926#issuecomment-5597676283) |
| 47 | [#6917](https://github.com/edwardkim/rhwp/pull/6917) | 2026-09-09 07:06:07 | CLOSED | 원제품PR | 있음 | `1af8d3eb961cd9d27af62fa1e335a490da439f9e` | 수용 기록: 셀 그림 host 줄 배치 [기록](https://github.com/edwardkim/rhwp/pull/6917#issuecomment-5597675231) |
| 48 | [#6915](https://github.com/edwardkim/rhwp/pull/6915) | 2026-09-09 07:06:03 | CLOSED | 원제품PR | 있음 | `4cc02ac252b379d6dafb31d44b65105f18d83b56` | 수용 기록: 선 굵기 격자 [기록](https://github.com/edwardkim/rhwp/pull/6915#issuecomment-5597674018) |
| 49 | [#6910](https://github.com/edwardkim/rhwp/pull/6910) | 2026-09-09 07:05:59 | CLOSED | 원제품PR | 있음 | `ba386ba856144c769063486d2bec0784a3d8548c` | 통합 중 focused 회귀/fixture 원장 보완; 제품 보정 여부 미단정 [기록](https://github.com/edwardkim/rhwp/pull/6910#issuecomment-5597672793) |
| 50 | [#6906](https://github.com/edwardkim/rhwp/pull/6906) | 2026-09-09 07:05:54 | CLOSED | 원제품PR | 있음 | `891eb812d73bda8762e2ff87a4313ca376b45f4b` | 수용: 서로게이트 치환; 세 번째 문서 미검증 분리 [기록](https://github.com/edwardkim/rhwp/pull/6906#issuecomment-5597671759) |
| 51 | [#6905](https://github.com/edwardkim/rhwp/pull/6905) | 2026-09-09 07:05:50 | CLOSED | 원제품PR | 있음 | `e3802579ace396ad3a1553e4033c876c3a811c40` | 수용: 스크롤 앵커; 문자열 계약의 동작 검증 한계는 #6908 참고 [기록](https://github.com/edwardkim/rhwp/pull/6905#issuecomment-5597670772) |
| 52 | [#6898](https://github.com/edwardkim/rhwp/pull/6898) | 2026-09-09 07:05:46 | CLOSED | 원제품PR | 있음 | `95bf64cb747db7681b723cbb98e3ce057997625e` | 기여자 후속: 앵커 판정을 TAC 형제 형상으로 축소 [기록](https://github.com/edwardkim/rhwp/pull/6898#issuecomment-5597669662) |
| 53 | [#6897](https://github.com/edwardkim/rhwp/pull/6897) | 2026-09-09 07:05:42 | CLOSED | 원제품PR | 있음 | `8daf2ce804abb9c60d414a68c7e3fc5c03de491e` | M: 표별 예약 인덱스 범위; 마지막 열 bbox 잔여 [기록](https://github.com/edwardkim/rhwp/pull/6897#issuecomment-5597668564) |
| 54 | [#6890](https://github.com/edwardkim/rhwp/pull/6890) | 2026-09-09 07:05:38 | CLOSED | 원제품PR | 있음 | `41a6953f64631d95f7dd494bc8d5d49e909c0e10` | 기여자 후속: 합성 설명 그림을 실제 렌더 증적으로 교체 [기록](https://github.com/edwardkim/rhwp/pull/6890#issuecomment-5597667452) |
| 55 | [#6886](https://github.com/edwardkim/rhwp/pull/6886) | 2026-09-09 07:05:33 | CLOSED | 원제품PR | 있음 | `438c932a3d75c2fb6b962d2e106f21dd6c81058d` | 수용: 회전 그림 hp:sz; rustfmt 후속 [기록](https://github.com/edwardkim/rhwp/pull/6886#issuecomment-5597666308) |
| 56 | [#6939](https://github.com/edwardkim/rhwp/pull/6939) | 2026-09-09 06:58:27 | MERGED | 통합 | 있음 | `12bca311002ac63ebe5488ed88f3880008b929fa` | I: 12개 PR 통합; 제품/시험/시각 도구 보정을 구별 [기록](https://github.com/edwardkim/rhwp/pull/6939#issuecomment-5597679442) |
| 57 | [#6930](https://github.com/edwardkim/rhwp/pull/6930) | 2026-09-09 04:25:21 | MERGED | 원제품PR | 미확인 | `641c8da175985e419f35c228f3bbf0bb6a6f74b8` | 자체 기능 개발: 제품/Gym 의존 분리; oracle mismatch와 구분 [기록](https://github.com/edwardkim/rhwp/pull/6930#issuecomment-5596026294) |
| 58 | [#6908](https://github.com/edwardkim/rhwp/pull/6908) | 2026-09-08 17:35:16 | CLOSED | 원제품PR | 있음 | `95983f66323cd915fe43bad4ec9e412eb3bc72ad` | 중복 종료: #6905; 브라우저 E2E 2건 제안, 통합 E2E 배선 제외 [기록](https://github.com/edwardkim/rhwp/pull/6908#issuecomment-5589274715) |
| 59 | [#6911](https://github.com/edwardkim/rhwp/pull/6911) | 2026-09-08 15:43:43 | MERGED | 문서 | 미확인 | `6906f0713606a7c29ec89d67054d88f2d5f86d1d` | D: #6901 운영 실증 기록 |
| 60 | [#6880](https://github.com/edwardkim/rhwp/pull/6880) | 2026-09-08 15:13:43 | MERGED | 원제품PR | 있음 | `44c5e50b83ef95e180d6c386b19e746e4dbe96bf` | M: 한컴 재저장 vpos reset 및 표 첫 줄 유지 [기록](https://github.com/edwardkim/rhwp/pull/6880#issuecomment-5587914743) |
| 61 | [#6903](https://github.com/edwardkim/rhwp/pull/6903) | 2026-09-08 14:17:43 | MERGED | CI | 미확인 | `f4a151e3d26969d1b66036f58d4f3887d5b04a3a` | O: CI 실패 증거; CodeQL inline 지적 및 후속 수정 |
| 62 | [#6885](https://github.com/edwardkim/rhwp/pull/6885) | 2026-09-08 13:04:26 | MERGED | 원제품PR | 있음 | `59ccf6df8fe9557ebcff4cefc6fe9f29d40f6d00` | M: 손상 CFB 경로 타입/안전 복구; Windows CRLF 테스트 보정 [기록](https://github.com/edwardkim/rhwp/pull/6885#issuecomment-5585602498) |
| 63 | [#6904](https://github.com/edwardkim/rhwp/pull/6904) | 2026-09-08 12:59:01 | MERGED | CI | 미확인 | `6abd02d58e231bb0661c73c687624893a1485846` | O: fork post-merge 재사용; 잔여 이슈 유지 [기록](https://github.com/edwardkim/rhwp/pull/6904#issuecomment-5585845686) |
| 64 | [#6881](https://github.com/edwardkim/rhwp/pull/6881) | 2026-09-08 12:18:23 | MERGED | 원제품PR | 미확인 | `6839d8af29b2ca10a9bb4fab2eefa0aa701a18ef` | M: 준비 전 예산/중첩 SVG viewport/공통 글꼴 상한 [기록](https://github.com/edwardkim/rhwp/pull/6881#issuecomment-5585361042) |
| 65 | [#6863](https://github.com/edwardkim/rhwp/pull/6863) | 2026-09-08 11:52:27 | MERGED | 원제품PR | 미확인 | `5dff5b07acafb7c217e46d14d6086d249cf53044` | M: 다중 인라인 도형 위치; 같은 줄/다른 줄/정렬 반례 [기록](https://github.com/edwardkim/rhwp/pull/6863#issuecomment-5584729066) |
| 66 | [#6894](https://github.com/edwardkim/rhwp/pull/6894) | 2026-09-08 10:45:52 | MERGED | 원제품PR | 미확인 | `60f2c0c51cb1cf8d4dbe31fb69920e62e257ce14` | 자체 기능 개발: 구조 기반 도형 구분; 비어 있음과 손상 구분 [기록](https://github.com/edwardkim/rhwp/pull/6894#issuecomment-5583953235) |
| 67 | [#6893](https://github.com/edwardkim/rhwp/pull/6893) | 2026-09-08 10:36:50 | MERGED | 원제품PR | 미확인 | `36fd7f3ee05c2dc0aae8d1ecdbd936121df01568` | 검토 도구 자체 보정: 로컬 폰트 선언 보존; 원 PR 제품 오류와 분리 [기록](https://github.com/edwardkim/rhwp/pull/6893#issuecomment-5583882121) |
| 68 | [#6867](https://github.com/edwardkim/rhwp/pull/6867) | 2026-09-08 09:43:11 | CLOSED | 원제품PR | 있음 | `ed196de9d2129adfae41698099ec0a468e604093` | 수용: tab bbox; 기존 #6801과 중복 기능 계산 제외 [기록](https://github.com/edwardkim/rhwp/pull/6867#issuecomment-5582932151) |
| 69 | [#6859](https://github.com/edwardkim/rhwp/pull/6859) | 2026-09-08 09:43:06 | CLOSED | 원제품PR | 있음 | `9bdea6db5abf1565608ed30eba35025f1ef9bf60` | 수용: 페이지 밖 문구 누락; 전체 페이지 수 차이 유지 [기록](https://github.com/edwardkim/rhwp/pull/6859#issuecomment-5582930801) |
| 70 | [#6857](https://github.com/edwardkim/rhwp/pull/6857) | 2026-09-08 09:43:01 | CLOSED | 원제품PR | 있음 | `5894984a402cecc9a605ade9970501c2d924a115` | 수용: 빈 문단 고아 페이지; 전체 fidelity 아님 [기록](https://github.com/edwardkim/rhwp/pull/6857#issuecomment-5582929316) |
| 71 | [#6853](https://github.com/edwardkim/rhwp/pull/6853) | 2026-09-08 09:42:57 | CLOSED | 원제품PR | 있음 | `ae0d816ac7ac0ac9f07b0c07c30cd792bd4a090e` | M: TAC 높이/간격 및 HWP5 계보 HWPX; 중간 보정 회귀도 수정 [기록](https://github.com/edwardkim/rhwp/pull/6853#issuecomment-5582927976) |
| 72 | [#6851](https://github.com/edwardkim/rhwp/pull/6851) | 2026-09-08 09:42:52 | CLOSED | 원제품PR | 있음 | `a1fe3cec58892f061ba168c5cb8f59a2299a62ba` | 수용: SVG/canvas 공통 gradient 축; Claude 표기 양성 대조 [기록](https://github.com/edwardkim/rhwp/pull/6851#issuecomment-5582926451) |
| 73 | [#6849](https://github.com/edwardkim/rhwp/pull/6849) | 2026-09-08 09:42:47 | CLOSED | 원제품PR | 있음 | `43308896b42c274553dc7ddcc6655897b3900d56` | 수용: 미참조 BinData 제거; 완전 sanitize 보장 아님 [기록](https://github.com/edwardkim/rhwp/pull/6849#issuecomment-5582925282) |
| 74 | [#6889](https://github.com/edwardkim/rhwp/pull/6889) | 2026-09-08 09:37:09 | MERGED | 통합 | 있음 | `13f7671ddf8ceff26ee5a009ac708a3a65f64712` | I: 6개 PR 중 #6853 보정; 나머지를 보정 실패로 세지 않음 [기록](https://github.com/edwardkim/rhwp/pull/6889#issuecomment-5582957180) |
| 75 | [#6878](https://github.com/edwardkim/rhwp/pull/6878) | 2026-09-08 06:26:33 | MERGED | 문서 | 미확인 | `e9156623e70c7e3a1ac7c842d45cf9cb7db58256` | D: #6877 운영 기록 |
| 76 | [#6877](https://github.com/edwardkim/rhwp/pull/6877) | 2026-09-08 06:18:25 | MERGED | CI | 미확인 | `04c71852cf7ada9acc59eef60058b2256cac5afa` | O: Rust CodeQL 통계 쿼리 축소 [기록](https://github.com/edwardkim/rhwp/pull/6877#issuecomment-5580351288) |
| 77 | [#6858](https://github.com/edwardkim/rhwp/pull/6858) | 2026-09-08 01:10:07 | MERGED | 원제품PR | 미확인 | `4b1c33576a76f600df29184fda3f395a81636f98` | 자체 기능 개발: 사각형 실선 보존 [기록](https://github.com/edwardkim/rhwp/pull/6858#issuecomment-5577585411) |
| 78 | [#6850](https://github.com/edwardkim/rhwp/pull/6850) | 2026-09-07 16:03:54 | MERGED | 원제품PR | 미확인 | `27299b4ece577821e79ce361ccea44e1776bf252` | 자체 기능 개발: 어울림 그림 뒤 TAC 표 줄 배치 |
| 79 | [#6846](https://github.com/edwardkim/rhwp/pull/6846) | 2026-09-07 14:50:15 | CLOSED | 원제품PR | 있음 | `89c373596247b1eeac9902f55a955d8c1417a7b8` | 수용: gradient stop; 각도 문제와 분리 [기록](https://github.com/edwardkim/rhwp/pull/6846#issuecomment-5572368729) |
| 80 | [#6843](https://github.com/edwardkim/rhwp/pull/6843) | 2026-09-07 14:50:11 | CLOSED | 문서 | 있음 | `ac5743ae342ec96a4e0508974d6747ad9dc847a0` | 문서 보정: fixture gate 목록/보안 입력; 제품 코드 오류로 세지 않음 [기록](https://github.com/edwardkim/rhwp/pull/6843#issuecomment-5572368029) |
| 81 | [#6836](https://github.com/edwardkim/rhwp/pull/6836) | 2026-09-07 14:50:08 | CLOSED | 원제품PR | 미확인 | `d964eea96d3837bd11181f887d3957b99b46a675` | 수용: Square 표 vertOffset; 완전 시각 일치 아님 [기록](https://github.com/edwardkim/rhwp/pull/6836#issuecomment-5572367392) |
| 82 | [#6809](https://github.com/edwardkim/rhwp/pull/6809) | 2026-09-07 14:50:04 | CLOSED | 원제품PR | 미확인 | `9233278ba49b9b52ea8341938cfc7a21ce590f75` | 수용: 수식 partial property bag 및 크기 보존 [기록](https://github.com/edwardkim/rhwp/pull/6809#issuecomment-5572366735) |
| 83 | [#6808](https://github.com/edwardkim/rhwp/pull/6808) | 2026-09-07 14:50:01 | CLOSED | 원제품PR | 미확인 | `6ceedc7ad4d87314b1e650e5080ab0f8e6594a89` | M: 실제 resize Undo의 raw/높이 복원 journal [기록](https://github.com/edwardkim/rhwp/pull/6808#issuecomment-5572366070) |
| 84 | [#6848](https://github.com/edwardkim/rhwp/pull/6848) | 2026-09-07 14:44:44 | MERGED | 통합 | 있음 | `fdbc608f9571e77cefef10c3050b6f6c10908379` | I: 5개 PR 통합; #6808 제품/#6843 문서 보정 구분 [기록](https://github.com/edwardkim/rhwp/pull/6848#issuecomment-5572369467) |
| 85 | [#6839](https://github.com/edwardkim/rhwp/pull/6839) | 2026-09-07 12:49:38 | CLOSED | 원제품PR | 있음 | `855e2386291556536c40f36f7eafae570dce0d6a` | 수용: 중첩 행 분할; #6792 잔여와 중복 집계 금지 [기록](https://github.com/edwardkim/rhwp/pull/6839#issuecomment-5570884857) |
| 86 | [#6817](https://github.com/edwardkim/rhwp/pull/6817) | 2026-09-07 12:49:33 | CLOSED | 원제품PR | 있음 | `5670dc13eb4e0e95517e5d58a381790f6a7e8580` | 수용: rowspan 끝 컷; 축소 입력/기준 PDF 후속 [기록](https://github.com/edwardkim/rhwp/pull/6817#issuecomment-5570883740) |
| 87 | [#6804](https://github.com/edwardkim/rhwp/pull/6804) | 2026-09-07 12:49:27 | CLOSED | 원제품PR | 있음 | `15a881d02a1e7a1ece63289bd633d54f15066842` | M: 필수 공개 fixture/분할 소유/새 sample 원장 보완 [기록](https://github.com/edwardkim/rhwp/pull/6804#issuecomment-5570882652) |
| 88 | [#6801](https://github.com/edwardkim/rhwp/pull/6801) | 2026-09-07 12:49:21 | CLOSED | 원제품PR | 있음 | `c7dade57c5f3b9d4bf2fe663613afe60b953612e` | M: bbox/charX/hit-test/탭 리더 경계 일치 [기록](https://github.com/edwardkim/rhwp/pull/6801#issuecomment-5570881609) |
| 89 | [#6798](https://github.com/edwardkim/rhwp/pull/6798) | 2026-09-07 12:49:16 | CLOSED | 원제품PR | 있음 | `deb4396ebe9b2ddd39f46a9e0dbd0a685c735e6b` | M: 실제 충돌/저장 앵커/전체 표 수용 조건; 임의 문턱 제거 [기록](https://github.com/edwardkim/rhwp/pull/6798#issuecomment-5570880545) |
| 90 | [#6796](https://github.com/edwardkim/rhwp/pull/6796) | 2026-09-07 12:49:10 | CLOSED | 원제품PR | 있음 | `097c499fef6e91e5f1efc0273caf30b38e12ec38` | M: 필수 원본/대상 존재/원본-축소본 기하 동일성 회귀 [기록](https://github.com/edwardkim/rhwp/pull/6796#issuecomment-5570879486) |
| 91 | [#6794](https://github.com/edwardkim/rhwp/pull/6794) | 2026-09-07 12:49:04 | CLOSED | 원제품PR | 있음 | `2bed19a6cf60757508df16c719fe24d593b382b3` | 기여자 수정 후 승인: 잘못된 1쪽 차례 oracle를 2쪽 소유로 정정 [기록](https://github.com/edwardkim/rhwp/pull/6794#issuecomment-5570878419) |
| 92 | [#6792](https://github.com/edwardkim/rhwp/pull/6792) | 2026-09-07 12:48:59 | CLOSED | 원제품PR | 있음 | `0b8e705a4b46dc1d63b38070b69c1a9f2ada15fe` | M: 페이지 수용 우회/진짜 양성 핀 및 범위 보완 [기록](https://github.com/edwardkim/rhwp/pull/6792#issuecomment-5570877424) |
| 93 | [#6789](https://github.com/edwardkim/rhwp/pull/6789) | 2026-09-07 12:48:54 | CLOSED | 원제품PR | 있음 | `74256bd5b25bd00f9e7bb67bb850cebbea90e223` | M: 공통 무리 판정과 공개 혼재/음수/겹침 경계 회귀 [기록](https://github.com/edwardkim/rhwp/pull/6789#issuecomment-5570876459) |
| 94 | [#6784](https://github.com/edwardkim/rhwp/pull/6784) | 2026-09-07 12:48:49 | CLOSED | 원제품PR | 있음 | `44432e34437c051575344a27129695f3fce7ca31` | M: 레인 연관/첫 비레인 paint 전 흐름 복구 [기록](https://github.com/edwardkim/rhwp/pull/6784#issuecomment-5570875418) |
| 95 | [#6847](https://github.com/edwardkim/rhwp/pull/6847) | 2026-09-07 12:42:10 | MERGED | 통합 | 있음 | `492730110e87d96eb6f7d2f60a46e6356578cd25` | I: 10개 PR 통합; 최종 승인/보정 판정은 개별 분리 [기록](https://github.com/edwardkim/rhwp/pull/6847#issuecomment-5570908310) |
| 96 | [#6820](https://github.com/edwardkim/rhwp/pull/6820) | 2026-09-07 08:54:18 | MERGED | CI | 미확인 | `9400653c343f70fb850167aa825d29a32cc3fed7` | O: 완료 audit의 controller 취소 방지 [기록](https://github.com/edwardkim/rhwp/pull/6820#issuecomment-5568047019) |
| 97 | [#6834](https://github.com/edwardkim/rhwp/pull/6834) | 2026-09-07 08:45:41 | CLOSED | 의존성 | 미확인 | `e0e102d5f7d4af7f5c9188b18045de8bc0fc1618` | B: Dependabot; 범위 밖 #6838 통합으로 종료 [기록](https://github.com/edwardkim/rhwp/pull/6834#issuecomment-5567907937) |
| 98 | [#6833](https://github.com/edwardkim/rhwp/pull/6833) | 2026-09-07 08:45:38 | CLOSED | 의존성 | 미확인 | `305d855e4b265bfae13645bc6fcdfebad1e964d5` | B: Dependabot; 범위 밖 #6838 통합으로 종료 [기록](https://github.com/edwardkim/rhwp/pull/6833#issuecomment-5567907040) |
| 99 | [#6832](https://github.com/edwardkim/rhwp/pull/6832) | 2026-09-07 08:45:35 | CLOSED | 의존성 | 미확인 | `3b7db11c339d057e8ced367c1df5d85acca71a1d` | B: Dependabot; 범위 밖 #6838 통합으로 종료 [기록](https://github.com/edwardkim/rhwp/pull/6832#issuecomment-5567906373) |
| 100 | [#6831](https://github.com/edwardkim/rhwp/pull/6831) | 2026-09-07 08:45:31 | CLOSED | 의존성 | 미확인 | `2a0e97af03d76a42dbec6ee7c959627e69d90eb0` | B: Dependabot Skia API 호환; Claude 오류 모집단에서 제외 [기록](https://github.com/edwardkim/rhwp/pull/6831#issuecomment-5567905515) |
