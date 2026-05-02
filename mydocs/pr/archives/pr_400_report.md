# PR #400 처리 보고서 — HWPX 수식 직렬화 보존 (#286)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#400](https://github.com/edwardkim/rhwp/pull/400) |
| 작성자 | [@cskwork](https://github.com/cskwork) — 첫 외부 컨트리뷰터 (PR #397 머지 후 두 번째 PR) |
| 이슈 | [#286](https://github.com/edwardkim/rhwp/issues/286) (closes #286, milestone v1.0.0) |
| 처리 결정 | **cherry-pick 머지** (작성자 보강 후) |
| 처리 일자 | 2026-04-29 |

## 처리 절차

### Stage 0: 1차 검토 (2026-04-28) — 옵션 A 보강 요청

1차 검토 ([comment-4336491440](https://github.com/edwardkim/rhwp/pull/400#issuecomment-4336491440)) 시 5 항목 안내:
1. 회귀 테스트 보강 — 자기검증 한계 (메모리 `feedback_self_verification_not_hancom.md` 위반)
2. 한컴 호환 검증 자료 (한컴 편집기 열람 증빙)
3. devel 기반 rebase
4. CI 실행
5. 이슈 #286 milestone v1.0.0 갱신 (메인테이너 처리)

### Stage 1: 작성자 5 항목 모두 정확 대응 (2026-04-28T15:58:36Z)

| 항목 | 작성자 대응 |
|------|------------|
| 1. 자기검증 한계 → 한컴 origin 회귀 | ✅ commit `ecd7d9a` 추가 — `equation_roundtrip_from_hancom_origin_hwp_sample` |
| 2. 한컴 호환 검증 (편집기 열람) | ✅ 한컴 한글 2020 (Office 2020 / HOffice110) PDF 검증 자료 첨부 |
| 3. devel 기반 rebase | ✅ linear 2 commits (`f25d986`, `ecd7d9a`) |
| 4. CI 실행 | ✅ 메인테이너 GitHub Actions API 승인 후 실행 |
| 5. 이슈 milestone | ✅ 메인테이너 처리 완료 |

### Stage 2: cherry-pick

`local/pr400` 브랜치 (`local/devel` 분기) 에서 2 commits cherry-pick — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `28d6eba` (← `f25d986`) | @cskwork | feat: HWPX 수식 직렬화 보존 |
| `43da8f9` (← `ecd7d9a`) | @cskwork | test: 한컴 origin hwp 기반 수식 HWPX 라운드트립 회귀 추가 |

cherry-pick 결과: 충돌 없이 자동 적용.

### Stage 3: 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1050 passed** (1046 → +4 신규 한컴 origin 회귀 + 기타) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo test --lib serializer::hwpx::tests::equation_roundtrip_from_hancom_origin_hwp_sample` | ✅ passed (한컴 origin 데이터 라운드트립) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 21s, 4,114,661 bytes |

## 변경 요약

### 본질 — HWPX section 직렬화에 `<hp:equation>` 출력 + XML entity 복원

기존: `render_paragraph_parts` 가 `Control::Equation` 무시 → HWPX 저장/재파싱 시 수식 소실.

정정 (3 files):

| 파일 | 변경 |
|------|------|
| `src/serializer/hwpx/section.rs` | `render_run_content` (controls inline 위치 추정) + `render_equation` (`<hp:equation>` XML 직렬화) + 모호 gap 보수적 처리 |
| `src/parser/hwpx/section.rs::parse_equation` | `<hp:script>` 내 XML entity 복원 (`&lt;`, `&gt;`, `&amp;`, `&quot;`, `&apos;`, numeric ref) |
| `src/serializer/hwpx/mod.rs` | 회귀 테스트 4건 (자기검증 3 + **한컴 origin 라운드트립 1**) |

### 한컴 origin 회귀 테스트 (보강 commit `ecd7d9a`)

```rust
let bytes = std::fs::read("samples/equation-lim.hwp")?;
let original = parse_hwp(&bytes)?;
let original_eqs = collect_equations(&original);
assert!(!original_eqs.is_empty());

let hwpx_bytes = serialize_hwpx(&original)?;
let reparsed = parse_hwpx(&hwpx_bytes)?;
let reparsed_eqs = collect_equations(&reparsed);

for (orig, rep) in original_eqs.iter().zip(reparsed_eqs.iter()) {
    assert_eq!(rep.script, orig.script);
    assert_eq!(rep.font_size, orig.font_size);
    assert_eq!(rep.baseline, orig.baseline);
    assert_eq!(rep.font_name, orig.font_name);
    assert_eq!(rep.color, orig.color);
    assert_eq!(rep.common.width, orig.common.width);
    assert_eq!(rep.common.height, orig.common.height);
    assert_eq!(rep.common.treat_as_char, orig.common.treat_as_char);
}
```

**자기검증 패턴 (Document::default + 수동 push) 정확히 회피** — 한컴 편집기가 만든 hwp 의 IR 을 입력으로 사용. 메모리 `feedback_self_verification_not_hancom.md` 원칙 부합.

### 한컴 호환 검증

작성자 본문:
- 입력: `samples/equation-lim.hwp` (한컴 편집기 origin)
- 처리: parse_hwp → serialize_hwpx (본 PR 변경사항 반영)
- 산출물: `output/pr400_equation_roundtrip.hwpx` (7,145 bytes)
- **한컴 한글 2020 (Office 2020 / HOffice110)** 열람 후 PDF 내보내기: 11,662 bytes
- 수식 렌더링: `lim_{h→0} {f(2+h) - f(2)} / h` (한컴 origin 과 시각적 일치)

[pr400_equation_roundtrip.pdf](https://github.com/user-attachments/files/27172903/pr400_equation_roundtrip.pdf) 첨부.

→ **한컴 호환 게이트 통과**.

## 시각 판정 정황

본 PR 은 **HWPX 직렬화 (저장 경로)** 정정 — dev server / 화면 시각 변화 없음. 검증 게이트:
- `parse_hwp → serialize_hwpx → parse_hwpx` 라운드트립 항등성 (cargo test 통과)
- 한컴 호환 (한컴 한글 2020 정상 열람 + PDF 일치 — 작성자 검증 자료)

→ **시각 판정 불필요** (PR #405, #411 와 같은 패턴).

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1050 + svg_snapshot 6/6 + clippy 0 + WASM + CI 승인 |
| 자기 라운드트립 ≠ 한컴 호환 | ✅ 한컴 origin 회귀 + 한컴 한글 2020 PDF 검증 자료 |
| PR 댓글 톤 | ✅ |
| output 폴더 가이드라인 | (본 PR 시각 자료 없음, 작성자 PDF 만 첨부) |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr400` 에서 커밋 |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr400` → `local/devel` → `devel` 머지 + push
3. PR #400 close + 작성자 댓글 (이슈 #286 자동 close)

## 참고

- 1차 검토: `mydocs/pr/pr_400_review.md`
- 재검토: `mydocs/pr/pr_400_review_v2.md`
- PR: [#400](https://github.com/edwardkim/rhwp/pull/400)
- 이슈: [#286](https://github.com/edwardkim/rhwp/issues/286) (milestone v1.0.0)
- 작성자 다른 머지 PR: [#397](https://github.com/edwardkim/rhwp/pull/397) (수식 ATOP)
- 한컴 호환 PDF: [pr400_equation_roundtrip.pdf](https://github.com/user-attachments/files/27172903/pr400_equation_roundtrip.pdf)
