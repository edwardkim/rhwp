---
kind: working
status: done
canonical: mydocs/working/task_m100_5447_stage1.md
last_verified: 2026-08-18
---

# #5447 Stage 1 — B2 구조 변종 하네스 + 판정 번들

- **계획서**: [`mydocs/plans/task_m100_5447.md`](../../plans/archives/task_m100_5447.md)
- **번들**: `output/issue_5447_b2_judgment/` (gitignored) — 38 파일 + `PANJEONG.md`
- **생성기**: `tests/issue_4100_chart_data_edit.rs::generate_b2_structure_judgment_bundle` (`#[ignore]`)
- **브랜치**: `task5447` (`upstream/devel` `e79f11308` 기준)

## 1. 무엇을 만들었나 — Stage 7 (기존 예외 타깃 확장, 신규 파일 0)

`tests/issue_4100_chart_data_edit.rs` 끝에 Stage 7 을 추가했다. 신규 테스트 파일을 만들지
않은 이유: 이 파일은 `suite-policy.json` exceptions 등재 예외 타깃이라 기존 헬퍼
(`core_of`·`replace_chart_representations`·`chart_streams`·`manifest`)를 그대로 쓸 수 있고,
새 파일로 가면 `tests/cases/` 배치 계약과 헬퍼 중복이 생긴다. 기존 파일 수정은 CI 의
`validateAddedSourcePlacement`(추가 파일만 검사) 대상이 아니며, `tests/provenance_contract.rs`
를 #4976 이후 수정한 merge 선례(PR #5122)가 있다.

### 바이트 수술 헬퍼 (재직렬화 금지 — 문자열 구간 치환)

구조 변종은 `set_chart_data_native` 의 fail-closed 검증을 지나갈 수 없으므로, 코퍼스 XML
(한컴 단일 라인 기계 생성 — 28종 균일 실측)을 문자열 수술로 가공했다. #5447 정책 3종을
그대로 구현한다:

| 헬퍼 | 역할 | 정책 |
|---|---|---|
| `b2_add_point` / `b2_remove_point` | cat/val/xVal/yVal 캐시에 점 추가·삭제 | `ptCount` ±1 재계산(§3-2), 삭제 후 `b2_renumber_pt_idx` 로 0..n-1 전수 재번호(§3-3), `c:f` 무갱신(§3-1) |
| `b2_clone_last_series` / `b2_remove_series` | 계열 복제(+`c:idx`/`c:order` 채번)·삭제(잔여 재번호) | 복제 계열의 `c:f` 는 일부러 원본 그대로(§3-1 실험) |
| `b2_rename_series` / `b2_relabel_category` | c:tx / c:cat 캐시 텍스트 교체 | 라벨은 전 계열 동기 수정(sharedCategoryRequired) |
| `replace_chart_nested_only` | HWP5(①없음) ②만 교체 | `replace_ole_stream` 경유 — 루트 CLSID·③④ 보존(#4097) |

### 자가검증 (산출마다)

`scan_chart_values` 재통과 → export → `from_bytes` 재개방 → 봉투 구조 확인(계열 수·라벨·값)
→ ①==②==변종 XML 바이트 → **③레거시·④프리뷰 바이트 불변**. 한컴이 못 열었을 때 편집 탓인지
조립 탓인지 가리기 위한 #4055 관례다.

### CI 상시 회귀 (비-ignore 프로브 2종)

- `b2_category_row_surgery_roundtrips_and_renders` — 행추가/행삭제 왕복 + **렌더 반영**
  (SVG 에 「추가항목」 등장, 지운 「항목 2」 소멸)
- `b2_series_surgery_renumbers_and_reopens` — 계열 채번/재번호 + 양 포맷 재개방

## 2. 판정 번들 — 38 파일

- 본선 6종(기준 문서 묶은세로막대형: 행추가·행삭제·계열추가·계열삭제·계열명변경·라벨변경) × 2포맷 = 12
- 경계 2종(원형대원형-계열추가, 시가고가저가종가-계열삭제) × 2 = 4
- 종류 커버리지 6종(묶은가로·꺽은선·분산형 점추가·특이케이스 numLit 점추가·**누적-계열삭제**·**3D-행추가**) × 2 = 12
- 대조군 9 + 변환 축 1(`묶은세로막대형-행추가-HWPX에서변환.hwp`)

`PANJEONG.md` 는 (a)오류창 (b)기대 모양 (c)편집기 열림 (d)**편집기 행·열 수** 4항목을
요청한다 — (d)가 S2(한컴 편집기의 `c:f` 재해석 여부 = `c:f` 무갱신 정책이 뒤집힐 유일 지점)다.
변종마다 「낡게 남긴 것」(c:f 범위·③·④)을 표에 명시해 한컴 반응의 원인을 가릴 수 있게 했다.

## 3. 검증 실측 (2026-08-18, dev profile)

| 게이트 | 결과 |
|---|---|
| `cargo test --test issue_4100_chart_data_edit` 전체 | **37 passed / 0 failed** (2 ignored = 판정 번들 생성기 2종), 398.8s |
| B2 프로브 2종 | ok (44.3s) |
| `generate_b2_structure_judgment_bundle -- --ignored` | ok — 38 파일 + PANJEONG.md, 자가검증 전건 통과, 488.8s |
| `cargo fmt --all` + `-- --check` | 통과 (기존 추적 파일 재포맷 0건) |
| `node scripts/rust-unit-test-tiers.mjs --check` | 통과 — src 무변경 (4225 tests / ratchet 그대로) |
| `cargo clippy --test issue_4100_chart_data_edit -- -D warnings` | **통과** (exit 0) |
| `node scripts/rust-test-suite-manifest.mjs --generate` → `--check` | **통과** — 32 harnesses / 9 exceptions, 693 sources / 3163 static test attrs |
| `cargo clippy --all-targets -- -D warnings` | **통과** (exit 0, `--generate` 후) |
| `git diff --check` | 통과 |

비고: `tests/generated/` 는 `.gitignore:6` 대상이라 fresh worktree 에 없다(CI 가 `--prepare` 로
만드는 파생물). 그 상태에서는 `rust-test-suite-manifest.mjs --check` 가 32건 drift 로 exit 1 이고
`cargo fmt --all` 도 그 경로들에 "does not exist" 를 낸다 — **검증 실패가 아니라 파생물 부재**다.
`--generate` 로 하니스를 만든 뒤 `--check`·`fmt --check`·`clippy --all-targets` 가 모두 성립하며,
생성물은 gitignore 대상이라 작업 트리를 더럽히지 않는다(`git status` 로 확인). 초판은 이 절차를
밟기 전이라 대상 테스트 크레이트를 명시 지정했고, 지금은 `--all-targets` 로 갈음한다.
`--profile release-test` 전체 회귀는 CI(nextest, `--prepare` 후)가 수행한다 — 로컬은 cold 빌드
비용(3h+) 때문에 dev profile 로 동일 로직을 검증했다.

## 4. 지킨 계약

- `samples/chart/` 무변경 — `checked == 56` 게이트 2곳·③ 일치 게이트·fixture baseline 무접촉
- `src/`·렌더러·studio 무변경 — 프로덕션 동작 변화 0, tiers 래칫 무접촉
- ③·④ 바이트 불변 — 산출 38건 전건 자가검증으로 고정
- 편집하지 않은 차트 blob 보존(`issue_3546`) — 전체 회귀에 포함돼 green

## 5. 다음

**완료** — 한컴 2022 판정 회신(PDF 38건 + 편집기 행 수)을 144DPI 래스터 해시로 갈랐다.
결과와 B2 본구현 권고는 [`../report/task_m100_5447_report.md`](../../report/archives/task_m100_5447_report.md).
정책 3종 전건 통과(S2 도 양성 — 편집기가 5행을 보여 `c:f` 갱신 재협의는 불요), 경계 2종은
한컴이 막지 않아 B2 엔진이 fail-closed 로 막는다.
