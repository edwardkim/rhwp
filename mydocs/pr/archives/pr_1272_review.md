# PR #1272 리뷰 — HWPX 탭 폭/leader/type 직렬화

- PR: https://github.com/edwardkim/rhwp/pull/1272
- 관련 이슈: https://github.com/edwardkim/rhwp/issues/1267
- 작성일: 2026-06-03
- 작성자: @Martinel2
- 처리 브랜치: `local/pr1272-integration`

## 1. PR 요약

PR #1272는 HWPX 시리얼라이저에서 `<hp:tab>`가 항상 `width="4000" leader="0" type="1"`로 저장되던 문제를 수정한다.

기존 경로:

- `src/serializer/hwpx/section.rs`
- `render_hp_t_content(text)`가 탭마다 `TAB_DEFAULT_WIDTH`만 사용
- `Paragraph.tab_extended`에 저장된 실제 탭 폭/leader/type 정보를 참조하지 않음

PR 변경:

- `render_hp_t_content(text, tab_extended, tab_idx)`로 시그니처 확장
- 탭 문자마다 `tab_extended[tab_idx]`를 순서대로 소비
- `ext[0]`을 width, `ext[2] & 0xff`를 leader, `ext[2] >> 8`을 type으로 복원
- `ref_mixed.hwpx` 기반 roundtrip 테스트 2개 추가

## 2. 변경 파일

| file | 판정 |
|---|---|
| `src/serializer/hwpx/section.rs` | 탭 직렬화 로직 수정 |
| `tests/issue_1267_hwpx_tab_and_diagonal.rs` | 탭 roundtrip 테스트 추가 |
| `mydocs/plans/archives/task_m100_1267.md` | archive 이동 완료 |
| `mydocs/plans/archives/task_m100_1267_impl.md` | archive 이동 완료 |

## 3. 현재 devel 기준 검증

PR head는 GitHub 기준 `BEHIND`였으므로, 현재 `devel`에서 `local/pr1272-verify`를 만들고 PR 커밋 3개를 cherry-pick했다.

```text
git checkout -B local/pr1272-verify devel
git cherry-pick 891f4b60 f1aeeeb2 9f6f9e9
```

결과:

- 충돌 없음
- `cargo fmt --all --check` 통과
- `cargo test --test issue_1267_hwpx_tab_and_diagonal -- --nocapture` 통과

GitHub PR head 기준 기존 체크도 통과 상태였다.

- Build & Test: pass
- CodeQL: pass
- WASM Build: skipped

## 4. 리뷰 관찰

### 4.1 탭 수정은 방향이 맞음

`Paragraph.tab_extended`가 이미 HWPX parser 및 HWP save adapter 경로에서 사용하는 IR이므로, HWPX 재직렬화에서도 동일 IR을 참조하는 것이 맞다.

특히 컨트롤 슬롯 때문에 텍스트가 여러 fragment로 나뉘는 경우에도 `tab_idx`를 `render_run_content` 전체에서 하나만 유지하므로, 문단 내 탭 순서를 보존하는 설계다.

### 4.2 문서 위치는 현행 archive 정책과 맞지 않음

PR이 추가한 문서는 다음 위치에 있다.

- `mydocs/plans/task_m100_1267.md`
- `mydocs/plans/task_m100_1267_impl.md`

최근 정리 정책상 `mydocs/plans`, `mydocs/report`, `mydocs/working` 루트에는 진행 중 문서만 두고 완료/PR 반영 문서는 `archives/` 아래로 이동해야 한다.

수용 처리에서 다음 위치로 이동했다.

- `mydocs/plans/archives/task_m100_1267.md`
- `mydocs/plans/archives/task_m100_1267_impl.md`

### 4.3 issue #1267 본문에는 사선 직렬화도 포함되어 있음

이슈 #1267의 제목은 탭 폭 고정이지만 본문에는 별도 문제도 같이 적혀 있다.

- HWPX parser는 `hh:slash` / `hh:backSlash` type을 `BorderFill.attr` bit로 파싱
- HWPX serializer는 현재도 `write_diag_line(w, "hh:slash")` / `write_diag_line(w, "hh:backSlash")`에서 항상 `type="NONE"` 출력

현재 `src/serializer/hwpx/header.rs` 상태:

```rust
write_diag_line(w, "hh:slash")?;
write_diag_line(w, "hh:backSlash")?;
```

따라서 PR #1272는 #1267의 탭 문제는 해결하지만, 사선 roundtrip 문제까지 해결하지는 않는다.

후속 추적 이슈를 별도로 등록했다.

- https://github.com/edwardkim/rhwp/issues/1278

## 5. 권장 처리

승인된 처리:

1. PR #1272는 탭 직렬화 수정으로 수용한다.
2. 문서 파일은 archive 정책에 맞춰 이동한다.
3. 사선 `slash/backSlash` 직렬화 문제는 PR #1272 범위 밖으로 분리한다.
4. issue #1267은 탭 이슈로 완료 처리하고, 사선은 #1278에서 추적한다.

관리상 권장은 첫 번째다. 이슈 제목이 탭 폭 고정이고, 사선은 다른 serializer 테이블인 `header.rs`의 borderFill 문제이므로 별도 이슈로 분리하는 편이 추적이 쉽다.

## 6. 다음 단계

승인 시 진행:

1. `local/pr1272-integration` 생성
2. PR 커밋 cherry-pick
3. 계획 문서 archive 이동
4. 필요한 경우 사선 직렬화 후속 이슈 등록
5. 로컬 검증
   - `cargo fmt --all --check`
   - `cargo test --test issue_1267_hwpx_tab_and_diagonal -- --nocapture`
   - `cargo check --target wasm32-unknown-unknown --lib`
   - 필요 시 전체 `cargo test --tests --quiet`
6. `devel` 병합 및 원격 CI 확인
