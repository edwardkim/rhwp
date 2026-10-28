# Stage 3 보고 — task_m100_3744 clause confidence 구현

- **일자**: 2026-08-04
- **이슈**: #3744
- **기준**: `upstream/devel` `0889974a01db3585df8ad2c1f13203e3cb9f51f8`
- **브랜치**: `codex/issue-3744-clause-context-confidence`
- **승인 범위**: 선택 정책 구현, focused green, 단계 보고와 로컬 커밋까지
- **단계 결론**: Stage 3 완료, Stage 4 corpus·release 검증 승인 대기

## 1. 구현 결과

`src/document_core/queries/structure.rs`의 explicit clause weak-marker gate에 Stage 2에서 선택한 세
정책만 반영했다. 공개 JSON schema, `StructureNode`/`StructureDoc`, CLI option, auto selector와
explicit outline 경로는 변경하지 않았다.

### 1.1 복합 번호와 anchor 만료

- `호` 후보 marker가 `.`으로 끝나고 marker 직후가 숫자이면 `N.N` 복합 번호로 거부한다.
- nearest `조|항`의 `(section, paragraph)`를 비공개 anchor 식별자로 삼아 해당 anchor의 후속 weak
  `호`를 거부한다.
- 새 `조|항`은 새 식별자로 시작하므로 이전 anchor의 만료가 전파되지 않는다.
- `1)1920년…`처럼 `)` marker 뒤 본문이 숫자로 시작하는 경우는 복합 번호로 보지 않는다.
- 거부 문단은 기존 body/preamble 경로에 보존한다.

Oracle fixture의 `4.1` 경계 뒤 SQL 세 좌표는 더 이상 `호` node가 아니며, synthetic 새 `항` 뒤의
정상 `1.`은 다시 `호`로 채택된다. Stage 2에서 문서화한 경계 전 네 `호` 잔존은 이번 좁은 정책의
의도한 trade-off다.

### 1.2 날짜 negative

- 선두 연도 4자리와 월·일 1~2자리를 점 구분자로 파싱한다.
- 월 1~12, 일 1~31인 `YYYY. M. D.`는 suffix와 무관하게 `호`에서 거부한다.
- `2022.1.1.5`처럼 마지막 점 뒤 숫자가 이어지는 버전 번호는 날짜로 보지 않는다.
- 날짜 거부는 anchor를 만료하지 않아 뒤의 정상 `1.`이 채택된다.

### 1.3 direct `목` positive

- 기존 열린 `호` 아래 `목`은 그대로 허용한다.
- 열린 `호`가 없고 `장|절` 조상이 있으면 탭+ASCII 쪽번호 tail이 없고 ParaShape가
  `margin_left=0`, `indent>=-1280`, `para_level=0`일 때만 허용한다.
- 편람의 대표 4좌표와 시장구조조사의 `가. 표준산업분류`가 green이고, synthetic
  `가. 개요\t9`는 body에 남는다.

시장구조조사 fixture의 heading 원문에는 끝 공백이 보존된다. 공개 heading을 정규화하지 않고 테스트의
제목 내용 비교만 `trim()` 기준으로 바꿔 이번 이슈의 API 비변경 원칙을 지켰다.

## 2. red → green

Stage 2의 영구 테스트는 **2 passed / 6 failed**에서 **8 passed / 0 failed**로 전환됐다.

| 축 | green 근거 |
| --- | --- |
| stale anchor | Oracle SQL 2303·2312·2313 node 부재와 body 보존 |
| anchor 범위 | `2.1` 뒤 현재 anchor 만료, 새 `②` 뒤 `1.` 회복 |
| 날짜 | 날짜 node 부재·body 보존·정상 후속 `호` 채택 |
| direct `목` | synthetic, 편람 대표 4좌표, 시장조사 대표 좌표 채택 |
| TOC | 탭+쪽번호 synthetic가 node가 아니며 body 보존 |
| 정상 `호` | 협정서 제1조 아래 `1.`·`2.`·`3.` 보존 |

달력 범위·버전 번호와 dotted/parenthesized 복합 번호 차이는 structure 단위 테스트 두 개로 추가
고정했다.

## 3. focused 검증

| 명령 | 결과 |
| --- | --- |
| `cargo test --lib document_core::queries::structure` | 8 passed |
| `cargo test --test issue_3744_structure_clause_confidence` | 8 passed |
| `cargo test --test issue_3693_structure_clause_context` | 3 passed |
| `cargo test --test issue_3695_structure_auto_policy` | 13 passed |
| `cargo test --test cli_json_contract export_structure_` | 4 passed |
| `cargo clippy --test issue_3744_structure_clause_confidence -- -D warnings` | 통과 |
| `cargo fmt --check` | 통과 |
| `git diff --check` | 통과 |

## 4. 남은 Stage 4 경계

이번 단계에서는 top-level 351개와 recursive 668개 sample의 제품 결과를 다시 생성하지 않았고
`--profile release-test --tests`와 all-targets clippy도 실행하지 않았다. 다음 승인 뒤 Stage 4에서
다음을 수행한다.

Stage 3 시작 시 `upstream/devel`은 승인 기준 `0889974a`였으나 최종 점검 시 공유 저장소의 추적 ref가
`976ed264`로 200커밋 전진해 있었다. 승인 범위를 넓혀 rebase하지 않았으며, Stage 4 첫 작업으로 최신
devel에 동기화한 뒤 focused green을 다시 확인한다.

1. 최신 devel 동기화와 focused 재검증을 마친다.
2. 기준/보정 결과의 문서별 mode, node_count, kind 증감을 재측정한다.
3. Stage 2 예측치와 실제 구현 결과를 대조하고 의도/잔여 trade-off를 분류한다.
4. release-test, fmt, diff check, all-targets clippy를 통과한다.
5. 최종 보고서를 로컬 커밋하고 원격 push·PR 승인 D에서 다시 중지한다.

GitHub comment, 이슈 상태, 원격 branch/PR은 변경하지 않았다.
