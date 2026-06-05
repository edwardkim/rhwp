# PR #1265 처리 보고서 — 편집 삽입 탭 저장 시 tab_extended 폴백 마커 보강

- PR: https://github.com/edwardkim/rhwp/pull/1265
- 관련 이슈: https://github.com/edwardkim/rhwp/issues/1244
- 처리일: 2026-06-03
- 통합 브랜치: `local/pr1265-current`

## 1. 처리 요약

PR #1265의 핵심 수정은 수용했다.

편집으로 삽입된 탭 문자는 `Paragraph.tab_extended`가 비어 있는 상태로 HWP 저장 경로에 들어올 수 있다.
기존 저장기는 이 경우 TAB 뒤의 7개 확장 데이터를 `[0; 7]`로 직렬화했고,
한컴 편집기는 `ext[6] != 0x0009`인 탭을 정상 탭으로 인식하지 않아 탭이 사라졌다.

이번 통합에서는 PR의 저장 폴백 보강을 반영하여 `tab_extended`가 없을 때도
`[0, 0, 0, 0, 0, 0, 0x0009]`를 직렬화하도록 했다.

## 2. 반영 내용

| file | 내용 |
|---|---|
| `src/serializer/body_text.rs` | TAB 확장 데이터 폴백의 `ext[6]`을 `0x0009`로 설정 |
| `tests/issue_1244_tab_extended_fallback.rs` | 편집 삽입 탭 1개/2개 roundtrip 및 HWPX→HWP 저장 경로 tab_extended 보존 검증 추가 |
| `mydocs/plans/archives/task_1244.md` | PR 수행계획서를 archives 위치로 정리 |
| `mydocs/pr/pr_1265_review.md` | PR 검토 보고서 작성 |

## 3. 검증

실행한 검증:

```text
cargo fmt --all --check
cargo test --test issue_1244_tab_extended_fallback -- --nocapture
cargo test --verbose
cargo clippy -- -D warnings
cargo check --target wasm32-unknown-unknown --lib
git diff --check
```

결과:

| 항목 | 결과 |
|---|---|
| 포맷 체크 | 통과 |
| 신규 issue #1244 테스트 | 3 passed |
| 전체 테스트 | 통과 |
| clippy | 통과 |
| wasm target check | 통과 |
| whitespace check | 통과 |

참고: `cargo test --verbose` 시작 시 Cargo 전역 캐시의 last-use DB 쓰기 실패 경고가 출력되었으나,
테스트 실패와는 무관하며 전체 테스트는 통과했다.

GitHub PR head 기준:

| workflow | run id | 결과 |
|---|---:|---|
| CI | 26864824145 | success |
| CodeQL | 26864824115 | success |

## 4. 판정

수용 완료.

이번 PR은 HWP 저장 호환성의 매우 좁은 폴백 보강이며, 기존 `tab_extended`가 있는 문서의 저장 경로는 변경하지 않는다.
따라서 기존 렌더링/저장 경로 회귀 위험은 낮고, 편집 삽입 탭의 한컴 호환성은 개선된다.

추가 확인 결과, HWPX 파서는 `<hp:tab>` 파싱 시 `tab_extended[6] = 0x0009`를 이미 생성하고,
HWPX 출처 HWP 저장 경로(`export_hwp_with_adapter`)도 이 값을 HWP 직렬화까지 보존한다.
이를 `samples/hwpx/ref/ref_mixed.hwpx` 기반 회귀 테스트로 고정했다.

## 5. 후속 절차

1. `local/pr1265-current`를 `devel`에 병합
2. `devel` push
3. PR #1265에 메인테이너 코멘트 추가 후 close
4. 이슈 #1244 close
