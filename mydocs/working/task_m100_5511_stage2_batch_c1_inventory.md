# #5511 Stage 2 기능군 배치 C1 inventory — field·text·privacy command

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 조사 HEAD: `3562e84e4e6f781b0c537865645227c27427a123`
- 통합 기준선: `upstream/devel` `cfe2c351e834d7579a521c8ed7f6839674cc9ad1`
- 작성일: 2026-08-20
- 상태: C1 진입 승인 — 이동 전 계약·소유권 고정

## 1. 범위 판정

기존 계획이 계측한 C1 구현은 `edit_fill_fields`부터 `edit_sanitize`까지 1,380줄·11함수다.
현재 root에는 이 블록과 떨어진 `parse_field_key` 20줄이 더 있다. 이 parser는 이름과 0 기준
occurrence를 나누는 field target 규약이며 fill command, batch fill, protocol plan, MCP session이
공유한다. C1 뒤에도 root에 두면 field 책임의 핵심 좌표 규약만 소유자 없이 남으므로 C1 실범위를
1,400줄·12함수로 보정한다.

| 책임 | 현재 규모 | 주요 소비자 |
|---|---:|---|
| field occurrence parser·fill command·fill core | 360줄 + parser 20줄 | 단건 edit, batch fill, plan, MCP session |
| replace-text command | 300줄 | 단건 edit, plan CAS 계약 |
| redact·sanitize와 privacy helper | 740줄 | 단건 edit, atomic output |

`edit_insert_text`와 `edit_delete_text`는 이 1,380줄 책임 블록에 속하지 않는다. 두 명령은 문단
좌표·분할·병합과 같은 구조 편집 seam을 공유하므로 C4 document-structure 기능군에서 함께
판정한다. C1은 field value, 문서 전역 replace, 개인정보 마스킹과 메타데이터 제거에 한정한다.

## 2. 보호 계약 기준선

현재 소스에서 다음 9개 직접 계약 모듈을 실행해 113/113 통과했다.

| 계약 축 | 모듈 | 건수 |
|---|---|---:|
| field 단건·occurrence | `edit_fill_fields_contract`, `edit_field_occurrence_contract` | 11 |
| replace text | `edit_replace_text_contract` | 5 |
| redact·sanitize | `redact_sanitize_contract` | 15 |
| batch·MCP·plan 재사용 | `batch_fill_contract`, `mcp_session_edit_contract`, `run_plan_contract` | 41 |
| JSON·provenance 봉투 | `cli_json_contract`, `provenance_contract` | 41 |

이 계약들은 field occurrence와 모호성·confusable 보고, dry-run 무쓰기, 입력 형식 보존, 저장 후
verify, replace 0건 무산출, CAS, redact 목적지 명시·원본 보호·`--no-raw`, HWP/HWPX metadata와
preview 정리, batch 행 격리, MCP session·plan 재사용, provenance 표지를 보호한다. 공개 규약이나
편집 알고리즘을 바꾸지 않는 물리 이동이므로 신규 characterization을 선행하지 않는다.

## 3. 복잡도 중단 조건과 처리

이동 전 cognitive-complexity 계측에서 다음 두 함수가 상한 25를 넘었다.

| 함수 | CC | 분해 기준 |
|---|---:|---|
| `edit_replace_text` | 29 | option parsing·검증과 CAS·치환·저장 실행 분리 |
| `edit_redact` | 33 | option/kind/mask/destination parsing과 탐지·치환·출력 실행 분리 |

두 함수를 그대로 다른 파일에 숨기지 않는다. parser는 기존 진단 순서와 exit code를 보존하는
private argument 구조로 분해하고 handler는 실행 수명주기만 소유한다. C1 새 경로의 CC 25 초과
경고가 0건인지 확인한다. parser·serializer·PII 탐지·hash 알고리즘 변경이 필요해지면 중단한다.

## 4. 목표 소유권

```text
src/cli/commands/edit/
├── mod.rs       # edit 하위 명령 dispatch
├── runtime.rs   # C0 serialize·verify·write seam
├── fields.rs    # occurrence parser·fill command·공유 fill core
├── text.rs      # replace-text command
└── privacy.rs   # redact·sanitize command와 metadata helper
```

`fields.rs`는 `parse_field_key`와 `fill_fields_core`를 `pub(crate)` 최소 API로 노출해 batch fill,
protocol plan과 MCP session이 같은 규약을 직접 재사용하게 한다. 세 command handler는 edit parent에만
노출한다. `text.rs`는 C0의 범용 integrity와 edit runtime을 소비하고, `privacy.rs`는 원본 덮어쓰기
보호를 위해 기존 atomic writer를 그대로 사용한다.

root wrapper와 helper 복제는 두지 않는다. 각 새 파일은 1,200줄 이하를 유지한다. Stage 3의
`DocumentService`, typed error와 전역 인증 제거를 선행하지 않으며, C0에서 판정한 대로 기능군 전체를
한 `EditContext`로 묶지 않는다.

## 5. 구현·커밋 순서

1. 이 inventory를 독립 커밋으로 고정한다.
2. occurrence parser·fill command·공유 fill core를 `fields.rs`로 이동하고 소비 경로를 바꾼다.
3. replace-text를 `text.rs`로 이동하며 option parser와 실행을 분리한다.
4. redact·sanitize를 `privacy.rs`로 이동하며 redact parser와 실행을 분리한다.
5. 113개 직접 계약과 전체·정적·정책 관문을 실행하고 완료 보고서를 커밋한다.

각 절편은 stdout/stderr, JSON, exit code, output 이름·형식, 원본 보호와 파일 부작용을 보존한다.
focused 계약과 format·diff 검사를 절편별로 실행한다.

## 6. 원격 위험과 중단 기준

조사 시점 `origin/devel`과 `upstream/devel`은 모두 `cfe2c351e`이고 현재 HEAD의 조상이다. 열린
devel 대상 PR #5647, #5689, #5691, #5695, #5707, #5709, #5710, #5718, #5719의 최신 변경
경로에는 `src/main.rs`, `src/mcp_serve.rs`, edit command·protocol, C1 직접 계약과 #5511 C1 문서가
없다. 이 판정은 시점 증거이므로 C1 완료와 push 전에 다시 조회한다.

다음 경우에는 같은 승인 배치 안에서도 이동을 멈추고 메인테이너에게 보고한다.

- 113개 기준 계약의 stdout/stderr·exit·파일 부작용·privacy 방어가 달라지는 경우
- field parser나 fill core가 edit·protocol·MCP 사이 양방향 의존을 만드는 경우
- 새 파일 1,200줄 또는 CC 25 상한을 지킬 수 없는 경우
- 최신 devel·열린 PR이 같은 함수, 테스트, 모듈 경계를 변경한 경우
- move-only 범위를 넘어 공개 schema, serializer, PII 탐지나 저장 알고리즘 변경이 필요한 경우
