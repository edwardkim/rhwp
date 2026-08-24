# 작업 기록 — task_m100_3790 Stage 5C

- **이슈**: [#3790](https://github.com/edwardkim/rhwp/issues/3790)
- **브랜치**: `task_m100_3790_stage5c_codeql_sparse_checkout`
- **기준**: `upstream/devel` `385e93b2c317d1f50d874fd655e88cf4b2a1ba07`
- **상태**: 구현 및 focused 검증 진행 중

## 배경

PR #5455는 PR Rust CodeQL 데이터베이스를 `src/**`, `crates/**`,
`rhwp-desk/src/**`, `build.rs`로 한정했다. 그러나 2026-08-24 PR #6017의
`Analyze (rust)` job은 shallow fetch만 3분 33초를 사용해 저장소 전체를 받은 뒤
861개 Rust 파일을 추출했다. checkout의 실제 파일 전개는 약 10초였으므로 병목은
worktree 전개가 아니라 분석에 필요 없는 blob 전송이다.

같은 job의 Rust 분석은 추출 및 데이터베이스 생성에 5분 35초, 기본 쿼리 평가에
7분 37초를 사용했다. 따라서 Cargo cache, `build-mode`, 또는 query suite를 바꾸지
않고 PR checkout 입력부터 분석 범위와 맞춘다.

## 설계

- 선택된 JavaScript/TypeScript와 Python lane은 기존 full checkout을 유지한다.
- 선택된 Rust PR lane만 cone sparse checkout으로 루트 파일, `.cargo`,
  `.github/codeql`, `crates`, `rhwp-desk`, `src`를 받는다.
- cone sparse checkout의 루트 파일에는 `Cargo.toml`, `Cargo.lock`, `build.rs`,
  `rust-toolchain.toml`이 남아 Rust workspace와 CodeQL 설정을 보존한다.
- `devel` push, schedule, workflow_dispatch의 Rust lane은 PR config를 쓰지 않으므로
  별도 full checkout을 유지한다.
- partial clone `filter`는 sparse checkout 입력을 덮어쓰므로 사용하지 않는다.

## 수용 기준

1. PR Rust lane에만 정확한 sparse checkout 경로가 선언된다.
2. non-PR Rust lane은 sparse checkout 없이 전체 분석을 유지한다.
3. CodeQL workflow 계약 테스트와 YAML/actionlint 검증이 통과한다.
4. PR CI에서 Rust checkout, 추출 성공·오류 파일 수, SARIF/CodeQL 결과, 전체 Rust job 시간을
   #6017 기준선과 비교한다. extraction 오류 증가 또는 결과 범위 이상이 보이면 merge하지 않는다.

## 비범위

- Rust `build-mode: none` 재변경, 기본 CodeQL query suite 축소, larger runner 도입
- trusted classifier의 fail-closed 언어 선택 정책 변경
- nextest archive B/C 구조 변경. 이는 독립 측정 PR에서 다룬다.
