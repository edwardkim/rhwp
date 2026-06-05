# PR #1272 처리 보고서 — HWPX 탭 폭/leader/type 직렬화

- PR: https://github.com/edwardkim/rhwp/pull/1272
- 관련 이슈: https://github.com/edwardkim/rhwp/issues/1267
- 후속 이슈: https://github.com/edwardkim/rhwp/issues/1278
- 작성일: 2026-06-03
- 작성자: @Martinel2
- 처리 브랜치: `local/pr1272-integration`
- 통합 방식: 최신 `devel` 기준 cherry-pick + 문서 archive 정책 반영

## 1. 반영 내용

PR #1272의 핵심 구현을 수용했다.

- HWPX 시리얼라이저의 `<hp:tab>` 출력에서 `TAB_DEFAULT_WIDTH=4000` 고정값 사용 제거
- `Paragraph.tab_extended`를 참조하여 탭별 width/leader/type 복원
- 문단 내 복수 탭 및 컨트롤 슬롯으로 분할된 텍스트 fragment에서도 `tab_idx`를 문단 단위로 유지
- `samples/hwpx/ref/ref_mixed.hwpx` 기반 roundtrip 테스트 추가

## 2. 문서 정리

PR이 추가한 계획 문서는 현행 archive 정책에 맞춰 이동했다.

| before | after |
|---|---|
| `mydocs/plans/task_m100_1267.md` | `mydocs/plans/archives/task_m100_1267.md` |
| `mydocs/plans/task_m100_1267_impl.md` | `mydocs/plans/archives/task_m100_1267_impl.md` |

## 3. 범위 분리

issue #1267 본문에는 탭 폭 고정 문제와 함께 HWPX borderFill 사선 `slash/backSlash` 직렬화 문제도 포함되어 있었다.

PR #1272는 `src/serializer/hwpx/section.rs`의 탭 직렬화만 수정한다. `src/serializer/hwpx/header.rs`의 `hh:slash` / `hh:backSlash` `type="NONE"` 고정 출력 문제는 별도 serializer 테이블 문제이므로 #1278로 분리했다.

## 4. 검증 결과

| 항목 | 결과 |
|---|---|
| PR head GitHub Build & Test | 통과 |
| PR head GitHub CodeQL | 통과 |
| 현재 `devel` 기준 cherry-pick | 충돌 없음 |
| `cargo fmt --all --check` | 통과 |
| `cargo test --test issue_1267_hwpx_tab_and_diagonal -- --nocapture` | 통과 |

## 5. 판정

탭 폭/leader/type 보존 문제는 PR #1272로 해결된다.

사선 직렬화 문제는 #1278 후속 이슈에서 별도로 처리한다.

승인 기준으로 `devel` 병합 가능하다.
