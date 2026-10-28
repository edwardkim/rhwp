# #7445 리베이스 보정 65단계: 닫힌 사다리 상자의 저장 원점 공유

## 범위와 원래 계약

기존 입력 목록 35번 `samples/issue6032/2912695_civil_petition_form.hwp`와 기존 독립 한컴
2020 Print PDF `pdf/2912695_civil_petition_form-2020.pdf` 전 2쪽을 사용했다. 새 HWP 보정
대상을 추가하지 않았다. 원본·PDF·기존 검사·허용치·기대값은 변경하지 않았다.

#4068의 [원 PR #7538](https://github.com/edwardkim/rhwp/pull/7538), 원 기여 `5e4a82579`,
통합 #7576의 `2743e27883`, 메인터너 관계 보강 `b26709b8ab`, 이전 닫힌 저장 프레임 보정
`a7703b07a2`를 추적했다. 원래 계약은 첫 표의 단머리 paint 여백을 바꿔도 별도 저장 원점을
가진 뒤 표를 밀지 않는 것이다. 현재 닫힌 lane 보정 `256c7f024d`의 상자 우선권과
공동 앵커 보정 `657aa5fa13`도 대조했다.

## 공통 원인과 수정

원본은 수정 전부터 Native/fresh WASM 전 2쪽 raw 100%였다. 기존 여백 0 대조는 첫 표를
110.3→108.5 CSS px로 정상 이동했지만 둘째 표가 888.9→897.9로 내려 기존 관계 검사만 실패했다.

`empty_float::prepare`의 `closed_source_placement`는 현재 커서 779.626667에서 먼저 만들어졌다.
뒤의 `closed_control_frame`은 원본 앞뒤 vpos와 닫힌 후속 줄로 anchor 770.586667을 복구했지만,
앞에서 만든 사다리 상자가 우선권을 가져 lane 789.44를 전달했다. 원본에서는 두 원점이 같아
문제가 보이지 않았고, 앞 표의 paint 여백이 바뀐 대조에서만 흐름이 갈라졌다.

기존 `native_empty_topbottom_rewind_anchor_saved_para_y` 질의를 먼저 수행하고 인증된 원점을
사다리 상자의 생성부터 전달했다. 원본 조건과 기존 24px drift 경계, 예산·다음 줄 닫힘·수용 제한을
바꾸지 않았다. 충돌 회피와 lane 이동은 계속 같은 확정 상자를 소비한다. 문서 ID·쪽 번호·
고정 이동량 분기를 추가하지 않았다. 보정 후 대조의 두 원점은 770.586667로 같고, 둘째 표는
원본·여백 0 모두 888.9에 남는다. 새 진단 코드는 생산 코드에 추가하지 않았다.

## 실제 검사와 전쪽 시각 비교

- 기존 #4068 실패 함수는 수정 전 FAIL → 수정 후 PASS다. 수정 전후 같은 진단 harness도
  1 FAIL → 1 PASS이며 실제 원점·두 표 위치를 출력한 로그를 보존했다.
- 관련 #4068·#6032·#6950·#6448·#7687 및 기존 잔여 목록의 집중 55건은
  **46 PASS / 9 FAIL**, 테스트 5.872초다. 전체 nextest가 아니다. 기존 실패 하나를 해결했고
  나머지 9개는 그대로 남았다. 검사나 기대값을 수정해 통과시키지 않았다.
- 96 DPI / compat 2022 / PDFium 153.0.7999.0 / `pdfium_smooth_rgb_v1` / raw 2px 90%로
  전 2쪽을 다시 산출했다. 보정 전후 Native와 fresh WASM은 각각 두 쪽 모두 **100%**이며
  같은 backend의 전후 TSV와 PNG는 byte 동일하다. backend 간 RGB 동일성을 주장하지 않는다.
- 보정 후 두 쪽 review와 overlay를 양쪽 backend에서 직접 판독해 표 테두리·작성요령·
  후속 용지규격 문장·내용 순서와 소속을 확인했다. 작은 획 농도·테두리의 엄격 픽셀 차이는
  남으며 raw 100%가 RGB 완전 일치라는 뜻은 아니다.
- 기존 `rowbreak-problem-pages.hwp`(compat 2024)와 HWPX(compat 2022)의 Native 전 18쪽씩
  RenderTree 36개가 직전 후보와 byte 동일하다. 이는 해당 출력 구조 보존의 근거이며
  새 head의 전체 WASM 시각 검사나 최종 devel 비교를 대신하지 않는다.
- fmt 및 manifest prepare는 빌드 전에 완료했다. manifest 확인은 1,736 sources /
  7,609 static test attrs / 48/48 targets, unit tier는 4,101 tests / 296 modules로 통과했다.
- Mac locked web/pkg/no-opt fresh WASM은 221.871초, exit 0이다. Docker 최적화 통과가 아니다.
  root pkg/Studio public/frozen package의 JS·WASM SHA가 같다. 실제 Studio 초기화 응답은 아직 미검증이다.

## 생산 출처

- 부모 HEAD: `ac4ba552d4561a8ca21208e354193db78145e7a4`
- base: `605ccfd92f2f12f18e14182548d729f9929fda00`
- `empty_float.rs` SHA-256: `6e30ce1f9f631d991d5272743e13c384ed7a98c95f8d2e20bcfb89b6cab8f256`
- Native SHA-256: `6446dc3dda80eee9defb263220235b04c2c5899240e2591d32b8615864eca57c`
- JS SHA-256: `aea5f6639676442e0628b640b75d0f70eb300481421112d23b661a082f695a68`
- WASM SHA-256: `889efac8f33740be1933c7155f792b3e457e24fe6286efbae1986964a4cfb2fb`
- 입력 SHA-256: `1c6569dc69b1fe99de058de2760249d322b393b7eb6b4f2a2fe60f2489c25e04`
- 독립 PDF SHA-256: `2e85455f13d0e911b495a7a152f1c799ffb2fc37dd2051312cafa4359c810f89`

정식 전후 TSV 4개와 review·overlay는
`mydocs/pr/assets/issue7445/completed-20261009/rebase-605ccfd9-20261022/civil-closed-frame-origin/`에 있다.
전체 로그·생산 manifest·원인 진단·구조 대조·검증 JSON은 ignored
`output/pr-review/7445-completed-20261009/rebase-8155-20261022/stage65-civil-closed-frame/`에 있다.
이전 64단계의 내용 감사는 공백·제어문자 정규화 후 대조였음을 해당 설명에도 명확히 했다.

## 남은 작업

5734, task1725, 1937, overlap partition 12/7, biotoxin, roadmap Finland, 7379, 7470의
기존 실패 9건을 계속 처리한다. 최신 전체 nextest, 범위별 필수 게이트, 승인된 문서의 전체
Native/fresh WASM과 고정 devel 비교, 실제 소비 경로·초기화 응답 검증 후에만 PR을 제출한다.
이번 로컬 보정이 전체 PR 준비 완료라는 뜻은 아니다. 목표와 PR 이후 후속 처리 승인은 유지한다.
