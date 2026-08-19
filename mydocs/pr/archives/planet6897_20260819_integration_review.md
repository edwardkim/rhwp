# planet6897 열린 PR 누적 검토 기록 (2026-08-19)

## 범위와 기준선

- 검토 branch: `review/planet6897-20260819`
- 초기 누적 기준: `upstream/devel` `2e852d7f730865cb32000519303df11b18c3f2fe`
- 최신 동기화 기준: `upstream/devel` `161820019cfb2931348992f9050109b62354ad54` (로컬 merge `466bfcce7`)
- 코드 보정 후보: `f9c34fbc837f8b43fc0e49438ea7442124d02ca8`; 현재 검토 head는 위 최신 기준을 포함한다.
- 대상: #5544, #5552, #5559, #5560, #5562, #5564, #5565, #5567, #5574, #5577
- 경로: collaborator 통합 PR / 접수·로컬 검증·다수 PR 누적·시각 fixture 보조 경로

원 PR은 모두 `planet6897`의 `devel` 대상 head를 fetch하여 오래된 PR 번호 순으로
`upstream/devel` 위에 누적 cherry-pick했다. 각 원 PR의 판정은 해당 번호의
`pr_<번호>_review.md`에 독립적으로 기록한다. 이 문서는 공통 기준선과 배치 검증 상태만
보관한다.

## 누적 순서와 메인터너 보정

| 원 PR | 원 head | 로컬 적용 commit |
| --- | --- | --- |
| #5544 | `3bd5b349291c5829be26dc62fd7ceebc3d83e963` | `42924f94e` |
| #5552 | `a90e85003a31bd36814a966f6487d2d19b6c71ae` | `f4b18606c` |
| #5559 | `2f399a1644dd36fd226a7f3ad59a01b9c0cd3b65` | `d5e36cbce` |
| #5560 | `b3d85b4f219522f104edb80fa7cc66ba38621b2d` | `04220d147` |
| #5562 | `706e2e34e38afa9f68ad42fe47e037c219c6ba26` | `efe22660d` |
| #5564 | `b91a458b06ecf404f335c3fc8eda6881b595f909` | `d99dd75a9` |
| #5565 | `3b496cd066de604c01f3b8bb13832615a9588a88`, `a1b1e8278920a14915000d6b07b8b2ed521d2123` | `453393787`, `7ba9e1a69` |
| #5567 | `cd05c27e4f559b6fbc5eca6d32b28f143db15d8d` | `e7e377eb9` |
| #5574 | `2125df67f9409f8306831c48c166910f4638498e` | `fbace2ffb` |
| #5577 | `ec6e06faba0200c4569d81149f9da32da10c63f8` | `ace18930f` |

누적 과정에서 다음 메인터너 보정을 분리했다.

- `fc56b1a2e`: Rustfmt가 요구하는 `drawing.rs` 공백 정리.
- `cd95d22f5`: #5560의 HWP3 정렬→줄나눔 파생을 `tests/cases/` 통합 회귀로 고정.
- `f9c34fbc8`: #5574의 lineseg 상한이 빈 누름틀 안내문을 DocumentCore IR에서 비운 뒤에도
  유효한 `textpos`를 버리지 않도록 실제 직렬화 UTF-16 축을 반영. 회귀는
  `tests/cases/issue_5563_hwpx_lineseg_axis.rs`에만 추가했다.

## 검증 상태

- `cargo-nextest 0.9.140`을 설치했다.
- 누적 후보의 기능별 focused integration 회귀 20건은 #5574의 후속 보정 전 통과했다.
- 누적 후보의 전체 `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 4 --no-fail-fast`는 보정 전 7,743건 중 7,726건 통과, 17건 실패로 종료했다. 실패에는 Windows CRLF 환경의 skill frontmatter 13건과 baseline에서도 재현된 WMF/EMF golden 2건이 포함됐다. #1893, corpus ratchet partition 1은 이 배치에서 별도 판단이 필요했다.
- `upstream/devel` baseline에서 선택 확인한 결과 #1893과 corpus ratchet partition 1은 통과했고, frontmatter·WMF/EMF golden 실패는 동일하게 재현됐다.
- #5574 후속 보정 뒤의 #1893/#5563 선택 nextest는 링크 단계에서 완료 요약 없이 종료됐다. 작업지시자의 재실행 금지 지시에 따라 PASS로 기록하지 않는다.
- 이후 작업지시자가 지정한 공식 #1893 회귀만 코드 보정 후보에서 다시 실행했다. `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 6 --no-fail-fast -E 'test(issue_1893_clickhere_form_roundtrip_render_is_self_consistent)'`는 `1 passed, 7781 skipped`로 종료 코드 0을 반환했다. 이후 최신 `upstream/devel`을 충돌 없이 병합했으므로, 최종 통합 head의 required check는 별도로 확인한다.
- LF 전용 검토 worktree에서 최종 후보 `f9c34fbc8`에 `cargo fmt --all` 및 `cargo fmt --all -- --check`를 실행해 통과했다. 원 Windows worktree의 전역 CRLF 변환은 포맷 판정에 사용하지 않았다.

## 공통 병합 조건

1. #5574의 추가 field-slot 회귀와 원인 미확정 corpus ratchet 실패를 최신 후보에서 판정한다. 공식 #1893 회귀는 통과했다.
2. #5544와 #5577의 renderer/layout 영향은 기준 PDF 또는 동등한 시각 증적을 남긴다.
3. 통합 PR의 최신 head에서 GitHub required checks를 다시 확인한다.
4. 작업지시자의 원격 push·PR 생성 및 최종 merge 승인을 각각 받는다.

따라서 이 기록의 개별 “수용 권고”는 누적 후보에 포함해 CI를 받는다는 뜻이며, 현 시점의
최종 merge 승인이나 원 PR의 원격 merge를 뜻하지 않는다.
