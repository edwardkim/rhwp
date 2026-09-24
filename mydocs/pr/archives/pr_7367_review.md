# PR #7367 self-review: #7339-#7343 renderer 통합 보정

## 최종 판정

**승인**. 이 PR의 제품 후보 `58769dd114985911527f038e872894d14d5cbfce`은 외부 contributor
PR #7339·#7340의 발견 사항을 제한적으로 보정하고, #7341·#7342·#7343을 최신 `devel` 기준으로
함께 검증한다. 이는 collaborator self-review이며 reviewer는 지정하지 않는다. 최신 trailing head의
GitHub Actions, mergeability와 작업지시자 merge 승인은 아직 미래 조건이다.

## 대상과 범위

| 항목 | 내용 |
| --- | --- |
| PR | [#7367](https://github.com/edwardkim/rhwp/pull/7367) |
| base / 후보 SHA | `upstream/devel` `7a95e46e025470a4d7a7b59ad68ec02958bda738` / `58769dd114985911527f038e872894d14d5cbfce` |
| branch | `integration/planet6897-7339-7343-20260923` |
| 포함 PR | #7339, #7340, #7341, #7342, #7343. #7338은 이미 `c1ac0f987`로 병합돼 재적용하지 않음 |
| 이슈 관계 | #6761, #7095, #7330은 잔여 축이 있어 `Refs`만 사용하며 자동 close하지 않음 |
| trailing 범위 | 이 파일과 `mydocs/orders/20260923.md`만 추가. 제품 코드, test, fixture, baseline, workflow는 바꾸지 않음 |

## Self-review와 완료 검증

- #7339의 실제 가로 배치 비교, #7340의 rewind 확정 길이 일치, #7338의 semantic-equal raw bit 보존을
  `a3f61dc`에서 보정했다. 수정 전 반례를 각각 Rust 계약 테스트로 고정했다.
- focused suite 653건, 전체 nextest 10,156건, fmt, `git diff --check`, native/WASM Clippy,
  workspace build/all-target Clippy, manifest 정책과 Native Skia 필수 경로를 통과했다.
- Docker와 `.env.docker`가 이 호스트에 없어 표준 Docker WASM은 실행하지 못했다. 대신 fresh no-opt
  WASM을 성공시켜 실제 HWP/HWPX Visual Sweep에 사용했다.
- #7339/#7340/#7343 chemical 5쪽, #7341 night-guard 1쪽, #7342의 두 실제 HWP 7쪽을 직접 확인했다.
  #7342 7062 4쪽의 미주 gap 후보는 base `-154.6px`, 후보 `-154.5px`로 동일했다.
- 상세 판정, 입력·PDF SHA-256, 대표 asset과 merge 후 contributor PR comment 계획은
  [통합 기록](pr_7339_7343_review_impl.md) 및 각 [개별 review](pr_7339_review.md)에 있다.

## PR 본문 증적 확인

PR 생성 시 후보 SHA `58769dd`로 고정한 대표 Visual Sweep PNG 6장을 본문 Markdown에 넣었다.
이 trailing commit을 push한 뒤에는 새 head SHA로 URL을 갱신하고, GitHub API와 PR 화면에서 본문·이미지
표시를 다시 확인한다. asset은 이미 이 PR의 후보 commit에 포함되며 raw raster, SVG, JSON 및 실행 로그는
포함하지 않는다.

## Merge 전 조건

1. trailing head의 required GitHub Actions가 성공해야 한다.
2. 최신 head SHA, `MERGEABLE`/`CLEAN`, PR 본문 asset URL을 재확인해야 한다.
3. 작업지시자의 별도 merge 승인이 있어야 한다.
