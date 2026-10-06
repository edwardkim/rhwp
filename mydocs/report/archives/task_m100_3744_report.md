# task_m100_3744 처리결과 보고서 — clause context confidence

- **Issue**: [#3744](https://github.com/edwardkim/rhwp/issues/3744)
- **상위 이슈**: [#1528](https://github.com/edwardkim/rhwp/issues/1528)
- **선행 작업**: [#3693](https://github.com/edwardkim/rhwp/issues/3693) / PR #3715,
  [#3695](https://github.com/edwardkim/rhwp/issues/3695) / PR #3749
- **브랜치**: `codex/issue-3744-clause-context-confidence`
- **Draft PR**: [#3933](https://github.com/edwardkim/rhwp/pull/3933)
- **최종 기준**: `upstream/devel` `301d0fe5f`
- **상태**: review 보정·최신 devel 결합·로컬 검증 완료, review 문서 갱신·push 진행 중

## 1. 결론

`export-structure --mode clause`의 weak marker 판정을 세 경계에서 보정했다.

1. `N.N` 복합 번호를 만나면 현재 nearest `조|항`의 weak-`호` 허용 상태를 만료해 뒤따르는
   SQL·일반 번호 목록을 `호`로 승격하지 않는다. 단, 같은 section의 바로 다음 문단이 경계 앞 번호
   또는 직전 정상 번호의 다음 번호이면 정상 목록 재개로 회복한다.
2. 유효한 월·일 범위의 `YYYY. M. D[.]` 날짜는 열린 조문 안에서도 `호`로 승격하지 않고 body에
   보존한다.
3. 열린 `호`가 없어도 `장|절` 아래의 `가.`~`하.` 문단은 탭·dotted-leader TOC tail이 없고 ParaShape가
   `margin_left=0`, `indent>=-1280`, `para_level=0`일 때 direct `목`으로 회복한다.

세 정책은 비공개 query context에만 구현했다. parser, renderer, 공개 구조체, CLI option, JSON 봉투,
explicit outline 및 auto 선택 정책은 바꾸지 않았다.

## 2. 구현과 red→green

- `src/document_core/queries/structure.rs`
  - nearest `조|항` 식별자별 만료 상태를 보관하는 비공개 `ClauseGateState`
  - 복합 번호 경계와 인접 문단 정상 목록 회복, 달력형 날짜 판정
  - `.`, `·`, `‥`, `…` dotted leader 쪽번호 판정
  - HWPUNIT 단위의 direct `목` indent 하한 명명 상수
  - 문단 원문·ParaShape·열린 stack을 함께 보는 direct `목` confidence gate
- `tests/issue_3744_structure_clause_confidence.rs`
  - 기존 8개에 복합 번호 뒤 정상 목록 회복, dotted-leader TOC, shape 하한 경계를 더한 총 11개 회귀
  - 날짜·TOC·거부된 weak marker가 사라지지 않고 body에 남는지 함께 단언

Stage 2에서 신규 테스트는 2 passed / 6 failed였고 Stage 3 구현 후 8 passed / 0 failed로 전환됐다.
PR review 보정은 9 passed / 2 failed red에서 11 passed / 0 failed로 전환됐다. #3693 clause 문맥,
#3695 auto 선택, CLI JSON 계약 focused test도 함께 통과했다.

## 3. 최신 devel 동기화

Stage 4에서 작업의 세 로컬 커밋을 당시 `upstream/devel` `2971a1d9a`에 conflict 없이 rebase했다.
최신 upstream이 추가한 `StructureDoc.node_count`의 `nodeCount` 직렬화 계약을 최종 트리에 그대로
보존했고, envelope integrity test를 포함한 전체 release test에서 이를 재확인했다.

PR review 보정 중 원격 head `ce6a23bca`를 기준으로 전체 release 검증을 다시 통과했고, 게시 직전
새로 병합된 `upstream/devel` `301d0fe5f`도 충돌 없이 결합했다. 최신 devel은 #3744 제품 파일과
겹치지 않았으며 결합 head에서 focused 5개 gate를 재실행했다. 전체 결합 head의 최종 판정은 push 뒤
GitHub Actions를 merge gate로 둔다.

## 4. 전체 sample corpus 영향

기준 checkout과 보정 checkout에 서로 다른 Cargo target을 사용해 artifact 혼입을 차단했다. 각
checkout에서 `samples/` top-level 및 recursive 입력을 독립 parse한 뒤 동일한 집계 형식으로 비교했다.

| 범위 | 후보 | parse 성공 | clause 문서 | parse 차이 | 변경 문서 |
| --- | ---: | ---: | ---: | ---: | ---: |
| top-level | 353 | 350 | 81 | 0 | 10 |
| recursive | 673 | 670 | 136 | 0 | 11 |

양쪽에서 동일하게 실패한 세 암호 sample은 password 없이 parse할 수 없어 집계에서 제외했다.

- `HWP3-password-123456.hwp`
- `HWP5-password-123456.hwpx`
- `hwp3-sample16-hwp5-2024-password-123456.hwp`

### 4.1 kind별 증감

| 범위 | 항목 | 기준 | 보정 | 증감 |
| --- | --- | ---: | ---: | ---: |
| top-level | 총 node | 10,698 | 6,491 | -4,207 |
| top-level | `호` | 5,816 | 1,465 | -4,351 |
| top-level | `목` | 794 | 938 | +144 |
| recursive | 총 node | 12,268 | 8,084 | -4,184 |
| recursive | `호` | 6,257 | 1,906 | -4,351 |
| recursive | `목` | 858 | 1,025 | +167 |

`편`, `장`, `절`, `조`, `항` 수는 두 범위 모두 변하지 않았다. recursive 기준으로 `편` 1,
`장` 59, `절` 110, `조` 148, `항` 4,835가 그대로다.

### 4.2 변경 문서 분류

`호` -4,351은 다음 6개 Oracle/Unix 기술문서에만 발생했다.

- `hwp3-sample10` HWP/HWP5/HWPX 세 변형: 각 `호` -1,224
- `hwp3-sample11.hwp`: `호` -227
- `hwp3-sample11` HWP5/HWPX 두 변형: 각 `호` -226, `목` -2

sample10의 오래된 원문자 anchor 뒤 `4.1` 복합 번호가 만료 경계가 되어 이후 SQL 번호 목록을
제거한다. sample11에서 함께 빠진 `목` 4개는 `# setenv LM_LICENSE_FILE...`,
`# setenv LD_LIBRARY_PATH...` 같은 shell 목록으로, direct 제목이 아닌 기존 오탐이다.

direct `목` 회복은 다음 5개 문서에서 총 +171개다.

- `2025 행정업무운영 편람(최종)` HWP/HWPX: 각 +44
- 시장구조조사 보고서: +23; recursive 하위 sample에만 존재
- 최신 upstream에 추가된 정책연구 보고서 HWP/HWPX: 각 +30

정책연구 보고서의 추가 60개도 `절` 또는 `절 > 항` 아래의 `일반 장기 이식 통계`, `간 이식 통계`,
`생존 장기기증 법적기준`, `적합성 평가`, `금기기준`, `동의 취득` 등 실제 제목이다. shape는
`margin_left=0`, `para_level=0`, `indent=0` 또는 `1992`로 승인된 문서 독립 규칙을 만족한다.

따라서 recursive `목` 순증가 +167은 실제 direct 제목 +171에서 shell 목록 오탐 4개를 뺀 값이다.
top-level 순증가 +144는 하위 디렉터리의 시장구조조사 +23을 제외한 같은 계산이다. parse 성공/실패,
변경 문서, kind 증감은 모두 선택 정책으로 설명되며 미분류 변화는 없다.

### 4.3 review 보정 영향

review에서 요청한 정상 목록 회복을 번호 일치만으로 허용한 최초 구현은 sample10 세 변형에서
node 8→1,145, `호` 4→1,141로 오래된 anchor를 다시 열었다. 이 설계는 즉시 기각했다.

최종 구현은 같은 section의 경계 바로 다음 문단에만 복귀를 허용한다. 원격 보정 전 head
`ce6a23bca`와 최종 정책을 recursive 673개(670 parse)에서 다시 비교한 결과, 파일별 node 수와 kind
분포는 모두 동일했고 parse 실패 3개도 같았다. 즉 이번 보정은 새 synthetic 정상/negative 경계를
고정하면서 기존 corpus 결과를 바꾸지 않는다. corpus 측정은 auto 선택 결과가 아니라
`StructureMode::Clause`를 명시한 explicit clause 출력이다.

## 5. 잔여 trade-off

- Oracle 문서의 복합 번호 경계 전 `1.`~`4.` 네 `호`는 기존 앵커 아래 남는다. 정상 조문의 반복
  번호를 대량 손실시키는 거리·엄격 연속성 정책을 피하기 위해 승인된 잔여 false positive다.
- `장|절` 아래라도 `margin_left != 0`, `indent < -1280` HWPUNIT, `para_level != 0`인 deeper `목` 후보는
  보수적으로 body에 남는다. 문서별 shape ID나 좌표를 하드코딩하지 않기 위한 잔여 false negative다.
- 날짜 gate는 마지막 점의 유무와 관계없이 월 1~12, 일 1~31의 lexical 경계만 확인하고 월별 실제
  일수·윤년까지 검증하지 않는다. 실제 corpus 변화는 0이며 synthetic oracle이 제품 경계를 고정한다.

## 6. 최종 검증

전체 release와 정적 검사는 review 보정 후보 `ce6a23bca` 기반 checkout에서 순차 실행했고,
최신 `upstream/devel` `301d0fe5f` 결합 뒤 focused 5개 gate를 다시 실행했다.

| 검증 | 결과 |
| --- | --- |
| structure 단위 | 8 passed |
| #3744 영구 회귀 | 11 passed |
| #3693 clause 문맥 | 3 passed |
| #3695 auto 정책 | 13 passed |
| CLI `export_structure_` 계약 | 4 passed |
| `cargo test --profile release-test --tests` | 3,200 passed / 7 ignored / 0 failed(lib 포함), 전체 test target exit 0 |
| `cargo fmt --check` | 통과 |
| `git diff --check upstream/devel...HEAD` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |

변경은 읽기 전용 structure query의 분류 결과에 한정되고 renderer·layout·pagination을 건드리지 않아
시각 검증 대상이 아니다.

## 7. 승인 경계와 후속 작업

Stage 4 범위인 동기화, corpus 영향 분류, 전체 검증과 최종 보고를 완료한 뒤 승인 D를 받아 원본
저장소 branch push와 `devel` 대상 draft PR #3933 생성을 완료했다. 이후 review 보정과 최신 devel
결합을 마쳤고, collaborator self-merge 근거는 `mydocs/pr/archives/pr_3933_review.md`와
`pr_3933_review_impl.md`에 분리해 기록한다.

GitHub review/comment, ready 전환, merge와 #3744/#1528 상태 변경은 수행하지 않는다. 두 커밋을 push한
최신 PR head의 required GitHub Actions가 통과하면 작업지시자가 merge하고, close comment와 상위
#1528 최종 검증은 별도 후속 처리한다.
