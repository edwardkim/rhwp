---
kind: investigation
status: active
---

# #7195 4단계 — 종료 전 검증과 잔여 실패 분리

## 범위와 고정 대상

- 작업지시: 중간 커밋 후 다음 절차 진행. 잔여 문제는 별건으로 분리하고 추가 조판 구현은 확대하지 않는다.
- 중간 커밋: `19c47f1430d939956ec09bcca4bebdddabdc8168`.
- 3단계 제품·테스트·교체 입력 및 승인 기록16파일을 커밋했다. 파생suite/output은 제외했다.
- 기준base: `8d45f242baa1a565357aaa38e9f459595b1e756c`.
- fetch한 현재upstream/devel: `222c8c4819de414898bf5b15608b0d0d394a3748`.
- 검증review: `/home/edward/mygithub/rhwp-review-7195` (base에 해당 수정 사본 적용).
  root/review 추적src/tests/crates/Cargo파일 및86712 입력3507개를 byte대조하여 차이0 확인.
  새tests/cases3건도 최종 검증 입력으로 포함한다.
- 이번 검증은 중간 커밋 코드 기준이며 최신devel 통합 검증으로 주장하지 않는다.
- PR/push/merge/close는 수행하지 않는다.

## 실행

review의suite준비 완료: 1343sources /5796staticattrs /48integrationtargets.
전체회귀는 `output/7195/final-gate.mjs regression`으로 시작했다.
16logicalCPU/가용메모리23GiB 및 다른Cargo작업 없음 확인 후 buildjobs4/testthreads4로 설정했다.
정확한명령·시각·결과는 `output/7195/stage4/regression.json`, 로그는 `regression.log`에 남긴다.
완료 전 PASS로 기록하지 않는다. 기존ignore를 우회하지 않으며 #3931쪽수1건도 제외 상태를 유지한다.

후속: 회귀결과 개별분류 → Rust lint/workspace/policy 및 변경범위추가게이트 → 최신base통합영향 확인.
별건추적은 #7009/#7202/#7204를 재사용하고 새실패는 중복검색 후 분리한다.

## 최신base와의 통합 사전 점검

작업트리를 변경하지 않는 `git merge-tree --write-tree HEAD upstream/devel`은 exit1이었다.
충돌은 `typeset.rs`의 재시도cut호출 한 곳이다. #7195는
`advance_row_cut_within_capacity`, 최신devel은
`advance_row_cut_with_mixed_nested_reserve` 및 실제retry_budget반환을 사용한다.
제품 조판 계약이 겹치는 충돌이므로 어느쪽을 기계적으로 선택하지 않았다.
실제merge는 시작하지 않았으며, 현재source검증과 최신base통합검증을 분리한다.

## 전체 회귀 결과 및 별건 추적

- nextest run: `cab4e48c-d293-4f9f-af59-df7ac4aac45c`.
- 9931실행: **9903 PASS /28 FAIL /47 skipped**, exit100. build3m23s, test433.663s.
- UTC15:03:34.586–15:14:12.640(한국시간2026-09-17 00:03–00:14).
- 파생suite는review에만 준비했다. root/review의신규tests3건도byte-identical 확인.
- fmt검사PASS. 최신base222c8c4819대비suite정책(1343sources/5796attrs/48targets),
  unit-tier정책(4205tests/298modules)PASS.
- nextest0.9.137/권장0.9.140 및 미지원설정키경고는 기존과 동일. default프로필 실행.

28건은 일반활성검사 실패이며, 이전부터ignore인47건과 구분한다. 새ignore나baseline변경은
하지 않았다. #3931개별쪽수1건ignore와별개로 편람코퍼스쪽수래칫도 실패한다.
동일문서의복수검사와partition의복수문서가섞여있으므로 테스트수와문서수를혼동하지 않는다.

| 후속트랙 | 분리대상 |
| --- | --- |
| [#7205](https://github.com/edwardkim/rhwp/issues/7205) 신규 | 거대셀live flush/resumable commit/bbox3건; #7114저장재열기와구분 |
| [#7206](https://github.com/edwardkim/rhwp/issues/7206) 신규 | 보도자료8–9쪽상자소유2건, nested snap91글자초과, overflow4줄/overlap1/offcanvas1 |
| [#7207](https://github.com/edwardkim/rhwp/issues/7207) 신규 | task2097두문서21→20/2→3, 가정통신문2→3의제목/그림/꼬리소유전제 및쪽수래칫 |
| [#7208](https://github.com/edwardkim/rhwp/issues/7208) 신규 | 1376496본문초과래칫신규1건 |
| [#7209](https://github.com/edwardkim/rhwp/issues/7209) 신규 | 76076물리4쪽저장프레임하단1034.9px의기준계약불일치 |
| [#7140](https://github.com/edwardkim/rhwp/issues/7140) 기존댓글 | giant overfill47→48/부록쪽소유, overflow76→157/overlap18→20 |
| [#7095](https://github.com/edwardkim/rhwp/issues/7095) 기존댓글 | 80168 HWP/HWPX157→158·108쪽줄소유·재열기전제, 30269물리10쪽끝줄소유 |
| [#7009](https://github.com/edwardkim/rhwp/issues/7009) 기존댓글 | 편람384→385코퍼스래칫 및Q7/Q8같은쪽소유 |
| [#7204](https://github.com/edwardkim/rhwp/issues/7204) 기존댓글 | 교체86712겹침3→24; pi161과같은위치/원인인지는미확정 |

정확한28개검출이름→이슈대응은 `output/7195/stage4/failures.json` 및 `failures.md`에
저장했다. 전부미분류로, 같은입력base/current/latest통합본대조와시각판정은 후속대상이다.
새5개이슈의OPEN상태와게시본문을API조회로로컬초안과대조했다. #7195에도전체결과를게시한다.
관련있는열린이슈는재사용하고닫힌이슈는재개하지않았다.

현재단계는중간커밋과전체회귀실행·잔여실패분리까지다. 실패를PASS로바꾸거나각partition을
통째ignore하지않았다. 이 시점의 Clippy3종/workspace build/NativeSkia/추가WASM등제출게이트는
미실행이다. 최신base충돌의조판계약통합방침확정전에는기계적으로merge하지않으며,
PR생성/push/merge/close도진행하지않았다.

## 작업지시자 검토용 Docker WASM 재빌드

2026-09-17 “wasm 빌드 합니다.” 지시에 따라 중간 커밋 `19c47f143` 코드로
`docker compose --env-file .env.docker run --rm wasm`을 실행했다.
최신 devel 통합이나 제품 코드 변경 없이 최적화 포함 **2분53초, exit0**으로 완료했다.

- 산출물: `pkg/rhwp_bg.wasm`, 11,199,509 bytes.
- SHA-256: `161810b14b2c421e0290fb89a6cd90551bb97bb5a58e297a7938f3c001d219f9`.
- `node output/7195/stage4/verify-wasm.mjs`: WASM 초기화 및 교체된
  `samples/86712_regulatory_analysis.hwp` 열기 성공, 64쪽.
- 4–8·11·12쪽 render tree와 SVG 생성 smoke 검사 통과. 이번 빌드의 새 시각 판정은 하지 않았다.
- Studio `http://127.0.0.1:7700/`와 WASM 요청 HTTP200, 제공 파일과 빌드 해시 일치.
- 로그: `output/7195/stage4/wasm-docker.log`, 검증: `wasm-verification.json`.

WASM만 추가 검증했으며 전체 회귀 결과, 미실행 lint/NativeSkia 및 최신 devel 충돌 상태는 그대로다.
