# #5511 Stage 2 기능군 배치 C0 — edit command runtime seam

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 최종 통합 기준: `upstream/devel` `cfe2c351e834d7579a521c8ed7f6839674cc9ad1`
- 최종 코드 HEAD: `4ee2e40ff1be81a21d86033d5b44b4ca9e22750f`
- 최종 통합 HEAD: `42f50aab21974affad59b68ba38d4411ca3d081c`
- 수행일: 2026-08-20
- 상태: 완료 — C0 종료, C1 진입 승인 대기

## 1. 결과

`src/main.rs`의 edit 하위 명령 dispatch, output format·serialize·verify·write와 CAS 무결성
구현을 책임별 소유 모듈로 분리했다.

```text
src/cli/
├── integrity.rs
└── commands/
    └── edit/
        ├── mod.rs
        └── runtime.rs
```

| 책임 | 최종 파일 | 줄 수 |
|---|---|---:|
| SHA-256·CAS path lock·debug 동시성 hook | `src/cli/integrity.rs` | 109 |
| edit 하위 명령 88개 dispatch | `src/cli/commands/edit/mod.rs` | 141 |
| output format·serialize·verify·write·edit CAS report | `src/cli/commands/edit/runtime.rs` | 278 |

세 파일은 모두 1,200줄 상한 이하다. `src/main.rs`는 C0 시작의 15,621줄에서 15,140줄로
481줄 줄었다. C0 inventory의 실제 구현 482줄을 이동했고 새 소유 경로 import와 최상위 dispatch
직접 호출의 순증감을 포함한 root 순감소가 481줄이다. root wrapper와 hash·serializer 복제는
남기지 않았다.

## 2. 최소 runtime API 결정

범용 `sha256_hex_of`, `CasPathLock`, debug CAS hook은 edit replace-text와 protocol plan이 함께
사용하므로 어느 command에도 속하지 않는 `cli::integrity`가 소유한다. 기대 hash 불일치 봉투는
provenance command가 `edit`으로 고정된 edit 전용 계약이므로 `edit::runtime`이 범용 integrity를
소비해 출력한다. 이 분리로 plan이 edit command를 역참조하거나 두 CAS 구현이 갈라지지 않는다.

광범위한 `EditContext`는 만들지 않았다. 현재 handler 88개는 서로 다른 인자, 좌표, 자산과 story를
가지며 parsing·load·domain mutation을 각자 소유한다. 이 단계에서 모든 의존을 한 구조체에 넣으면
C1~C6 책임을 미리 결합한 새 god object가 된다. 공통 runtime은 기존처럼 저장할 문서, 원본 bytes,
경로, 출력·검증 옵션과 변경 문단을 명시적으로 받는다. 기능군별 handler를 실제 이동하면서 반복되는
입력 구조가 확인될 때 좁은 argument/context를 결정한다. `DocumentService`, typed error와 전역 인증
제거는 계획대로 Stage 3 범위다.

CSV import, batch fill, protocol plan, MCP session과 아직 root에 있는 edit handler는 새 소유 경로를
직접 참조한다. output 형식 보존, HWP adapter, session snapshot, 재파싱 검증, provenance와 CAS의
공개 동작은 바뀌지 않았다. C0 경로의 `clippy::cognitive_complexity` 25 초과 경고는 0건이다.

## 3. 커밋 계보

| 커밋 | 역할 |
|---|---|
| `c8803fe0a` | C0 범위·101개 보호 계약·최소 runtime API inventory |
| `896677e69` | edit와 protocol plan이 공유하는 범용 CLI integrity seam 이동 |
| `4ee2e40ff` | edit dispatch와 output·verify·write runtime 이동 |
| `42f50aab2` | 완료 직전 전진한 최신 `upstream/devel`의 #5693 Studio 변경 정상 merge |

## 4. 직접 계약

이동 전과 최종 코드 HEAD에서 9개 직접 계약 모듈 101/101을 통과했다.

| 계약 축 | 모듈 | 건수 |
|---|---|---:|
| edit 적용·형식·검증 | `edit_fill_fields_contract`, `edit_replace_text_contract`, `edit_format_preserve_contract`, `edit_verify_contract` | 23 |
| MCP session snapshot | `mcp_session_edit_contract` | 8 |
| plan·edit CAS | `run_plan_cas_contract` | 15 |
| CSV command 저장 | `table_csv_contract` | 14 |
| JSON·provenance 봉투 | `cli_json_contract`, `provenance_contract` | 41 |

debug-only CAS 경합 계약은 동일한 기대 hash를 가진 in-place plan 둘 중 정확히 하나만 commit함을
다시 확인했다. inventory 중 오래된 release artifact로 처음 실행한 provenance 2건 실패는 현재
CLI보다 앞선 embedded binary 때문이었고, 현재 소스로 다시 빌드한 계약은 10/10 통과했다.

## 5. 최종 검증

| 검증 | 결과 |
|---|---|
| C0 직접 focused 계약 | 이동 전·최종 101/101 통과 |
| 최종 release-test 전체 nextest | 8,005/8,005 통과, 3 slow, 38 skipped, 159.763초 |
| `cargo fmt --all -- --check`·`git diff --check` | 통과 |
| `cargo check --locked --all-targets` | 통과 |
| `cargo clippy --locked --all-targets -- -D warnings` | 통과 |
| C0 모듈 CC 25 상한 | 초과 경고 0건 |
| `cargo test --locked --doc` | 8/8 통과, 3 ignored |
| integration manifest 정책·현재 상태 | 18/18, 803 sources / 3,956 attrs / 41/48 targets 통과 |
| unit-tier 정책·현재 상태 | 12/12, 4,225 tests / 299 modules 통과 |
| CI impact Node·Python workflow 계약 | 62/62, 163/163 통과 |

전체 회귀는 매뉴얼의 고정 target 명령인 `--cargo-profile release-test --target-dir
target/pr-review --tests --test-threads 8 --no-fail-fast`로 실행했다. 변경분 release-test 재컴파일은
1분 9초였고 실제 8,005개 실행은 159.763초였다. 로컬 nextest 0.9.137이 저장소 권고 0.9.140보다
낮다는 경고가 있었지만 전체 모집단은 정상 실행되어 전건 통과했다.

`rust-test-suite-manifest --prepare`가 확인한 파생 harness와 Cargo 파생 순서는 추적 변경에
포함하지 않았다. C0는 move-only CLI runtime 변경이므로 renderer·layout·WASM·native-skia·시각
검증 발생 조건에 해당하지 않는다.

## 6. 최신 devel과 열린 PR

전체 회귀 뒤 `upstream/devel`과 `origin/devel`이 `b914bdf4b`에서 `cfe2c351e`로 한 커밋
전진했다. 이 커밋은 #5693의 `rhwp-studio` master-page picture hit 처리와 해당 문서만 변경했고
C0 경로와 겹치지 않았다. merge-tree 무충돌을 확인한 뒤 `42f50aab2`에서 정상 merge했다.
`4ee2e40ff..42f50aab2`의 Rust·Cargo·tests·scripts·workflow tree는 완전히 같으므로 C0 전체
검증의 코드 모집단도 최종 통합 HEAD에서 그대로다.

최종 통합 뒤 열린 devel 대상 PR은 #5647, #5689, #5691, #5695, #5707, #5709, #5710,
#5718, #5719다. 각 최신 head의 변경 경로를 다시 조회했으며 C0의 root, MCP, CLI integrity·
command·protocol plan, 직접 계약과 #5511 C0 문서 경로에 겹침이 없다. 최종 통합 HEAD는 최신
`upstream/devel`을 조상으로 포함하며 원격보다 35개 커밋 앞서고 뒤처진 커밋은 없다.

이 판정은 시점 증거다. 향후 push 직전에 exact base SHA, 열린 PR head와 merge 가능성을 다시
확인한다. 이 보고서 작성 시점에는 remote push를 수행하지 않았다.

## 7. 다음 승인 단위

C0 완료로 후속 edit command가 공유할 물리 runtime 경계를 확정했다. 다음 기능군은 C1
field·text·replace·redact·sanitize이며 입력 무훼손, target occurrence, 개인정보 제거와 저장 검증
계약을 먼저 inventory한다. C1은 메인테이너의 C0 완료 승인과 별도 진입 승인 전 시작하지 않는다.
