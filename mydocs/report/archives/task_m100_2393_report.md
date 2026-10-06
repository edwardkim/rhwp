# 최종 보고 — #2393 CI 기본 테스트 병렬화 (nextest archive + 8샤드)

- PR #2395 merge (2026-07-19, admin). 목표: wall 17분 → **10분 이내**
  (작업지시자 조정 — 5분대 한정 아님)

## 결과 (4차 PR CI 실측, cold cache)

| 항목 | before | after |
|------|--------|-------|
| run 전체 wall | ~17분 | **12.0분** (cold) / warm 6~7분 추정 |
| 테스트 임계 경로 | 8.4분 직렬 | 최장 샤드 2.1분 (8샤드 병렬) |
| lint | 테스트 뒤 직렬 | 1.9분 병렬 job |
| 공허 통과 게이트 | 없음 | sum(run)=3,277 = runnable, 8샤드 total 일치 |

cold 병목은 아카이브 job 9.5분(빌드 7.2) — rust-cache 저장은 devel/main push
한정이라 첫 PR 은 miss. devel 적재 후 warm 재실측으로 10분 이내 확정 예정.

## 디버깅 3건 (모두 PR CI 실측으로 검출·해소)

1. **CARGO_BIN_EXE 절대경로**: CLI 테스트의 `env!` 가 빌드 러너 경로를 컴파일
   타임에 고정 — 샤드 임시 추출로 NotFound. GitHub 러너의 job 간 동일
   workspace 경로를 이용해 `--extract-to .` 로 해소. 로컬 시험이 못 잡은
   이유(빌드=실행 동일 머신)도 기록.
2. **ANSI 파싱**: CARGO_TERM_COLOR=always 로 Summary 라인 색코드 — 파서가
   설계대로 "모르면 죽음". 스트립 후 파싱.
3. **ignored 의미론**: 게이트가 sum 3,277 ≠ total 3,300 검출 — 차이 23 =
   `#[ignore]`. 기대치를 `cargo nextest list` runnable 수로 정정.

②③은 공허 통과 방지 게이트의 실효 증명 — 조용히 green 이 되는 대신 두 번
죽어서 의미론 오류를 드러냈다.

## 불변 확인

required check 'Build & Test'(집계) 유지 — ruleset 무변경. preflight
fast-pass 는 마지막 docs 커밋에서 실작동 확인. artifact 액션 v7/v8 승격
(Node 20 경고 해소). native-skia/frontend/CodeQL 무접촉.

## 후속

- warm-cache wall 재실측 (다음 PR 1건) — 10분 이내 확정
- 타 워크플로(render-diff·npm-publish·full-renderer-sweep) artifact v4 잔존
  — 별도 보수 후보
- 5분대가 필요해지면 native-skia job rust-cache 화가 다음 지렛대

## 추가 — 캐시 예산 차단·정리 (2026-07-19, 옵션 A 승인)

devel 첫 run 에서 캐시 저장 거부(계정 예산 도달·읽기 전용) 발견 — 총량
10.48GB 로 무료 쿼터 초과가 원인. 스테일 14건 삭제로 4.79GB 확보(활성 키
보존: native-skia cargo 최근 2세대, codeql/frontend/wasm/node 현행). 다음
devel push 에서 저장 재개 확인 → warm 재실측으로 10분 이내 확정 예정.
