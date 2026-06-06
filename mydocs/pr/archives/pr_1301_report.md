# PR #1301 완료 보고서 — 키 큰 base 위첨자 상단 정렬

## 1. 개요

| 항목 | 내용 |
|---|---|
| PR | #1301 |
| 관련 이슈 | #1300 |
| 통합 브랜치 | `local/pr1301-integration` |
| 통합 방식 | `src/renderer/equation/layout.rs` 코드 변경만 현재 `local/devel` 위에 3-way 적용 |
| 추가 문서 | `mydocs/troubleshootings/stale_wasm_build_phantom_bug.md` 경로 보정 후 신규 작성 |
| 제외 | 기여자 `mydocs/plans`, `mydocs/report`, `mydocs/working` 작업 문서 |

## 2. 처리 내용

`layout_superscript`에서 키 큰 base의 위첨자 위치를 base 상단에 정렬하도록 변경했다.

기존 로직은 `sup_shift = b.baseline - s.height * 0.7`에 따라 `base_y`를 키웠다. 괄호 분수처럼 base baseline이 큰 경우 base가 과하게 아래로 밀리고, 위첨자가 렌더 박스 최상단에 붙으면서 윗줄을 침범했다.

수정 후:

```rust
base_y = (s.height - b.height).max(0.0);
```

효과:

- 키 큰 base: base를 아래로 밀지 않아 위첨자 상단이 base 상단과 정렬
- 짧은 base: 메인테이너 시각 판정 통과

## 3. 시각 판정

메인테이너 시각 판정: **통과**

판정 대상:

- `samples/3-09월_교육_통합_2022.hwpx` 17쪽 [다른 풀이]
- `(1/6)^4` 형태의 지수가 윗줄을 침범하지 않음
- 짧은 base 수식도 허용 범위로 확인

시각 판정용 SVG:

```text
output/poc/pr1301-superscript-top-align/3-09월_교육_통합_2022_017.svg
```

## 4. 검증 결과

통합 브랜치에서 직접 실행:

```text
cargo fmt --all -- --check
통과

cargo test --lib renderer::equation -- --nocapture
134 passed

cargo clippy --lib -- -D warnings
통과

cargo test --lib
1587 passed, 0 failed, 6 ignored
```

## 5. 문서 보강

PR에 포함된 stale WASM troubleshooting 내용은 유지 가치가 있어 보정 후 새 문서로 추가했다.

보정 내용:

- contributor 로컬 경로(`/home/planet/iop/rhwp`) 제거
- 프로젝트 일반 경로와 표준 WASM 빌드 명령으로 정리
- PR #1301 사례를 일반적인 stale WASM 진단 흐름으로 기록

## 6. 판정

**수용 완료 가능**.

승인 후 절차:

1. 통합 브랜치 커밋
2. `local/devel` merge
3. `devel` merge
4. `origin/devel` push
5. PR #1301 코멘트 및 close
6. 이슈 #1300 close
