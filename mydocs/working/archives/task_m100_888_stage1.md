# Task #888 Stage 1 검증 보고서

## 1. 기준선

- 기준 브랜치: `local/devel`
- 기준 커밋: `d4ece849`
- 작업 브랜치: `local/task_m100_888_integrate_854`
- 관련 이슈: https://github.com/edwardkim/rhwp/issues/888

## 2. 코드 검증

### HWPX -> HWP adapter 테스트

```text
cargo test --test hwpx_to_hwp_adapter
```

결과:

```text
30 passed; 0 failed
```

### 라이브러리 전체 테스트

```text
cargo test --lib --quiet
```

결과:

```text
1246 passed; 0 failed; 2 ignored
```

기존 경고만 출력됨:

- `renderer/equation/parser.rs` duplicate `#[test]`
- `renderer/layout/integration_tests.rs` unnecessary parentheses
- 기존 test function snake_case 경고
- 기존 unused `Result` 경고

## 3. 작업지시자 판정용 HWP 산출물

아래 산출물은 프로젝트 루트 `/home/edward/mygithub/rhwp/output/` 하위에 배치한다. `output/` 하위이므로 git 추적 대상이 아니다.

```text
output/poc/hwpx2hwp/task888/stage1/basic-table-01.hwp
output/poc/hwpx2hwp/task888/stage1/expense_report.hwp
```

생성 경로:

- `DocumentCore::from_bytes(hwpx)`
- `DocumentCore::export_hwp_with_adapter()`
- HWP bytes write

## 4. 외부 판정 요청

### basic-table-01.hwp

판정 항목:

- 한컴 에디터 파일 손상 판정이 없는지
- 문단 배경 무늬 오류가 없는지
- 표 배치가 #854 Stage 14 정답 수준인지

### expense_report.hwp

판정 항목:

- 한컴 에디터 파일 손상 판정이 없는지
- rhwp-studio/한컴에서 TAC table 배치가 종이 왼쪽으로 붙지 않는지
- 셀 border가 유지되는지
- 쪽 배경이 출력되는지
- #854 Stage 14 정답지와 유사한지

## 5. 현재 판정

자동 테스트 기준은 통과했다. 다음 단계는 작업지시자 환경에서 위 두 HWP 파일을 한컴 에디터와 rhwp-studio로 시각 판정하는 것이다.

주의: 판정용 산출물은 worktree 내부 `/tmp/rhwp-task-888/output/` 가 아니라 작업지시자 프로젝트 루트 `/home/edward/mygithub/rhwp/output/` 에 배치해야 한다.
