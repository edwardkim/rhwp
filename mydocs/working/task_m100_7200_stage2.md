# PR #7200 메인터너 보정 2회차

## 분석

- 시작 head: `4ce8d253c`. 1회차 결과를 덮어쓰지 않고 남은 시각 보류를 해결한다.
- 남은 경로: 별도 PageItem으로 배치되는 TAC 표는 텍스트 run의 글자 테두리 paint를 거치지 않는다.
  한컴 통제 입력에서 호스트 글자 테두리만 끄면 추가 외곽선이 사라진다. 표 테두리·문단 테두리와 구분한다.
- 독립 기대값은 같은 엔진 PDF에서 객체 높이·줄간격·글자 크기를 독립적으로 변경해 확인한다.
  하단 차이 약 14pt를 설명 없이 상수로 넣지 않는다.
- 원본 합성 width 입력의 무효 저장 줄과 정상 재조판 입력을 구분한다. 원본 실패를 숨기거나
  대조군 통과를 원본의 PDF 일치라고 보고하지 않는다.
- 소비 경로: control 소속 char shape → TAC 실물 bbox/host line → 글자 테두리 paint → Native/WASM.
  focused 계약과 Visual Sweep 직접 판독 후 전체 검증을 진행하고 결과보고 뒤 커밋한다.

## 독립 통제 결과

- Hancom 12.0.0.4605의 동일 PDF 경로에서 표 높이 20000/30000/40000 HU, 폭
  12000/24000/28000 HU, 글자 크기 500/1000/2000 HU, 줄간격 100/160/200%,
  중첩 표 높이, 쪽 테두리 여백, 글꼴 및 글자 테두리 색을 각각 변경했다.
- 표의 **글자 테두리 상자**는 객체 물리 높이에 위·아래 각각 최소 2.5mm(708 HU)의
  여백을 더한다. 바깥여백 700/800 HU의 경계, 한쪽 1000/2000/5000 HU,
  양쪽 1000/2000 HU에서 같은 규칙을 확인했다. 처음 관측한 약 14pt는 임의 보정값이
  아니라 이 최소 여백의 합이다. PDF의 가는 선 끝점에는 프린터 양자화가 있다.
- 테두리의 시작은 객체의 줄 상자 원점이며 실제 표 위치에 최소 여백을 추가하지 않는다.
  실제 배치·흐름에 이 테두리 전용 여백을 반영하면 표·뒤 문단이 잘못 이동한다.
- 단독 TAC 객체 run의 경계 그리기에 적용한다. 비-TAC은 글자 상자가 아니며, 여러 텍스트/객체가
  한 run을 이루는 연결 테두리는 별도 줄/run 소유 문제이므로 이 보정의 일반화 주장에 포함하지 않는다.

## 전체 검증에서 발견한 경계와 재분석

- 1차 전체 nextest에서 `stored_inline_table_suffix`의 정상 한컴 저장 숫자 표 2개가
  다른 줄로 갈라졌다. `compose_lines`는 `max(line_height, text_height)`를 쓰지만
  `stored_tac_line_assignment`의 저장 정보 재사용 판정은 아직 `line_height` 원값을 비교한다.
  두 값의 의미를 맞추고 기존 실제 저장본 검사를 유지한다. 단순 허용치 확대가 아니다.
- 원 PR의 `overlay-{top,center,bottom}.hwpx`는 배경 표와 다음 표가 모두 LEFT/offset=0이다.
  명시한 가로 앵커를 복원하면 `End`와 `Second`가 같은 좌표에서 겹친다. 이는 두 표가 같은 높이에
  있어야 한다는 합성 흐름 계약과 별개인 입력 구성 결함이다. renderer를 다시 가운데 정렬하거나
  겹침 baseline을 허용하지 않는다. 두 번째 표를 첫 표 오른쪽으로 명시 배치해 두 라벨을 보존하고,
  실제 가로 원점·같은 세로 원점·세로 정렬 이동을 정식 검사한다. 기존 파일을 수정하며 복사본을 추가하지 않는다.
- 전체 실행은 중단하지 않고 완료 결과를 받은 뒤 같은 회차에서 보정·재검증한다.

### overlay 호스트 줄의 추가 독립 증거

보정한 overlay 입력을 실제 PDF로 비교하자 `Second`의 y가 `End`보다 15.9968px 아래였다
(Top 110.511093/122.508693pt, Center 144.584277/156.581877pt,
Bottom 178.537485/190.535085pt). 앞서의 “같은 y” 가정은 한컴 출력으로 지지되지 않는다.
배경 객체가 흐름을 전진시키지 않는 것과 빈 호스트 문단의 줄이 공간을 갖는 것은 별개다.

`sequential_nested_cell_layout`은 모든 block 표 호스트에서 구성 줄 높이를 생략했다.
배경 표만 가진 문단은 구성 줄 공간을 보존하고 객체 높이만 흐름에 가산하지 않아야 한다.
앞선 문단이 빈 경우만 고친 1회차의 원칙을 이 실제 소비 경로에도 적용한다. 기대값은
원본 LineSeg의 1000+200HU = 16px 및 위 PDF 관측값이며, 테이블 높이 4000HU가 아니다.

## 보정 및 집중 검사 결과

- 객체 글자 테두리 정식 검사: 1회차 코드에서 5 PASS / 새 1 FAIL, 보정 후 6 PASS.
- 저장 줄 소속: `stored_line_box_height`의 동일 메트릭을 구성과 저장 줄 수용 판정이 함께 사용한다.
  기존 실제 한컴 저장 숫자 표의 같은 줄/명시적 개행/너비 줄바꿈/뒤 텍스트 경계 2개 검사 PASS.
- overlay 호스트 줄: PDF로 검증한 16px assertion을 먼저 실행해 5 PASS / 해당 1 FAIL을 확인했다.
  측정·실제 배치가 공유하는 sequential 계획에서 배경 객체 호스트 줄을 보존한 후 6 PASS다.
- 최종 focused 8개 PASS이며, renderer 코드와 입력을 고친 뒤 Native CLI를 다시 빌드했다.
  최종 전체 게이트 결과는 아래에 별도로 기록한다. 1차 전체 9941 PASS / 4 FAIL은 숨기지 않는다.
- PDF 버전 지침은 `fbc14758f`로 독립 커밋했다. Creator 연도·빌드와 PDF 형식 버전은 허용 목록이
  아니며 `Hwp 2020 0.0.0.0`/PDF 1.4도 출처·원문 대응이 맞으면 재사용한다. 변경 문서 5개의
  상대 링크와 권위 문서 3개의 메타데이터를 검사했다.

## 최종 결과보고

- 최종 전체 nextest: **9945 PASS / 0 FAIL / 51 skipped**, exit 0, 실행 317.275초.
  첫 전체의 4 FAIL은 코드·입력 보정 후 모두 통과했다. 사용자 중단한 1회차 exit 143과 구분한다.
- fmt, Native/WASM/workspace-all-targets Clippy 3종, workspace build, 고정 base 대비 suite 정책: PASS.
- Native Skia lib: rhwp 3930 + 공용 crate 182 PASS / 13 ignored. 그림 placeholder 2 PASS,
  direct PDF export 4 PASS. 모든 Cargo 작업은 전용 target에서 순차 실행했다.
- 최종 Native/fresh WASM 23쪽의 compare·standalone overlay·review를 다시 생성했다.
  backend별 페이지 PNG는 23/23 바이트 동일하다. 기존 17쪽은 두 추가 원인 보정 전후 동일하며,
  추가 날짜 표 3쪽과 overlay 3쪽의 의미상 줄 소속·16px 호스트 줄을 직접 확인했다.
- sample 28개는 전체 회귀의 새 문서 환경변수로 검사했다. fixture까지 포함한 39개는 최종 CLI의
  hidden-text/injection(include-fields)/unicode를 각각 실행하여 **117 clean**을 확인했다.
  sample 전용 필터가 fixture 경로를 제외한다는 점을 확인해 CLI 직접 검사로 보완했다.
- 원문·기준 PDF·생성 job·SHA256 및 최종 production/CLI/WASM 해시는
  `mydocs/pr/archives/pr_7200_review.md`와 fixture README에 기록했다. raw log/JSON/TSV는 커밋하지 않는다.
- 객체 글자 테두리, 저장 TAC 줄 소속, 배경 표의 빈 호스트 줄은 해결했다. 원본 합성 width의
  한컴 중복 인쇄 PDF와 여러 줄 재조판 계약은 상충하므로 그 원본의 PDF 일치까지 완료했다고
  보고하지 않는다. 정상 대조군의 개선·남은 래스터/합성 차이를 구분하고 머지 보류는 유지한다.
- 결과보고 뒤 분석·코드·입력/PDF·시각 증적·검토 기록을 함께 커밋한다. 원격 push/comment/merge는 수행하지 않는다.
