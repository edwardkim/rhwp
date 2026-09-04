# task_m100_3261 처리결과 보고서 — export-structure `--json`·batch 확장

- **이슈**: [#3261](https://github.com/edwardkim/rhwp/issues/3261) (#2659 Stage 2-③)
- **브랜치**: `pr/task3261-structure-json` (PR #3258 위에 적층)
- **범위**: `src/main.rs`, `tests/cli_json_contract.rs`, `mydocs/manual/cli_commands.md`,
  `mydocs/manual/cli_json_pipeline_guide.md`
- **분류**: 기능 추가 (조회 기계화 — 구조 축)

## 1. 문제

`export-structure`(#1523)는 개요/조문 계층을 JSON 트리로 추출하지만, ① 출력에
`schemaVersion`·`source` 가 없어 #3237 조회 계약 밖이고, ② `batch`(#3238)가 이 축을
지원하지 않아 아카이브 구조 스윕이 파일당 프로세스 1회였다.

## 2. 분석

기존 산출물이 이미 serde 직렬화 구조(`StructureDoc`)라, 필요한 것은 **봉투와 배선**뿐이다.

- 기본 stdout(무봉투 pretty JSON)과 `-o` 저장은 문서화된 기존 소비자 계약 — 무변경 원칙.
- `--json` 은 형제(#3237)와 같은 규약: 계약 봉투를 씌운 **한 줄** JSON.
- `mode`/`nodeCount` 를 봉투 톱레벨로 올려 스윕 선별(`jq select(.mode=="clause")`)이
  트리 파싱 없이 성립하게 했다.
- batch 는 #3238 골격(병렬·순서 보존·panic 격리·부분 실패 exit 1)에
  `BatchMode::Structure(StructureMode)` 축만 추가 — 동시성 코드는 한 줄도 변하지 않는다.
- `--mode` 는 구조 축 전용 플래그로, 타 서브커맨드와 병용 시 사용법 오류(2)다.

## 3. 변경

- `structure_json_value()` — 단건 `--json` 과 batch 레코드가 공유하는 봉투 빌더
- `export_structure()` 에 `--json` 분기 (기본 경로 무변경)
- `run_batch()` 서브커맨드 3축화 + `--mode` 파싱, `batch_structure_record_inner()`
- help·`cli_commands.md`·파이프라인 가이드(시나리오 4: 구조 스윕) 갱신

## 4. 검증

- **red→green**: 계약 테스트 5종 신설 — 봉투 스키마·한 줄 보장, **기본 출력 무변경 가드**
  (기본 stdout 에 `schemaVersion` 없음 단언), batch 봉투 공유, `--mode` 오류 2종
- 기존 계약 테스트 10·`cli_exit_codes.rs` 10 무회귀 (합계 25 green)
- `cargo clippy --release --bin rhwp -- -D warnings`·`rustfmt --check` clean
- 실측(32코어 Windows, samples 271건): **구조 스윕 2.7s 전건 성공** —
  auto 판별 분포 clause 258건/7,239노드·outline 13건/905노드. 가이드 예시는 전부 실행 검증.

## 5. 남긴 것

- `dump-pages`·`ir-diff` 의 `--json` — 전자는 휴리스틱 부기가 많은 내부 진단이라
  Stage 4 `rhwp dev` 격리 논의와 함께, 후자는 검증 파이프라인 수요 확인 후 별도 이슈로.
- 단건 명령 stdin(`-`) 입력, 인자 파서 공통 헬퍼 — Stage 2 잔여로 유지.
