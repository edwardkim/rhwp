# #4586 Stage 1 완료보고서 — RED 계약과 최신 기준선

- **Issue**: [#4586](https://github.com/edwardkim/rhwp/issues/4586)
- **브랜치**: `task/4586-gym-t12-hwpx`
- **기준**: `upstream/devel` `d30e5d4af`
- **계획 커밋**: `d25375877`

## 1. 최신 devel 재현

`cargo build --bin rhwp`로 최신 기준 바이너리를 만든 뒤 `samples/field-01.hwp`를 다시 측정했다.
원격 통합 뒤 82개 커밋이 추가됐지만 #4586의 두 현상은 그대로였다.

| 경로 | 실제 형식 | 종료 코드 | 판정 |
| --- | --- | --- | --- |
| `convert ... conv.hwpx --verify --json` | HWP5 | 0 | `identical:true`, `diffCount:0` |
| `export-hwpx ... conv.hwpx --verify --json` | HWPX | 3 | `identical:false`, `diffCount:11` |
| 독립 `ir-diff` | HWP↔HWPX | 3 | `identical:false`, `diffCount:6` |

원문 봉투와 `file`/`info` 결과는 `output/4586/red/`에 저장했다. 이 폴더는 저장소 정책대로
gitignore이며 작업지시자가 VS Code에서 확인할 수 있는 로컬 증적이다.

## 2. Rust RED 계약

신설: `tests/issue_4586_gym_t12_contract.rs`

```bash
cargo test --test issue_4586_gym_t12_contract -- --nocapture
```

결과: **1 passed / 2 failed**.

- PASS: `.HWP` 대문자 확장자는 유효한 HWP5 출력으로 유지된다.
- FAIL: 기존 `convert`가 `.hwpx` 출력에 HWP5를 쓰고 exit 0을 반환한다.
- FAIL: 잘못된 출력 확장자보다 존재하지 않는 입력 IO를 먼저 검사해 exit 1을 반환한다.

두 실패는 구현 뒤 각각 exit 2, stdout 0바이트, `export-hwpx` 안내, 산출물 없음으로 바뀌어야 한다.

## 3. Python RED 계약

신설: `scripts/tests/test_gym_score.py`

```bash
python3 -m unittest scripts/tests/test_gym_score.py
```

결과: **1 passed / 3 failed**.

- PASS: 기존 `expect_exit:0` 단일 종료 코드 검사는 그대로 동작한다.
- FAIL: exit 3 + `identical:false`를 정답 `false`와 비교하지 않고 `기대 0`에서 버린다.
- FAIL: `expect_exits:[0,3]`을 인식하지 않아 허용 집합 오류에 3이 표시되지 않는다.
- FAIL: T12가 여전히 `convert`를 안내하며 형식 검사와 `expect_exits`가 없다.

## 4. 판정

RED 분포는 수행계획의 세 근인과 일치한다.

1. CLI 출력 확장자 가드 부재
2. gym 복수 판정 종료 코드 부재
3. T12 실제 형식 검사 부재

따라서 설계 변경 없이 Stage 2 구현으로 진행할 수 있다. 실제 HWPX의 IR 차이 11건/6건은 이번
단계에서도 수정 대상으로 확장하지 않는다.

## 5. 형식 검사

```bash
cargo fmt --all -- --check
git diff --check
```

둘 다 통과했다. 렌더러·레이아웃·WASM 변경이 없어 시각 증적은 적용하지 않았다.
