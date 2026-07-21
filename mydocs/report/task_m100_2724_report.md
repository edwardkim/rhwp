# task_m100_2724 처리결과 보고서 — 패스스루 무효화 누락 저작 시점 가드

- **이슈**: [#2724](https://github.com/edwardkim/rhwp/issues/2724)
- **브랜치**: `task/m100-2724-invalidation-guard` (base `devel`)
- **범위**: 신규 파일 `tests/issue_2724_passthrough_invalidation_guard.rs` 1개 + 이 문서.
  **기존 소스·테스트 무수정**
- **분류**: 재발 계급 봉인 (저작 시점 소스 가드 신설)

## 1. 문제

rhwp 는 완전 라운드트립을 위해 원본 바이트를 세 층에 보존한다.

| 층 | 필드 | 직렬화 시 동작 | 근거 |
|---|---|---|---|
| 레코드 | `Control::*.raw_ctrl_data` | 비어 있지 않으면 원본 그대로 방출 | `src/serializer/control.rs:482-483`, `2383-2386` |
| 구역 | `Section::raw_stream` | `Some` 이면 **구역 전체**를 원본 그대로 반환 | `src/serializer/body_text.rs:26-30` |
| DocInfo | `DocInfo::raw_stream_dirty` | `false` 면 원본 스트림 그대로 반환 | `src/serializer/doc_info.rs:23-33` |

`serialize_section` 은 `raw_stream` 이 `Some` 이면 **함수 첫 줄에서** 원본을 반환한다.
그리고 `raw_stream` 은 HWP5 CFB 파싱 경로에서 **항상** 채워진다
(`src/parser/mod.rs:465`, `521`). 즉 사용자가 실제로 여는 `.hwp` 는 예외 없이 이 조건에
걸린다.

따라서 IR 을 고친 `&mut self` 메서드가 해당 층을 무효화하지 않으면 **컴파일 에러도,
테스트 실패도, 런타임 경고도 없이** 편집이 저장 시점에 사라진다.

- 컴파일러: `section.raw_stream = None;` 은 선택적 부수효과 — 빠뜨려도 타입이 맞는다.
- 단위 테스트: 대부분 IR 을 직접 검증한다. IR 은 정확히 바뀌었으므로 통과한다.
- clippy: 검출 대상이 아니다.
- 사용자: 편집 직후 화면은 정상(렌더는 IR 기준). 저장하고 **다시 열었을 때** 드러난다.

`#2698`(`object_ops/connector.rs` 347줄에 무효화 0건, PR #2704 로 `devel` 반영)이 이
계급의 최신 사례다. 핵심은 **밀도 0 이 한눈에 보이는 신호였는데도 아무도 보지
않았다**는 것이다. 사람이 매번 `grep` 할 것을 기대하는 규약은 규약이 아니다.

메인테이너는 이미 TypeScript 쪽에 같은 형태의 방어를 만들어 두었다
(`rhwp-studio/src/core/mutation-method-registry.ts` +
`rhwp-studio/tests/mutation-routing-guard.test.ts`, `#2327`). 이 작업은 그 설계를
코드베이스의 나머지 절반(Rust)으로 포팅한 것이다.

## 2. 전수 조사 결과

### 2.1 방법

`src/document_core/**` 의 `.rs` 41개(그중 `&mut self` 보유 25개)를 스캔했다. 문자열·문자·
주석을 공백으로 치환한 뒤 중괄호 매칭으로 본문을 잘라냈고, `#[cfg(test)]` 블록은
제외했으며, `impl ... DocumentCore` 블록 안의 메서드만 셌다.

- **무효화 신호**: 본문에 `raw_stream = None` 또는 `raw_stream_dirty = true`
- **위임**: 직접 무효화가 없고, 호출하는 다른 함수가 무효화에 도달

조사는 Python 스크립트와 최종 Rust 가드 두 구현으로 **독립 교차 검증**했다. 두 구현이
`pub fn (&mut self)` 개수를 **144개로 동일하게** 산출했다(가드 자기검사 하한을 일시로
9999 로 올려 실측치를 확인 — 5.2절 캡처).

### 2.2 결과 — `impl DocumentCore` 의 `&mut self` 총 202개

`impl DocumentCore {` 201개 + `impl crate::document_core::DocumentCore {` 1개
(`object_ops/common.rs:341 insert_new_number_native` — 완전 경로 표기). 이슈 본문의
"201개 / pub 143" 은 후자를 빼고 센 초안 수치이고, 아래가 최종본이다.

| 가시성 | 개수 | 무효화함 | 위임 | 그 외 |
|---|---:|---:|---:|---:|
| `pub` (가드 범위) | **144** | **108** | **9** | **27** |
| `pub(crate)` / private | 58 | 15 | 3 | 40 |

`pub` 144개 최종 분류:

| 분류 | 개수 | 판정 |
|---|---:|---|
| **무효화함** (본문에서 직접) | 108 | 정상 |
| **`DelegatesTo`** — 실제 무효화는 피위임자가 | 11 | 정상 (근거 기재) |
| **`SessionState`** — 문서 IR 비변경 | 16 | 정상 (근거 기재) |
| **`WholeDocument`** — 문서 전체 교체 | 3 | 정상 (근거 기재) |
| **`SurgicalRawEdit`** — 원본 스트림 직접 수술 | 3 | 정상 (근거 기재) |
| **`NoPassthrough`** — 패스스루 부재 경로 | 1 | 정상 (근거 기재) |
| **`CallerResponsibility`** — 원시 핸들 반환 | 1 | 정상 (근거 기재) |
| **`Pending`** — 판정 보류 | 1 | 2.4절 |
| **무효화 누락(진짜 결함)** | **0** | — |

> 이슈 #2724 본문의 분류 내역(SessionState 17 / WholeDocument 5 / Surgical 2 …)은
> 초안 집계다. 구현하면서 `convert_to_editable_native`(→ `SurgicalRawEdit`),
> `export_hwp_with_adapter`·`serialize_hwp_with_verify`(→ `DelegatesTo`),
> `document_mut`(→ `CallerResponsibility`) 를 더 정확한 분류로 옮겼다. **총계
> 27 면제 + 9 위임 = 36 항목은 동일**하고, 위 표가 최종본이다.

`pub(crate)`/private 58개 중 무효화·위임이 없는 40개는 전부
① 가변 접근자(`get_table_mut`, `resolve_shape_control_mut`, `find_equation_mut` 등 12개)
② 리플로우 헬퍼(`reflow_paragraph`, `reflow_cell_paragraph`, `reflow_hf_paragraph` 등 5개)
③ 렌더 캐시·페이지네이션 헬퍼(`mark_section_dirty`, `paginate`, `recompose_section`,
`rebuild_section` 등 15개) ④ 기타 내부 헬퍼 8개로, **호출자가 무효화하는 것이 이
코드베이스의 확립된 패턴**이다. 예: `text_editing.rs:1116 mark_cell_control_dirty` 의
호출 지점 21곳을 전수 확인했고, 모두 호출부에서 `raw_stream = None` 을 수행한다.

### 2.3 DocInfo 층 별도 확인

`doc_info` 컬렉션(`char_shapes`/`para_shapes`/`border_fills`/`bin_data_list`/`font_faces`)을
직접 변경하는 `commands/**` 함수 6개를 따로 셌다. **6개 전부** `raw_stream_dirty = true`
또는 surgical 편집을 수행한다. 이 층은 대부분 `model/document.rs` 의
`find_or_create_char_shape` / `find_or_create_para_shape` / `find_or_create_tab_def` 가
책임지며(각각 `raw_stream_dirty = true` 보유) 현재 누락이 없다.

### 2.4 판정 보류 1건 — `reflow_linesegs_on_demand` (메인테이너 판단 요청)

`src/document_core/commands/document.rs:995`. wasm 공개 API `reflowLinesegs` 이며
`mutation-method-registry.ts:70-71` 이 `MUTATING_METHODS` 에 등재하면서
`// lineseg 재계산 (#177 — 저장 lineseg 를 실제로 변경)` 이라고 못박아 두었다.

- **기계적 사실**: 모든 구역을 순회하며 `reflow_line_segs()` 로 `para.line_segs` 를
  재작성하고 `recalculate_section_vpos()` 로 vpos 를 갱신하지만 `section.raw_stream` 을
  건드리지 않는다. `line_segs` 는 `PARA_LINE_SEG` 레코드로 직렬화되는 필드다.
- **관측된 사용자 영향: 없음.** 호출 지점을 전수 확인했다 — 저장소 전체에서 실호출은
  `src/wasm_api.rs:5481` 한 곳(단순 위임)뿐이고, studio 는 `#2527` 이후 이 API 를
  호출하지 않는다(`rhwp-studio/src/ui/validation-modal.ts:4` 에 "미사용" 명시).
- **의도적일 가능성**: docstring 은 효과를 "호출 이후 렌더 캐시·페이지네이션이 갱신되므로
  즉시 렌더링하면 보정된 결과가 보인다"라고 **렌더 기준으로** 서술하고, 한컴이 계산한
  lineseg 를 강제로 다시 푸는 것에 대해 명시적으로 보수적이다.

**추정으로 고치지 않았다.** `raw_stream` 을 비우면 해당 구역이 통째로 재직렬화되는데,
이 코드베이스는 그 경로를 의도적으로 피해 왔다(`text_editing.rs:1328`: "DocInfo
raw_stream은 유지 (전체 재직렬화 시 FIX-4 문제 발생)"). 지속성 계약은 메인테이너 결정
사항이므로 `Exempt::Pending` 으로 등재하고 근거를 남겼다. **가드의 부수 효과가 바로
이것이다 — 보이지 않던 질문을 저작 시점에 표면화한다.**

## 3. 변경

`tests/issue_2724_passthrough_invalidation_guard.rs` (신규, 1,058줄) 1개 파일.
기존 소스·테스트는 한 줄도 건드리지 않았다.

`tests/issue_1402_enum_token_whitelist.rs` 가 이미 확립한 관례를 따랐다 — **근거 주석이
달린 화이트리스트 + 위반을 모아 한 번에 실패시키는 테스트**.

### 3.1 왜 목록과 테스트를 한 파일에 두었나

TS 쪽은 런타임 코드가 같은 목록을 `import` 하므로 `src/core/` 와 `tests/` 로 분리돼 있다.
Rust 쪽은 참조자가 가드뿐이라 `src/` 에 두면 아무도 쓰지 않는 상수가 생긴다
(`dead_code = "allow"` 정책이 있어 경고는 없지만, 라이브러리에 검사 전용 데이터를 싣게
된다). 그래서 테스트 타깃 한 파일에 **권위 목록 + 검사**를 함께 두었다. 목록을 파싱할
필요가 없어져 TS 가드의 `parseStringArray` 대응물이 사라진 것은 부수적 이득이다.

### 3.2 검사 5개

| # | 테스트 | 잡는 것 |
|---|---|---|
| 1 | `classification_drift_is_blocked` | 범위 내 `pub fn (&mut self)` 가 무효화도 등재도 없음 = `connector.rs` 계급 |
| 2 | `stale_exemptions_are_reclaimed` | 면제 항목이 실재하지 않음 / 이제 무효화함 / 중복 / 근거 누락 / 판정 보류 상한 초과 |
| 3 | `delegation_targets_actually_invalidate` | 위임 대상이 사라졌거나 무효화를 잃음 |
| 4 | `invalidation_density_ledger_is_ratcheted` | 파일별 무효화 사이트 수 감소 (갈래 일부 제거) |
| 5 | `guard_scanner_self_check` | 스캐너 손상으로 1~4 가 **공허하게 통과**하는 것 |

### 3.3 baseline / 래칫 설계 근거

**빨간 상태로 태어난 가드는 꺼지고, 꺼진 가드는 없는 것보다 나쁘다.** 그래서 오늘의
상태를 그대로 동결했다.

- `EXEMPT` 36항목 전부가 **한 줄짜리 근거를 동반**한다. 근거 없는 allowlist 는 가치가
  없으므로 형식으로 강제했다(분류 enum + 사유 문자열, 검사 2가 길이까지 확인).
- **면제는 줄어들기만 한다**: 면제받던 함수가 무효화를 갖게 되면 검사 2가 "항목을
  삭제하라"고 실패시킨다. 삭제되고 나면 그 뒤에 무효화를 없앨 때 검사 1이 잡는다.
- **무효화 밀도는 늘어나기만 한다**: 21파일 135사이트 하한을 동결했다. 감소는 실패,
  증가는 통과 + 갱신 안내(TS 가드의 `stale` 안내와 대칭).
- `DelegatesTo` 는 단순 메모가 아니라 **기계가 검증하는 주장**이다(검사 3).

### 3.4 정밀도 우선 — 의도적으로 검사하지 않은 것

오탐 하나가 가드를 죽인다. TS 쪽이 런타임 `opDepth` 가드를 폐기한 경위(오탐 → 경고
소진 → 진짜 미라우팅까지 침묵, PR #2329)가 그 증거다.

- **`pub(crate)`/private 헬퍼 58개**: 40개가 "호출자가 무효화" 패턴이라 근거 없는 면제만
  40건 늘어난다. TS 가드가 `WasmBridge` **공개** 메서드만 보는 것과 같은 선택이다.
- **"실제로 IR 을 바꾸는가" 판정**: 정적으로 불가능하다. 대신 "`pub` + `&mut self`" 라는
  구문적으로 확정적인 조건을 쓰고, 조회성 메서드는 면제 목록이 흡수한다.
- **갈래별 분석**(어느 return 경로가 무효화를 안 타는가): 시도했다. 휴리스틱이 후보
  11건을 냈으나 상위 2건을 직접 읽어보니 전부 오탐이었다 —
  `merge_paragraph_in_cell_native`(text_editing.rs:2486)의 조기 return 3개는 모두
  뮤테이션 **이전**이다. **채택하지 않았다.** 이 축은 검사 4(밀도 원장)가 근사한다.
- **구역 인덱스 정합성**(A 구역을 고치고 B 구역을 무효화): 데이터플로 분석 필요. 범위 밖.
- **`syn` 도입**: 의존성에 없고 이 목적으로 추가할 무게가 아니다. TS 가드도 실제 타입
  분석이 아니라 소스 파싱이다.

### 3.5 못 잡는 것 (정직 고지 — 파일 상단 doc 주석에도 명시)

1. **무효화 대상이 틀린 경우** — 호출이 본문에 있기만 하면 통과한다.
2. **처음부터 일부 갈래만 무효화하는 신규 코드** — 검사 4는 "감소"로만 근사한다.
3. **헬퍼로 감싼 무효화 이관** — 검사 4가 감소로 잡아 baseline 갱신을 요구한다(의식적
   갱신 강제가 목적이지만, 정당한 리팩터에도 한 번 걸린다).
4. **`src/wasm_api.rs`** — 범위 밖. 이 어댑터 층에도 `self.core.document.sections.get_mut()`
   직접 뮤테이션이 있다(`wasm_api.rs:5800` 근방 `update_style` 계열). 현재는 무효화가
   정상적으로 붙어 있음을 확인했다. 다수의 병행 작업이 이 파일을 건드리는 중이라
   baseline 충돌 위험이 커 후속 PR 로 분리한다.
5. **레코드 층(`raw_ctrl_data`)** — 무효화 관용구가 단일하지 않아 이번 범위 제외.

## 4. 검증

### 4.1 red→green 실증 (실제 실행 캡처)

가드가 실제로 동작함을 증명하기 위해 **소스에서 무효화를 일시 제거하고 실행한 출력을
그대로** 붙인다. 세 시나리오 모두 실행 후 `git checkout --` 로 복구했고, 최종 트리에는
아무 변경도 남아 있지 않다.

#### 실증 1 — 뮤테이터의 무효화 전부 제거 (검사 1·4·5 동시 적중)

`connector.rs::update_connector_subject_ids` 의 `section.raw_stream = None;` 을
`let _ = section;` 로 치환.

```
running 5 tests
test delegation_targets_actually_invalidate ... ok
test invalidation_density_ledger_is_ratcheted ... FAILED
test classification_drift_is_blocked ... FAILED
test stale_exemptions_are_reclaimed ... ok
test guard_scanner_self_check ... FAILED

failures:

---- invalidation_density_ledger_is_ratcheted stdout ----

thread 'invalidation_density_ledger_is_ratcheted' (35744) panicked at tests\issue_2724_passthrough_invalidation_guard.rs:973:5:
무효화 밀도가 원장 아래로 내려갔다 1건 (#2724):
  ↓ src/document_core/commands/object_ops/connector.rs: 4 → 3 (무효화 사이트 감소)

한 함수 안에 무효화 갈래가 여럿일 때 일부만 지우면 함수 단위 검사는 통과한다.
PR #2704 의 커넥터 곡선 갈래 누락이 실제로 그 형태였다(3갈래 중 1갈래만 방어).
무효화를 의도적으로 이관·통합했다면 INVALIDATION_LEDGER 를 갱신하라.
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- classification_drift_is_blocked stdout ----

thread 'classification_drift_is_blocked' (13084) panicked at tests\issue_2724_passthrough_invalidation_guard.rs:806:5:
미분류 문서 뮤테이터 1건 (#2724):
  + src/document_core/commands/object_ops/connector.rs:14 update_connector_subject_ids() — 패스스루 무효화 없음

`pub fn (&mut self)` 가 문서 IR 을 바꾸면 본문에서 패스스루를 무효화해야 한다
(`section.raw_stream = None` / `doc_info.raw_stream_dirty = true`).
빠뜨리면 `serialize_section`(serializer/body_text.rs:26-30)이 원본 바이트를 그대로
반환해 편집이 저장 결과에서 사라진다 — 컴파일 에러도 테스트 실패도 없이.
바꾸지 않거나 다른 뮤테이터에 위임한다면 이 파일
(tests/issue_2724_passthrough_invalidation_guard.rs)의 EXEMPT 에 분류와 근거를
적어 등재하라.

---- guard_scanner_self_check stdout ----

thread 'guard_scanner_self_check' (16176) panicked at tests\issue_2724_passthrough_invalidation_guard.rs:1030:9:
connector.rs::update_connector_subject_ids() 가 무효화하지 않는다 — #2698 회귀


failures:
    classification_drift_is_blocked
    guard_scanner_self_check
    invalidation_density_ledger_is_ratcheted

test result: FAILED. 2 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s
```

#### 실증 2 — 갈래 하나만 제거 (검사 4만 적중 = 원장의 존재 이유)

`recalculate_connector_routing` 의 **직선 조기반환 갈래** 무효화 1개만 제거하고 나머지
갈래의 무효화는 남겨 둔 상태. 함수 단위 검사(1)는 통과하고 밀도 원장(4)만 잡는다 —
PR #2704 에서 실제로 발생했던 "3갈래 중 1갈래만 방어" 형태다.

```
running 5 tests
test invalidation_density_ledger_is_ratcheted ... FAILED
test classification_drift_is_blocked ... ok
test delegation_targets_actually_invalidate ... ok
test stale_exemptions_are_reclaimed ... ok
test guard_scanner_self_check ... ok

failures:

---- invalidation_density_ledger_is_ratcheted stdout ----

thread 'invalidation_density_ledger_is_ratcheted' (37976) panicked at tests\issue_2724_passthrough_invalidation_guard.rs:973:5:
무효화 밀도가 원장 아래로 내려갔다 1건 (#2724):
  ↓ src/document_core/commands/object_ops/connector.rs: 4 → 3 (무효화 사이트 감소)

한 함수 안에 무효화 갈래가 여럿일 때 일부만 지우면 함수 단위 검사는 통과한다.
PR #2704 의 커넥터 곡선 갈래 누락이 실제로 그 형태였다(3갈래 중 1갈래만 방어).
무효화를 의도적으로 이관·통합했다면 INVALIDATION_LEDGER 를 갱신하라.


failures:
    invalidation_density_ledger_is_ratcheted

test result: FAILED. 4 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s
```

#### 실증 3 — 스캐너 파손 (검사 5)

`SCAN_ROOT` 를 잘못된 하위 경로로 바꿔 스캐너가 대상을 못 찾게 만든 상태.
**검사 1이 대상 0건으로 공허하게 통과(`ok`)** 하는 것을 확인할 수 있다 — 자기검사가
없으면 이 상태로 가드가 영원히 초록이 된다.

```
running 5 tests
test invalidation_density_ledger_is_ratcheted ... FAILED
test classification_drift_is_blocked ... ok
test guard_scanner_self_check ... FAILED
test stale_exemptions_are_reclaimed ... FAILED
test delegation_targets_actually_invalidate ... FAILED

---- guard_scanner_self_check stdout ----

thread 'guard_scanner_self_check' (33596) panicked at tests\issue_2724_passthrough_invalidation_guard.rs:1001:5:
스캐너가 범위 내 `pub fn (&mut self)` 를 0개만 찾았다(하한 130).
경로·파싱이 깨지면 검사 1~4 가 대상 0건으로 **공허하게 통과**한다.
구조 변경이 실제라면 하한을 의식적으로 낮춰라.
```

#### 실측치 확인 — 스캐너가 찾는 뮤테이터 개수

자기검사 하한을 일시로 9999 로 올려 실측치를 출력시켰다. Python 사전 조사와 **동일한
144** 로, 두 독립 구현이 교차 검증됐다.

```
---- guard_scanner_self_check stdout ----

thread 'guard_scanner_self_check' (20416) panicked at tests\issue_2724_passthrough_invalidation_guard.rs:1001:5:
스캐너가 범위 내 `pub fn (&mut self)` 를 144개만 찾았다(하한 9999).
경로·파싱이 깨지면 검사 1~4 가 대상 0건으로 **공허하게 통과**한다.
구조 변경이 실제라면 하한을 의식적으로 낮춰라.
```

#### 복구 후 green

```
running 5 tests
test invalidation_density_ledger_is_ratcheted ... ok
test classification_drift_is_blocked ... ok
test delegation_targets_actually_invalidate ... ok
test stale_exemptions_are_reclaimed ... ok
test guard_scanner_self_check ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
```

### 4.2 가드 런타임

**dev 0.24초 / release-test 0.18초** (5개 테스트 합계). `.rs` 41개 읽기 + 문자열 스캔뿐이라 CI 부담이
사실상 없다. 느린 가드는 꺼진다는 전제로 설계했다.

### 4.3 CI 3종

| 검사 | 명령 | 결과 |
|---|---|---|
| clippy | `cargo clippy --all-targets -- -D warnings` | **통과** (`Finished dev profile in 1m 49s`, 경고 0) |
| fmt | `rustfmt --edition 2021 tests/issue_2724_passthrough_invalidation_guard.rs` 후 재실행 해시 비교 | **통과** (md5 `95a26b7d…` 동일 — 멱등) |
| test | `cargo test --profile release-test --tests` | **통과** (테스트 바이너리 292개 / 3,485 passed, 0 failed, 23 ignored, exit 0) |

> fmt 는 지시된 방법(변경 파일에 `rustfmt` 직접 실행 후 diff 확인)을 따랐다.
> `cargo fmt --all -- --check` 는 이 Windows 체크아웃에서 CRLF 파일에 대해
> `Incorrect newline style` 만 출력하고 diff 를 내지 않아 **거짓 통과**한다.

release-test 프로필에서의 가드 실행 결과(전체 회귀 로그 발췌):

```
     Running tests\issue_2724_passthrough_invalidation_guard.rs (target\release-test\deps\issue_2724_passthrough_invalidation_guard-bc196448cee187aa.exe)

running 5 tests
test invalidation_density_ledger_is_ratcheted ... ok
test classification_drift_is_blocked ... ok
test delegation_targets_actually_invalidate ... ok
test guard_scanner_self_check ... ok
test stale_exemptions_are_reclaimed ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
```

## 5. 미실행 항목

- **`reflow_linesegs_on_demand` 지속성 계약 확정**(2.4) — 메인테이너 판단 사항이라
  건드리지 않았다. 결론이 "저장돼야 한다"면 무효화 추가 + 라운드트립 회귀 테스트,
  "렌더 전용이 맞다"면 docstring 명시 + `mutation-method-registry.ts:70` 주석 정정.
- **저장→재로드 왕복 실측** — 이번 작업은 정적 계약 검사이지 데이터 유실 보고가 아니다.
  전수 조사에서 진짜 결함이 0건이었으므로 재현할 유실이 없다. **관측하지 않은 것을
  관측했다고 쓰지 않았다.**

## 6. 잔여 (범위 밖)

1. **`src/wasm_api.rs` 로 가드 범위 확장** — 공개 wasm 표면의 직접 뮤테이션 지점.
   커버리지가 크게 늘지만 현재 다수의 병행 작업이 이 파일을 건드리고 있어 baseline
   충돌 위험이 크다. 후속 PR.
2. **레코드 층(`raw_ctrl_data`) 가드** — 3.5-5.
3. **런타임 가드** — 채택하지 않는다. TS 쪽에서 이미 폐기된 접근이다(PR #2329).
4. **`connector.rs` 중복 고지** — PR #2704 가 `devel` 에 이미 반영돼 있다. 이 작업의
   baseline 은 그 상태를 기준으로 동결했다(`connector.rs` 무효화 사이트 4).
