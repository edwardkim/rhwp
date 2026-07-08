# PR #2033 리뷰 - floating 그림 restrictInPage 하단 클램프

- PR: #2033 `Task #2032: floating 그림 restrictInPage 하단 클램프 — off-page 완전 소실 수정`
- URL: https://github.com/edwardkim/rhwp/pull/2033
- 작성자: `lpaiu-cs`
- 작성자 관계: `FIRST_TIME_CONTRIBUTOR`
- 기준 브랜치: `devel`
- head branch: `lpaiu-cs:fix/picture-offpage-loss`
- 문서 작성 시점 참고 head: `c8c9ed1eec46ff1806f84b7b2783baaaf780d130`
- 관련 이슈: #2032, 배경 이슈 #2027
- reviewer: `jangster77` 지정 완료
- merge commit: `2b213ab706f7824f32bbb64b06831d9b794bce5d`
- 처리 방식: 옵션 2. 원 코드 PR merge 후 review 문서와 대표 visual asset 을 별도 docs-only 후속 PR 로 반영

## 결론

승인 및 merge 완료.

PR 은 `vertRelTo=Para` + `restrictInPage=true` 인 floating 그림이 큰 `vertOffset` 때문에
페이지 캔버스 아래로 완전히 사라지는 결함을 좁은 조건으로 막는다. 수정 범위가
`layout_body_picture` 의 Para-relative floating picture 하단 clamp 에 한정되어 있고,
`restrictInPage=false`, `vpos_accounts_for_height`, `pr-149` gap-accounting 회귀 축을 테스트로
분리한 점이 좋다.

first-time contributor PR 이므로 공개 코멘트는 환영 인사와 함께, merge 판단 근거와 남은 범위 외 항목을
구체적으로 남기는 방식이 적절하다.

## 변경 요약

- `src/renderer/layout/picture_footnote.rs`
  - `!treat_as_char && flow_with_text && vert_rel_to == Para && !vpos_accounts_for_height` 조건에서
    그림 프레임 전체 높이(`total_height`, 캡션 포함)가 현재 column body 하단을 넘으면 `base_y` 를 위로
    clamp 한다.
  - 상단 top bleed 는 기존 table 예외와 같은 취지로 허용한다.
  - `vpos_accounts_for_height` 경로는 기존 #1079 보정처럼 clamp 비대상으로 둔다.

- `tests/issue_2032_picture_offpage_restrict_loss.rs`
  - `restrictInPage=true` + off-page offset: 그림 ImageNode 가 페이지 안에 남는지 확인.
  - `restrictInPage=true` + partial overflow: 하단이 페이지 하단을 넘지 않는지 확인.
  - `restrictInPage=false`: 페이지 밖 좌표가 안쪽으로 clamp 되지 않는지 확인.

## 코드 리뷰 메모

- clamp 위치가 캡션 offset 계산 전에 있어 top/bottom/side caption 을 포함한 전체 frame 기준으로 동작한다.
- `base_y.min(body_bottom.max(col_area.y))` 형태라 하단 초과만 교정하고, 기존 top bleed 는 보존한다.
- `TextWrap::BehindText` / `InFrontOfText` 는 flow cursor advance 에는 영향을 주지 않지만, `flow_with_text`
  가 켜진 Para-relative 개체의 하단 제한은 동일하게 적용된다. PR 설명의 restrictInPage 의미와 어긋나지 않는다.
- 새 테스트는 문제 재현을 synthetic picture 삽입으로 직접 고정한다. PR 본문에서 언급한 ta-pic 쌍은
  한컴 동작 근거와 범위 외 셀 앵커 비교 자료로 보는 것이 맞다.

## 검증

- `cargo fmt --check`: 통과.
- `git diff --check upstream/devel...HEAD`: 통과.
- `CARGO_INCREMENTAL=0 cargo test --test issue_2032_picture_offpage_restrict_loss -- --nocapture`: 통과, 3 passed.
- `CARGO_INCREMENTAL=0 cargo test --lib test_task683_pr149_image_cluster_spacing -- --nocapture`: 통과, 1 passed.
- `CARGO_INCREMENTAL=0 cargo test --test issue_1079_picture_pushdown_vpos -- --nocapture`: 통과, 2 passed.
- `CARGO_INCREMENTAL=0 cargo test --test issue_2027_picture_wrap_toggle_loss -- --nocapture`: 통과, 4 passed.
- `CARGO_INCREMENTAL=0 cargo build --bin rhwp`: 통과.
- `wasm-pack build --target web --out-dir pkg`: 통과.
- GitHub Actions: PR head `c8c9ed1eec46ff1806f84b7b2783baaaf780d130` 기준 CI, CodeQL, Render Diff 통과 확인.
- 이전 head `b7aea4734f4a2f066ac99349d03a226d58961f3a` 의 CI/CodeQL/Render Diff run 은 모두 completed/success 로
  남은 취소 대상 없음.

## 시각 검증

`mydocs/manual/visual_sweep_guide.md` 에 따라 PR 본문이 한컴 동작 근거로 든 ta-pic on/off 쌍을 확인했다.

```bash
python3 scripts/task1274_visual_sweep.py \
  --file-target ta-pic-restrict-on samples/ta-pic-001-r-쪽영역안제한.hwp pdf/ta-pic-001-r-쪽영역안제한-2024.pdf \
  --file-target ta-pic-restrict-off samples/ta-pic-001-r-쪽영역안제한no.hwp pdf/ta-pic-001-r-쪽영역안제한no-2024.pdf \
  --out output/pr2033_visual
```

| 대상 | 페이지 수 | 자동 후보 | proxy | 대표 파일 |
|------|-----------|-----------|-------|-----------|
| restrict on | SVG 1 / PDF 1 | 0/1 | 20.49541% | `mydocs/pr/assets/pr_2033_ta_pic_restrict_on_review_p001.png` |
| restrict off | SVG 1 / PDF 1 | 0/1 | 37.42514% | `mydocs/pr/assets/pr_2033_ta_pic_restrict_off_review_p001.png` |

두 샘플 모두 자동 후보는 없다. 다만 ta-pic 쌍은 그림 세로 위치와 폰트/래스터 차이가 커서 완전 시각 일치
판정 자료로 쓰기보다는, PR 설명처럼 `restrictInPage` on/off 동작의 한컴 기준 근거와 범위 외 셀 앵커 차이를
확인하는 보조 자료로 보는 것이 안전하다. 이번 PR 의 직접 결함은 새 #2032 통합 테스트가 더 정확히 고정한다.

## 범위 외 확인

PR 본문이 남긴 후속 후보는 타당하다.

- shape 경로의 동일 결함은 이번 PR 범위 밖이다.
- `restrictInPage=false + 자리차지` 시 본문 push-down 은 이번 PR 범위 밖이다.
- 셀 앵커 그림의 행 확장은 ta-pic 기준 PDF에서 여전히 별도 경로로 남는다.

## 공개 review 기록

PR merge 전에 다음 내용으로 approve review 를 게시했다.

```text
@lpaiu-cs rhwp 첫 PR 감사합니다.

검토 결과, 이번 PR은 merge 후보로 봅니다. `vertRelTo=Para` + `restrictInPage=true` floating 그림이 큰 `vertOffset` 때문에 페이지 아래로 완전히 사라지는 문제를 좁은 조건으로 막고 있고, `restrictInPage=false`와 기존 `pr-149` gap-accounting 경로를 분리해 회귀 테스트로 확인한 점이 좋았습니다.

확인한 내용은 다음과 같습니다.

- GitHub Actions: CI, CodeQL, Render Diff 통과
- 로컬 검증: `cargo fmt --check`, `git diff --check`, `cargo test --test issue_2032_picture_offpage_restrict_loss`, `cargo test --lib test_task683_pr149_image_cluster_spacing`, `cargo test --test issue_1079_picture_pushdown_vpos`, `cargo test --test issue_2027_picture_wrap_toggle_loss`, `cargo build --bin rhwp`, `wasm-pack build --target web --out-dir pkg`
- visual sweep: PR 본문에서 근거로 든 ta-pic on/off 기준 PDF 쌍은 SVG/PDF 모두 1쪽이고 자동 후보 0/1로 확인했습니다.

ta-pic 쌍은 셀 앵커/행 확장 등 별도 경로 차이가 남아 있어서 완전 시각 일치 자료라기보다는 `restrictInPage` on/off 동작 근거로 보는 것이 맞고, 이번 PR의 직접 회귀는 새 `tests/issue_2032_picture_offpage_restrict_loss.rs`가 잡는 것으로 판단했습니다.

shape 경로, `restrictInPage=false + 자리차지` push-down, 셀 앵커 그림의 행 확장은 PR 본문에 적어주신 대로 후속 범위로 남겨두면 됩니다.
```
