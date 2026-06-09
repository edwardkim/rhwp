# PR #1325 리뷰 — mini_cfb DIFAT 지원으로 7.14MB 초과 CFB 손상 방지

## 1. PR 개요

| 항목 | 내용 |
|---|---|
| PR | #1325 |
| 제목 | fix: mini_cfb DIFAT 미작성으로 7.14MB 초과 CFB 손상 (#1227) |
| 작성자 | oksure |
| 대상 이슈 | #1227 |
| 대상 브랜치 | `devel` |
| PR base SHA | `ea23403786649c5c601d2577a637605579a52cec` |
| PR head SHA | `09fb6ce9355ef692c7e3580cdb8fde7a560e84c2` |
| 현재 devel | `222a16272aa61dc7ca484459ff885fcb1cb6732d` |
| 상태 | open / draft 아님 / mergeable |
| CI | PR head 기준 CI/CodeQL success |

## 2. 변경 범위

변경 파일은 1개다.

| 파일 | 변경 내용 | 판단 |
|---|---|---|
| `src/serializer/mini_cfb.rs` | CFB v3 DIFAT 섹터 계산/작성, 헤더 `first_difat`/`num_difat` 설정, 회귀 테스트 추가 | 실제 수용 대상 |

PR 변경량:

- +123 / -15
- 커밋 2개
- 문서 파일 추가 없음

## 3. 문제 구조

현재 `mini_cfb` writer는 WASM에서 `cfb` crate의 `SystemTime::now()` 의존을 피하기 위해 자체 CFB v3 바이너리를 작성한다.

CFB v3 헤더에는 FAT 섹터 포인터를 최대 109개만 직접 담을 수 있다. FAT 섹터 1개는 128개의 FAT entry를 담으므로, 헤더만으로 표현 가능한 대략적인 한계는 다음과 같다.

```text
109 FAT sectors * 128 entries * 512 bytes = 7,143,424 bytes
```

출력 CFB가 이 크기를 넘으면 남은 FAT 섹터 포인터를 DIFAT 섹터에 기록해야 한다. 기존 구현은 다음처럼 처리했다.

- `first_difat = ENDOFCHAIN`
- `num_difat = 0`
- 헤더의 109개 DIFAT slot만 작성

따라서 FAT 섹터가 109개를 넘으면 초과 FAT 섹터 위치가 어디에도 기록되지 않고, reader가 FAT chain을 따라가다 `FREESECT`를 만나 파일 열기에 실패한다.

## 4. PR 수정 방식

핵심 변경:

1. FAT/DIFAT 섹터 수를 함께 고정점 반복으로 계산
2. FAT 섹터 뒤에 DIFAT 섹터 배치
3. FAT에서 DIFAT 섹터를 `DIFSECT(0xFFFFFFFC)`로 표시
4. DIFAT 섹터에 header 109개 이후의 FAT 섹터 SID를 127개씩 기록
5. 각 DIFAT 섹터 마지막 entry에 다음 DIFAT 섹터 SID 또는 `ENDOFCHAIN` 기록
6. 헤더 offset 68/72에 `first_difat`, `num_difat` 기록

이 방식은 CFB v3의 표준 구조와 맞다.

## 5. Copilot 리뷰 반영 확인

Copilot은 최초 테스트가 대용량 버퍼를 여러 개 동시에 보유하고 전체 버퍼 비교를 수행해 CI 메모리/시간 부담이 크다고 지적했다.

컨트리뷰터는 `09fb6ce9`에서 다음처럼 반영했다.

- 8MB/7.2MB 테스트를 7.2MB 단일 테스트로 통합
- `build_cfb` 직후 입력 버퍼 `drop`
- 별도 기대 대용량 버퍼 없이 결정적 패턴(`i % 251`)으로 검증
- 전체 `assert_eq!(read_data, big)` 대신 길이 + 패턴 검사

DIFAT 경로 자체가 7.14MB 초과에서만 발동하므로, 테스트 payload를 임계값 아래로 낮출 수는 없다. 현재 반영은 합리적이다.

## 6. 로컬 검증

검증 브랜치: `local/pr1325-upstream`

```text
cargo fmt --all -- --check
통과

cargo test --lib mini_cfb -- --nocapture
7 passed

cargo clippy --lib -- -D warnings
통과
```

GitHub Actions:

```text
CI 27091879437: success
CodeQL 27091879438: success
```

## 7. 검토 의견

기능 수정은 수용 가능하다. 대용량 HWP 저장 시 파일 손상으로 이어지는 심각한 CFB writer 결함을 직접 해결한다.

현재 테스트는 `cfb::CompoundFile::open`으로 생성 CFB를 실제 라운드트립 검증하므로, 단순 헤더 필드 확인보다 의미가 크다. 테스트 시간도 로컬 기준 `mini_cfb` 전체 7개가 약 0.25초 실행으로 부담이 낮다.

추가로 고려할 수 있는 후속 보강:

- DIFAT 섹터가 2개 이상 필요한 초대형 케이스는 현재 테스트하지 않는다.
- 다만 실제 문서 크기/CI 비용을 고려하면 현재 PR의 blocker로 보기는 어렵다.

## 8. 권장 처리

권장안: **수용**

통합 방식은 두 가지가 가능하다.

1. GitHub admin merge
   - PR 변경 파일이 1개이고 PR head CI가 success라 단순하다.
   - 다만 PR base가 현재 `devel`보다 뒤처져 있어 merge commit이 필요하다.
2. maintainer-side 통합
   - 현재 `local/devel` 위에 `src/serializer/mini_cfb.rs` 변경만 반영 후 push
   - 최근 PR 처리 방식과 일관되고, push 후 `devel` CI를 다시 확인할 수 있다.

권장 절차: **maintainer-side 통합**

1. `local/devel` 기준 통합 브랜치 생성
2. PR의 `src/serializer/mini_cfb.rs` 변경만 적용
3. `cargo fmt --all -- --check`
4. `cargo test --lib mini_cfb -- --nocapture`
5. `cargo clippy --lib -- -D warnings`
6. `devel` push
7. PR #1325 코멘트 및 close
8. Issue #1227 close
9. GitHub CI/CodeQL 확인

## 9. PR 코멘트 초안

```markdown
검토했습니다. CFB v3 헤더의 109개 FAT 포인터 한계를 넘는 경우 DIFAT 섹터를 작성하지 않아 대용량 HWP 저장 파일이 손상되는 문제를 정확히 해결한 것으로 확인했습니다.

로컬에서 다음 검증을 통과했습니다.

- `cargo fmt --all -- --check`
- `cargo test --lib mini_cfb -- --nocapture` (7 passed)
- `cargo clippy --lib -- -D warnings`

PR head 기준 GitHub CI/CodeQL도 성공입니다.

Copilot이 지적한 테스트 메모리/시간 부담도 `09fb6ce9`에서 합리적으로 줄어든 것을 확인했습니다. 프로젝트 통합 정책상 maintainer-side로 현재 `devel`에 반영하겠습니다. 감사합니다.
```
