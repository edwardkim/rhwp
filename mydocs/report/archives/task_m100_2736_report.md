# task_m100_2736 처리결과 보고서 — 순회 × 컨테이너 전수 조사와 캡션 컨테이너 누락 수정

- **이슈**: [#2736](https://github.com/edwardkim/rhwp/issues/2736)
- **브랜치**: `task/m100-2736-container-recursion` (base `devel` @ `1658d0bb`)
- **범위**: `src/document_core/converters/hwpx_to_hwp.rs` 1개 파일 (+ 본 보고서)
- **분류**: 결함 수정 (순회 누락) + 전수 조사

## 1. 문제

`Document` 는 문단을 14종 컨테이너(본문 / 표 셀 / 중첩 표 셀 / 글상자 / 각주 / 미주 /
머리말 / 꼬리말 / 바탕쪽 / 표 캡션 / 그림·도형 캡션 / 숨은설명 / 묶음 개체 자식 / 메모)에
중첩 보관한다. 그런데 문서를 순회하는 함수들은 **각자 독립적으로 어느 컨테이너로 내려갈지
결정**하며, 완전성을 강제하는 공유 방문자가 저장소에 없다 (`grep -rn "visitor\|Visitor" src/`
→ 0건).

그래서 "본문·표 셀은 처리하지만 각주 본문은 조용히 건너뛰는" 워크가 만들어지고, 그 컨테이너를
쓰는 문서에서만 결과가 틀리므로 결함이 오래 산다. 같은 계열의 수정이 이미 5건 머지됐다
(`632f4258`, `05af4fe7` #2467 / `ea82eb33` #2715 / `761544e7` #2651 / `395ebedc` #2717).
전부 "다른 워크는 이미 그 컨테이너를 도는데 이 워크만 빠져 있다"는 동일 구조다.

본 작업의 목적은 하나씩 찾는 방식을 끝내고 **행렬을 만드는 것**이었다.

## 2. 행렬 전수 조사

### 2.1 방법

추측이 아니라 순회 관용구 검색으로 후보를 뽑았다.

```
grep -rn "for para in" src/                      → 126건
grep -rn "\.paragraphs\.iter()" src/             → 204건
grep -rn "for cell in\|cells\.iter()" src/       → 194건
grep -rn "fn .*_recursive\|fn walk_\|fn visit_\|fn collect_" src/ → 98건
```

이후 스크립트로 `src/document_core` · `serializer` · `renderer` · `model` · `parser` · `paint`
의 모든 `fn` 본문을 brace 매칭으로 잘라 "표 셀로 내려간다(= 문서 워크)" 마커를 가진 함수
**269개**를 추출하고, 테스트 헬퍼·`#[cfg(test)]` 내부·호출자가 컨테이너를 이미 지정하는 지역
루프를 제외해 **컨테이너를 놓치면 사용자 결과가 틀리는 워크 24개**로 좁혔다.

### 2.2 분류별 집계 (우선 범위 `converters/` · `queries/` · `commands/`)

| 분류 | 개수 |
| --- | --- |
| 대상 컨테이너를 모두 재귀 (또는 누락이 결과에 영향 없음) | **8** |
| 의도적 미재귀 (문서화된 범위 한정 / path 해석기 / 파라미터 제어) | **6** |
| **누락 (재귀해야 하는데 안 함)** | **10** |

24행을 빠짐없이 배분한 결과다 (8 + 6 + 10 = 24). 행렬 전문(24행 × 14열)과 셀별 판정 근거는
이슈 #2736 §2.2 에 있다. 본 보고서는 중복하지 않고 결론만 옮긴다.

- **재귀함 8** — bin order 수집, `copy_control_native`, `clear_missing_lineseg_placeholders_*`,
  `materialize_missing_lineseg_paragraphs_*`, `get_cell_paragraph*_by_path`,
  `renumber_footnotes_in_section`, `collect_header_footer_controls`,
  `assign_auto_numbers_in_controls`.
- **의도적 미재귀 6** — `converters/diagnostics.rs:106`(진단 전용),
  `search_first_body`/`replace_one_native`(doc 주석에 본문 한정 명시),
  `search_all_text_native(include_cells=false)`(파라미터 제어),
  `normalize_xml_import_paragraphs`(코드에 TODO 명시), `validate_linesegs`(doc 주석 명시),
  `resolve_paragraphs`/`resolve_paragraph_by_path`(path 해석기 — 호출자가 컨테이너 지정).
- **누락 10** — §7.1 잔여표 참조. 그중 `collect_all_fields` 는 위치 모델(`FieldLocation`)
  한계에서 오는 **설계 제약형 누락**이라 성격이 다르다.

본 PR 의 수정 범위는 **누락 10건 중 adapt 워크 1건 전부 + bin remap·border_fill 워크 2건의
캡션 축**이다.

**중요 판정 정정** — `collect_bin_order_from_control` 도 그림 캡션을 빠뜨리지만, 뒤이은
`for id in 1..=bin_count` 폴백(`hwpx_to_hwp.rs:296-298`)이 남은 id 를 전부 order 에 채워
remap 이 항상 전단사 순열이 되므로 **참조 무결성은 유지되고 순서만 달라진다.** 결함이
아니라고 판정하고, 형제 워크와의 대칭 유지 목적으로만 수정했다.

## 3. 확정 누락 (수정 대상) — 실측

`adapt` 워크(`adapt_paragraph_with_context` / `adapt_shape_with_context`)가 **그림 캡션과
도형·묶음·차트·OLE 캡션을 방문하지 않는다.** 같은 워크가 표 캡션은
`adapt_table_with_context` 에서 `#2443` 근거로 방문한다.

**실측** — 한컴 산출 실파일 `samples/hwpx/aift.hwpx` 를 파싱해 `convert_hwpx_to_hwp_ir()` 를
돌린 뒤, adapt 방문 표식인 `Paragraph::raw_header_extra.len()`(= `materialize_para_header_tail`
이 12로 늘림) 분포를 셌다.

```
para_header_tail_materialized = 6330
본문     raw_header_extra 길이 분포: {12: 921}   ← 921/921 방문
표캡션   raw_header_extra 길이 분포: {12: 2}     ←   2/2   방문
그림캡션 raw_header_extra 길이 분포: {10: 9}     ←   0/9   방문
```

본문 921/921, 표 캡션 2/2 가 방문되는 동안 **그림 캡션 9/9 전부 미방문**(파서가 넣은 10바이트
그대로). 코드 확인이 아니라 실파일 계량이다.

**사용자에게 나타나는 잘못된 결과**

1. `adapt_picture_href_ctrl_data` 미실행 → 캡션 안 그림의 하이퍼링크(href)가 HWP 저장 시 유실.
   `#2467` 이 각주·표 캡션에 대해 고친 것과 같은 증상.
2. `adapt_table_with_context` 미실행 → 캡션 안 표의 `raw_ctrl_data` 미합성 → 표 개체 공통
   속성(위치·크기·배치) 유실.
3. `adapt_equation` 미실행 → 캡션 안 수식 `ctrl_header attr`·글꼴 버전 보정 누락.
4. `materialize_para_header_tail` 미실행 → 직렬화기(`src/serializer/body_text.rs:406-408`)가
   `raw_header_extra[6..]` 를 그대로 흘리므로 그림 캡션 문단만 `PARA_HEADER` 22바이트
   (18+instanceId 4), 본문·표 캡션은 24바이트(+변경추적 2). 한 파일에 형식이 두 종류로 섞인다.

그림 캡션은 희귀 구조가 아니다. `aift.hwpx` 한 파일에만 `<hp:caption>` 이 그림에 11개 붙어
있고, HWPX 파서는 `#1403` 이래 그림·도형·묶음 캡션을 모두 `Caption` 으로 적재한다
(`src/parser/hwpx/section.rs:1985,2602,3929,4112`).

같은 캡션 축에서 형제 워크 2건도 그림 캡션을 빠뜨리고 있어 함께 고쳤다.

- `remap_bin_refs_in_control` — `Control::Picture` arm 이 자기 `bin_data_id` 만 리맵하고
  `pic.caption` 은 안 봄. 표 캡션에 대한 동형 결함이 이미
  `table_caption_picture_bin_ref_is_remapped` 로 회귀 고정돼 있는데 그림 캡션이 그 미수정 형제.
- `collect_object_border_fill_refs_from_paragraph` — `Control::Picture` arm 자체가 없음.
  미수집 시 `normalize_paragraph_char_border_fills` 가드가 실패해 캡션 안 개체 채우기가
  no-fill 로 정규화된다(`05af4fe7` 와 동일 메커니즘).

## 4. 변경

`src/document_core/converters/hwpx_to_hwp.rs` 한 파일, 캡션 축만. 참조 구현은 같은 저장소의
`DocumentCore::clear_missing_lineseg_placeholders_in_control`
(`src/document_core/commands/document.rs:702-751`) — `Table`(셀+캡션)/`Shape`/`Picture`(캡션)/
`Header`/`Footer`/`Footnote`/`Endnote`/`HiddenComment`/`Field`(memo) 를 모두 처리하는,
저장소에서 가장 완전한 워크다.

| # | 함수 | 추가한 재귀 |
| --- | --- | --- |
| 1 | `adapt_paragraph_with_context` `Control::Picture` arm | `pic.caption.paragraphs` |
| 2 | `adapt_shape_with_context` | `drawing.caption.paragraphs` — `DrawingObjAttr` 공유로 사각형·타원·선·호·다각형·곡선·글상자·묶음·차트·OLE 일괄 적용 |
| 3 | `remap_bin_refs_in_control` `Control::Picture` arm | `pic.caption.paragraphs` |
| 4 | `collect_bin_order_from_control` `Control::Picture` arm | `pic.caption.paragraphs` (대칭 유지 목적, 결과는 순서만) |
| 5 | `collect_object_border_fill_refs_from_paragraph` | `Control::Picture` arm 신설 → `pic.caption.paragraphs` |

각 추가에 `[#2736]` 접두 한국어 주석으로 **어느 형제 워크가 이미 그 컨테이너를 도는지**를
명시해 다음 사람이 비대칭을 즉시 볼 수 있게 했다.

`ParagraphContext` 는 캡션에서도 바깥 문맥을 그대로 전달한다(표 캡션 처리와 동일). 캡션 전용
컨텍스트를 새로 만들면 `materialize_master_page_autonum_placeholder` 의 바탕쪽/머리말 판정이
흔들리므로 도입하지 않았다.

**추가 테스트 4건** (`hwpx_to_hwp.rs` 테스트 모듈)

- `picture_href_materializes_inside_picture_and_shape_caption` — 그림 캡션·도형 캡션 안 그림 href
- `picture_caption_picture_bin_ref_is_remapped` — 그림 캡션 안 그림 `bin_data_id` 리맵
- `border_fill_refs_collected_inside_picture_caption` — 그림 캡션 안 표 `border_fill` 수집
- `aift_picture_caption_paragraphs_are_adapted` — 실파일 회귀(`samples/hwpx/aift.hwpx` 9문단)

## 5. 검증

### 5.1 red → green (실제 실행, 캡처)

수정 5곳만 되돌리고(테스트는 유지) `cargo test --lib document_core::converters::hwpx_to_hwp` 실행.
되돌린 상태의 diff 는 테스트 모듈 단일 hunk(`@@ -2257,0 +2258,194 @@ mod tests`)만 남았음을
`git diff -U0` 로 확인했다.

**RED (실측 출력)**

```
running 41 tests
test ...::border_fill_refs_collected_inside_picture_caption ... FAILED
test ...::picture_href_materializes_inside_picture_and_shape_caption ... FAILED
test ...::picture_caption_picture_bin_ref_is_remapped ... FAILED
test ...::aift_picture_caption_paragraphs_are_adapted ... FAILED

---- ...::border_fill_refs_collected_inside_picture_caption stdout ----
panicked at src\document_core\converters\hwpx_to_hwp.rs:2395:9:
그림 캡션 안 표의 border_fill 이 수집돼야 함

---- ...::picture_href_materializes_inside_picture_and_shape_caption stdout ----
panicked at src\document_core\converters\hwpx_to_hwp.rs:2288:9:
assertion `left == right` failed: 그림 캡션 문단 안 그림의 href 가 물질화돼야 함(캡션 문단 미방문)
  left: 0
 right: 1

---- ...::picture_caption_picture_bin_ref_is_remapped stdout ----
panicked at src\document_core\converters\hwpx_to_hwp.rs:2362:9:
assertion `left == right` failed: 그림 캡션 안 그림의 bin_data_id 가 remap 되지 않음(캡션 문단 미방문)
  left: 1
 right: 2

---- ...::aift_picture_caption_paragraphs_are_adapted stdout ----
panicked at src\document_core\converters\hwpx_to_hwp.rs:2446:9:
assertion `left == right` failed: 그림 캡션 문단이 adapt 워크의 방문 표식(header tail 12바이트)을 받아야 함
  left: 0
 right: 9

test result: FAILED. 37 passed; 4 failed; 0 ignored; 0 measured; 2434 filtered out; finished in 0.70s
```

RED 상태에서 통과한 신규 테스트는 **없다** — 4건 전부 실패했다.

**GREEN (수정 복원 후, 실측 출력)**

```
test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 2434 filtered out; finished in 0.91s
```

### 5.2 CI 3종

| 검사 | 결과 |
| --- | --- |
| `cargo clippy --all-targets -- -D warnings` | **통과** — `Finished dev profile ... in 58.72s`, 경고 0 |
| `cargo test --profile release-test --tests` | **통과** — 292 바이너리 / 3,494 passed / 0 failed / 23 ignored (§5.3) |
| `rustfmt --edition 2021` (변경 `.rs` 파일) | **통과** — 1차 실행에서 신규 테스트 코드 2곳 정정 후 커밋에 포함, 2차 실행 무변경(멱등) |

`cargo fmt --all -- --check` 는 이 Windows 체크아웃에서 CRLF 파일에 대해 `Incorrect newline
style` 만 찍고 diff 를 내지 않는 **거짓 통과**라 사용하지 않았다.

### 5.3 `--tests` 전체 결과

`cargo test --profile release-test --tests` 종료 코드 **0**.

| 항목 | 수치 |
| --- | ---: |
| 테스트 바이너리 | 292 |
| passed | **3,494** |
| failed | **0** |
| ignored | 23 |

`test result:` 라인 292개 전부 `ok`, `FAILED`/`error[` 0건. 그중 lib 타깃 단독 결과는
`2468 passed; 0 failed; 7 ignored`. 신규 4건 모두 통과:

```
test ...::border_fill_refs_collected_inside_picture_caption ... ok
test ...::picture_caption_picture_bin_ref_is_remapped ... ok
test ...::picture_href_materializes_inside_picture_and_shape_caption ... ok
test ...::aift_picture_caption_paragraphs_are_adapted ... ok
```

`cargo test --lib` 은 `tests/` 디렉터리를 통째로 건너뛰므로 판정 근거로 쓰지 않았다.

## 6. 미실행 항목

- **한컴 대조 없음.** 캡션 안 href/표 속성 유실을 한/글에서 열어 확인하지 않았다. 근거는
  (a) 실파일 방문 계량, (b) 표 캡션·각주에 대한 동형 결함이 `#2467`/`#2443` 에서 이미
  같은 형태(합성 IR 단위 테스트)로 수용된 선례다.
- **시각 회귀 없음.** 변경이 HWPX→HWP 저장 경로 한정이고 렌더 트리를 건드리지 않아
  golden SVG 대조는 돌리지 않았다.
- **`PARA_HEADER` 22 → 24 의 한컴 정합 미검증.** 본문 921/921·표 캡션 2/2 가 24바이트이므로
  **동일 문서 내 일관성** 근거로만 정당화한다. "22가 손상"이라고 주장하지 않는다.

## 7. 잔여

### 7.1 이번에 고치지 않은 누락

| # | 누락 | 위치 | 근거 강도 | 미수정 사유 |
| --- | --- | --- | --- | --- |
| 1 | `replace_all_native` 가 중첩 표·각주·머리말·캡션 미치환 | `queries/search_query.rs:77` | 코드 확인 | `SearchHit.cell_context` 4-튜플 → 경로형 일반화 필요, `search_all_text_native` 공개 JSON `cellContext` 계약 변경 동반 |
| 2 | `collect_bookmarks` 글상자 미방문 | `queries/bookmark_query.rs:221` | 코드 확인 | 책갈피 위치 모델(`host_para`)의 글상자 표현 미정 |
| 3 | `collect_all_fields` 가 각주/머리말/캡션 필드 미수집 | `queries/field_query.rs:50,968` | 코드 확인 | `FieldLocation::NestedEntry` 가 `TableCell`/`TextBox` 만 모델링 |
| 4 | 누름틀 `field_id` 채번이 각주/머리말 필드 미계수 | `field_query.rs:1193`, `clipboard.rs:200` | **영향 미증명** | "문서 내 고유 ID" 불변식 위반은 확정이나, rhwp 소비자(`get/set_field_value_by_id`)가 `collect_all_fields` 를 쓰고 그 워크도 같은 컨테이너를 안 봐서 충돌한 두 필드가 API 상에서 만나지 않음. 한컴 오동작 미실측 |
| 5 | `clear_initial_field_texts` 중첩 표·글상자 미방문 | `commands/document.rs:1687` | 코드 확인 | 실파일 재현 미실시 |
| 6 | `reflow_zero_height_paragraphs` 중첩 표·각주 미방문 | `commands/document.rs:296` | 코드 확인 | 렌더 회귀 위험 — 별도 시각 검증 필요 |
| 7 | bin remap·border_fill 워크의 숨은설명·메모 축 미방문 | `hwpx_to_hwp.rs:470,732` | **영향 미증명** | 파서가 해당 컨테이너에 그림을 적재하는 경로 미확인 |
| 8 | `normalize_xml_import_paragraphs` 글상자 미방문 | `commands/document.rs:1670` | 코드에 TODO 명시 | 원 저자가 "정확한 API 미식별" 로 남긴 기지 항목 |

### 7.2 감사하지 않은 영역

- `src/renderer/**`(typeset·layout·pagination) — 순회 함수 100+개, 레이아웃 정합이 별도 축이라 제외.
  `collect_header_footer_controls` 1건만 표본 포함.
- `src/serializer/**`, `src/parser/hwpx/**`, `src/parser/hml/**` — 다른 PR 이 점유 중이라 판정만 하고
  미수정. `assign_auto_numbers_in_controls` 1건만 표본 포함.
- `src/paint/**`, `src/diagnostics/**` — 결과가 사용자 문서에 반영되지 않아 제외.
- HWP3 경로(`src/parser/hwp3/`) — 컨테이너를 넓게 도는 것으로 보이나 개별 검증 미실시.

### 7.3 근본 대책 제안

행렬이 보여주듯 이 계열은 개별 수정으로 수렴하지 않는다. `Control` 의 문단 보유 arm 을 한
곳에서 열거하는 **공유 방문자**(예: `model::visit::for_each_paragraph_container`)를 도입하고
새 워크가 그것을 쓰게 하면, `Control` 에 컨테이너가 추가될 때 `match` 가 non-exhaustive 로
깨져 컴파일러가 누락을 잡아준다. 본 작업 범위를 넘어서므로 이슈 #2736 §7.3 에 제안만 남겼다.
