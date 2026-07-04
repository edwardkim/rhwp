# CI cache 정책 결정 기록 — #1664

## 목적

이 문서는 #1664 작업의 정책/의사결정 원천 기록이다. 하이퍼-워터폴 작업 문서가 아니라, 이후 #1667
Rust cache 전략 검토와 #1665 병렬화 검토 때 다시 참조할 CI cache 운영 정책을 보존한다.

관련 이슈:

- #1664: Actions cache quota/read-only 상태 해소 및 PR cache 저장 정책 정리
- #1668: CI Build & Test 실행 시간 단계별 단축 tracking/RFC
- #1666: PR용 Rust profile 재검토
- #1667: Rust cache 전략 개선
- #1665: Build & Test job 병렬 분리 설계

## 배경

`CI / Build & Test (pull_request)`가 20분 이상 소요되고, Actions cache 로그에서 다음 경고가 관측됐다.

```text
Cache reservation failed: You have reached your configured budget, your cache is now read only
```

현행 `actions/cache@v5` 단일 step은 PR merge ref에서도 cache save를 시도한다. quota/read-only 상태에서는
새 cache key가 저장되지 않아 PR마다 stale cache restore 뒤 대량 컴파일을 반복할 수 있다.

## 메인테이너 결정 요약

### 1. 회귀 가드 1:1 추적성 보존

CI 단축은 profile, cache, 병렬화 축에서만 추진한다. 다음은 수용하지 않는다.

- 통합 테스트 파일 통합
- 회귀 가드 명명 규칙 변경
- `tests/golden_svg/issue-NNN/` 자산 구조 변경

`Run integration tests`가 PR마다 실행되어 회귀 가드 162개가 빠짐없이 돈다는 전제가 모든 변경의 우선 조건이다.

### 2. PR profile 정책

PR은 장기적으로 `release-test` 중심으로 전환하고, release-grade 검증은 `devel` push / tag workflow로 이동한다.

#1664 적용 범위:

- profile은 변경하지 않는다.
- `cargo build` / `cargo test` command는 유지한다.
- 이 결정은 #1666에서 별도 처리한다.

### 3. Cache save 정책

채택 정책:

```text
PR = restore-only
devel/main = restore + save
```

근거:

- cache read-only 경고는 quota 문제이므로 PR save를 끄는 것이 직접 대응이다.
- PR은 trusted branch가 아니므로 cache poisoning 위험도 있다.
- `devel` / `main`이 충분히 자주 갱신되면 trusted branch save만으로도 hit rate 손실을 제한할 수 있다.

### 4. Job 병렬화 정책

`Build & Test` 2-3개 job 병렬 분리는 #1666 적용 후 효과를 재평가한 뒤 결정한다.

#1664 적용 범위:

- job 구조를 바꾸지 않는다.
- runner-minutes 증가를 만들지 않는다.
- #1665 범위를 침범하지 않는다.

### 5. Third-party Rust cache action 정책

`Swatinem/rust-cache` 검토는 허용하지만 다음 조건을 만족해야 한다.

- SHA-pinned
- 별도 PR에서 도입 검토
- #1664 적용 후 `actions/cache`만으로 quota 안정화 여부를 먼저 측정

#1664 적용 범위:

- `actions/cache` 기반을 유지한다.
- third-party action을 도입하지 않는다.

## #1664 채택 구현 정책

`Build & Test` job의 cargo cache를 restore/save 분리 구조로 전환한다.

restore:

- action: `actions/cache/restore@v5`
- 모든 run에서 실행
- path:
  - `~/.cargo/registry`
  - `~/.cargo/git`
  - `target`
- key: `${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}`
- restore key: `${{ runner.os }}-cargo-`

save:

- action: `actions/cache/save@v5`
- `push` 이벤트에서만 실행
- `refs/heads/devel` 또는 `refs/heads/main`에서만 실행
- exact cache hit이면 save 생략
- `pull_request`, tag, `workflow_dispatch`에서는 실행하지 않음

## Non-goals

#1664에서는 다음을 하지 않는다.

- `Cargo.toml` profile 변경
- `cargo build` / `cargo test` command 변경
- `Build & Test` job 분리
- `Swatinem/rust-cache` 도입
- scheduled warm-cache workflow 신설
- cache cleanup 자동화 workflow 신설
- 회귀 가드 파일/이름/자산 구조 변경

## 문서 / 코드 PR 분리 원칙

메인테이너 요청에 따라 PR 단위는 다음처럼 분리한다.

- `mydocs/`: 정책, 의사결정, measurement 기록
- `.github/workflows/ci.yml`: 실제 CI 변경
- `Cargo.toml` profile 변경: 필요 시 단독 PR

이 문서는 문서 PR #1701 쪽의 정책/의사결정 원천 기록이다. 실제 workflow 변경은 후속 코드 PR #1702에서만
다루며, #1701 자체에는 `.github/workflows/ci.yml` 변경을 포함하지 않는다. #1702의 draft PR run 관측값은
정책 검토와 후속 #1666/#1667 비교 기준으로만 기록한다.

## required check 표면

#1664는 required check 표면을 바꾸지 않는다.

- `Build & Test` job 이름 유지
- `CI / Build & Test` check 이름 유지
- branch protection 설정 변경 없음

save step은 `Build & Test` job 내부 step일 뿐이며 별도 required check를 만들지 않는다.

## 운영 확인 기준

PR run에서 확인:

- `Restore cargo registry & build cache`가 실행됨
- `Save cargo registry & build cache`가 skipped 됨
- build/lib test/integration test/native-skia/clippy command가 기존처럼 실행됨
- `Run integration tests`가 유지되어 회귀 가드 162개 실행 전제가 보존됨

`devel` / `main` push run에서 확인:

- restore가 실행됨
- exact cache hit가 아니면 save가 실행됨
- cache reservation/read-only 경고가 사라졌는지 확인

누적 측정:

- PR checks 완료 시간 P50/P90
- `CI / Build & Test` job 시간
- 주요 step 시간
- cache hit/miss/save 성공 여부
- cache 크기
- runner-minutes 변화

누적 measurement 원천 기록은 `mydocs/report/task_m100_1664_measurement.md`에 보존한다.
