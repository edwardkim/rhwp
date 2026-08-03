---
kind: guide
status: active
canonical: mydocs/tech/agent_roadmap/track_a_envelope.md
last_verified: 2026-08-04
---

# 트랙 A — 봉투 무결 (R1~R10)

**질문**: 에이전트가 받는 모든 `--json` 봉투를 기계가 믿어도 되는가.

L1 은 이 저장소의 가장 낮은 층이자 가장 자주 무너지는 층이다. 지금까지의 실측이
보여준 실패 모드는 셋이다 — ① 실패가 성공처럼 보인다(침묵 무시·stdout 오염),
② 자기서술이 실물과 다르다(선언한 필드가 봉투에 없음), ③ 표지가 빠진다(문서
유래 값이 표식 없이 실림). R1~R6 이 이 셋을 각각 봉합한 실적이고, R7~R10 이
남은 표면이다.

## R1 종료 코드 계약 `[완료]`

- **한 줄** — 전 명령이 0(성공)/1(런타임 실패)/2(사용법 오류)/3(단언 실패)을 지킨다.
- **지금** — #2707 → PR #2711 로 export 계열, #3169→#3171 로 진단 계열,
  #3172 로 `dump`/`diag`, #3221 로 마감. 미지 명령·명령 누락은 stderr + exit 2
  (종전에는 stdout + 0 이라 오타가 스크립트에서 성공으로 보였다).
- **설계(보장)** — `tests/cli_exit_codes_*.rs` 계열이 명령군별로 0/1/2 를 고정.
  `run` 의 의도된 예외(실패도 저널 봉투)는 R4 에서 자기서술에 명문화됐다.
- **DoD** — 달성: 임의 명령의 실패를 종료 코드만으로 분류할 수 있다.
- **의존** — 없음(전 트랙의 바닥).

## R2 출처 표지(S1) 규약 + 출처 지도 `[완료]`

- **한 줄** — 모든 봉투가 `untrustedContent`/`untrustedFields` 로 "이 값은 문서에서
  왔다"를 기계가 읽게 선언한다.
- **지금** — #3787 S1 구현, #3804 출처 지도(`export-provenance-map`) + 드리프트
  가드. `capabilities.jsonContract.provenance.policy` 가 계약 원문이다:
  *"표지는 항상 실린다 — 문서를 열지 않는 명령의 봉투도 untrustedContent:false 를
  명시한다"*.
- **설계(보장)** — 표지는 `provenance::marked(envelope, command)` 한 곳에서만
  찍힌다. 지도(`src/provenance.rs MAP`)가 선언의 단일 출처이고, 실재 경로 필터
  (`present_fields`)가 "봉투에 없는 경로를 표지가 주장"하는 거짓을 구조로 막는다.
- **DoD** — 달성 + R5·R6 의 가드가 재발을 막는다.
- **의존** — R1.

## R3 info·structure 정합 (T1·T3) `[완료]`

- **한 줄** — `info` 가 건너뛴 것을 숨기지 않고, `structure` 봉투에서 snake_case
  혼입을 제거했다.
- **지금** — #3880 T1·T3 → #3882 (통합 머지 #3895 반영).
- **설계(보장)** — 봉투 필드명은 camelCase 단일 규약. "건너뜀"은 데이터로 보고된다.
- **DoD** — 달성.
- **의존** — R1.

## R4 진단 명령 침묵 무시 제거 `[완료]`

- **한 줄** — `bench`·`dump`·`diag` 가 모르는 옵션을 조용히 무시하지 않는다.
- **지금** — #3884 G1·G2·G3 → PR #3897. 수정 전 실측: `bench <문서> --json` 은
  `--json` 을 **파일 이름으로 접어** exit 1 + stdout 518 B(배너+반쪽 표)를 냈고,
  `dump <문서> --bogus-flag` 는 exit 0 + 18,643 B, `diag` 는 `args[1..]` 를 아예
  보지 않았다. 수정 후: 전부 exit 2 + stdout 0 B + stderr 안내. 전건 실패 시
  bench 는 배너·표·TSV 를 내지 않는다(redact 의 "탐지 0건 = 산출물 미생성" 원칙).
- **설계(보장)** — `tests/diagnostics_flag_contract.rs` 11건이 거부 3면(exit 2·
  stdout 0 B·stderr 에 문제 플래그 명시)을 고정. red 검증에서 devel 원복 시
  9건 실패/회귀 가드 2건 통과 — 그물 모양 그대로다.
- **DoD** — 달성. `run` 의 stdout 예외는 `jsonContract.failure` 에 현행 실물
  기준으로 명문화(입력 오류 1+`error` / 계획 무효 2+`invalid[]` / 단언 3+`verify`).
- **의존** — R1.

## R5 표지 누락 봉투 5건 봉합 `[완료]`

- **한 줄** — S1 선언과 실물의 괴리 5건(`edit redact`·`edit sanitize`·
  `edit insert-image`·`export-ir-schema`·`export-capabilities-schema`)을 닫았다.
- **지금** — #3885 → PR #3903. 가장 민감했던 것은 redact: `findings[].raw` 에
  **마스킹 전 개인정보 원문**이 표지 없이 실렸다. 이슈의 미확정 2건은 실측으로
  갈렸다 — `run` 은 전 경로가 이미 marked(반박), insert-image 는 무표지(확정).
- **설계(보장)** — 방출부 5곳 `marked` + 지도에 `findings[].raw`·`findings[].masked`·
  `removed[].before` 선언. **선언이 하중을 받는 부분이다** — 선언 없이 감싸기만
  하면 raw 를 실은 봉투가 `untrustedContent:false` 로 거짓말한다(표지 부재보다
  나쁘다). `--no-raw` 면 raw 가 봉투에 없으므로 표지에서도 빠진다(실재 경로 필터,
  양방향 테스트).
- **DoD** — 달성. 재발 방지는 R6·트랙 B 의 가드 4중 확장.
- **의존** — R2.

## R6 recordFields ↔ 실물 전수 대조 `[완료]`

- **한 줄** — 자기서술이 광고하는 필드가 실물 봉투에 실제로 나타나는지를 스윕
  전수로 대조한다.
- **지금** — PR #3903 의 `declared_record_fields_actually_appear_in_envelopes`.
  파이썬 바인딩의 대표 4개 검사(`test_declared_fields_actually_appear`)를 본체
  전수로 승격했고, **도입 즉시 7개 명령 28건 괴리를 발굴**했다. 전부 허용목록
  없이 "그 필드를 실제로 내는 호출" 레시피 10종으로 회수: `digest --sections`,
  `inspect hidden-text`/`injection`, `run` 무효 계획, `table-to-csv -o`,
  `csv-to-table -o --verify`, `fill-fields --verify`, `replace-text`,
  `batch` 오류 레코드, `batch fill`.
- **설계(보장)** — 검사 기준은 최상위 키 합집합 ⊇ 선언(중첩 경로는 제외 — 바인딩
  파리티와 같은 기준). 조건부 필드는 사유 필수의 `CONDITIONAL_RECORD_FIELDS` 로만
  허용 — 현재 항목 0. **28건의 뿌리는 전부 "하위 명령·조건부 경로를 스윕이 안
  돌린 것"이었다** — 이 발견이 R7 의 직접 근거다.
- **DoD** — 달성(허용목록 0 유지가 지속 조건).
- **의존** — R2·R5, 트랙 B 의 스윕 기구.

## R7 edit·inspect 하위 명령 자기서술 등재 `[이슈]`

- **한 줄** — `capabilities` 만 읽는 에이전트가 `edit` 6종·`inspect` 3종 하위
  명령의 존재를 알게 한다.
- **지금** — #3884 G4. 실측: `capabilities` 의 commands 에는 `edit`·`inspect`
  부모만 있고, 실제 하위는 `edit <fill-fields|replace-text|set-cell|insert-image|
  redact|sanitize>`·`inspect <hidden-text|injection|unicode>` 9종. 실패 자체는
  규약을 지킨다(exit 2 + stdout 0 B) — 깨짐이 아니라 **발견 가능성의 공백**이라
  `capabilities --search`(R31)가 이 위에서 절반만 동작한다. R6 의 28건 중 15건이
  같은 뿌리(하위 명령 사각)였다는 것이 정량 근거다.
- **설계** — 저장소에 이미 선례가 있다: `capabilities` 본문의
  `batch.subcommands` 배열. 같은 모양으로 `edit`·`inspect` 항목에
  `subcommands: [...]` 를 더하고, 하위별 `recordFields` 분화가 필요한지가 설계
  질문이다(현재 `edit` 는 부모 하나에 전 하위의 필드 합집합을 선언 — R6 가드가
  합집합 기준이라 동작은 하지만, 소비자는 "어느 하위가 어느 필드를 내는지"를
  모른다). 1차: subcommands 배열 + 요약 한 줄. 2차(별도 판단): 하위별 선언 분화.
- **착수 게이트** — 없음(추가 근거 불요, 선례 존재). 메인테이너의 모양 승인만.
- **DoD** — `capabilities --search "redact"` 가 edit 하위를 찾는다. 드리프트
  가드: 디스패치의 실제 하위 목록 ↔ 선언 대조 테스트 1본.
- **의존** — R6(정량 근거), R31(효과 수혜자).

## R8 진단 명령 부류 판정 `[이슈]`

- **한 줄** — `bench`·`dump`·`diag` 류를 (a) 자기서술에 등재해 전 규약을 지키게
  할지, (b) "자기서술 밖 명령" 부류를 명시 정의할지 정한다.
- **지금** — #3884 G2 의 열린 질문. R4 가 어느 쪽을 택해도 필요한 최소 불변식
  ("모르는 옵션은 조용히 무시하지 않는다")까지만 세웠다. 현재는 **정의되지 않은
  회색지대**다 — 이 셋은 `capabilities` 에 `json`·`flags` 를 선언하지 않아
  드리프트 가드의 시야 밖이고, "자기서술에서 빠진 것이 검증에서도 빠지는" 구조를
  R4~R6 이전까지 그대로 겪었다.
- **설계** — 판단 자료: (a)안은 표면 일관성이 얻는 것이고 비용은 진단 명령의
  봉투 설계(R9·R53)가 선행된다는 것. (b)안은 비용이 낮지만 "밖" 부류의 규약
  문서(무엇은 지키고 무엇은 면제인지)가 필요하고, 부류 소속을 기계가 읽을 자리
  (`capabilities.outOfContract: [...]` 류)가 필요하다. 어느 쪽이든 **"부류 없음"
  상태보다는 낫다.**
- **착수 게이트** — 메인테이너 판단. 판단 요청은 #3884 에 이미 걸려 있다.
- **DoD** — 판정이 `capabilities` 또는 규약 문서에 기계가 읽게 명문화되고,
  preflight(R12)가 부류 기준으로 검사 범위를 정한다.
- **의존** — R4.

## R9 `dump --json` 실봉투 `[가설]`

- **한 줄** — 문서 내부 구조 덤프(`dump`)에 기계 계약 봉투를 단다.
- **지금** — R4 이후 `dump --json` 은 정직한 거부(exit 2)다 — 침묵 무시(사람용
  텍스트 + exit 0)보다 낫지만, 구조 질의 자체는 에이전트 수요가 실재한다
  (`export-structure` 는 문서 논리 구조, `dump` 는 IR·레코드 관점이라 용도가
  다르다).
- **설계** — 봉투 스케치: `{schemaVersion, source, borderFills[], controls[],
  paragraphs[]?}` — 사람용 출력의 정보를 그대로 옮기되, **양이 문제다**(사람용
  18 KB 실측). `--section/--para` 필터를 1급 계약으로 유지하고, 상한·`truncated`
  규약(digest 선례)을 재사용한다.
- **착수 게이트** — ① R8 이 (a)안으로 판정될 것. ② 실수요 근거 1건 이상
  (에이전트가 export-structure 로 못 푸는 질의 실측 — 예: 특정 컨트롤의 IR
  좌표가 필요한 디버깅 시나리오).
- **DoD** — `dump --json` 이 계약 봉투를 내고 capabilities 에 선언되며, R6 전수
  대조와 출처 스윕에 편입된다.
- **의존** — R8.

## R10 L1 완결 게이트 `[가설]`

- **한 줄** — "봉투 규약을 통과하지 못한 표면은 저장소에 못 들어온다"를 CI
  불변식으로 합성한다.
- **지금** — 부품은 이미 있다: 종료 코드 계열 테스트(R1), 출처 스윕·표지 존재·
  면제 존재검사·recordFields 전수(R2·R5·R6), 플래그 거부 계약(R4), preflight
  (R11~12). 없는 것은 **"새 명령 추가"를 감지해 전 검사를 자동 적용하는 합성
  층**이다 — 지금은 새 명령이 스윕 레시피·선언·지도에 스스로 등록해야 하고,
  등록을 빼먹으면 일부 가드(커버리지 대조)가 잡지만 전부는 아니다.
- **설계** — ① `capabilities.commands[]` 를 기준 목록으로 삼아, 목록의 모든
  `json:true` 명령이 {스윕 레시피 또는 면제, 지도 항목, recordFields, 실패 경로
  검사} 4종에 전부 등재됐는지 교차 대조하는 단일 메타 가드. ② 등재 누락 시
  실패 메시지가 "무엇을 어디에 더하라"를 그대로 적는다(오늘의 가드들이 이미 이
  문체를 쓴다). ③ preflight(R12)와 중복되지 않게 역할을 가른다 — preflight 는
  빌드 전 빠른 검사, 메타 가드는 CI 의 최종 판정.
- **착수 게이트** — R7·R8 판정(하위 명령·부류가 기준 목록의 모양을 정한다),
  그리고 4종 등재처의 명세가 안정될 것(R12 머지 포함).
- **DoD** — 신규 명령을 등재 없이 추가한 가짜 브랜치가 CI 에서 메타 가드 1개로
  잡히고, 메시지만 따라 하면 통과하는 것을 red→green 으로 실증.
- **의존** — R4·R6·R7·R8·R12.
