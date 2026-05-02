# PR #400 검토 — HWPX 수식 직렬화 보존

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#400](https://github.com/edwardkim/rhwp/pull/400) |
| 작성자 | [@cskwork](https://github.com/cskwork) (Agentic-Worker, 본인 표기) — 신규 컨트리뷰터 (PR #397 와 동일 작성자) |
| base / head | `devel` ← `cskwork:feature/hwpx-equation-serialization` |
| state | OPEN |
| mergeable | MERGEABLE |
| mergeStateStatus | BEHIND |
| 변경 통계 | +416 / -1, 3 files |
| **CI** | **statusCheckRollup 비어있음** — CI 실행 안 됨 (PR #397 와 같은 정황) |
| 관련 이슈 | [#286](https://github.com/edwardkim/rhwp/issues/286) |

## 작성자 정황

@cskwork — 신규 컨트리뷰터, PR #397 와 동일 작성자 (작업지시자 git 정보 확인 — 실제 사람)

## 이슈 #286 정황 — 마일스톤 변경

원래 이슈 #286 본문은 "M200: HWPX 수식 직렬화 - Phase 1 설계 (01-01)" 로 v2.0.0 마일스톤에 설계 단계 (소스 변경 없음) 만 명시했음. 그러나 작업지시자가 본 검토 시점에서 **M100 (v1.0.0) 으로 변경 결정**:

- 이슈 #286 milestone: 미설정 → **v1.0.0** (메인테이너 갱신)
- HWPX 수식 직렬화는 v1.0.0 범위에 포함됨
- 본 PR 의 코드 변경도 정당한 진행 (시기상조 아님)

## 변경 내용

### 결함 (작성자 분석)

`render_paragraph_parts()` 의 기존 흐름이 `render_hp_t_content(&para.text)` 만 호출 → `Control::Equation` 이 IR 에 있어도 HWPX section.xml 에 `<hp:equation>` 출력 안 됨. 결과:

- HWPX 저장/재파싱 라운드트립에서 수식이 소실
- `<hp:script>` 의 XML entity (`<`, `&`) 복원 누락 (parser 가 `Event::GeneralRef` 무시)

### 정정 (3 files / +416 / -1)

#### 1. `src/serializer/hwpx/section.rs` (대부분 변경)

- `render_paragraph_parts` 내 `render_hp_t_content` → `render_run_content` 교체
- `render_run_content` 신규: 문단의 `char_offsets` / `char_count` 로 수식 컨트롤 inline 위치 추정
- `render_equation` 신규: `Control::Equation` → `<hp:equation>` XML 직렬화
- 모호한 control gap (slot 수와 controls 수 불일치) 시 보수적 처리 — 수식을 텍스트 뒤로 배치
- 헬퍼 함수: `inferred_control_slot_count`, `is_hwpx_inline_slot`, `flush_text_fragment`, `render_control_slot`, `char_utf16_width`, `color_ref_to_hwpx`, `text_wrap_to_hwpx`, `vert_rel_to_hwpx`, `horz_rel_to_hwpx`, `vert_align_to_hwpx`, `horz_align_to_hwpx`

#### 2. `src/parser/hwpx/section.rs` (`parse_equation`)

`<hp:script>` 내 `Event::GeneralRef` 처리 추가:
- `&lt;` → `<`, `&gt;` → `>`, `&amp;` → `&`, `&quot;` → `"`, `&apos;` → `'`
- numeric character reference
- 알 수 없는 entity 는 원문 보존

#### 3. `src/serializer/hwpx/mod.rs` (회귀 테스트 3 개)

- `equation_control_roundtrip_preserves_script`
- `equation_control_between_text_runs_roundtrips_position`
- `equation_control_does_not_consume_unmapped_control_gap`

## 검증

### 본 검토에서 dry-run merge 결과

devel 위에 자동 머지 성공. 머지 후 검증:
- `cargo test --lib`: **1034 passed** (1031 → +3 신규)
- `cargo test --test svg_snapshot`: 6/6
- `cargo test --test issue_418`: 1/1 (Task #418 보존)
- `cargo clippy --lib -- -D warnings`: warning 0건

코드 자체는 컴파일 / 테스트 통과.

## 평가

### 강점

1. **결함 분석 명확** — `render_paragraph_parts` 가 controls 무시하는 정황 정확히 짚음
2. **변경 범위 합리** — section.rs / parser.rs / mod.rs 3 파일
3. **헬퍼 함수 분리 잘됨** — render_run_content / render_equation / inferred_control_slot_count / 변환 헬퍼들 분리
4. **모호한 gap 보수적 처리** — 잘못된 위치에 수식 배치 방지 (텍스트 뒤로 fallback)
5. **XML entity 복원** — parser 측 `Event::GeneralRef` 처리 추가 (round-trip 안전성)
6. **dry-run merge** — 자동 성공 + 1034 passed
7. **clippy 통과**
8. **이슈 #286 milestone v1.0.0 으로 갱신됨** — 본 PR 의 정당한 범위 확정

### 약점 / 점검 필요

#### 1. 자기검증 테스트 정황 (메모리 위반)

본 PR 의 회귀 테스트 3개 모두 **자기검증 패턴**:

```rust
// 작성자가 IR 생성 → 직렬화 → 재파싱 → 자기 데이터와 일치 검증
let mut doc = Document::default();
para.controls.push(Equation { script: "x < y & z", ... });   // ← 자기 정의
serialize_hwpx(&doc) → bytes
parse_hwpx(&bytes) → parsed_eq
assert_eq!(parsed_eq.script, "x < y & z");                    // ← 자기 검증
```

메모리 (`feedback_self_verification_not_hancom.md`) 위반:
> 자기 라운드트립 통과해도 한컴 거부 가능. HWP 저장 작업은 한컴 수동 검증 게이트 필수

본 PR 의 테스트는 **rhwp 자체 roundtrip 항등성** 만 검증. 다음이 검증 안 됨:
- **한컴 spec 일치성** — `<hp:equation>` 의 속성 / 자식 요소 순서 / 네임스페이스 등이 한컴 spec 과 정확히 일치하는지
- **한컴 호환성** — 본 직렬화 결과를 한컴 편집기가 정상 열람하는지
- **실제 hwpx 샘플 회귀** — `samples/hwpx/` 의 한컴 생성 hwpx (수식 포함) 를 parse → serialize → 재parse 후 의미 일치하는지

#### 2. CI 실행 안 됨

PR #397 와 동일한 정황. statusCheckRollup 비어있음. rebase + push 후 자동 트리거 예상.

#### 3. devel base BEHIND

PR #395, #396 머지 전 base. devel rebase 필요.

#### 4. 한컴 hwpx 샘플 활용 안 함

본 저장소 `samples/hwpx/` 에 한컴 생성 hwpx 다수 보유. 본 PR 의 회귀 테스트가 **자체 IR 대신 실제 한컴 hwpx 를 사용**하면 한컴 spec 일치성을 더 잘 검증 가능.

예시 패턴:
```rust
let bytes = std::fs::read("samples/hwpx/exam_math.hwpx")?;
let doc = parse_hwpx(&bytes)?;
let serialized = serialize_hwpx(&doc)?;
let reparsed = parse_hwpx(&serialized)?;
// 원본 doc 과 reparsed doc 의 수식 정보 비교 (rhwp 자체가 만든 데이터 아님)
```

## 메인테이너 작업과의 관계

### 충돌 가능성

본 PR 의 영향 파일 (3 files) 중 PR #395, #396 (오늘 머지) 와의 영향:
- `src/parser/hwpx/section.rs` — PR #395, #396 미변경 ✅
- `src/serializer/hwpx/mod.rs` — PR #395, #396 미변경 ✅
- `src/serializer/hwpx/section.rs` — PR #395, #396 미변경 ✅

dry-run merge 자동 머지 통과 확인.

## 처리 방향 — 옵션 A (PR #397 와 같은 패턴, 작업지시자 결정)

작성자에게 다음 항목 보강 후 재제출 요청:

1. **devel 기반 rebase** — base 가 PR #395, #396 머지 전 (BEHIND)
2. **회귀 테스트 보강** — 자기 IR 생성 대신 **실제 한컴 hwpx 샘플** (`samples/hwpx/`) 를 활용한 테스트 추가
3. **한컴 호환성 검증 자료** — 본 PR 의 직렬화 결과 hwpx 를 한컴 편집기에서 정상 열람하는 증빙 (스크린샷 / PDF)
4. **CI 실행** — rebase + push 후 자동 트리거 예상

본 PR 은 OPEN 유지 (close 안 함). 작성자 보강 재제출 시 다시 검토.

## 검토 항목 (Claude 점검 완료)

- [x] 작성자 신뢰도 — 신규 컨트리뷰터 (PR #397 와 동일) ⚠️
- [x] 코드 품질 — 합리적 ✅
- [x] dry-run merge — 자동 성공 ✅
- [x] cargo test --lib — 1034 passed ✅
- [x] cargo clippy — warning 0 ✅
- [x] 이슈 #286 milestone v1.0.0 갱신 ✅
- [ ] **자기검증 테스트 한계** — 한컴 hwpx 샘플 활용 보강 필요 ⚠️
- [ ] **한컴 호환 검증** — 한컴 편집기 정상 열람 증빙 필요 ⚠️
- [ ] CI 실행 — rebase 후 트리거 예상
- [ ] devel rebase — 필요

## 다음 단계

1. 작성자 댓글 — devel rebase + 한컴 hwpx 샘플 회귀 테스트 보강 + 한컴 호환 검증 자료 요청
2. 본 PR OPEN 유지 (재제출 대기)

## 참고

- PR: [#400](https://github.com/edwardkim/rhwp/pull/400)
- 이슈: [#286](https://github.com/edwardkim/rhwp/issues/286) (milestone v1.0.0 으로 갱신)
- 작성자 다른 PR: [#397](https://github.com/edwardkim/rhwp/pull/397) (수식 ATOP, OPEN, 재제출 대기)
- 메모리: `feedback_self_verification_not_hancom.md` (자기 라운드트립 ≠ 한컴 호환)
