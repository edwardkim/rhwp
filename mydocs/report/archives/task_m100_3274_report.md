# task_m100_3274 처리결과 보고서 — `ir-diff --json` + 오류 종료 코드 정정

- **이슈**: [#3274](https://github.com/edwardkim/rhwp/issues/3274) (#2659 Stage 2-⑤)
- **브랜치**: `pr/task-irdiff-json` (**upstream/devel 직분기 — 기존 스택 #3258/#3262/#3264 와 독립, 공유 커밋 없음**)
- **범위**: `src/main.rs`(ir_diff 함수·디스패치 1행·help), `tests/ir_diff_json_contract.rs`(신규),
  `mydocs/manual/cli_commands.md`, `mydocs/manual/ir_diff_command.md`
- **분류**: 기능 추가 + 종료 코드 결함 정정 (조회 기계화 — 검증 축)

## 1. 문제

① `ir-diff` 판정이 사람용 텍스트뿐이라 대량 변환 파이프라인이 `=== 비교 완료: 차이 N 건 ===` 을
문자열 파싱해야 했다 — #2659 Stage 2 계획의 `--json` 대상 중 마지막 남은 축.
② 파일 없음·파싱 실패·인자 부족이 전부 조용히 `return`(**exit 0**) — #2707 계약 위반 잔존.
"비교했고 차이 없음"과 "비교 자체를 못 함"을 스크립트가 구별할 수 없었다.

## 2. 분석 — 설계 결정

- **기존 기계 재사용**: `--summary` 의 카테고리 버킷(`IrDiffEmitter.summary_buckets`)이 이미
  차이의 구조화 집계다. json 모드는 같은 수집 전용 경로를 타고(텍스트 무출력 = stdout 순수성),
  끝에서 봉투 한 줄만 낸다. **새 비교 로직 0줄.**
- **종료 코드**: 차이 발견 = **3** — `convert --verify` 가 이미 문서화한 "IR 차이" 코드와 같은
  의미라 소비자 학습 비용이 없다. `--json` 모드 전용이며, **기본 모드의 정상 비교는 차이가
  있어도 종전대로 0**(기존 소비자·`ir_diff_summary_mode` 보호).
- **오류 정정은 양 모드 공통**: 읽기·파싱 실패 1, 인자 부족 2 — 결함의 수정이므로 모드를
  가리지 않는 게 맞다. `--summary --json` 병용 시 JSON 이 이긴다(순수성 우선).
- **카테고리 맵은 BTreeMap** — 키 정렬이 결정적이라 스냅샷 비교·diff 가 안정적이다.

## 3. 변경

- `fn ir_diff(&[String]) -> i32` 로 전환, 디스패치 `exit_with` 연결(1행)
- `--json` 파싱·봉투 방출·exit 3, 오류 경로 종료 코드 정정, help·사용법 문자열 갱신
- `cli_commands.md`·`ir_diff_command.md` 갱신 (front matter `last_verified` 는 진행 중인
  #3258 이 같은 줄을 갱신하므로 본 PR 에서 건드리지 않음 — 충돌 방지)

## 4. 검증

- **red→green**: 신규 계약 테스트 6종(`tests/ir_diff_json_contract.rs`) — 동일 문서 exit 0·
  `identical:true`, 차이 문서 exit 3·categories 비어있지 않음, 실패 시 stdout 0바이트·exit 1,
  인자 부족 exit 2, **기본 모드 차이=exit 0 무회귀 가드**
- 기존 `ir_diff_summary_mode` 3종·`cli_exit_codes` 10종 무회귀 (합계 19 green, release)
- `cargo clippy --release --bin rhwp -- -D warnings`·`rustfmt`·문서 검사 스크립트 clean
- 실측: 동일 파일 → `{"identical":true,"diffCount":0}` exit 0 /
  이종 파일 → `{"identical":false,"diffCount":1939,"categories":24종}` exit 3

## 5. 리뷰 반영 (적대적 3렌즈 검증 후 2차 커밋)

PR 제출 후 자체 적대적 리뷰에서 확정된 결함을 같은 PR 에서 수정:

- **[major] 구역 수 차이 미집계**: `total_diffs` 선언이 구역 수 비교 블록 뒤에 있어
  그 차이가 `diffCount` 에 반영되지 않았다. 텍스트 모드에선 무해했으나 `--json` 게이트에서는
  구역 하나 덧붙은 변환본이 `identical:true·diffCount:0·exit 0` 으로 통과하는 치명적 누락.
  → 선언을 앞으로 올리고 집계, **봉투 계약 불변식**(`identical` ⇔ `diffCount==0` ⇔
  `categories` 비어있음)을 `assert_envelope_invariants` 로 전 테스트에 고정 +
  단일↔다구역 회귀 테스트 추가.
- **[minor] 플래그 삼킴**: `--max-lines --json` 처럼 값을 빠뜨리면 `--json` 이 값으로
  소비돼 게이트가 조용히 꺼졌다. → 값 자리 토큰이 `-` 로 시작하면 값으로 삼키지 않도록
  방어(-s/-p/--max-lines 는 비음수만 받음), 회귀 테스트 추가.
- **[docs] 종료 코드 표·병용 규칙**: 코드 3 을 `convert`/`export-hwpx` "전용" 으로만
  서술하던 표에 `ir-diff --json` 추가, `--summary`/`--max-lines` 의 `--json` 병용 시
  거동(JSON 우선)과 알 수 없는 옵션 무시 단서를 매뉴얼에 명시.

## 6. 남긴 것

- 알 수 없는 옵션 무시(`_ => i += 1`)의 exit 2 정렬(#3178)은 기존 스크립트 호환성 조사
  필요 — 매뉴얼에 현재 거동을 단서로 공시하고 별도 이슈로 남김.
- `batch ir-diff`(쌍 목록 배치 검증)는 stdin 쌍 구분 형식 설계가 필요해 수요 확인 후.
