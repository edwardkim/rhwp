---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-13
---

# PR #4684 검토 - HWPX curve를 `hp:seg` 체인으로 저장

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4684](https://github.com/edwardkim/rhwp/pull/4684) |
| 작성자 / source | @planet6897 / `fix/4676-hwpx-curve-seg` |
| 원 source head | `2c185af979f71bdf4ae80352a3db90c7cf6b0616` |
| 기준 devel | `6f70cd1b6` |
| 가시성 검토 branch | `review/planet6897-4684-20260812` |
| 관련 이슈 | [#4676](https://github.com/edwardkim/rhwp/issues/4676) |
| 원 PR 상태 참고값 | 원 source `2c185af` 기준 `CONFLICTING` / `DIRTY`; local merge commit `87fab352a`로 해소, push 뒤 재확인 필요 |
| reviewer | @jangster77 지정 완료 |

원 PR은 `hp:curve` 내부의 점을 `hc:pt`로 저장하던 경로를 인접 점 쌍의 `hp:seg` 체인으로
바꾼다. 한글 2022 오라클에서 `hc:pt`를 포함한 curve가 프로세스 종료를 유발했다는 #4676의
원인 분석에 대응한다. 최신 `devel`과 원 PR은 충돌했으나, 가시성 branch에서 원 source
`2c185af`를 second parent로 보존하는 merge commit `87fab352a`를 만들고 #4675·#4676 테스트 및
메인터너 보정을 함께 유지해 해소했다. 원 contributor commit을 재작성하거나 force-push하지 않으며,
이 head를 기존 source branch `fix/4676-hwpx-curve-seg`에 fast-forward push한다.

## 메인터너 보정

기여자 변경은 `hp:seg type="CURVE|LINE"`를 HWP5의 `CurveShape.segment_types`에 `1|0`으로
옮겼다. 그러나 HWPX `CURVE` segment는 끝점 하나만 담는 점-대-점 체인이고, renderer의 HWP5
값 `1`은 제어점 둘과 끝점, 총 세 점을 소비하는 cubic Bezier 계약이다. 그대로 매핑하면 5개
HWPX segment가 `LineTo` 둘과 Bezier 하나로 바뀌어 실제 경로가 달라진다.

메인터너 보정 `3741441a2`는 HWPX `hp:seg`의 type을 HWP5 `segment_types`에 넣지 않고 비워,
기존 `LineTo` 체인 렌더 계약을 보존했다. CURVE와 LINE가 섞인 입력에서도 빈 값이 유지됨을
회귀로 고정했다. 이 보정은 writer의 `hp:seg` 출력 경로를 변경하지 않는다.

`278eee6ea`는 최신 `devel`에 먼저 들어온 #4675 테스트의 닫히지 않은 블록을 해소한 충돌
정리다. 동작을 바꾸지 않고 #4676 serializer 회귀를 독립 테스트로 유지한다. 커밋별 적용 순서와
검증 단계는 [구현 기록](pr_4684_review_impl.md)에 분리했다.

## 완료한 검증

| 범위 | 명령 또는 근거 | 결과 |
| --- | --- | --- |
| parser 회귀 | `cargo test --profile release-test --target-dir target/pr-review --lib curve_seg -- --nocapture` | `test_parse_curve_seg_populates_points_without_hwp5_bezier_types` 통과 |
| serializer 회귀 | `cargo test --profile release-test --target-dir target/pr-review --lib issue4676 -- --nocapture` | `hp:seg` 출력 회귀와 HWPX `CURVE` XML→IR→XML 경계 회귀 2건 통과 |
| 전체 Rust | `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast` | 5,910 passed, 37 skipped, 7 slow, 418.930초 |
| 품질 | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `git diff --check` | 통과 |
| 실제 HWPX 재저장 | 비공개 코퍼스의 `156550355` HWPX를 `rhwp export-hwpx --verify --json`으로 재저장 | 1,710,915 bytes, `diffCount: 0`, `identical: true` |
| 출력 구조 | 재저장 `Contents/section0.xml`의 curve 두 개 검사 | curve 내부 `hp:seg` 10개, `hc:pt` 0개 |
| ZIP 무결성 | `unzip -t` | 모든 엔트리 통과 |

메인터너 보정 전후의 같은 실제 HWPX 재저장 산출물은 SHA-256
`1ec49710ac9c9eae2b822227e4ad5b0f901a1efe08d89deade8c0d1c2943a122`로 동일했다. 따라서
보정은 #4676의 한글 호환 writer 바이트를 바꾸지 않고 parser와 renderer 사이의 잘못된 의미
전달만 제거한다.

추가한 XML→IR→XML 회귀는 HWPX `CURVE` 세 구간을 재저장한 뒤에도 segment 세 개와 네 점을
유지하고 HWP5 `segment_types`가 비어 있음을 확인한다. 즉 parser가 다시 `1`을 넣거나 serializer가
curve를 `hc:pt`로 되돌리는 두 회귀를 함께 차단한다.

renderer 소스나 조판 규칙은 바뀌지 않았다. parser/serializer 구조 보존 변경이므로 PDF pixel
sweep은 적용하지 않았고, 실제 HWPX 구조와 자기 재파싱으로 판단했다. 독립 한글 2022 COM 재개방은
검증 호스트의 COM factory가 `0x80080005`로 실패해 이 작업에서 재실행하지 못했다. 이는 생성
파일의 실패 판정이 아니며, contributor가 #4676에 기록한 17/17 오라클 결과는 writer 바이트가
동일한 이 후보에도 그대로 적용된다.

## 잔여 범위와 판단

HWPX의 `LINE|CURVE` 구분을 HWP5 Bezier 구간 타입과 공용 필드 하나로 표현할 수는 없다. native
HWPX segment 의미를 보존하려면 제어점 형식이 다른 별도 IR 모델이 필요하며, 이번 크래시 수정에서
잘못된 `1` 매핑을 유지할 수는 없다.

**통합 수용 권고.** 최신 `devel` 위 후보에 code/test 보정이 포함됐으므로 review-only fast-pass를
적용하지 않는다. 기존 PR #4684의 최신 code head에서 Full CI와 CodeQL을 확인하고 작업지시자 승인을
받은 뒤 merge한다. merge 뒤 원 PR #4684에는 충돌 해소 commit과 메인터너 보정 이유를 남기고 close하며,
#4676의 close 조건은 한글 2022 오라클 재개방 범위까지 재확인한다.
