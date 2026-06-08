# PR #1327 리뷰 — HWPX inline control serialization

**작성일**: 2026-06-08  
**PR**: https://github.com/edwardkim/rhwp/pull/1327  
**작성자**: `oksure`  
**제목**: `fix: HWPX 직렬화에서 누락되던 인라인 컨트롤 6종 (pageHiding/pageNum/newNum/머리말/꼬리말/autoNum) — 저장 시 데이터 손실 (#1326)`

## 1. 메타

| 항목 | 값 |
|---|---|
| base | `devel` |
| head | `contrib/hwpx-page-controls` |
| head sha | `668b4bfd57d93cd44fd0b66b4ebd016e219ef62f` |
| draft | false |
| mergeable | true |
| GitHub merge state | `BEHIND` |
| commits | 3 |
| changed files | 3 |
| 규모 | +409 / -2 |
| 관련 이슈 | Refs #1326 |

## 2. 변경 범위

이번 PR은 HWPX 저장 시 누락되던 inline control을 다시 HWPX로 직렬화하는 보존성 개선 PR이다.

대상 control:

- `hp:pageHiding`
- `hp:pageNum`
- `hp:newNum`
- `hp:header`
- `hp:footer`
- `hp:autoNum`

주요 변경:

- `src/serializer/hwpx/section.rs`
  - page hiding/page number/new number/header/footer/auto number 렌더링 함수 추가.
  - paragraph part serialization에서 위 control들을 출력하도록 분기 추가.
- `src/parser/hwpx/section.rs`
  - `autoNumFormat type` 파싱에서 HWPX enum 문자열을 내부 코드로 매핑하도록 보강.
- `tests/hwpx_roundtrip_integration.rs`
  - pageHiding/pageNum/newNum/header/footer/autoNum 보존 roundtrip 테스트 추가.

## 3. GitHub 상태

GitHub checks:

- Build & Test: pass
- CodeQL: pass
- Analyze javascript-typescript/python/rust: pass
- WASM Build: skipped

검토 당시 주의 사항:

- PR head는 현재 `origin/devel`보다 뒤처져 있어 GitHub merge state가 `BEHIND`이다.
- 이전 maintainer-side 통합 보조 PR #1338은 closed/unmerged 상태이며, #1327 변경은 아직 devel에 반영되지 않았다.

Copilot unresolved thread:

- `page_num_pos_to_str()` / `page_num_format_to_str()` unknown code fallback 지적
  - 현재 parser가 unknown HWPX enum을 이미 기본값으로 정규화하므로, serializer에서 원문 unknown 값을 보존할 구조가 없다.
  - 이번 PR의 회귀 요인은 아니며, unknown enum 보존이 필요하면 별도 IR 확장 이슈로 분리하는 편이 맞다.
- `side.to_string()` allocation 지적
  - 단일 문자 문자열 생성에 대한 미세 최적화 의견이다.
  - 기능/보존성/CI 관점에서는 non-blocking으로 판단한다.

## 4. 로컬 검증

PR branch: `local/pr1327-upstream`

최신 `origin/devel` 병합 시뮬레이션:

- `local/pr1327-merge-test`에서 `origin/devel` merge
- conflict 없음

통과:

- `cargo fmt --all -- --check`
- `cargo test --test hwpx_roundtrip_integration`
- `cargo clippy --all-targets -- -D warnings`

`cargo test --test hwpx_roundtrip_integration` 결과:

- 22 passed

## 5. 리스크 평가

- 변경 범위는 HWPX serialization/parser/test에 한정된다.
- 기존 HWP/HWPX 렌더링 경로의 layout 의미 변경은 없다.
- header/footer serialization은 기존 paragraph part serializer를 재사용하므로 control 보존 목적에는 적합하다.
- #1326은 "form"까지 남아 있는 상태로 보이며, 이번 PR merge 후 자동 close 대상으로 보기는 어렵다.
- unknown HWPX enum 보존은 현재 IR가 원문 enum 문자열을 들고 있지 않으므로 이번 PR에서 해결할 범위를 넘어선다.

## 6. 권고

**수용 권고**.

사유:

- HWPX roundtrip에서 누락되던 inline control 6종 보존 문제가 명확히 개선된다.
- 테스트가 실제 fixtures 기반으로 추가되어 회귀 방지 효과가 있다.
- GitHub checks와 로컬 검증이 모두 통과했다.
- PR이 `BEHIND` 상태지만 최신 devel 병합 시뮬레이션에서 conflict가 없다.

권장 절차:

1. maintainer-side로 최신 `devel` 위에 PR #1327 변경을 통합한다.
2. Copilot unresolved thread는 non-blocking 사유를 남기고 필요 시 resolve한다.
3. PR #1327에 메인테이너 코멘트를 남긴다.
4. PR #1327을 종료 처리한다.
5. #1326은 form 관련 후속 처리가 남아 있으므로 유지한다.
6. `local/devel` sync, 리뷰 문서 archive, `mydocs/orders/20260608.md` 기록을 진행한다.

## 7. 처리 결과

승인 후 처리:

- `local/devel`에 PR #1327 변경을 병합 커밋으로 통합.
- 통합 커밋: `d7ff712b` (`Merge PR #1327 from oksure/contrib/hwpx-page-controls`)
- `origin/devel` push 완료.
- GitHub PR #1327은 `MERGED` 상태로 자동 처리됨.
- 메인테이너 코멘트 작성: https://github.com/edwardkim/rhwp/pull/1327#issuecomment-4648702544
- #1326은 form 관련 후속 범위가 남아 있어 유지.
