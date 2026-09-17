---
kind: investigation
status: completed
---

# #7195 7단계 — 종료 전 로컬 검증

2026-09-17 작업지시자의 다음 처리 승인에 따라 stage6 수정·증적을
`d54459747`에 커밋했다. 추가 조판 수정, 기준값·ignore 변경, 원격 push와
PR 생성은 이번 실행 범위가 아니다.

## 검증 기준

- 제품·테스트: `d54459747`.
- 격리 실행: `/home/edward/mygithub/rhwp-review-7195-integration`의 cb151 HEAD에
  동일 소스·테스트 diff를 적용한 상태. 실행 전 두 파일의 바이트 동일성을 확인한다.
- stage6 전체 회귀: 9932 PASS /30 FAIL /47 skipped. 신규 실패0, 해결0이다.
  커밋 외 제품 변경이 없어 이 결과를 재사용하며 전체 통과로 보고하지 않는다.
- Clippy 3종, workspace build, suite/unit 정책, Native Skia 3종을 검증한다.
  Rust 명령은 순차 실행하고 현재 자원에 맞춰 build jobs와 test threads를 4로 제한한다.
- 각 명령·시각·소스 diff/hash·종료값은 `output/7195/stage7/`에 보존한다.

## 결과

`git fetch upstream devel` 후 원격은 `fcbd00e0fabc4b309a887357033f92e2d511cd75`로
변경되지 않았다. review worktree의 수정 두 파일은 커밋본과 바이트 동일하다.

| 게이트 | 결과 |
| --- | --- |
| suite prepare, format, fmt check | PASS |
| Native Clippy | PASS (55초) |
| WASM Clippy | PASS (52초) |
| workspace build | PASS (1분28초) |
| workspace all-targets Clippy | PASS (1분22초) |
| suite policy | PASS: 1350 sources, 48 targets |
| unit tier policy | PASS: 4205 tests |
| Native Skia lib | 3929 PASS /3 FAIL /11 ignored |
| Native Skia picture | 2 PASS (선택하지 않은 202건 제외) |
| Native Skia PDF | 4 PASS (선택하지 않은 192건 제외) |

실행 명령은 `output/7195/stage7/gates.mjs`, 각 결과는 같은 폴더의
`<gate>.json`과 `<gate>.log`에 보존한다. 기존 실패와 새 실패를 분리하며,
미실행 게이트는 통과로 간주하지 않는다.

Native Skia lib 실패3건은 기존 전체 회귀의 다음 항목과 동일하다.

- `issue2214_scoped_cache_coherence_preserves_transient_pagination`
- `issue2424_resumable_pagination_commits_only_after_final_fragment`
- `test_get_table_bbox_at_page_for_giant_multi_page_cell`

`skia-failure-comparison.json`에서 세 항목의 assertion 및 비교값까지 동일함을 확인했다.
nextest와 libtest의 로그 들여쓰기만 정규화했다. #7205 후속 범위이며, 이번에 새로
검출된 Skia 전용 실패로 세지 않는다. 그렇다고 실패한 게이트를 PASS로 바꾸지 않는다.

## 최종 판정과 다음 경계

- 모든 명령은 동일 제품·테스트 diff SHA256
  `7c6f7e8a1fce4223f68669e1eed06c25320b60ddfa14f10cd36c628ea2883987`로 실행했다.
- Clippy 3종과 workspace build·정책 게이트는 통과했다. Native Skia 3종은 모두
  실행했으며, lib의 기존 실패3건 때문에 묶음 전체는 통과가 아니다.
- 전체 회귀의 30건 실패는 여전히 남는다. 이번 추가 검증에서 별도의 신규 실패는
  검출되지 않았다. 기존 결함을 해결했다고 보고하거나 기준값·ignore로 숨기지 않았다.
- 설치 nextest 0.9.137이 저장소 권고 0.9.140보다 낮다는 경고와 관측 profile의
  `junit.report-skipped` 키 경고가 있다. 이번 default profile 실행은 완료되었으며,
  이 경고를 없애기 위한 도구·정책 변경은 하지 않았다.
- Docker WASM·시각 증거는 같은 제품의 stage6 결과를 참조한다. 이번에는 새 WASM을
  만들지 않았고 한컴 fidelity 통과를 추가 선언하지 않았다.
- 다음은 기존 후속 이슈/잔여 실패의 제출 처리 방침을 확정하고 완료 보고·PR 준비를
  진행하는 단계다. 전체/Skia 실패가 남아 있는 사실을 명시해야 하며, 이번 실행으로
  예외 승인이나 원격 push·PR 생성 권한이 생긴 것으로 간주하지 않는다.

이번 단계에서 추가 제품 변경, baseline·ignore 변경, 원격 게시·push·PR 생성은 하지 않았다.
