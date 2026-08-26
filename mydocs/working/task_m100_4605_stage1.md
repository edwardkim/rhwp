# #4605 Stage 1 — 죽은 측정 상태 정리와 폴백 전용 표기

- **Issue**: [#4605](https://github.com/edwardkim/rhwp/issues/4605)
- **브랜치**: `fix/renderer-measurement-unification`
- **기준**: `upstream/devel` `4f9e4ae69`

## 1. 사실 확인

### 1.1 `MeasuredParagraph::has_picture` / `picture_height` — 읽는 곳 0건

```
$ grep -rn "\.has_picture\b\|\.picture_height\b" --include="*.rs" src/ tests/
(0건 — 계산·저장만 하고 소비자가 없다)
```

두 필드는 `measure_section`(`:583`) · `measure_section_incremental`(`:2483`) ·
`measure_section_selective`(`:2584`) 세 곳에서 계산돼 `measure_paragraph` 인자로 전달되고,
`measure_paragraph` 본문은 두 값을 **구조체 리터럴에 그대로 담기만 한다**(`:945-955`, 술어로도
쓰지 않는다). 유일한 생산자 `measure_pictures_in_paragraph`(`:979`)도 이 셋 외에는 호출부가 없다.

### 1.2 `MeasuredSection.paragraphs` — 프로덕션 페이지네이션이 읽지 않는다

`DocumentCore::paginate_pass`(`queries/rendering.rs:4260`)가
`TypesetEngine::typeset_section_with_variant` 에 넘기는 측정값은 `measured.tables` 뿐이다.
본문 문단 높이는 `TypesetEngine::format_paragraph` 가 자기 안에서 다시 만든다.

`paragraphs` 를 읽는 곳은 셋이다.

| 소비자 | 위치 | 조건 |
|---|---|---|
| `Paginator::paginate_with_measured_opts` | `rendering.rs:4180` | `RHWP_USE_PAGINATOR=1` |
| `Paginator::paginate_with_measured` | `rendering.rs:3990` | 구역 0개 문서의 빈 결과 |
| `dump-pages` 진단(JSON/텍스트) | `rendering.rs:5226`, `:5534` | 진단 전용 |
| 증분 측정의 자기 캐시 | `height_measurer.rs:2517` | 재측정 스킵 |

## 2. 결정 — `RHWP_USE_PAGINATOR` 폴백은 유지한다

이슈가 "필요 없으면 `Paginator` 경로와 필드가 같이 정리된다"고 열어 둔 선택지를 **택하지 않았다.**
근거 셋:

1. **`Paginator` 는 env 게이트 전용이 아니다.** `rendering.rs:3990` 의 구역 0개 문서 경로가
   `paginate_with_measured` 를 무조건 부른다. 지우려면 이 경로도 `TypesetEngine` 으로 옮겨야 하고,
   그건 별개의 동작 변경이다.
2. **회귀 비교 기준선으로 실제 쓰인다.** `mydocs/troubleshootings/*` 와 `mydocs/plans/*` 에서
   `RHWP_USE_PAGINATOR=1` 을 "옛 Paginator 로 fallback (회귀 비교 기준)"으로 지목한 문서가 30건이 넘고,
   `typeset.rs` 테스트가 `Paginator::with_default_dpi()` 를 대조군으로 직접 부른다(6곳).
3. **삭제 규모가 이 커밋의 위험 등급을 벗어난다.** `pagination/engine.rs` 2,920줄 + `typeset.rs`
   테스트 재작성이다. #4605 는 "수정 위치를 두 번 오도했다"는 비용을 없애는 이슈이고, 그 비용은
   이름으로 없앨 수 있다.

따라서 이슈가 제시한 두 번째 선택지 — **이름이 폴백 전용임을 말하게 한다** — 를 택했다.

## 3. 변경

1. `MeasuredParagraph::has_picture` · `picture_height` 삭제. 세 계산 지점,
   `measure_paragraph` 의 두 인자, `measure_pictures_in_paragraph`(유일한 생산자)까지 함께 지웠다.
2. `MeasuredSection.paragraphs` → **`fallback_paragraphs`**. 필드 doc 에 위 §1.2 표의 소비자 셋과
   "본문 문단 높이를 바꾸려면 `format_paragraph`/`layout_partial_paragraph` 를 고쳐야 한다"를 적었다.
   이 필드를 읽는 세 접근자(`get_paragraph_height`·`get_measured_paragraph`·`paragraph_has_table`)
   에도 같은 표기를 달았다.

## 4. 검증 — 컴파일러가 증거다

이 커밋은 **순수 삭제 + 이름 변경**이다. 동작을 바꾸는 줄이 없으므로 새 회귀 테스트를 만들지 않았다.

- 삭제한 두 필드에 소비자가 없다는 주장은 `cargo check --all-targets` 가 통과하는 것으로 증명된다.
  소비자가 하나라도 있었으면 E0609 로 멈춘다.
- 이름 변경이 모든 소비자에 반영됐다는 주장도 같은 근거다. 실제로 첫 `cargo check` 는
  `height_measurer.rs:3171-3172` 의 기존 단위 테스트 두 줄을 E0609 로 잡아냈다(수정 후 통과).

```
$ cargo fmt --all && cargo check --profile release-test --all-targets
(error·warning 0건)
```

## 5. 남긴 것

`Paginator`(`pagination/engine.rs`, 2,920줄)와 `RHWP_USE_PAGINATOR` 게이트는 그대로 둔다.
이 경로의 존폐는 구역 0개 문서 경로 이관과 함께 판단해야 하는 별개 결정이다.
