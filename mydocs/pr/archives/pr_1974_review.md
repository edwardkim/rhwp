# PR #1974 리뷰 - lineseg vpos=0 새 쪽 시작 신호 보존

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1974 |
| 제목 | Issue #1920: lineseg 재계산 시 저장 vpos=0 새 쪽 시작 신호 보존 (#1969 후속 커밋 분리) |
| 작성자 | planet6897 |
| base | `devel` |
| 문서 작성 시점 head SHA | `a7884b707d1ca2b124404f13902f502b9b85bc38` |
| 체리픽 commit | `02be39697` |
| 규모 | 1 file, +49 / -2 |
| 변경 파일 | `src/document_core/commands/document.rs` |
| 처리 방식 | planet6897 PR 8건 통합 체리픽 브랜치에서 오래된 PR 번호 순으로 적용 |

## 변경 범위

- 문서 command 경로의 lineseg 재계산에서 저장된 `vpos=0`을 새 쪽 시작 신호로 보존한다.
- #1969에서 다룬 하단 고정 틀/쪽 하단 흐름 보정의 후속 분리 커밋이다.
- 특정 샘플명이나 PR 번호가 아니라, 저장 lineseg 값의 의미를 근거로 동작한다.

## 체리픽 검토

- 적용 순서: 1/8
- 원 commit: `a7884b707d1ca2b124404f13902f502b9b85bc38`
- 로컬 commit: `02be39697`
- 충돌: 없음
- 선행 PR 의존: 통합 브랜치 기준 선행 PR 없음. 내용상 #1969 후속 성격이지만 `upstream/devel`에는 #1969가 이미 반영되어 있다.

## 시각 검증

이 PR은 직접 렌더 출력 파일을 추가하지 않는 command/lineseg 의미 보존 보정이다. PR 자체의 기준 PDF는 첨부되어 있지 않아 별도 visual sweep은 수행하지 않았다. 렌더 영향 가능성은 통합 브랜치 전체 회귀 테스트와 관련 targeted test로 확인했다.

## 로컬 검증

검토 시작 전 `/Users/tsjang/rhwp/target` 하위 항목을 삭제한 뒤 순차 실행했다.

- `git diff --check`: 통과
- `cargo fmt --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과

통합 브랜치 전체 검증에서 `tests/svg_snapshot.rs`도 함께 실행되어 통과했다.

## 검토 결과

보정 근거가 저장 lineseg 의미에 있고, 통합 체리픽 상태에서 전체 회귀 테스트와 clippy가 통과했다. 최종 권고는 planet6897 8건 통합 PR로 merge 후보다.

