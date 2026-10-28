# PR #7200 메인터너 보정 3회차

## 분석

- 시작 head: `78fd17cf8`. 대상은 원본 width 합성 입력의 잘못된 저장 줄과 empty 입력의 시각 차이다.
- 원본 width 입력의 200자/1개 LineSeg는 #2291/#2287의 부실 저장 복구 경로를 발동한다.
  Hancom은 이 무효 저장 정보를 그대로 사용해 글자를 겹친다. 복구 렌더링과 그 겹침 PDF를
  동시에 정답으로 요구한 검토 설계가 잘못됐다. renderer에 겹침을 재현하는 예외를 넣지 않는다.
- 원본 입력과 실패 PDF/PNG는 이전 commit에 보존한다. 현재 파일을 정상 한컴 저장 정보로
  보정하고, 깨진 캐시의 복구 계약은 정상 문서를 메모리에서 변형하는 별도 검사로 유지한다.
  정상 저장본의 직접 PDF 비교와 의도적으로 깨뜨린 캐시의 재조판 검사를 서로 대체하지 않는다.
- empty 입력은 저장 메트릭을 지운 뒤 정상 저장/출력한 통제 실험으로 텍스트와 후행 테두리의
  차이가 입력의 모순인지 renderer 문제인지 확인한다. 정답 PDF를 rhwp 출력으로 생성하지 않는다.
- PDF 벡터의 선 두께는 0.24pt(96DPI에서 0.32px)로 rhwp의 0.3px와 대응한다.
  흐린 선을 보고 두께 상수를 늘리지 않는다. 직접 overlay에서는 선 위치·누락도 함께 판독한다.
- 소비 경로: 입력 LineSeg → composer의 저장 줄 수용/복구 → 공통 sequential 셀 계획 →
  표/뒤 문단/테두리 실제 배치. 최종 유효 입력과 캐시 손상 반례를 focused 검사하고,
  동일 최종 입력/PDF로 Native·fresh WASM Visual Sweep 후 결과보고·커밋한다.

## 정상 저장본으로 발견한 추가 원인과 보정

- 정상 empty HWPX의 첫 빈 줄은 `textheight=1056, spacing=632HU`이며 다음 호스트의
  저장 vpos는 1688HU다. PDF의 표 상단은 Top/Center/Bottom 각각
  112.418 / 143.971 / 175.525pt다. 같은 입력에서 보정 전 Top rhwp y=127.20px,
  PDF y=149.891px로 22.69px 차이가 확인됐다.
- `sequential_nested_cell_layout`은 저장 위치가 reset된 경로를 처리한다. 정상 저장 경로에서는
  `layout_cell_content`의 앵커/선행 줄 배치가 `para_y_before_compose`를 구해도,
  `has_preceding_text=false`인 마지막 fallback이 `inner_area.y`로 덮어썼다.
  이 fallback을 이미 결정한 호스트 문단 원점을 소비하도록 고쳤다. 새 수치 상수나
  문서 ID 분기를 추가하지 않았다. 앞 글자가 있는 기존 flow와 reset 공통 계획은 유지한다.
- 정식 `saved_empty_leading_paragraph_preserves_pdf_table_and_border_positions`는 보정 전
  의도한 22.69px 오류로 FAIL했다. 보정 후 Top/Center/Bottom의 표/End/문단 테두리와 기존
  정상 저장·손상 캐시 경계를 포함하여 focused 7 PASS다. 처음 prepare 이후 fmt로 suite가
  바뀐 실행은 0 tests/exit4로 실패했으며 통과로 세지 않았다. fmt 뒤 prepare하여 실제 7개를 실행했다.
- Native 최종 20쪽을 재캡처했다. empty 3쪽의 표와 후행 테두리는 PDF 위치에 맞춰졌고,
  다른 17쪽은 직전 보정 결과와 PNG 바이트가 같다. 수정 전 실패 증거와 별도로 보존한다.
  fresh WASM·최종 전체 게이트 완료 후 최종 결과를 아래에 덧붙인다.

## 최종 결과보고

- 전체 nextest: **9946 PASS / 0 FAIL / 51 skipped**, exit 0, 실행 302.072초.
- fmt, Native/WASM/workspace-all-targets Clippy 3종, workspace build, 고정 base 대비 suite 정책 PASS.
- fresh WASM 빌드는 wasm-opt까지 exit 0. Native/fresh WASM 20쪽의 PNG는 20/20 바이트 동일하다.
- 최종 입력 36개에 hidden-text / injection(include-fields) / unicode를 각각 실행해 **108 clean**.
  삭제한 width 중복 fixture 3개는 samples 경로로 검증하며 같은 문서를 중복 집계하지 않았다.
- 보정 후 empty 표 y는 149.7 / 191.8 / 233.9px, 독립 PDF는 149.891 / 191.961 / 234.033px다.
  후행 End와 문단 테두리도 정식 assertion 및 compare/overlay/review 직접 판독으로 확인했다.
- empty 재생성 job 9개가 보고한 한컴은 **11.0.0.9136**이다. 앞선 width의 **12.0.0.4605**와
  구분하며 PDF 버전/Creator를 허용 목록으로 사용하지 않았다.
- 문서 5개 상대 링크와 review 메타데이터 PASS. raw log/JSON/TSV는 커밋하지 않는다.
- Native Skia lib **4112 PASS / 13 ignored**, 그림 focused **2 PASS**, direct PDF export **4 PASS**.
  전체 순차 게이트 스크립트 exit 0. 검증 후 source 해시를 재확인했고 코드 변경은 없다.
- 판정: 이번 PR의 머지 보류 사유 해소, 로컬 검토 승인. 잘못된 과거 합성 PDF의 겹침을
  renderer에 복제한 것이 아니라 최종 입력을 고치고 정상/손상 경로를 각각 검증했다.
  기존 실문서 차이와 픽셀 래스터 차이를 해결했다고 확대하지 않으며 #7150은 종료하지 않는다.
- 분석 → 코드·입력 보정/검증 → 이 결과보고 순서를 마쳤다. 결과를 사용자에게 보고한 뒤
  source·fixture/PDF·대표 compare/overlay/review·검토 기록을 함께 로컬 커밋한다.
  원격 push/comment/merge는 수행하지 않았다.
