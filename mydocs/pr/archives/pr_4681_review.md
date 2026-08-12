---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4681 검토 - HWPX 고정폭 빈칸 직렬화

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4681](https://github.com/edwardkim/rhwp/pull/4681) |
| 작성자 / source | @planet6897 / `fix/4675-hwpx-fwspace-element` |
| base / source head | `devel` / `5e8bec8877daae881db8ec539d40799552ab66ce` |
| 규모 | 1 file, +55 / -0, 2 commits |
| reviewer | @jangster77 지정 완료 |
| mergeable 참고값 | 작성 시점 `MERGEABLE` / `CLEAN` |
| 관련 이슈 | [#4675](https://github.com/edwardkim/rhwp/issues/4675), 본문 `closes #4675` |
| 통합 검토 branch | `review/planet6897-20260812-r2` |

파서가 `<hp:fwSpace/>`를 U+2007로 읽은 뒤 serializer가 리터럴 문자로 강등하던 경로를,
고정폭 빈칸 요소로 되돌린다. 두 번째 commit은 U+00A0을 요소로 강제한 초기 변경을 되돌려,
묶음 빈칸은 리터럴로 보존한다.

renderer geometry를 바꾸지 않는 XML 표현·텍스트 추출 계약 변경이므로 PDF fidelity sweep은
적용 대상이 아니다. serializer의 XML과 parse-serialize-parse roundtrip을 직접 검증했다.

## 완료한 검증

- `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --lib hwpx
  --no-fail-fast`: 638/638 통과.
- `hwpx_roundtrip_baseline`, `hwpx_roundtrip_integration`,
  `issue_1868_export_hwpx_cli`, `hwpx_form_roundtrip`: 31/31 통과.
- 새 `issue4675_fixed_width_space_serializes_as_element`는 U+2007 세 개의
  `<hp:fwSpace/>` 방출, 리터럴 U+2007 부재, U+00A0 리터럴 보존, 재파싱 뒤 원 IR text
  보존을 확인했다.
- 통합 candidate 전체 `nextest`: 5,881/5,881 통과. `cargo fmt --check`,
  `cargo clippy --target-dir target/pr-review --all-targets -- -D warnings`,
  `wasm-pack build --target web --out-dir pkg`, `git diff --check`도 통과했다.

## 판단

**통합 수용 권고.** U+2007과 U+00A0의 원 표현 구분은 현재 IR에 없으므로 U+00A0 표현
fidelity를 더 높이는 작업은 별도 과제로 남는다. 이번 변경은 U+2007의 관측된 한컴 표기와
roundtrip 계약을 고정한다. 통합 PR 병합 뒤 본문의 closing keyword에 따라 #4675 자동 close
여부를 확인하고, 자동으로 닫히지 않으면 검증 근거를 적어 close한다.
