# Stage 202: HWP3 미주 저장 줄 범위 복원

## 목적

Stage 156의 HWP3 변환 본문 orphan `LineSeg` 보정이 미주 흐름까지 fresh reflow를
적용해, `issue_1139_inline_picture_duplicate`의 PDF 기준 쪽/단 경계를 밀어낸 회귀를
원인 단위로 분리한다.

## 증거

1. `upstream/devel`을 good, Stage 200(`b84df94cc`)을 bad로 한 이진 탐색에서
   `195cc2a355ded9136c968a5c5f871da39a09f510`이 최초 회귀로 확정됐다.
2. 해당 커밋은 HWP3 변환본에서 `fresh lines < stored lines`이면 문단 위치와 무관하게
   `stored_body_lines_stale`을 true로 만들었다.
3. `RHWP_DIAG_REWRAP=1`로 실패 사례를 재현하면 미주 단 너비에서
   `stored=13, fresh=12` 등 저장 줄보다 짧은 fresh 결과가 반복됐다. 미주의 저장
   `LineSeg`는 본문 폭 추정치가 아니라 이미 기록된 쪽/단 흐름을 표현한다.

## 변경 계약

`stored_body_lines_stale`과 `recompose_stale_body_lines`에 `hwp3_body_reflow` 권한을
명시적으로 전달한다.

1. 일반 본문은 기존과 같이 HWP3 변환본에서만 권한을 켠다. 따라서 Stage 156의
   orphan tail 보정은 유지한다.
2. 미주 조판, 미주 scratch layout, 그리고 `endnote_para_sources`로 식별되는 가상
   미주 렌더 문단은 권한을 끈다. HWP3 스타일의 폰트/간격 규칙은 그대로 유지하고,
   stale 판정의 fresh 줄 수 축소만 막는다.
3. `HeightMeasurer`, `TypesetEngine`, `LayoutEngine`이 같은 권한을 사용해 측정,
   페이지네이션, 렌더링의 줄 수가 달라지지 않게 한다.

문단 지문, fixture 이름, 페이지 번호, 단 너비, 픽셀 여유값을 판정에 사용하지 않는다.

## 검증 명령

```sh
cargo test --profile release-test --test issue_1139_inline_picture_duplicate -- --nocapture
cargo test --profile release-test --test issue_1035_alignment --test issue_1105 --test issue_1086 --test issue_3820_body_top_table_border_clip --test issue_3820_rowbreak_rowspan_band --test issue_3820_stored_reset_fragment_geometry --test issue_3820_tac_caption_first_text_owner --test issue_3930_hwpx_hwp_save_layout -- --nocapture
cargo test --profile release-test --lib && cargo test --profile release-test --tests
```

## 상태

구현 완료.

`cargo test --profile release-test --test issue_1139_inline_picture_duplicate -- --nocapture`에서
Stage 156 이후의 23건 중 미주 저장 줄 재조판으로 발생한 20건이 해소되어 `82 passed,
3 failed`가 되었다. 남은 세 건은 HWP3 본문에서 TAC/어울림 그림이 줄 폭을 좁히는
별도 흐름이며, 다음 Stage에서 본문 stale 재조판과 그림 앵커의 공존 계약으로 다룬다.
