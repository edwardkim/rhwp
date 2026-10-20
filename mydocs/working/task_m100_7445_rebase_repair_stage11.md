---
kind: working
status: draft
canonical: mydocs/working/task_m100_7445_rebase_repair_stage11.md
last_verified: 2026-10-20
---

# 7445 리베이스 보정11: 저장 표 프레임의 원본 계보 유지

parent `bd21c14458d138eeb613b5a53710fcb056894a19`, base `4a07d3ae4dd3255a57fb0166b2b2a2dc335a61fa`.
#7445 전체가 아니라 기존 완료 범위와 실제 리베이스 회귀를 복구한다. PR은 새 사용자 승인 후 진행한다.

## 원 이슈 → 해결·후속 PR → 현재 실패

[#3930](https://github.com/edwardkim/rhwp/issues/3930)의 [원PR4058](https://github.com/edwardkim/rhwp/pull/4058)
본문·코멘트·diff와 merge `efed25b43a972f79e948f8c28abf863eb18aa1d8`,
[후속4763](https://github.com/edwardkim/rhwp/pull/4763)의 실제 merge
`f8c784235d7c56fb7ed4c72ca8f70762d278a734` 및 Stage182 기록을 확인했다.
원 계약은 실제 TAC 흐름·희소 바탕쪽·HWPX→HWP 저장 전후 표/이어받기 보존이다.
당시에도 전쪽 fidelity를 모두 해결했다고 하지 않았다. Stage182의 원본 저장 프레임 소유는
ZIP 컨테이너가 아니라 HWPX/HWP5 조판 계보다. 정확한 기존 검사를 완화하지 않는다.

현재 기존 main 검사는294쪽에서 FAIL이다. 원본/저장본의 해당 문단·표 속성과 앞 flow 끝은
동일한데, whole-fit의 `hwpx_container || native_hwp5_layout` 조건만 저장 HWPX→HWP를
제외한다. 원본은 저장 전체 프레임을 발행하고 저장본은 발행하지 않아 Q26의 실제 원점이
3.306667 CSS px 낮아진다. 음수 줄간격 −248HU가 원인이라는 초기 가설은 흐름 추적에서
양쪽623.146667px로 같아 기각했다. 임시 trace는 frozen495에만 보존하고 소스에서 제거했다.

## 공통 보정과 소비 경로

whole-fit 조건을 `hwpx_stored_layout || hwp5_stored_pagination_layout`로 바꾼다.
기존 RowBreak·비TAC·여러 행·단일 개체·빈 host·각주 없음·offset·다음 물리 쪽·
내부 reset 없음·실제 예산 수용 조건은 유지한다. HWPX→HWP와 HWP5 원본 HWPX도
같은 독립 원본 프레임을 소비한다. 문서/쪽 번호 예외나 좌표 clamp를 추가하지 않는다.

`query_whole_table_fit` → `WholeFit.saved_table_source_frame` → entry의 원본 top 정렬 및
저장 전체 표 배치/수용 높이 → `paragraph_float_placements` → layout의 최종 table_y_start.
예산 질의와 최종 원점이 같은 source-frame 결과를 소비한다. 별도 폰트 원인으로 단정하지 않는다.

## 실행 결과와 남은 범위

- frozen482의 기존3930 main FAIL → Native497에서 기존2검사 모두 PASS. 검사·fixture 기대값 변경0.
- 정식 Native nextest500은 원3930 및3820·7336와 원1937 대조35PASS/0FAIL/선택제외1730,
  실행5.196초,exit0. manifest check1643sources/7175attrs/48targets,fmt check도 완료했다.
- 동일 저장본 SHA `16b9510359b23c3be3ce25ae6190448101446b3ecbb4ed0c54e29b0c8320fabb`는
  보정 전후 동일하다. 원본383쪽 logical render tree 변화0, 저장본은294쪽만 변경되어 원본과 같아졌다.
  나머지382쪽은 같고 기존33쪽 source/saved 형식 차이는 이번 수정으로 새로 생기거나 해결되지 않았다.
  이는 전체383쪽 Visual Sweep을 대신하지 않는다.
- 저장본 INFO는 Hancom2010이다. 원본HWPX2024와 혼동하지 않고 exact saved 입력을
  engine2020/timeout1800/한컴11.0.0.9136/Print method0/one-up으로 출력했다. 독립PDF383쪽,
  6,320,966byte/SHA `56183a15a3cf8f3c5adfb0976afefaf94ee3258a9ce697fdafd1056a6d882076`.
- 같은 입력/그 PDF의294쪽192DPI·compat2022·PDFium smooth_rgb_v1·2px raw90%:
  Native92.38356→96.74475%, fresh WASM92.99938→98.88533%. 각383/383쪽수 일치.
  Q26 상자와 본문 원점·앞 Q24/25·쪽 소유를 review/overlay로 직접 판독했다.
  Native의 글자 굵기/간격 차이는 남으며 Native<98%여서 새 px 핀은 추가하지 않는다.
- fresh499는 locked web/pkg/no-opt 실제 재컴파일224.669초의 Mac 로컬 대체 빌드다.
  Docker 최적화 통과가 아니다. root pkg/Studio public/frozen 및504의 실제 초기 GET200·
  no-cache·JS Vite rewrite·WASM byte hash와 ready를 확인했다. actual36 문서 paint 수용은 아니다.

[전후 TSV·review/overlay·생산 원장](../pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/stored-frame-lineage/README.md).
ignored output의491~505에 원PR 조사·원인 진단·빌드/검사/Print 로그를 보존한다.
실패480/임시 trace495의 종료된 frozen CLI/lib만 hash/lsof 확인 후 정리했으며 해당 바이너리는
재사용 증거가 아니다. 원본/PDF/PNG/TSV/log·공유 target과 다른 작업장은 보존했다.

전체 PR 게이트와 완료 범위 전쪽 Native/fresh WASM·새 base 기능 보존·actual36은 아직 완료가 아니다.
다음은8064의 원본 face/수식 쪽 소유 계약과 남은 기존 실패의 독립 기대값을 확인한다.
