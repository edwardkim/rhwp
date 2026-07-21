# Task #2715 최종 보고서 — 그리기 도형·묶음·차트 캡션 HWP5 직렬화 누락

- 이슈: [#2715](https://github.com/edwardkim/rhwp/issues/2715) / 브랜치: `task/m100-2715-hwp5-drawing-caption` (base `origin/devel` `49f38446`)
- 관련: #1403(HWPX 캡션 방출), #1387(표 캡션 공유), #1283·#2696(OLE base-only 계약 — 본 작업 범위 밖)
- 변경 파일: `src/serializer/control.rs`, `src/serializer/control/tests.rs`

## 결론 요약

`serialize_shape_control` 의 **10개 arm 중 캡션(`LIST_HEADER`)을 방출하는 arm 이 하나도 없어**
그리기 도형·묶음·차트 캡션이 HWP5 저장에서 전량 소실됐다. 파서는 전 변종의 캡션을 적재하고
(`parser/control/shape.rs:205/213/222/230/240`), 같은 파일에서 표(`:492`)·그림(`:998`)은
`serialize_caption` 을 호출하는데 도형 계열만 비어 있었다.

한컴 저장본 실측에서 `$rec`(사각형)·`$con`(묶음)이 `$pic`(그림)과 **동일한 30B `LIST_HEADER`** 를
`SHAPE_COMPONENT` 앞에 갖는 것을 확인했으므로 포맷 제약이 아니라 미구현이다. 8개 arm에 방출을
추가했다.

| 코퍼스 | 도형 캡션 | 표 캡션 | 그림 캡션 |
|---|---:|---:|---:|
| `samples/*.hwp` 21개 파일 (수정 전) | **38 → 0** | 597 → 597 | 13 → 13 |
| `samples/*.hwp` 21개 파일 (수정 후) | **38 → 38** | 597 → 597 | 13 → 13 |
| `samples/hwpx/aift.hwpx` → HWP5 (수정 전) | **2 → 0** | 2 → 2 | 9 → 9 |
| `samples/hwpx/aift.hwpx` → HWP5 (수정 후) | **2 → 2** | 2 → 2 | 9 → 9 |

## 1. 문제

파서의 캡션 인식 규칙은 "`SHAPE_COMPONENT` **앞**의 `LIST_HEADER`"다
(`src/parser/control/shape.rs:134-147`). `SHAPE_COMPONENT` **뒤**의 `LIST_HEADER` 는 글상자로
별도 처리되므로(`:149-167`) 두 경로는 레코드 위치로 구분된다.

직렬화 쪽 `grep -n "serialize_caption" src/serializer/control.rs` 전수 결과는 3줄뿐이었다 —
정의(`:666`)와 표(`:492`)·그림(`:998`) 호출 2건. `serialize_shape_control` 안에는 0건.

| arm | 캡션 방출(수정 전) | IR 필드 |
|---|---|---|
| `Line` / `Rectangle` / `Ellipse` / `Polygon` / `Arc` / `Curve` | 없음 | `drawing.caption` |
| `Group` | 없음 | `group.caption` |
| `Chart` | 없음 | `chart.caption` |
| `Ole` | 없음 | `ole.caption` (본 작업 범위 밖 — §5) |
| `Picture(_)` | 해당 없음 | `serialize_picture_control` 위임 |

## 2. 분석

### 2.1 의도적 누락이 아님을 확인

- `git log -S "serialize_caption" -- src/serializer/control.rs` → **`f0f7f1a4`(Initial commit) 1건뿐.**
  도형 arm 에서 캡션 방출을 제거한 커밋이 없다. "호환성 때문에 뺀" 이력이 아니라 처음부터
  표·그림에만 구현된 미구현 구간이다. `-S "pic.caption"` / `-S "table.caption"` 도 동일.
- #1403 계열 커밋(`3baf8724`, `897066c8`, `2a176d09`)은 전부 HWPX 경로만 수정했다.
  메인테이너 검토 문서 `mydocs/pr/archives/pr_1406_review.md` §3.2 가 오히려
  **"HWP5 파서는 전 도형 캡션 적재 중"** 을 명시한다 — 적재는 되는데 HWP5 방출만 비어 있는
  현 상태와 일치.
- `mydocs/` 전체에서 "HWP5 도형 캡션 미방출" 계약 문서를 찾지 못했다.

### 2.2 한컴 실물 레코드 배치 (결정적 근거)

`samples/3-09월_교육_통합_2023.hwp` (한컴 저장, 캡션 `<보기>`):

```
tag=71  CTRL_HEADER      lv=1 sz=46  ctrl_id="gso "
  tag=72  LIST_HEADER      lv=2 sz=30      ← 캡션
  tag=66  PARA_HEADER      lv=2 sz=24
    tag=67  PARA_TEXT        lv=3 sz=10
    tag=68  PARA_CHAR_SHAPE  lv=3 sz=8
    tag=69  PARA_LINE_SEG    lv=3 sz=36
  tag=76  SHAPE_COMPONENT  lv=2 sz=239 comp_id="$rec"
```

`samples/draw-group.hwp` 는 동일 배치로 `comp_id="$con"`(sz=270), `samples/aift.hwp` 는
`comp_id="$pic"`(sz=196). **세 경우 모두 캡션 `LIST_HEADER` 가 30B 로 동일**하며, 이는 기존
`serialize_caption`(`control.rs:666-706`)이 방출하는 크기와 정확히 같다
(`n_para(2)+list_attr(4)+width_ref(2)+attr(4)+width(4)+spacing(2)+max_width(4)+예약(8)=30`).
해당 함수 주석도 `// 예약 필드 8바이트 (한컴 호환성: 원본 파일은 30바이트 LIST_HEADER)` 로
이미 이 계약을 명시하고 있었다 — **방출 함수는 준비돼 있고 호출만 없었다.**

### 2.3 `raw_stream` 지름길이 결함을 가리고 있었음

`serialize_section`(`src/serializer/body_text.rs:26-30`)은 원본 섹션 바이트를 그대로 반환한다:

```rust
if let Some(ref raw) = section.raw_stream {
    return raw.clone();
}
```

따라서 **파싱 직후 무편집 저장은 직렬화기를 타지 않아 손실이 관측되지 않는다.** 최초 계측에서
`38 → 38` 이 나와 결함이 없어 보였으나, 이는 직렬화기를 우회한 결과였다. 실사용 경로는
`src/document_core/commands/` 의 편집 명령들이 예외 없이 `raw_stream = None` 을 설정하므로
(`formatting.rs` 16곳, `header_footer_ops.rs` 9곳, `footnote_ops.rs` 6곳, `clipboard.rs` 4곳 등)
**문서를 한 글자라도 편집하면 그 섹션의 도형 캡션이 전부 사라진다.** HWPX 입력은 `raw_stream`
자체가 없어 무조건 손실된다.

§ 결론 요약 표의 수치는 `raw_stream` 을 무효화한 상태(=실사용 경로)의 계측이다.

## 3. 변경

`src/serializer/control.rs`:

1. `serialize_shape_control` 에 `emit_caption` 클로저 신설(`emit_ctrl_data` 옆). 방출 위치·OLE
   제외 사유·호출 순서 제약을 주석으로 고정.
2. 8개 arm(`Line`/`Rectangle`/`Ellipse`/`Polygon`/`Arc`/`Curve`/`Group`/`Chart`)에서
   `emit_caption(...)` 호출. 캡션 필드 해석은 `serializer/hwpx/roundtrip.rs:1322-1335`
   `shape_caption()` 과 동일 매핑(그리기 6종 → `drawing.caption`, Group/Chart → 각자 필드).

**호출 순서 제약**: `emit_top_level_synthesized_ctrl_data` **뒤**, `SHAPE_COMPONENT` push **앞**.
`parse_caption`(`parser/control.rs:417-462`)은 `records[0]` 을 LIST_HEADER 로, `records[1..]` 전체를
캡션 문단으로 넘기므로, 캡션과 `SHAPE_COMPONENT` 사이에 `CTRL_DATA` 가 끼면 문단 파싱이 오염된다.

`src/serializer/control/tests.rs`: 회귀 테스트 4건 + 픽스처 헬퍼 2개 추가.

## 4. 검증

### 4.1 red→green (실제 실행 캡처)

`git stash push -- src/serializer/control.rs` 로 수정만 되돌린 상태 (**RED**):

```
running 4 tests
test serializer::control::tests::issue2715_shape_without_caption_emits_no_list_header ... ok
test serializer::control::tests::issue2715_caption_precedes_shape_component_and_textbox_follows ... FAILED
test serializer::control::tests::issue2715_rectangle_caption_roundtrips ... FAILED
test serializer::control::tests::issue2715_group_caption_roundtrips ... FAILED

---- serializer::control::tests::issue2715_caption_precedes_shape_component_and_textbox_follows stdout ----
thread '...' (14364) panicked at src\serializer\control\tests.rs:897:5:
캡션 LIST_HEADER 는 SHAPE_COMPONENT 앞이어야 함 (caption=2, comp=1)

---- serializer::control::tests::issue2715_rectangle_caption_roundtrips stdout ----
thread '...' (9824) panicked at src\serializer\control\tests.rs:819:10:
[#2715] 사각형 캡션이 왕복 보존돼야 함

---- serializer::control::tests::issue2715_group_caption_roundtrips stdout ----
thread '...' (35892) panicked at src\serializer\control\tests.rs:848:14:
[#2715] 묶음 캡션이 왕복 보존돼야 함

test result: FAILED. 1 passed; 3 failed; 0 ignored; 0 measured; 2471 filtered out
```

RED 의 `caption=2, comp=1` 은 발견된 유일한 `LIST_HEADER` 가 `SHAPE_COMPONENT`(idx 1) **뒤**의
글상자용(idx 2)이었음을 뜻한다 — 캡션이 아예 방출되지 않았다는 직접 증거다.

`git stash pop` 후 (**GREEN**):

```
running 4 tests
test serializer::control::tests::issue2715_shape_without_caption_emits_no_list_header ... ok
test serializer::control::tests::issue2715_group_caption_roundtrips ... ok
test serializer::control::tests::issue2715_rectangle_caption_roundtrips ... ok
test serializer::control::tests::issue2715_caption_precedes_shape_component_and_textbox_follows ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 2471 filtered out
```

### 4.2 CI 3종

| 명령 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | **통과** (`Diff in` 0건 — CRLF 노이즈 `Incorrect newline style` 제외) |
| `cargo clippy --all-targets -- -D warnings` | **통과** (경고 0, `Finished dev profile`) |
| `cargo test --profile release-test --tests` | **3484 passed, 0 failed** (`test result: FAILED` 0건, panic 0건) |

### 4.3 회귀 위험 테스트 개별 확인

레코드 시퀀스에 `LIST_HEADER` 를 추가하므로 레코드 인덱스·크기를 고정한 테스트를 사전 조사 후
개별 확인했다.

| 테스트 | 위험 | 결과 |
|---|---|---|
| `issue_1283_hwpx_to_hwp_save_keeps_ole_as_storage` (`shape_component.size == 196` 고정) | OLE 계약 | ok |
| `issue2696_ole_shape_component_stays_base_only` | OLE 196B base-only | ok |
| `task903_stage45_*` (`SHAPE_COMPONENT_INDICES = &[21,35,807,808,810,812]` 고정 인덱스) | 인덱스 이동 | ok — 기준 fixture `samples/hwpx/hwpx-h-01.hwpx` 는 도형 캡션 0건이라 이동 없음 |
| `issue_1403_captions_roundtrip_aift` | HWPX 캡션 | ok |
| `tests/issue_1251_ole_chart_contents.rs` 전 10건 | OLE 전반 | 전건 ok |

### 4.4 실물 코퍼스 재계측

`raw_stream` 무효화 상태에서 `samples/*.hwp` 270건 전수 재계측 — 도형 캡션 보유 21개 파일
**38건 전량 보존**(수정 전 0건). 동일 실행에서 표 캡션 597건·그림 캡션 13건도 불변.
`samples/hwpx/aift.hwpx` → HWP5 변환도 2 → 2 로 보존.

## 5. 미실행·범위 밖

- **OLE 캡션** — `Ole` arm 은 손대지 않았다. #1283/#2696 의 196B base-only 계약 영역이고,
  캡션 보유 OLE 한컴 실물 샘플을 확보하지 못해 방출 위치를 실측 검증할 수 없다. 별도 이슈 권장.
- **묶음 자식 도형의 캡션** — `serialize_group_child` 는 `CTRL_HEADER` 없이 `SHAPE_COMPONENT` 만
  방출하는 경로다. 한컴이 묶음 **자식**에 캡션을 쓰는지 실물 확인하지 못했고, 현 파서도
  `parse_container_children` 에서 자식 캡션을 적재하지 않으므로 범위 밖.
- **캡션 내 `atno`(자동 번호) 의미 검증** — `draw-group.hwp` 캡션 문단에 `atno` 컨트롤이 있다.
  캡션 문단은 `serialize_paragraph_list` 공용 경로를 타므로 레코드는 동승하나, 번호 재계산
  의미까지는 검증하지 않았다.
- **시각 검증(렌더 비교)** — 본 변경은 저장 레코드 계약 수정이고 렌더러 경로는 불변이라
  `visual_baseline_all_samples` 통과로 갈음했다. 별도 SVG 대조는 수행하지 않았다.
- **`raw_stream` 지름길 자체** — 무편집 저장이 직렬화기를 우회하는 설계는 의도된 것(완전
  라운드트립)이라 건드리지 않았다. 다만 직렬화기 결함이 무편집 라운드트립 테스트에서 가려지는
  구조적 사각이므로 별도 관찰 대상으로 남긴다.
