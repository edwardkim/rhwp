# #2353 최종 결과보고 — e2e 스크립트 인벤토리·관리 체계

- 이슈: [#2353](https://github.com/edwardkim/rhwp/issues/2353)
- 마일스톤: M100 / 브랜치: `local/task2353`
- 계획서: `mydocs/plans/task_m100_2353.md` (2단계 설계안이 구현계획서 대행)
- 단계 보고: `working/task_m100_2353_stage{1,2,3}.md`

## 결론

e2e 76개(디스크) 스크립트에 단일 권위 목록과 생명주기·드리프트 방지 체계를
수립했다. 관리 갱신 지점 = `rhwp-studio/e2e/MANIFEST.md` 하나, 검사 =
`scripts/check_e2e_manifest.py` (로컬 원칙).

## 산출물

1. **MANIFEST.md 72행 전수** (git tracked 기준) — 분류(상시 60/진단 6/유틸 7
   중 tracked 조정분)·상태(active/hold)·용도·샘플·배선·비고
2. **대조 검사기** — 양방향 대조(미등재/유령 FAIL) + 명명 정합(신규 강제,
   legacy-name 면제) + 배선 실재 검증 + 열거값 + **7열 명시 파싱**(셀 원문
   보존·malformed 행 FAIL·결손 집계) + **배선 열 양방향 교차 대조**
   (작업지시자 파싱 팁 반영 — 즉시 실불일치 1건·용도 결손 6건 검출·정정).
   npm `e2e:manifest-check`
3. **정리**: debug 3건 개명(비-test 접미), 폐기 3건(해소 완료 프로브·임시
   검증·중복 host 변형 — git history = archive), `.check.mjs` 관례 폐지
4. **e2e-cdp.md** 목록 절 → MANIFEST 참조 (매뉴얼 = 실행 가이드로 역할 분리)

## 과정의 발견 2건

- **공허 통과**: 존재하지 않는 파일명 e2e 실행이 FAIL 0 으로 집계되던 검증
  위생 결함을 메인테이너 자신의 최근 검증에서 발견 — 실파일 정정 실행으로
  공백을 메우고, 검사기의 "배선 실재 검증" 요건으로 구조화
- **의도적 로컬 전용**: kps-ai 계열이 .gitignore 로 로컬 전용이었음 —
  manifest 범위를 tracked 로 확정하는 근거가 됨

## 게이트

manifest 검사 72/72 · 링크 384+/메타데이터 380+ green · 대표 e2e 3종 0 FAIL
(관리 변경 무영향) · 개명 파일 기동 확인

## 운용 규칙 (요약)

- 테스트 추가 = MANIFEST 행 등재 의무 (미등재 시 검사 FAIL)
- 폐기 = 파일 삭제 + 행 제거 (stub 없음)
- 신규 명명: 상시 `.test.mjs` / 진단 `probe-`·`debug-` / 유틸 무접미
