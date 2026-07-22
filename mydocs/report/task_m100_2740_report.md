# Task #2740 — IR 필드 전수 스윕 하네스 + 문단 텍스트 공백 무한 누적 수정

- **이슈**: [#2740](https://github.com/edwardkim/rhwp/issues/2740)
- **브랜치**: `task/m100-2740-roundtrip-invariant-harness`
- **범위**: 왕복 불변식 하네스 신설(비교기 + 회귀 관문 + baseline) / 하네스가 찾아낸
  결함 **1건 수정** / 나머지 발산은 분류해서 인벤토리로 인계

---

## 1. 문제

`fix(...): ... 유실/소실` 형태의 **왕복 1속성 소실** 수정이 최근 30일 37건 머지됐다
(전체 44건 중 84%). 전부 사람이 코드를 눈으로 읽어 찾았다.

이 부류가 **기존 왕복 게이트를 통과한 채로** 발생한다는 것이 핵심이다.
`tests/hwp5_roundtrip_baseline.rs`·`tests/hwpx_roundtrip_baseline.rs` 는 코퍼스를 전수로
돌리고 XFAIL 이 양쪽 다 0 인 초록 상태다. 그럼에도 소실이 계속 나온 이유는 두 게이트가
쓰는 `diff_documents` 가 그 필드들을 **보지 않기** 때문이다.

## 2. 분석

### 2.1 이미 있는 것 (새로 만들지 않았다)

| 자산 | 상태 |
|---|---|
| 코퍼스 전수 스윕 | `hwp5_roundtrip_baseline.rs` / `hwpx_roundtrip_baseline.rs` — **이미 있음** |
| baseline·래칫 관례 | `opengov_corpus_snapshot.rs` + `tests/fixtures/opengov_snapshot.tsv` — **이미 있음** |
| 배치 CLI | `rhwp hwp5-roundtrip --batch` / `hwpx-roundtrip --batch` — **이미 있음** |
| IR 비교 CLI | `rhwp ir-diff [--summary]` — **이미 있음** |

따라서 새 배치 CLI·새 스윕 러너·새 CLI 서브커맨드는 만들지 않았다.
`src/main.rs` 는 손대지 않았다.

### 2.2 비어 있던 것 — 비교 대상의 망라성

`serializer/hwpx/roundtrip.rs::diff_documents` 는 사건 대응으로 누적된 수작업
화이트리스트다. `IrDifference` variant 22종이 전부 손으로 짠 `if a.x != b.x` 이고
파생·매크로·reflection·`PartialEq` 를 일절 쓰지 않는다.

비교되지 **않는** 영역: `CharShape`/`ParaShape`/`BorderFill`/`Numbering`/`Bullet`/
`Style`/`Font` 의 내용(개수만 비교), `Paragraph.text`·`para_shape_id`·`style_id`,
표/셀 속성 전부, `CommonObjAttr` 배치 모델 대부분, 그림 외 도형 기하,
`footnote_shape`/`endnote_shape`/`page_border_fill`/바탕쪽.
추가로 하위 재귀가 전부 `.zip()` 이라 `cells.len()`·`paragraphs.len()`·
`Group.children.len()` 불일치가 **조용히 통과**한다.

최근 37건을 이 목록에 겹치면 정확히 들어맞는다.

### 2.3 실측 중 드러난 사실 — HWP5 게이트는 직렬화기를 보고 있지 않다

`hwp5` 레인(현행 저장 경로) 발산은 **0건**이다. 그러나 무손실이라서가 아니다.
`serializer/body_text.rs:28`·`serializer/doc_info.rs:25` 가 `raw_stream` 이 있으면
**원본 바이트를 그대로 되돌려준다**. 편집하지 않은 문서의 HWP5 왕복은 바이트 재생이다.

제품에서 한 글자라도 고치면 `document_core/commands/*` 가 `section.raw_stream = None` /
`doc_info.raw_stream_dirty = true` 로 무효화하고 **레코드를 다시 만드는 경로**로 저장한다.
그 무효화를 그대로 재현한 `hwp5rb` 레인을 추가하니 **481행**이 나왔다.

## 3. 설계 근거

### 3.1 왜 `Debug` 기반인가

비교 단위를 코드에 적는 방식은 이 결함 부류를 못 막는다. 필드를 추가한 사람이 비교기도
같이 고쳐야 하는데 그걸 잊는 것이 곧 결함의 발생 경로다.

`#[derive(Debug)]` 는 구조체의 **모든 필드**를 출력한다. 그 문자열을 구조적으로 재귀
분해해 비교하면 IR 에 필드를 추가해도 비교기에 손대지 않고 자동 편입된다.

보강으로, 큰 노드(`Paragraph`·`Table`·`Cell`·`FormObject`)는 **`..` 없는 철저 구조
분해**로 잡아 필드가 추가되면 **컴파일이 깨진다**. 런타임 망라 + 컴파일 타임 관문의
이중 안전장치다.

설계상 주의 2가지를 처리했다.
- 비용: `CappedWriter` 가 `Err` 를 돌려 파생 `Debug` 문자열화를 노드당 96KB 에서 중단시킨다.
  거대한 `Vec<u8>`·긴 텍스트를 만나도 비용이 묶인다.
- 결정성: `FormObject.properties`(IR 내 유일한 `HashMap`)는 `Debug` 순서가 비결정적이라
  정렬해 비교한다. 놓치면 하네스 자체가 flaky 해진다.

### 3.2 왜 baseline(래칫)인가 — 첫날 초록이어야 한다

스윕은 첫날부터 빨갛다(729행). **첫날 빨간 게이트는 반드시 꺼진다.** 그래서 현 상태를
`tests/fixtures/ir_field_sweep_baseline.tsv` 로 동결하고 **증가분만** 실패시킨다.
목적은 "지금 전부 무손실"이 아니라 "오늘보다 나빠지면 즉시 실패"다.

baseline 키는 `(레인, 샘플, **인덱스를 지운** 정규화 경로)` → 건수다.
`sections[0].paragraphs[37]...` 를 `sections[].paragraphs[]...` 로 접는다.
문서가 조금 바뀌면 인덱스는 흔들리지만 **결함의 종류**는 경로 모양으로 식별되므로,
인덱스로 고정하면 래칫이 오작동한다.

### 3.3 래칫 방향은 회귀만 — 의도적 관례 이탈

`opengov_corpus_snapshot.rs` 는 개선도 실패시켜 스냅샷 승격을 강제한다.
본 게이트는 **회귀만** 실패시키고 개선은 메시지로만 알린다.

이유: 스윕 범위가 직렬화기 전반이라 **무관한 수정도 수치를 낮춘다.** 그때마다 남의 PR 이
깨지면 게이트가 미움받아 꺼진다 — 그러면 게이트를 만든 의미가 없다. 대신 baseline
재생성 명령을 모듈 문서에 고정해 두었다. 관례에서 벗어난 선택이므로 근거를 여기 남긴다.

### 3.4 CI 범위 — 전체 코퍼스

794회 왕복 실측 **53.4~70.9초**(release-test, 단일 스레드)로 예산 안에 들어와 전체를
기본으로 돌린다. 부분집합만 돌리면 대형 실문서 회귀를 놓치므로 줄이지 않았다.
로컬 반복 확인용으로만 `RHWP_IR_SWEEP_FAST=1` 을 남겼다.

## 4. 변경

| 파일 | 성격 | 내용 |
|---|---|---|
| `src/diagnostics/ir_field_sweep.rs` | 신규 | `Debug` 기반 전수 비교기 + 3개 왕복 진입점 + 단위 테스트 17개 |
| `tests/ir_field_sweep_baseline.rs` | 신규 | 코퍼스 스윕 + 회귀 관문 2건 + 덤프/상세 모드 |
| `tests/fixtures/ir_field_sweep_baseline.tsv` | 신규 | 동결 baseline 729행 (기계 판독) |
| `tests/issue_2740_para_text_space_growth.rs` | 신규 | 결함 회귀 테스트 3건 |
| `src/diagnostics/mod.rs` | 수정 | `pub mod ir_field_sweep;` 1줄 |
| `src/serializer/body_text.rs` | 수정 | 결함 수정 (+12/-1) |

`src/main.rs` · `src/serializer/control.rs` · `src/serializer/hwpx/*` 는 손대지 않았다
(동시 작업 충돌 회피 — §7).

### 결함 수정 내용

`serialize_para_text` 의 자동번호 placeholder 판정:

```rust
- && next_offset.map_or(false, |n| n >= offset + 8);
+ let is_last_text_char = i + 1 == text_chars.len();
+ && next_offset.map_or(is_last_text_char, |n| n >= offset + 8);
```

파서(`parser/body_text.rs:334`)는 자동번호 컨트롤(`0x0012`)을 만나면 **항상** `text` 에
공백 placeholder 를 넣는다. 직렬화기는 그 공백을 다시 컨트롤로 되돌려야 하는데, 판정식이
다음 문자의 오프셋을 요구해서 **placeholder 가 문단의 마지막 문자면 판정이 실패**했다.
그러면 공백을 리터럴로 쓰고 컨트롤도 뒤에 다시 방출하므로 재파싱 때 placeholder 가
하나 더 생긴다 — 저장 N회 → 공백 N개.

마지막 문자를 placeholder 로 봐도 안전한 근거: 진짜 공백이었다면 그 뒤에 파서가 만든
placeholder 가 하나 더 붙어 마지막이 아니게 된다.

## 5. 검증

### 5.1 red → green (실제 실행 캡처)

**RED** — 수정 전 `cargo test --profile release-test --test issue_2740_para_text_space_growth`:

```
running 3 tests
test para_text_is_stable_across_repeated_saves ... FAILED
test para_text_growth_is_not_cumulative ... FAILED
test footer_para_text_is_stable_across_repeated_saves ... FAILED

---- para_text_is_stable_across_repeated_saves stdout ----
thread 'para_text_is_stable_across_repeated_saves' panicked at
tests\issue_2740_para_text_space_growth.rs:50:5:
저장을 반복할수록 문단 텍스트가 자랐다 (공백 누적) — 라운드마다: ["  ②", "  ② ", "  ②  ", "  ②   "]

---- para_text_growth_is_not_cumulative stdout ----
assertion `left == right` failed: 저장 횟수에 따라 텍스트가 계속 자란다 (무한 누적)
  left: "  ② "
 right: "  ②   "

---- footer_para_text_is_stable_across_repeated_saves stdout ----
assertion `left == right` failed: 1회 저장에서 꼬리말 문단이 변했다
  left: Some(("  ", 10))
 right: Some((" ", 9))

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out
```

**GREEN** — 수정 후 같은 명령:

```
running 3 tests
test footer_para_text_is_stable_across_repeated_saves ... ok
test para_text_growth_is_not_cumulative ... ok
test para_text_is_idempotent_after_first_save ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

> 1번 테스트는 red 캡처 후 이름과 단언을 조정했다. 원본 → 1회차 저장에서 공백 1개가
> 생기는 것은 **원인이 다른 별개 사안**(컨트롤 문자 계열 `0x0015` → `0x0012` 재매핑)이라
> 이 수정의 단언 대상이 아니기 때문이다. 이 수정이 실제로 세우는 성질은 **누적되지
> 않는다(멱등)** 이고, 테스트는 그것을 고정한다. 별개 사안은 §7 에 잔여로 남겼다.

### 5.2 코퍼스 효과 (하네스 자체로 측정)

| | 수정 전 | 수정 후 |
|---|---:|---:|
| baseline 행 수 | 786 | **729** |

수정 1건으로 발산 **57행**이 사라졌다. 하네스가 수정 효과를 코퍼스 규모로 재는 데
쓰인다는 것을 보여준다.

### 5.3 CI 3종

| 검사 | 명령 | 결과 |
|---|---|---|
| clippy | `cargo clippy --all-targets -- -D warnings` | **통과** (exit 0, 경고 0) |
| 테스트 | `cargo test --profile release-test --tests` | **통과** (exit 0, 3512 passed / 0 failed / 23 ignored) |
| fmt | 변경 `.rs` 5개에 `rustfmt --edition 2021` 후 md5 재비교 | **통과** (변경 0) |

fmt 는 `cargo fmt --all -- --check` 를 쓰지 않았다. 이 Windows 체크아웃에서는 CRLF 파일에
대해 `Incorrect newline style` 만 출력하고 diff 를 내지 않아 **거짓 통과**한다.
대신 변경 파일에 직접 `rustfmt` 를 돌리고 md5 로 무변경을 확인했다.

> `rustfmt src/diagnostics/mod.rs` 는 `mod` 선언을 따라 형제 모듈 17개를 함께
> 재작성한다(줄바꿈만 바뀜). 내용 변화가 없음을 `git diff --stat` 으로 확인하고
> 전부 `git checkout` 으로 되돌렸다. 커밋에는 포함하지 않는다.

### 5.4 하네스 런타임

| 항목 | 값 |
|---|---|
| 왕복 횟수 | 794회 (hwp5 365 + hwp5rb 365 + hwpx 64) |
| 단독 실행 | 53.4s / 56.7s / 70.9s (3회, release-test 단일 스레드) |
| `--tests` 스위트 내 | **35.19s** (nextest 없이 순차, 병렬 실행 시 더 짧음) |
| baseline 행 | 729 |
| 총 발산 잎 | 110,012 |

CI 는 nextest 로 샤딩 병렬 실행하므로 실제 wall time 증가는 이보다 작다.

## 6. 미실행 항목

- **한컴 실제 저장본과의 대조 미실행**. 수정한 결함은 rhwp 자기 왕복의 **멱등성**
  위반이라 외부 정답지 없이 판정 가능하지만, 저장 결과가 한컴에서 어떻게 보이는지는
  확인하지 않았다.
- 시각 회귀(렌더 비교) 미실행. 변경이 텍스트 스트림 직렬화 1곳이라 레이아웃 영향은
  `--tests` 의 기존 pagination/렌더 핀 테스트로 갈음했다.
- `samples/hwpx/opengov/` 는 스윕에서 제외(기존 `opengov_corpus_snapshot.rs` 관할).

## 7. 잔여 — 인벤토리 인계

수정하지 않은 발산은 **분류해서** 남긴다. 분류 없는 목록은 오히려 해롭다는 판단이다.
경로·샘플·건수는 `tests/fixtures/ir_field_sweep_baseline.tsv` 에 기계 판독 형태로 있고,
`RHWP_IR_SWEEP_DETAIL="샘플명조각"` 으로 값까지 즉시 확인할 수 있다.

수정 후 baseline 729행의 분류 집계:

| 분류 | 행 | 잎 |
|---|---:|---:|
| 결함 | 46 | 2,794 |
| 의도된 정규화 | 426 | 106,644 |
| 판단보류 | 257 | 574 |
| 합계 | 729 | 110,012 |

**의도된 정규화가 결함보다 9배 많다.** 이 비율이 이 하네스의 사용법을 규정한다 —
스윕 결과는 결함 목록이 아니라 **분류 대상 목록**이다.

### 7.1 결함으로 판단하나 이번 PR 범위 밖 (파일 충돌 회피)

두 건 모두 수정 지점이 `src/serializer/control.rs` 인데, 이 파일은 현재 다른 작업들이
동시에 손대고 있어 충돌 위험이 크다. 근거와 재현 샘플을 남기고 별도 PR 로 넘긴다.

| 항목 | 행 | 샘플 | 근거 |
|---|---:|---:|---|
| 바탕쪽(master page) 2개 → 1개 소실 | 5 | 5 | `control.rs:318` 의 가드가 all-or-nothing — raw 로 보존된 LIST_HEADER 가 **하나라도** 있으면 모델의 바탕쪽을 **전부** 방출하지 않는다. `exam_math.hwp` 는 바탕쪽 2개(`ext_flags` 0x0000 / 0x0004) 중 1개만 살아남는다 |
| 도형 채우기 `fill.solid` `Some(...)` → `None` | 14 | 5 | `basic/BookReview.hwp` 에서 `SolidFill { background_color: 16777215, pattern_color: 0, pattern_type: -1 }` 이 통째로 사라진다 |

### 7.1a 텍스트 계열 잔존 27행 — 이번 수정과 원인이 다르다

`.text`/`char_count`/`char_offsets` 계열이 27행 남아 있다. 이번에 고친 **무한 누적**과
달리 이들은 1회차 저장에서만 변하고 이후 멱등이다. 확인된 원인 하나는 자동번호 컨트롤의
**문자 계열 재매핑**이다 — 원본은 쪽 컨트롤 계열(`0x0015`)로 담고 있는데
`control_char_code_and_id` 가 `Control::AutoNumber` 를 항상 `0x0012` 로 쓴다.
그러면 재파싱 때 파서가 placeholder 공백을 새로 만든다.

`0x0012` vs `0x0015` 선택은 같은 함수의 주석이 밝히듯 한컴 호환 판단이 얽힌 영역이라
(`NewNumber` 는 한컴 정답지에 맞춰 의도적으로 `0x0015` 로 쓴다) 근거 없이 건드리지 않았다.
나머지 잔존분(글상자 문단 등)은 원인 미확인이다.

### 7.2 의도된 정규화 — 결함 아님 (기록만)

| 항목 | 행 | 잎 | 근거 |
|---|---:|---:|---|
| 셀 `list_header_width_ref` `0` → `0x0400` | 242 | 65,799 | **task #1633 의도** |
| `raw_header_extra`·`instance_id` 재생성 | 178 | 37,590 | 모델 주석의 계약 (count 필드 재생성 + instanceId) |
| `line_segs[].vertical_pos` 등 | 6 | 3,234 | 레이아웃 캐시 재조판 결과물 |

**`list_header_width_ref` 는 특히 주의가 필요하다.** 173개 샘플 / 65,799개 잎으로
인벤토리 최대 항목이고, 코드만 보면 전형적인 "파서는 읽는데 직렬화기가 상수를 쓴다"
패턴이라 결함으로 올리기 쉽다. 출처를 확인하면 의도다:

```
$ git log -S "0x0400" -- src/serializer/control.rs
25a9316f task 1633: 셀존 origin 렌더링과 HWP 저장 보정
         - 신규 셀 HWP LIST_HEADER를 한컴식 47바이트 구조로 보강
```

다만 이 치환은 "신규 셀"뿐 아니라 **한컴 파일에서 `0` 으로 파싱된 셀**에도 걸린다.
의도 범위를 넘는지는 메인테이너 판단 영역이라 결함으로 올리지 않고 기록만 한다.

### 7.3 판단보류 (257행) — 증거 부족

| 항목 | 행 | 레인 |
|---|---:|---|
| `section_def.extra_child_records.len` (증가 방향) | 70 | hwp5rb |
| `drawing.shape_attr.offset_x/offset_y` | 62 | hwpx |
| `drawing.caption` | 17 | hwp5rb |
| `doc_info.extra_records.len` / `memo_shape_count` | 19 | hwpx |
| 글상자 `list_attr` | 8 | hwpx |
| `shape_attr.group_level` | 4 | hwpx |
| **컨트롤 종류 자체가 바뀜** (`controls[]`) | 4 | hwpx |
| `footnote_shape`/`endnote_shape`/`page_def.margin_*` | 14 | hwpx |
| `img_dim`·`raw_picture_extra`·`chars[]`·`bullet_count` 등 | 59 | 혼합 |

상당수가 **증가**(0 → 값) 방향이라 소실이 아니라 materialize 일 수 있다.
방향만 보고 결함이라 부르지 않는다.

추가 잔여: 자동번호 컨트롤 문자 계열 재매핑(`0x0015` → `0x0012`, §5.1 주석),
교차 포맷 왕복(A→B→A) 검증 부재(저장소 전체에 없음 — 이슈 #2740 §2.3/§7).
