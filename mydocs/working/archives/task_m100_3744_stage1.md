# Stage 1 보고 — task_m100_3744 최신 재현·계획

- **일자**: 2026-08-03
- **이슈**: #3744
- **상위 추적**: #1528
- **기준**: `upstream/devel` `0889974a01db3585df8ad2c1f13203e3cb9f51f8`
- **브랜치**: `codex/issue-3744-clause-context-confidence`
- **단계 결론**: 이슈 유효, 작업 가치 있음. 구현은 미착수이며 계획 승인 대기

## 1. 조사 범위

다음을 읽기 전용으로 대조했다.

- #3744 본문과 발견 근거 PR #3715 comment `5152400380`
- 상위 #1528 및 선행 #3693·#3695/PR #3715·#3749 상태
- `src/document_core/queries/structure.rs`
- `tests/issue_3693_structure_clause_context.rs`
- #3693 계획·최종 보고와 PR #3715 collaborator review 기록
- 최신 `samples/` top-level 및 recursive 문서

GitHub comment·이슈 상태, 원격 branch/PR은 변경하지 않았다.

## 2. 최신 기준선

| 명령 | 결과 |
| --- | --- |
| `CARGO_INCREMENTAL=0 cargo test --lib document_core::queries::structure -- --nocapture` | 6 passed |
| `CARGO_INCREMENTAL=0 cargo test --test issue_3693_structure_clause_context -- --nocapture` | 3 passed |

#3693의 marker/context 테스트와 #3695가 통합된 최신 `devel`이 green이므로, 아래 실패는 선행 결과가
깨진 것이 아니라 #3744에 남겨 둔 별도 한계다.

## 3. red 재현

조사용 임시 통합 테스트 네 개를 만들어 최신 코드를 직접 호출했다. 기대한 세 red와 한 guard가
관측됐다.

| 조사 케이스 | 최신 결과 |
| --- | --- |
| stale anchor 뒤 SQL | failed — 문단 2303 `1)`이 `호` |
| 열린 `조` 안 개정 날짜 | failed — `2022.`가 `호` |
| `절` 직속 `가.` 제목 | failed — `목` node 없음 |
| 탭+쪽번호 TOC | passed — body 보존 |

임시 테스트는 계측을 마친 뒤 삭제했으며 이 단계의 최종 diff에는 문서만 남긴다.

## 4. 실문서 측정

### 4.1 `hwp3-sample10.hwp`

- 전체 node 1,232, `호` 1,228
- 유일한 anchor는 문단 2269 `① Partition 제거`, 2270 `② UNION-ALL view 의 Parallel 수행`
- anchor 전 `호` 0, anchor 뒤 `호` 1,228
- 문단 2303/2312/2313의 SQL 명령도 마지막 `항`의 자손 `호`로 남음

현행 stack은 같은/더 낮은 level heading이 들어올 때만 pop되고 section 전환에서도 초기화되지 않는다.
따라서 문맥의 시작 증거만 있고 종료 증거가 없다는 이슈 진단이 최신 코드에도 그대로 맞는다.

### 4.2 날짜

recursive sample 665개에서 현재 `호`로 채택된 완전한 `YYYY. M. D.` 날짜는 0개였다. 실물 재현이
없으므로 synthetic `제1조(목적) → 2022. 1. 1. 일부개정`을 최소 oracle로 사용했다. 열린 `조`가
있으면 현행 gate는 marker 모양만 보고 이를 `호`로 채택했다.

### 4.3 편람 `목`

- 최신 결과: 591 nodes, `목` 327
- `장|절` node direct body의 `가.`~`하.` 후보: 125
- 열린 `장|절` 조상을 단순 허용하면 들어올 후보: 128
- 후보 shape 분포 중 `(margin_left, indent, para_level)=(0,0,0)` 41개와 `(0,-1280,0)` 3개의
  합이 발견 코멘트의 44개와 정확히 일치
- 44개에는 코멘트 예시 `가. ‘업무’의 개념`이 포함되고 탭+쪽번호 tail은 0개

이 결과는 ParaShape가 positive evidence가 될 가능성을 높이지만, 128개 전체와 다른 문서의 같은
shape를 대조하기 전에는 채택 규칙으로 확정할 수 없다.

### 4.4 corpus 규모

| 범위 | 후보 | 성공 | 비고 |
| --- | ---: | ---: | --- |
| top-level `samples/*.{hwp,hwpx}` | 351 | 348 | 기존 리뷰와 같은 top-level 규모 |
| recursive `samples/**/*.{hwp,hwpx}` | 668 | 665 | password 문서 3개 실패 |

파싱 성공 665개 중 clause node가 있는 문서는 134개, 총 node는 12,016개다. 현재 heading 중
탭+숫자 tail은 79개여서 이 신호를 모든 heading에 일괄 적용하면 회귀 위험이 있다.

## 5. 코드 원인

`classify_clause()`는 약한 문자열 모양을 후보로 만드는 역할이고, `clause_heading_allowed()`가 열린
stack만 보고 최종 채택한다. 후자는 텍스트·위치·ParaShape를 받지 않으므로 다음을 표현할 수 없다.

1. 마지막 anchor가 얼마나 오래됐는가 또는 section이 바뀌었는가
2. `2022. 1. 1.`이 목록 번호가 아니라 날짜인가
3. `장|절` 아래 `가.`가 본문 제목인지 목차인지

#3695에서 추가한 `has_toc_page_number_tail()`은 auto selector의 strong `조` 증거에만 적용되고 explicit
clause weak-marker gate와 연결되지 않는다.

## 6. 가치 판정

작업 가치가 있다.

- stale anchor는 한 문서에서 `호` 1,228개라는 큰 출력 오염을 만들고 SQL도 구조화한다.
- 날짜 오검출은 현재 sample에는 없지만 법령 개정연혁의 일반적인 형식을 열린 조문에서 잘못 해석하는
  결정적 최소 재현이 있다.
- `목`은 현행 과검출 방지 정책의 대가로 실제 본문 제목을 잃고 있으며, 44개 shape 군을 최신 파서에서
  다시 식별했다.
- 세 문제는 모두 같은 explicit clause confidence/context 경계에 있어 #3744 한 이슈에서 순차적으로
  다루는 것이 응집도가 높다.

## 7. 다음 단계 권고와 승인 요청

수행계획서와 구현계획서대로 Stage 2에서 먼저 영구 red를 고정하고, 다음을 제품 구현과 분리해 비교한다.

1. section reset·거리 상한·번호 연속성의 anchor 만료 효과
2. 달력 범위를 포함한 날짜 lexical negative
3. 44개 shape 가설·128개 broad 후보·TOC tail의 corpus 판별력

정책 비교 결과를 별도 Stage 2 보고서와 계획서 보정 커밋으로 제시한 뒤 구현 승인 지점에서 다시
멈춘다. 현재는 Stage 2 착수 승인을 기다리며 제품 소스·영구 테스트·원격 상태를 변경하지 않는다.
