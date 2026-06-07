# PR #1301 리뷰 — 키 큰 base 위첨자 상단 정렬

## 1. PR 개요

| 항목 | 내용 |
|---|---|
| PR | #1301 |
| 제목 | fix(#1300): 수식 위첨자를 base 상단에 정렬 — 키 큰 base 윗줄 침범 해소 |
| 작성자 | planet6897 |
| 관련 이슈 | #1300, #1297 |
| 대상 브랜치 | `devel` |
| 검토 기준 | `local/pr1301-upstream` |

## 2. 변경 범위

변경 파일:

| 파일 | 판단 |
|---|---|
| `src/renderer/equation/layout.rs` | 실제 기능 수정 및 회귀 테스트 |
| `mydocs/troubleshootings/stale_wasm_build_phantom_bug.md` | 유용하나 contributor 로컬 경로 수정 필요 |
| `mydocs/plans/task_m100_1297.md` | 기여자 작업 문서 |
| `mydocs/plans/task_m100_1297_impl.md` | 기여자 작업 문서 |
| `mydocs/plans/task_m100_1300.md` | 기여자 작업 문서 |
| `mydocs/plans/task_m100_1300_impl.md` | 기여자 작업 문서 |
| `mydocs/report/task_m100_1297_report.md` | 기여자 작업 문서 |
| `mydocs/report/task_m100_1300_report.md` | 기여자 작업 문서 |
| `mydocs/working/task_m100_1300_stage*.md` | 기여자 작업 문서 |

코드 변경은 `layout_superscript`에서 위첨자 배치를 조정하는 것이다.

## 3. 문제와 수정 방향

문제 샘플:

- `samples/3-09월_교육_통합_2022.hwpx`
- 17쪽 [다른 풀이]
- `(1/6)^4` 형태에서 `4`가 위로 과하게 올라가 윗줄을 침범

기존 로직은 `sup_shift = b.baseline - s.height * 0.7`을 사용하고, 키 큰 base일수록 `base_y`가 커졌다. 결과적으로 합성 baseline이 커지고, 위첨자가 렌더 박스 최상단에 붙으면서 base 상단보다 위로 떠올랐다.

PR은 다음처럼 base를 아래로 밀지 않는 상단 정렬로 바꾼다.

```rust
base_y = (s.height - b.height).max(0.0);
```

효과:

- 키 큰 base에서는 `base_y = 0`이 되어 위첨자 상단이 base 상단과 정렬된다.
- 윗줄 침범은 해소되는 방향이다.

## 4. 검증 결과

PR head(`local/pr1301-upstream`)에서 직접 실행:

```text
cargo fmt --all -- --check
통과

cargo test --lib renderer::equation -- --nocapture
134 passed

cargo clippy --lib -- -D warnings
통과

cargo test --lib
1577 passed, 0 failed, 6 ignored
```

시각 판정용 SVG 생성:

```text
output/poc/pr1301-superscript-top-align/3-09월_교육_통합_2022_017.svg
```

## 5. 검토 의견

수정 지점은 문제 원인과 잘 맞는다. 키 큰 base에서 `base_y`가 baseline에 비례해 커지는 것이 지수 과상승의 원인이므로, base를 아래로 밀지 않는 방식은 타깃 증상을 직접 해소한다.

주의할 점:

- 이 변경은 키 큰 base뿐 아니라 짧은 base(`x^2`, `6^4`)에도 영향을 줄 수 있다.
- 기존 `test_superscript_layout`은 `x^2`의 위치를 정밀하게 고정하지 않고 width/height만 확인한다.
- 새 테스트는 “sup가 base 상단보다 위로 뜨지 않는다”를 고정하므로, 수학적으로 자연스러운 위첨자 높이와 한컴 정합은 시각 판정으로 확인해야 한다.

따라서 수용 전 메인테이너 시각 판정은 다음 둘을 같이 보는 것을 권장한다.

1. 타깃 샘플 17쪽 `(1/6)^4`: 윗줄 침범 해소
2. 짧은 base 수식(`6^4`, `x^2`, `25^{1/3}`): 위첨자가 너무 낮아지지 않았는지 확인

## 6. 문서 처리 의견

`mydocs/troubleshootings/stale_wasm_build_phantom_bug.md`는 최근 반복된 stale WASM 혼동을 줄이는 데 유용하다. 다만 그대로 수용하기 전에 다음 보정이 필요하다.

- `cd /home/planet/iop/rhwp` 같은 contributor 로컬 경로를 제거하거나 프로젝트 일반 경로로 변경
- Task #1297 보고서 경로를 수용하지 않을 경우 사례 링크 문구도 조정

`mydocs/plans`, `mydocs/report`, `mydocs/working`의 기여자 작업 문서는 운영 문서 흐름과 섞일 수 있으므로 실제 통합에서는 제외하는 것을 권장한다.

## 7. 권장 처리

권장안: **조건부 수용**

- 코드 변경은 작고, 관련 테스트와 `cargo test --lib`가 통과했다.
- 단, 메인테이너 시각 판정으로 짧은 base 수식까지 확인한 뒤 통합한다.
- 통합 시에는 코드 변경을 우선 수용하고, troubleshooting 문서는 경로 보정 후 선택 수용한다.

권장 절차:

1. 메인테이너 SVG/웹 시각 판정
2. `local/devel` 기준 통합 브랜치 생성
3. `src/renderer/equation/layout.rs` 코드 변경 적용
4. 필요 시 `mydocs/troubleshootings/stale_wasm_build_phantom_bug.md` 경로 보정 후 추가
5. 테스트 재실행
6. 완료 보고서 작성 후 승인 게이트

## 8. PR 코멘트 초안

```markdown
검토했습니다. 키 큰 base에서 `base_y`가 baseline에 비례해 커지면서 위첨자가 과하게 상승하던 원인 분석과 수정 방향은 타당해 보입니다.

로컬에서 다음 검증을 통과했습니다.

- `cargo fmt --all -- --check`
- `cargo test --lib renderer::equation -- --nocapture` (134 passed)
- `cargo test --lib` (1577 passed, 0 failed, 6 ignored)
- `cargo clippy --lib -- -D warnings`

메인테이너 시각 판정에서는 타깃 샘플 17쪽 `(1/6)^4`뿐 아니라 짧은 base 수식(`6^4`, `x^2`, `25^{1/3}`)의 위첨자가 너무 낮아지지 않았는지도 함께 확인하겠습니다.

`stale_wasm_build_phantom_bug.md` 문서는 유용하지만 contributor 로컬 경로가 포함되어 있어 통합 시 보정하겠습니다. 감사합니다.
```
