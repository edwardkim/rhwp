# Stage 1 보고 — Task M100-1196

## 범위

- 이슈: [#1196](https://github.com/edwardkim/rhwp/issues/1196)
- 단계: 재현 고정과 baseline 기록
- 브랜치: `local/task1196`
- 기준 커밋: `62f3bf74`

## 목적

#1276 및 후속 바탕쪽 번호 배치 보정이 반영된 최신 기준에서, #1196 결함이 여전히 남아 있는지 확인했다.

확인 대상:

- 대상 HWPX가 46페이지로 페이지네이션되는지
- page 4가 section 1 본문 시작 `page_num=4` 상태를 유지하는지
- page 4/5/6의 `body_area.x`가 아직 동일해 맞쪽 편집 여백 교대가 미적용인지
- HWPX `gutterType="LEFT_RIGHT"` 파서는 이미 `DuplexSided`로 동작하는지
- #1271 회귀 테스트가 통과하는지

## dump-pages 결과

대상 문서:

```text
samples/hwpx/[2027] 온새미로 1 본교재.hwpx
```

명령:

```text
cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 3
cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 4
cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 5
```

결과 요약:

```text
문서 로드: 46페이지

page 4: global_idx=3, section=1, page_num=4
  body_area: x=94.5 y=113.4 w=510.2 h=895.8
  첫 문단: "강의 01."

page 5: global_idx=4, section=1, page_num=5
  body_area: x=94.5 y=113.4 w=510.2 h=895.8

page 6: global_idx=5, section=1, page_num=6
  body_area: x=94.5 y=113.4 w=510.2 h=895.8
```

판단:

- #1276 이후 앞부분 페이지 대응은 유지된다.
- page 4가 section 1 본문 시작이며 `page_num=4`로 잡힌다.
- page 4/5/6의 `body_area.x`가 모두 `94.5`로 동일하다.
- 따라서 #1196의 핵심 증상인 `LEFT_RIGHT` 맞쪽 편집 여백 교대 미적용이 재현된다.

## 파서 테스트

명령:

```text
cargo test --lib test_parse_page_pr_gutter_type_materializes_hwp5_binding_attr
```

결과:

```text
test parser::hwpx::section::tests::test_parse_page_pr_gutter_type_materializes_hwp5_binding_attr ... ok
test result: ok. 1 passed; 0 failed
```

판단:

- HWPX `pagePr@gutterType="LEFT_RIGHT"`를 `BindingMethod::DuplexSided`로 materialize하는 파서 경로는 이미 통과한다.
- 이번 이슈의 구현 범위는 파서가 아니라 레이아웃 적용 쪽이다.

## #1271 회귀 테스트

명령:

```text
cargo test --test issue_1271_hwpx_behind_text_table -- --nocapture
```

결과:

```text
running 3 tests
test onsaemiro_front_matter_is_not_shifted_by_behind_text_table_fragment ... ok
test onsaemiro_odd_master_page_number_stays_after_title_text ... ok
test onsaemiro_master_page_bottom_textbox_is_not_microscopic ... ok

test result: ok. 3 passed; 0 failed
```

판단:

- #1271에서 고정한 글뒤로 표 분할/바탕쪽 홀짝 관련 회귀는 없다.
- #1196 구현은 이 상태를 유지해야 한다.

## Stage 1 결론

- baseline 재현 완료.
- `gutterType` 파서는 동작한다.
- `PageContent.layout.body_area.x`는 최종 page 4/5/6에서 아직 교대되지 않는다.
- Stage 2에서는 `PageAreas` / `PageLayoutInfo`의 페이지별 여백 계산 API와 단위 테스트를 추가한다.
