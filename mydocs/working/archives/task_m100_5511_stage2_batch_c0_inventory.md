# #5511 Stage 2 기능군 배치 C0 inventory — edit command runtime seam

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 조사 HEAD: `af99a2ad6fc869c2ecb31bba1354d4314b2aa50f`
- 통합 기준선: `upstream/devel` `b914bdf4bf1a8f922f03ea6b141f0d9c2b10a98f`
- 작성일: 2026-08-20
- 상태: C0 진입 승인 — 이동 전 계약·최소 runtime API 고정

## 1. 범위 판정

C0의 실제 공통 구현은 `src/main.rs`에 흩어진 482줄이다.

| 책임 | 현재 줄 | 규모·소비자 |
|---|---:|---|
| `run_edit` dispatch | 112 | edit 하위 명령 88개 |
| 출력 형식·직렬화·재파싱 검증 | 138 | root, CSV import, plan, MCP session 등 6개 파일 |
| CAS hash·경로 잠금·동시성 hook·기대 해시 | 147 | edit replace-text와 protocol plan |
| `finish_edit_write` | 85 | 정의를 제외한 edit handler 호출 81곳 |

`run_edit`는 편집 알고리즘이 아니라 하위 명령 이름을 handler에 연결하는 command adapter다.
출력 형식 판정, HWP/HWPX 직렬화, 저장본 재파싱 검증과 공통 저장 봉투는 모든 후속 C1~C6
handler가 공유할 process runtime이다. P1에 임시로 남긴 CAS seam도 C0에서 최종 소유권을
확정한다.

바로 앞의 agent manifest와 schema export는 metadata adapter이므로 제외한다. 바로 뒤부터의
개별 edit handler와 field key parsing, cell·shape helper는 C1~C6 소유이므로 이동하지 않는다.

## 2. 보호 계약 기준선

현재 소스와 일치하는 다음 9개 직접 계약 모듈을 profile별로 실행해 101/101 통과했다.

| 계약 축 | 모듈 | 건수 |
|---|---|---:|
| edit 적용·형식·검증 | `edit_fill_fields_contract`, `edit_replace_text_contract`, `edit_format_preserve_contract`, `edit_verify_contract` | 23 |
| MCP session snapshot | `mcp_session_edit_contract` | 8 |
| plan·edit CAS | `run_plan_cas_contract` | 15 |
| CSV command 저장 | `table_csv_contract` | 14 |
| JSON·provenance 봉투 | `cli_json_contract`, `provenance_contract` | 41 |

`run_plan_cas_contract`의 release-visible 14건과 debug-only 경합 1건을 모두 실행했다. 경합 계약은
동일한 기대 hash를 가진 in-place plan 둘 중 정확히 하나만 commit하는 것을 확인했다.

처음 직접 실행한 오래된 release `provenance_contract` artifact는 현재 CLI보다 앞선 바이너리를
내장해 신규 명령 8개를 알지 못했고 8/10이었다. 현재 소스로 regression suite를 재빌드하자
10/10이 통과했으므로 코드 기준선 결함이 아니라 stale artifact로 판정해 폐기했다.

공통 seam은 이미 dry-run 무쓰기, 입력 형식 보존, HWPX→HWP 경고, adapter 경유, 저장 후
재파싱 검증, session snapshot 비변이, provenance 표지, CAS 단일 commit을 직접 보호한다.
따라서 C0는 신규 characterization 없이 move-only로 진행한다.

## 3. 최소 소유권과 API

```text
src/cli/
├── integrity.rs             # 범용 SHA-256·경로 잠금·동시성 test hook
└── commands/
    └── edit/
        ├── mod.rs           # 88개 하위 명령 dispatch
        └── runtime.rs       # output format·serialize·verify·write·edit CAS report
```

범용 hash·경로 잠금·동시성 hook은 edit와 protocol plan이 함께 쓰므로 어느 command 아래에도
두지 않는다. 반면 기대 hash 불일치 봉투는 provenance command가 `edit`으로 고정된 edit 전용
표면이므로 `edit/runtime.rs`가 소유하고 범용 integrity를 소비한다.

`EditContext` 또는 큰 의존 묶음은 C0에서 만들지 않는다. 현재 88개 handler는 인자 parsing,
문서 load, 도메인 mutation을 각자 소유하고 공통 runtime에는 저장할 문서·원본 bytes·경로·
출력 옵션·변경 문단만 명시적으로 넘긴다. 이 단계에서 광범위한 context를 만들면 C1~C6의
서로 다른 좌표·자산·story 의존까지 한 구조체에 선반영해 다시 god object가 된다. 기존
`finish_edit_write`의 명시적 인자 계약을 보존하고, handler를 실제 책임군으로 옮기며 반복되는
입력 구조가 확인될 때 기능군별 argument/context를 결정한다. `DocumentService`와 typed error,
전역 인증 제거는 계획대로 Stage 3 범위다.

root wrapper는 두지 않는다. 최상위 dispatch, root의 아직 이동하지 않은 handler, CSV import,
protocol plan, MCP session은 각각 새 소유 경로를 직접 참조한다. 새 파일은 1,200줄 이하를
유지하고 C0 경로의 CC 25 초과 경고는 0건이어야 한다. 이동 전 계측에서도 C0 함수 자체의
CC 25 초과는 0건이다.

## 4. 구현·커밋 순서

1. 이 inventory를 독립 커밋으로 고정한다.
2. 범용 CAS integrity seam을 `src/cli/integrity.rs`로 이동하고 edit·plan 소비 경로를 바꾼다.
3. output format·serialize·verify·공통 write와 edit 전용 기대 hash 판정을
   `src/cli/commands/edit/runtime.rs`로 이동한다.
4. 88개 하위 명령 dispatch를 `src/cli/commands/edit/mod.rs`로 이동하고 root가 직접 호출한다.
5. 101개 직접 계약과 전체·정적·정책 관문을 실행하고 완료 보고서를 커밋한다.

각 절편은 공개 schema, hash·serializer 알고리즘, stdout/stderr, exit code와 파일 부작용을
바꾸지 않는다. parser·serializer·domain service 변경이 필요해지면 C0를 중단한다.

## 5. 원격 위험과 중단 기준

조사 시점 `origin/devel`과 `upstream/devel`은 모두 `b914bdf4b`이고 현재 HEAD의 조상이다. 열린
devel 대상 PR #5647, #5689, #5691, #5693, #5695, #5707, #5709, #5710, #5718, #5719의
최신 변경 경로에는 `src/main.rs`, `src/mcp_serve.rs`, `src/cli/mod.rs`, `src/cli/commands/`, 계획한
`src/cli/integrity.rs`, protocol plan, C0 직접 계약과 #5511 C0 문서가 없다. 이 판정은 시점
증거이므로 C0 완료와 push 전에 다시 조회한다.

다음 경우에는 같은 승인 배치 안에서도 이동을 멈추고 메인테이너에게 보고한다.

- 101개 기준 계약의 stdout/stderr·exit·파일 부작용·보안 방어가 달라지는 경우
- edit와 plan 사이에 양방향 모듈 의존 또는 hash·잠금 구현 복제가 생기는 경우
- 공통 runtime이 개별 C1~C6 도메인 mutation을 소유해야만 이동할 수 있는 경우
- 새 파일 1,200줄 또는 CC 25 상한을 지킬 수 없는 경우
- 최신 devel·열린 PR이 같은 함수, 테스트, 모듈 경계를 변경한 경우
