# PR #1500 시리즈 통합 검토 - render-diff 게이트와 HWPX 직렬화 충실도

- 작성일: 2026-06-24
- 검토 시각: 2026-06-24 18:29 KST
- 대상 PR: #1500, #1502, #1507
- 비대상: 별도 foundation 후보 PR
- 관련 이슈: #1499, #1501, #1505, #1506
- 작성자: [@planet6897](https://github.com/planet6897)
- 처리 기준 브랜치: `pr1500-series-integration`
- 통합 PR: #1511
- 통합 방식: 원 PR non-merge 커밋 cherry-pick + maintainer 보정 커밋 1개
- 최종 push/PR comment 조건: 작업지시자 승인

## 1. 요약 판단

**#1500 -> #1502 -> #1507 시리즈는 통합 처리 후보로 적합**하다.

세 PR은 순서가 있는 stack이다. #1500이 `render-diff` 시각 정합성 게이트를 만들고, #1502/#1507은 그
게이트가 검출한 HWPX 직렬화 누락을 단계적으로 줄인다. 로컬에서 이 순서로 cherry-pick했을 때 충돌은 없었다.

별도 foundation 후보 PR은 이번 시리즈에서 제외한다. 작업지시자 지시에 따라 해당 PR에는 GitHub comment,
review, close, cross-reference를 남기지 않는다.

## 2. 적용 커밋

`local/devel` 위에 적용한 커밋:

| 새 SHA | 출처 | 원 커밋 | 제목 |
|--------|------|---------|------|
| `64ba6468` | #1500 | `1fc7f8d0` | `Task #1499: HWPX 라운드트립 시각 정합성 게이트 (render-diff)` |
| `65ab3e03` | #1500 | `a6ec47b4` | `Task #1499: rustfmt 정렬 (CI fmt 체크)` |
| `19230f82` | #1500 | `88e9c302` | `render-diff 게이트 보강: 구조 불일치 노드 타입 델타 출력` |
| `e3cf5409` | #1502 | `c3b1a51e` | `Task #1501: HWPX 묶음 개체 자식 좌표 변환 보존` |
| `104e10b7` | #1502 | `ae100f23` | `HWPX 직렬화: 레거시 도형 shape_attr 블록 보존` |
| `a50bfe3d` | #1507 | `d547bea7` | `HWPX 직렬화: 쪽 테두리 pageBorderFill 보존` |
| `95f558f1` | #1507 | `4c1c9464` | `HWPX 직렬화: 바탕쪽 MasterPage 직렬화 구현` |
| `31881776` | maintainer | - | `maint: render-diff gate failure semantics 보강` |

## 3. 변경 범위

| 범위 | 주요 파일 | 내용 |
|------|-----------|------|
| 시각 게이트 | `src/diagnostics/render_geom_diff.rs`, `tests/visual_roundtrip_baseline.rs` | RenderNode bbox diff, batch TSV, struct_delta, visual baseline |
| CLI 등록/문서 | `src/main.rs`, `src/diagnostics/mod.rs`, `mydocs/manual/cli_commands.md` | `render-diff` 명령 등록, 비-PASS exit code 문서화 |
| 그룹/도형 직렬화 | `src/serializer/hwpx/picture.rs`, `src/serializer/hwpx/shape.rs`, `src/serializer/hwpx/section.rs` | shape_attr, raw_rendering, 레거시 도형 회전/뒤집힘 보존 |
| 쪽 테두리/바탕쪽 | `src/serializer/hwpx/content.rs`, `src/serializer/hwpx/master_page.rs`, `src/serializer/hwpx/mod.rs`, `src/serializer/hwpx/section.rs` | pageBorderFill, MasterPage XML/manifest/secPr 직렬화 |

## 4. Maintainer 보정

원 #1500 계열에는 두 가지 리뷰 지적이 있었다.

1. `render-diff` CLI가 `OVER`/`STRUCT_MISMATCH`에서도 exit 0으로 종료해 자동화 gate로 쓰기 어렵다.
2. `VISUAL_XFAIL` 테스트가 `is_err()`만 확인해 known drift 샘플이 더 심한 실패로 악화되어도 통과할 수 있다.

통합 브랜치에서 다음을 보정했다.

- `PASS` 외 status는 batch/single 모두 exit 1.
- `non_pass_statuses_are_hard_failures` 단위 테스트 추가.
- visual xfail은 기록 사유에서 기대 failure class를 유도하고, 현재 실패 종류가 바뀌면 실패.
- CLI 매뉴얼에 exit code 정책 명시.

## 5. 로컬 검증

targeted 검증:

| 명령 | 결과 |
|------|------|
| `cargo fmt` | 통과 |
| `cargo fmt --check` | 통과 |
| `git diff --check` | 통과 |
| `cargo test --profile release-test --lib diagnostics::render_geom_diff -- --nocapture` | 통과, 8 passed |
| `cargo test --profile release-test --test visual_roundtrip_baseline -- --nocapture` | 통과, 3 passed |

전체 merge 후보 검증:

| 명령 | 결과 |
|------|------|
| `cargo build --release` | 통과 |
| `cargo test --release --lib` | 통과, 1934 passed / 6 ignored |
| `cargo test --profile release-test --tests` | 통과 |
| `cargo fmt --check` | 통과 |
| `git diff --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `cargo test --doc` | 통과, 0 passed / 1 ignored |
| `wasm-pack build --target web --out-dir pkg` | 통과 |
| `cd rhwp-studio && npx tsc --noEmit` | 통과 |
| `cd rhwp-studio && npm test` | 통과, 147 passed |

검증 중 이전 세션에서 남은 중복 cargo/clippy 프로세스가 build directory lock을 잡아 종료했고, 최종 foreground
검증 명령은 모두 exit 0으로 완료했다.

## 6. GitHub 처리 방침

- #1500, #1502, #1507: 통합 branch PR로 대체 반영 후, 원 PR에는 cherry-pick 반영 코멘트와 함께 close 예정.
- 별도 foundation 후보 PR: 이번 시리즈가 아니므로 GitHub상 언급 없이 보류.
- 관련 이슈 #1499, #1501, #1505, #1506은 integration PR merge 후 close 상태를 확인한다.

## 7. 권고

작업지시자 승인 후 `pr1500-series-integration`을 원본 저장소 임시 branch로 push했고, `devel` 대상 단일 PR
#1511을 생성했다. 이 방식은 개별 contributor PR branch에 review 문서를 올리지 않아 CI 재실행을 반복하지 않는다.
