# PR #4931 검토 - kevin9327 공개 PR 통합 및 출력 계약 보정

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4931](https://github.com/edwardkim/rhwp/pull/4931) |
| 관련 PR | [#4919](https://github.com/edwardkim/rhwp/pull/4919)~[#4924](https://github.com/edwardkim/rhwp/pull/4924) |
| 작성자 | `jangster77` (`Taesup Jang`) |
| 검토 방식 | collaborator self-review, kevin9327 공개 PR 누적 통합 및 메인터너 보정 |
| base / head | `devel` / `integrate/kevin9327-open-prs-20260816` |
| code candidate | `210b3ee377944088c28191da0967469a9fc8abc1` |
| review 기록 전 head | `4560ee432b83d6b6471a56441f099cc36bfafe60` |
| 규모 | 작성 시점 9 commits, 39 files, +7,148 / -68 |
| 작성 시점 상태 | `MERGEABLE`, `BLOCKED` (최신 CI 대기) |

collaborator self PR이므로 reviewer는 지정하지 않았다. `mergeable`, `mergeStateStatus`, head SHA와 CI 결과는
작성 시점의 참고값이며, 최종 merge 전에 이 trailing 기록을 포함한 최신 head에서 다시 확인한다.

## 개별 원 PR 검토 기록

- [#4919 검토](pr_4919_review.md): service 공통 문서 열기·조회 축
- [#4920 검토](pr_4920_review.md): render backend 공통 trait 계층
- [#4921 검토](pr_4921_review.md): 문서 의미 diff 라이브러리
- [#4922 검토](pr_4922_review.md): CAS 판정과 재계획 hint
- [#4923 검토](pr_4923_review.md): CI agent preflight 배선
- [#4924 검토](pr_4924_review.md): 실물 대형 문서 scale ladder

각 원 PR의 source head, 변경 범위와 원 PR CI 결과는 위 개별 기록에 남겼다. #4931의 전체 회귀는 이들을
한 tree로 누적한 결과의 최종 검증이며, 원 PR을 직접 merge하는 판단을 대체하지 않는다.

## 체리픽 범위와 판단

- [#4919](https://github.com/edwardkim/rhwp/pull/4919)의 service 공통 축, [#4920](https://github.com/edwardkim/rhwp/pull/4920)의
  render backend trait, [#4921](https://github.com/edwardkim/rhwp/pull/4921)의 `docdiff`,
  [#4922](https://github.com/edwardkim/rhwp/pull/4922)의 CAS 판정 계약,
  [#4923](https://github.com/edwardkim/rhwp/pull/4923)의 agent preflight CI 배선,
  [#4924](https://github.com/edwardkim/rhwp/pull/4924)의 실물 scale ladder 측정을 최신
  `upstream/devel` 위에 오래된 번호 순으로 누적했다.
- [#4925](https://github.com/edwardkim/rhwp/pull/4925)~[#4927](https://github.com/edwardkim/rhwp/pull/4927),
  [#4929](https://github.com/edwardkim/rhwp/pull/4929), [#4930](https://github.com/edwardkim/rhwp/pull/4930)은
  이미 `upstream/devel`의 조상임을 확인해 중복 체리픽하지 않았다. #4928은 Open non-draft 대상에 없었다.
- `SvgBackend`는 clip을 평면화하고 여러 SVG root를 연결하면서도 해당 capability를 지원한다고 선언하던
  계약 불일치를 보였다. 메인터너 보정은 `ClipRect`와 `MultiPage` capability 광고를 제거하고, 두 번째
  `begin_page`를 명시 오류로 거부해 생성 불가능한 SVG를 반환하지 않게 했다.
- service text `char_offset`의 논리 좌표와 scale ladder의 최대 RSS 플랫폼 범위를 문서·출력 계약과 맞췄다.
- PR merge 뒤 검토만 수행한 local worktree도 종료 정리 대상임을 workflow와 post-merge 절차에 명시했다.

## 렌더 영향과 시각 검증 판정

`src/render_backend`의 새 reference adapter와 capability 계약은 변경됐지만, 기존 renderer/layout/typeset,
페이지 geometry, HWP/HWPX fixture, PDF 기준 자료, golden은 바뀌지 않았다. 따라서 별도 PDF/시각 sweep은
이번 통합 PR의 필수 근거가 아니며, SVG backend의 단일 쪽·지원 capability는 새 Rust 계약 테스트와 전체
회귀로 검증했다.

## 완료된 검증

- `cargo fmt --check`를 통과했다.
- `python3 -m py_compile tools/scale_ladder_real.py`를 통과했다.
- `cargo clippy -- -D warnings`를 통과했다.
- 전용 target `target/pr-review-kevin9327-integration-20260816`에서
  `cargo test --profile release-test --tests`를 종료 코드 `0`으로 완료했다. 라이브러리 단위 4,020건 통과와
  전체 integration test binary의 통과를 확인했다.
- `git diff --check`를 통과했다.
- `mydocs/manual/pr_review_workflow.md`와 `mydocs/manual/pr_review/post_merge.md`의 내부 Markdown 상대 링크를
  검사해 이상 없음을 확인했다.
- 전체 문서 메타데이터 검사는 변경하지 않은 `mydocs/tech/benchmark_vs_alternatives.md`의 기존 front matter
  누락 4건으로 실패했다. 해당 파일은 `upstream/devel`과 동일하므로 이번 PR의 변경 범위 밖 기존 결함이다.

## 위험과 후속 범위

- 새 service, render backend, docdiff는 공통 API 축을 신설하는 단계이며 기존 CLI/MCP/WASM 표면의 전면 이관은
  포함하지 않는다. 실제 소비 표면 이관은 후속 PR에서 별도 계약 검증과 함께 진행해야 한다.
- `SvgBackend`의 multi-page 거부는 조용히 잘못된 다중 root SVG를 반환하는 기존 동작보다 안전한 실패로
  바꾸는 보정이다. multi-page SVG 산출이 필요하면 단일 문서 결합 규격을 별도 설계해야 한다.
- 추가 결함은 현재 검토 범위에서 발견하지 못했다.

## 최종 권고

merge를 권고한다. 이 review·오늘할일 trailing commit의 최신 GitHub Actions가 성공하고, merge 직전에 최신
head SHA, `MERGEABLE`, `CLEAN`을 재확인한 뒤 작업지시자가 승인한 collaborator self-merge를 수행한다.
