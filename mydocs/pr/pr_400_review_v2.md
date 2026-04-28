# PR #400 재검토 — 작성자 보강 후 (HWPX 수식 직렬화 보존)

## 1차 검토 후 작성자 대응 (5 항목 모두)

1차 검토 ([comment-4336491440](https://github.com/edwardkim/rhwp/pull/400#issuecomment-4336491440)) 의 5 항목 작성자 대응 결과:

| 항목 | 작성자 대응 |
|------|------------|
| 1. 회귀 테스트 보강 (자기검증 한계 → 한컴 origin 활용) | ✅ commit `ecd7d9a` 추가 — `samples/equation-lim.hwp` (한컴 origin) 기반 회귀 테스트 |
| 2. 한컴 호환 검증 자료 (한컴 편집기 열람 증빙) | ✅ 한컴 한글 2020 (Office 2020) 열람 + PDF 내보내기 검증 (PDF 첨부) |
| 3. devel 기반 rebase | ✅ linear rebase (2 commits) — `f25d986`, `ecd7d9a` |
| 4. CI 실행 | ✅ rebase + push 후 자동 트리거 (메인테이너 승인 대기) |
| 5. 이슈 #286 milestone | ✅ 메인테이너 v1.0.0 갱신 완료 (1차 검토 시) |

## 신규 회귀 테스트 평가 — `equation_roundtrip_from_hancom_origin_hwp_sample`

작성자가 1차 검토의 핵심 지적 (자기검증 패턴은 메모리 `feedback_self_verification_not_hancom.md` 위반) 을 정확히 보강:

```rust
// 한컴 origin 데이터 입력 (자체 IR 생성 회피)
let bytes = std::fs::read("samples/equation-lim.hwp")?;
let original = parse_hwp(&bytes)?;
let original_eqs = collect_equations(&original);
assert!(!original_eqs.is_empty(), "한컴 origin 샘플에 수식이 존재해야 회귀 비교가 의미있음");

// 직렬화 + 재파싱
let hwpx_bytes = serialize_hwpx(&original)?;
let reparsed = parse_hwpx(&hwpx_bytes)?;
let reparsed_eqs = collect_equations(&reparsed);

// 한컴 origin 의 수식 필드가 hwpx 라운드트립에서 보존되는지
for (orig, rep) in original_eqs.iter().zip(reparsed_eqs.iter()) {
    assert_eq!(rep.script, orig.script);
    assert_eq!(rep.font_size, orig.font_size);
    // ...
}
```

**자기검증 패턴 (Document::default + 수동 push) 을 정확히 회피** — 한컴 편집기가 만든 hwp 의 IR 을 입력으로 사용. 메모리 원칙 부합.

## 한컴 호환 검증 자료

작성자 본문:
> 한컴 한글 2020 (Office 2020 / HOffice110) 에서 본 PR 의 직렬화 결과 hwpx 를 정상 열람 + PDF 내보내기 검증
> - 입력: `samples/equation-lim.hwp` (한컴 편집기 origin)
> - 처리: parse_hwp → serialize_hwpx (본 PR 변경사항 반영)
> - 산출물: `output/pr400_equation_roundtrip.hwpx` (7,145 bytes)
> - 한컴 한글 2020 열람 후 PDF 내보내기: 11,662 bytes
> - 수식 렌더링: `lim_{h→0} {f(2+h) - f(2)} / h` (한컴 origin 과 시각적으로 일치)

[pr400_equation_roundtrip.pdf](https://github.com/user-attachments/files/27172903/pr400_equation_roundtrip.pdf) 첨부.

→ **한컴 호환 게이트 통과** — rhwp serialize → 한컴 편집기 정상 열람 + PDF 출력 일치.

## dry-run merge 결과

devel 위에 자동 머지 성공. 머지 후 검증:

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1050 passed** (1046 → +4 신규 한컴 origin 회귀 + 기타) |
| `cargo test --test svg_snapshot` | ✅ 6/6 |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo test --lib serializer::hwpx` | ✅ **65 passed** (`equation_roundtrip_from_hancom_origin_hwp_sample` 통과 ✅) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |

## CI 정황

`action_required` 상태 — 메인테이너 GitHub Actions API 로 승인 후 실행 가능:

```bash
gh api -X POST repos/edwardkim/rhwp/actions/runs/25063569310/approve
gh api -X POST repos/edwardkim/rhwp/actions/runs/25063569348/approve
```

## 평가

### 강점

1. **자기검증 한계 정확 보강** — 한컴 origin 데이터 활용 회귀 테스트 추가
2. **한컴 호환 검증 게이트 통과** — 한컴 한글 2020 열람 + PDF 출력 일치
3. **rebase 깔끔** — linear (2 commits) 처리
4. **메모리 원칙 부합** — `feedback_self_verification_not_hancom.md` 정확히 인지
5. **dry-run merge 자동 성공** + 1050 passed
6. **PR #397 와 같은 패턴** — 1차 검토 안내에 정확히 대응

### 약점 / 점검 필요

#### 1. CI action_required 정황

@cskwork 의 두 번째 PR 인데 여전히 `action_required` — PR #397 머지 후 신뢰 컨트리뷰터로 자동 등록 안 된 정황. 본 저장소의 fork PR 정책이 옵션 3 (all outside collaborators) 일 가능성. 메인테이너 승인 후 CI 실행 가능.

#### 2. samples/hwpx 의 수식 포함 hwpx 부재

작성자 본문 명시:
> samples/hwpx/ 폴더에는 `<hp:equation>` 포함 hwpx 가 부재하여 hwp 샘플로 대체했습니다.

→ 본 PR 회귀는 hwp 샘플 (한컴 origin) 만 사용. 한컴 hwpx (직접 origin) 회귀는 별도 task 후보.

## 처리 방향

옵션 A (cherry-pick 머지, PR #397 와 같은 패턴) — 권장.

이유:
1. 작성자가 1차 검토 5 항목 모두 정확 대응
2. 한컴 origin 회귀 테스트 보강 + 한컴 호환 PDF 검증 자료
3. dry-run merge 자동 성공 + 1050 passed
4. PR #397 머지 통과 후 이어지는 같은 작성자 PR — 신뢰성 확정

### 시각 판정 정황

본 PR 은 **HWPX 직렬화 (저장 경로)** 정정. dev server / 화면 시각 변화 없음 — `parse_hwp → serialize_hwpx → parse_hwpx` 라운드트립 항등성 + 한컴 호환 (작성자 PDF 검증 자료) 이 검증 게이트.

→ **시각 판정 불필요** (PR #405, #411 와 같은 패턴 — API/저장 경로 변경, 화면 변화 없음).

## 다음 단계 — 작업지시자 결정

cherry-pick 머지 진행 시:
1. `local/pr400` cherry-pick 2 commits (`f25d986`, `ecd7d9a`)
2. WASM 빌드 (Rust 변경 있으므로)
3. 머지 + push + close + 작성자 댓글 (이슈 #286 자동 close)

작업지시자 결정 부탁드립니다.

## 참고

- 검토 문서 (1차): `mydocs/pr/pr_400_review.md`
- PR: [#400](https://github.com/edwardkim/rhwp/pull/400)
- 이슈: [#286](https://github.com/edwardkim/rhwp/issues/286) (milestone v1.0.0)
- 작성자 다른 머지 PR: [#397](https://github.com/edwardkim/rhwp/pull/397) (수식 ATOP, 머지 완료)
- 한컴 호환 검증 PDF: [pr400_equation_roundtrip.pdf](https://github.com/user-attachments/files/27172903/pr400_equation_roundtrip.pdf)
