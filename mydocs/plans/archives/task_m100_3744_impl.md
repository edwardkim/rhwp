# 구현계획서 — task_m100_3744

- **이슈**: #3744
- **상위 이슈**: #1528
- **브랜치**: `codex/issue-3744-clause-context-confidence`
- **Draft PR**: [#3933](https://github.com/edwardkim/rhwp/pull/3933)
- **수행계획서**: `mydocs/plans/task_m100_3744.md`
- **기준 commit**: `upstream/devel` `0889974a01db3585df8ad2c1f13203e3cb9f51f8`
- **최종 동기화 기준**: `upstream/devel` `301d0fe5f`
- **절차 상태**: review 보정·최신 devel 결합·로컬 검증 완료, review 기록 갱신·push 진행 중
- **다음 승인 경계**: 승인 E — 최신 head CI 통과 뒤 작업지시자 merge

## 1. 변경 경계

주 변경점은 `src/document_core/queries/structure.rs`의 explicit clause weak-marker 채택 경로다.
`classify_clause()`의 marker 문법과 공개 `StructureNode`/`StructureDoc` 형태는 유지한다.

예상 변경 단위는 다음과 같다.

1. 문단 위치·텍스트·ParaShape와 열린 계층을 담는 비공개 clause evidence/context
2. 날짜형 `호` negative 판정 helper
3. `호` anchor 만료/연속성 판정 helper
4. `장|절` 아래 `목` confidence와 TOC tail 판정 helper
5. `build_structure()`의 section/paragraph 순회 중 context 갱신

정확한 필드와 helper 이름은 Stage 2 정책 선택 뒤 고정한다. 공개 API나 직렬화 필드는 추가하지 않는다.

## 2. Stage 2 — red 고정과 정책 선택

### 2.1 영구 회귀 테스트

새 통합 테스트 `tests/issue_3744_structure_clause_confidence.rs`에 최소 다음을 둔다.

1. `제1조 → ① → 일반 body 간격 → 1)`에서 승인된 만료 경계 뒤 후보가 body로 남는 synthetic negative
2. `hwp3-sample10.hwp` 문단 2303·2312·2313 SQL negative와 문단 2269·2270 anchor 존재 확인
3. `제1조(목적) → 2022. 1. 1. 일부개정` 날짜 negative와 body 보존 단언
4. `제1장 → 제1절 → 가. 본문 제목` direct `목` synthetic positive
5. `제1장 → 가. 개요\t9` TOC negative와 body 보존 단언
6. 편람에서 발견 코멘트와 일치한 44개 shape 후보의 대표 좌표·positive 가설과 목차 대표 negative
7. 협정서 `조 → 호 → 목` 및 #3693 괄호형 marker 무회귀

숫자 총량만 단언하지 않고 대표 좌표, marker, parent kind, body 보존을 함께 확인해 공허한 통과를 막는다.

### 2.2 앵커 정책 비교

각 후보를 제품 코드에 바로 넣지 않고 조사 helper 또는 test-local evaluator로 비교한다.

- section reset 단독
- 마지막 `조|항`에서의 거리 상한 후보
- 첫 번호·연속 번호·중단 후 재시작 신호
- section reset + 거리/연속성 조합

각 후보마다 다음을 표로 남긴다.

- `hwp3-sample10.hwp`의 1,228 `호` 중 제거/잔존 수와 SQL 3좌표 판정
- `hwp3-sample16-hwp5.hwp` 정상 조문 `호` 보존
- top-level 351개와 recursive 668개 sample의 문서별 `호` 증감
- false positive로 설명할 수 없는 감소 목록

최소 감소율 자체를 목표로 하지 않는다. 정상 조문을 보존하면서 명시한 false positive를 결정적으로
설명하는 정책을 선택한다.

### 2.3 `목` evidence 비교

편람의 broad-allow 후보 128개를 현재 계층, `margin_left`, `indent`, `para_level`, `head_type`,
탭+쪽번호 tail별로 분류한다.

- 발견 코멘트의 44개와 일치한 `(0, 0, 0)` 41개 + `(0, -1280, 0)` 3개는 positive 가설이다.
- style/shape ID는 문서 로컬 식별자이므로 판정에 쓰지 않는다.
- 탭+숫자 tail은 weak marker negative 후보로 평가하되 strong marker에는 blanket 적용하지 않는다.
- 다른 sample에서 같은 shape가 목차·일반 목록에 쓰이는지 반드시 역대조한다.

선택 규칙과 기각한 대안을 Stage 2 보고서 및 수행계획서 4절에 반영한 뒤 승인 B를 받는다.

### 2.4 Stage 2 선택 결과

1. anchor: `N.N` 복합 번호를 body로 거부하고 현재 nearest `조|항`의 weak-`호` 상태를 만료한다.
2. date: 유효 범위의 `YYYY. M. D.`를 suffix와 무관하게 거부한다.
3. direct `목`: TOC tail이 없고 `margin_left=0`, `indent>=-1280`, `para_level=0`인 문단만
   열린 `장|절` 조상 아래에서 허용한다.
4. section reset, 거리 cap, blanket strict sequence는 편람의 정상 반복 목록 손실 때문에 구현하지 않는다.

세부 실측과 잔여 trade-off는 `mydocs/working/task_m100_3744_stage2.md`가 canonical 근거다.

## 3. Stage 3 — 구현 (완료)

### 3.1 context 전달

`clause_heading_allowed()`가 다음 중 승인된 최소 evidence만 받도록 확장한다.

- 원문 `para_text`
- section/paragraph 위치
- `ParaShape`의 문서 독립 속성
- 열린 clause stack
- 마지막 strong/weak anchor와 번호 진행 상태

context는 `build_structure()` 내부 순회에서 갱신하고 section 전환, strong heading 채택, weak 후보 거부
시점의 상태 전이를 단위 테스트로 고정한다.

구현은 공개 context 구조를 추가하지 않고 `ClauseGateState`가 nearest `조|항`의 `(section,
paragraph)` 식별자별 만료 상태만 보관하도록 제한했다. 새 strong anchor는 새 식별자를 사용하므로 이전
anchor의 만료가 전파되지 않는다.

### 3.2 날짜 gate

- 선두 연도 4자리, 월·일 1~2자리와 점 구분자를 파싱하며 마지막 점은 선택 사항이다.
- 월 1~12, 일 1~31 범위를 확인해 일반 `1. 항목`과 분리한다.
- 열린 `조|항`이 있어도 날짜 후보는 heading으로 채택하지 않고 기존 body 경로로 보낸다.
- `일부개정` 같은 특정 suffix가 없어도 같은 날짜 문법은 일관되게 판정한다.

### 3.3 anchor 만료와 `목` confidence

- marker가 `.`으로 끝나고 marker 직후 숫자가 이어지는 `N.N` 후보를 거부하고 현재 nearest
  `조|항`의 weak-`호` 상태를 만료한다.
- 만료 경계 바로 다음 문단에서 경계 앞 번호 또는 직전 정상 번호의 다음 번호가 나타나면 같은
  section의 정상 목록 재개로 회복한다. 비인접 후보에는 이 예외를 적용하지 않는다.
- 그 밖의 만료는 같은 anchor 아래 후속 weak `호`에만 적용하고 새 `조|항`에서 초기화한다.
- `)` marker 뒤 본문 숫자, 날짜 거부, 일반 body는 anchor를 만료하지 않는다.
- strong 편·장·절·관·조와 원문자 항 분류를 바꾸지 않는다.
- 기존 열린 `호` 아래 `목`은 보존한다.
- `장|절` 아래 `목`은 열린 `호`가 없고 탭 또는 dotted leader의 TOC tail이 없으며 ParaShape가
  `margin_left=0`, `indent>=-1280`, `para_level=0`일 때만 허용한다.
- 거부 문단은 삭제하지 않고 현재 node body 또는 preamble에 그대로 보존한다.

## 4. Stage 3 focused 검증 (완료)

- `CARGO_INCREMENTAL=0 cargo test --lib document_core::queries::structure -- --nocapture`
- `CARGO_INCREMENTAL=0 cargo test --test issue_3744_structure_clause_confidence -- --nocapture`
- `CARGO_INCREMENTAL=0 cargo test --test issue_3693_structure_clause_context -- --nocapture`
- `CARGO_INCREMENTAL=0 cargo test --test issue_3695_structure_auto_policy -- --nocapture`
- `CARGO_INCREMENTAL=0 cargo test --test cli_json_contract export_structure_ -- --nocapture`

red→green 결과와 focused 실측은 `mydocs/working/task_m100_3744_stage3.md`에 기록했다. 이후 Stage 4에서
최신 기준의 focused gate를 다시 통과하고 전체 corpus·release 측정을 완료했다.

## 5. Stage 4 corpus·release 검증 (완료)

1. 작업 커밋을 최신 `upstream/devel` `2971a1d9a`에 conflict 없이 rebase하고 focused green을
   재확인했다. upstream의 `StructureDoc.node_count` `nodeCount` rename도 최종 트리에 보존했다.
2. 기준과 보정 checkout에 서로 다른 Cargo target을 사용해 top-level 353개(350 parse)와 recursive
   673개(670 parse)를 비교했다. 양쪽 parse 결과는 일치했고 변경 문서는 recursive 11개다.
3. recursive kind 증감은 `호` -4,351, `목` +167이며, 오래된 anchor 제거 6문서와 direct 제목 회복
   5문서로 분류했다. 대표 좌표와 문서별 근거는 최종 보고서에 기록했다.
4. 공개 JSON key와 CLI 계약은 기존 계약 테스트 및 최신 upstream envelope integrity test로 확인했다.
5. 다음 게이트를 순차 통과했다.

- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`
- `CARGO_INCREMENTAL=0 cargo fmt --check`
- `git diff --check`
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`

최종 결과는 `mydocs/report/task_m100_3744_report.md`에 기록했다. CLI 사용법·공개 schema는 바뀌지
않아 별도 CLI 문서 수정은 하지 않았으며, 승인 D를 받아 원격 게시 단계로 이동했다.

## 6. Stage 5 draft PR·self-merge 준비 (완료)

1. 원본 저장소의 `task_m100_3744` branch로 push하고 `devel` 대상 draft PR #3933을 생성했다.
2. `mydocs/pr/archives/pr_3933_review.md`에 역할·규모·검증·visual 판정·merge 조건을 기록했다.
3. 최초 review 뒤 제품 보정이 추가되어 `pr_3933_review_impl.md`를 만들고 구현·기각안·검증을
   독립 기록한다.
4. 최신 PR head의 GitHub Actions와 별도 code review를 확인하고 승인 E에서 ready·merge 여부를
   결정한다.

## 7. Stage 6 review 보정 (완료)

1. 보정 전 새 회귀 3개 축 중 복합 번호 뒤 정상 목록 회복과 dotted-leader TOC가 실패하는
   9 passed / 2 failed red를 확인했다. ParaShape 경계는 `indent=-1280` positive와 `-1281`,
   nonzero margin, nested level negative로 확대했다.
2. 만료 상태에 번호만으로 복귀시키는 최초 안은 sample10 세 변형에서 node 8→1,145,
   `호` 4→1,141로 회귀해 기각했다. 경계의 바로 다음 문단·같은 section 조건을 추가한 최종 안은
   기준/보정 recursive 673개(670 parse)의 파일별 kind 결과가 동일했다.
3. dotted leader는 `.`, `·`, `‥`, `…`를 점수화해 3점 이상이고 뒤에 쪽번호가 있을 때만 TOC
   tail로 본다. direct `목` indent 하한은 HWPUNIT 단위의 명명 상수로 문서화했다.
4. latest `upstream/devel` `301d0fe5f` 결합 후 structure 8, #3744 11, #3693 3, #3695 13,
   CLI 4 tests를 통과했다. 결합 직전 보정 후보에서는 전체 release tests, fmt, diff check,
   all-targets clippy를 통과했고 최신 결합 head는 GitHub CI를 최종 merge gate로 둔다.
5. 구현·task 문서를 보정 커밋으로, `pr_3933_review.md` 갱신과 새 review_impl을 별도 문서 커밋으로
   분리한다. push 뒤 GitHub comment/review/ready/merge 없이 승인 E에서 중지한다.

## 8. 금지 사항

- 파일명·section/paragraph 좌표·para shape ID를 제품 판정에 하드코딩하지 않는다.
- sample 하나의 44건을 맞추기 위한 임의 threshold를 채택하지 않는다.
- #3695 auto selector와 explicit outline 동작을 함께 리팩터링하지 않는다.
- parser/render/serializer 변경과 시각 fixture 추가를 이 이슈에 섞지 않는다.
- 승인 B 전 제품 소스를 구현하지 않고, 승인 D 전 push·PR을 수행하지 않는다.
- 승인 E 전 GitHub review/comment, ready 전환, merge와 이슈 상태 변경을 수행하지 않는다.
