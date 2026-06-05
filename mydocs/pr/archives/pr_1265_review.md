# PR #1265 검토 — 편집 삽입 탭 저장 시 tab_extended 폴백 마커 보강

- PR: https://github.com/edwardkim/rhwp/pull/1265
- 관련 이슈: https://github.com/edwardkim/rhwp/issues/1244
- 작성일: 2026-06-03
- PR 작성자: @Martinel2
- base: `devel`
- head: `fix-1244` / `be04d389fa2a75fb2df2fde4fcccbff343204d64`

## 1. 문제 요약

rhwp-studio에서 새 문서 또는 편집 과정으로 탭 문자를 삽입한 뒤 HWP로 저장하면,
한컴 편집기에서 탭이 사라지는 문제다.

HWP5 `PARA_TEXT`의 TAB `0x0009`는 문자 코드 뒤에 7개 `u16` 확장 데이터를 가진다.
기존 저장기는 `Paragraph.tab_extended`가 비어 있으면 `[0; 7]`을 직렬화했고,
이 경우 마지막 슬롯 `ext[6]`가 `0x0009`가 아니라 `0`이 되어 한컴이 탭으로 인식하지 못한다.

## 2. PR 변경 내용

변경 파일:

| file | 내용 |
|---|---|
| `src/serializer/body_text.rs` | `tab_extended` 폴백을 `[0, 0, 0, 0, 0, 0, 0x0009]`로 변경 |
| `tests/issue_1244_tab_extended_fallback.rs` | 편집 삽입 탭 1개/2개 저장 후 재파싱해 `ext[6] == 0x0009` 검증 |
| `mydocs/plans/task_1244.md` | 수행계획서 추가 |

핵심 코드:

```rust
for cu in [0u16, 0, 0, 0, 0, 0, 0x0009] {
    code_units.push(cu);
}
```

## 3. 검토 결과

수정 방향은 타당하다.

- 기존 `tab_extended`가 존재하는 HWP/HWPX 파싱 문서의 저장 경로는 그대로 보존한다.
- 편집 삽입 탭처럼 `tab_extended`가 없는 경우에만 폴백 값을 바꾼다.
- `ext[0]=0`, `ext[2]=0`은 기본 탭으로 한컴이 TabDef 기준 재계산할 수 있는 형태다.
- `ext[6]=0x0009`는 기존 분석 문서(`hwp_spec_errata`, `hwp_save_guide`)와도 정합한다.

주의점:

- PR이 추가한 `mydocs/plans/task_1244.md`는 현재 프로젝트 문서 정리 규칙상 `mydocs/plans/archives/`로 이동해야 한다.
- 이 이슈는 HWP 저장 호환성 문제이므로 SVG 시각 판정보다는 HWP 저장 후 한컴 편집기 확인이 최종 게이트에 가깝다.

## 4. 검증

PR head에서 실행:

```text
cargo test --test issue_1244_tab_extended_fallback -- --nocapture
```

결과:

```text
running 2 tests
test issue_1244_multiple_inserted_tabs_all_have_marker ... ok
test issue_1244_inserted_tab_has_marker_after_roundtrip ... ok

test result: ok. 2 passed; 0 failed
```

GitHub Actions:

| workflow | run id | conclusion |
|---|---:|---|
| CI | 26864824145 | success |
| CodeQL | 26864824115 | success |

## 5. 권장 처리

권장: **PR 변경을 수용하되, 메인테이너 통합 브랜치에서 문서 위치를 정리한 뒤 병합**.

진행 순서:

1. `devel` 기준 통합 브랜치 생성
2. PR #1265 커밋 반영
3. `mydocs/plans/task_1244.md`를 `mydocs/plans/archives/task_1244.md`로 이동
4. `cargo fmt --all --check`
5. `cargo test --test issue_1244_tab_extended_fallback -- --nocapture`
6. 필요 시 저장 산출물로 한컴 편집기 확인
7. `devel` 병합, 전체 회귀 확인, push
8. PR #1265 및 이슈 #1244 종료 처리

## 6. PR 코멘트 초안

```markdown
검토했습니다. 편집 삽입 탭에서 `tab_extended`가 없는 경우 `ext[6]=0x0009` 마커를 보강하는 방향이 이슈 #1244의 원인과 잘 맞습니다.

신규 roundtrip 테스트도 탭 1개/2개 케이스를 직접 검증하고 있어 수용 가능하다고 판단했습니다.

프로젝트 문서 정리 규칙에 맞춰 `mydocs/plans/task_1244.md`는 메인테이너 통합 과정에서 `mydocs/plans/archives/`로 이동해서 반영하겠습니다.
```
