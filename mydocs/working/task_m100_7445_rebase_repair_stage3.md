# Issue #7445 — 전체 재검증이 드러낸 경계 보정

## 기준과 실패를 보존한 재검증

고정 upstream/devel은 `86a3c88fc89de9a2306a10bdda9a31023f0e24a5`입니다.
stage2의 `0e9154c63b7e6444e5b3790bd5859dc72ae2e547`에서 실행한 `final-v13` 전체
nextest는 **10279 PASS / 13 FAIL / 45 skip**, 372.816초, exit100입니다.
Native 선행 8조건 406쪽은 모두 90% 이상이었지만 생물독소 42·43·53쪽이 각각
61.16744%·68.17089%·73.92247%여서 전쪽 비교를 중단했습니다. Native는 9조건
611쪽까지 산출했으며 fresh WASM 전쪽·실제 소비 36조건은 실행하지 않았습니다.
fresh WASM 빌드 성공과 전쪽 검증 성공을 구분합니다.

추가 게이트 중 Skia lib의 캡션 3건, manifest 생성본 drift와 PDF 검사 0건 선택도
실패입니다. ignored `tests/generated` 28개를 `rust-test-suite-manifest.mjs --prepare`로
재생성한 뒤 같은 base의 `--check`는 통과했습니다. 추적된 Cargo target/검사 원문은
바꾸지 않았으며 새 최종 검증은 준비된 매핑으로 실제 PDF 검사까지 다시 실행합니다.

## 관측·원인과 생산 → 측정 → 배치의 연결

| 경계 | 보정과 독립 근거 | 유지하는 반례 |
| --- | --- | --- |
| 자동 번호의 마지막 자리 | raw PARA_TEXT의 마지막 공백은 8 UTF-16 유닛의 번호 슬롯입니다. `control_text_positions`의 슬롯 추론을 복원하고 `trailing_auto_number`가 마지막 소유 공백과 기존 EOF 표현을 받도록 했습니다. split/merge/undo가 같은 소유 위치를 소비합니다. aa9의 EOF4 가정은 잘못이었으며 합성 원문 검사 3곳을 실제 슬롯3으로 바로잡았습니다. | 일반 말미 공백이나 번호의 순수 갭을 소유 문자로 바꾸지 않습니다. 캡션 번호 위치·undo·삽입/Enter의 내용 소속을 함께 검사합니다. |
| 생물독소 작은 첫 조각 | upstream의 일반 단일 줄 fragment 분기가 기존 최소 잔여와 실제 세로 정렬 끝점을 거치지 않아 다음 행을 42·53쪽에 앞당겼습니다. `row_step`은 일반 컷에서도 기존 `MIN_TOP_KEEP_PX`와 공통 `row_aligned_content_bottom`의 점유 끝을 대조합니다. | 저장 첫 프레임으로 인증한 작은 빈 띠의 별도 소유 경로는 유지합니다. 문서 ID나 현재 페이지 좌표로 분기하지 않습니다. |
| ordinary 빈 밴드 이월 | stage2는 순수 저장 행의 generic carry를 너무 넓게 막았습니다. `emit`은 전체 source row가 **단일 저장 프레임**일 때만 기존 measured carry를 다시 사용합니다. 여러 쪽의 raw 합계는 현재 조각의 물리 최소로 삼지 않습니다. | #5585의 내용 뒤 빈 밴드와 다음 행의 쪽 소유를 유지하며 개인정보의 여러 source frame 이월과 구분합니다. |
| float 본체 원점 | host 줄의 높이만으로 원점을 추론한 실험은 기각했습니다. 동일한 host 값이어도 독립 HWP/HWPX Print의 본체 원점이 달랐습니다. layout profile의 저장 출처를 공통 query에 전달해 HWPX의 닫힌 object frame은 top margin을 소비하고 HWP5의 기존 저장 anchor 계약은 유지합니다. | stage2의 전역 top-margin 적용은 철회합니다. 측정·typeset·최종 paint가 같은 출처/원점을 소비하며 edit/reflow·닫힘 없는 원점은 승격하지 않습니다. |
| native 첫 완료 프레임 | stage2의 closing-span raw minimum을 HWP5의 수용된 첫 물리 프레임에 다시 더하면 종료 빈 띠가 이월되어 1741000이 3쪽이 됐습니다. HWP5의 완료 컷은 수용된 물리 행 원장을 사용하고, HWPX의 cellSz 최소 공간 경로는 유지합니다. | 개인정보의 HWPX colspan/rowspan 최소 공간과 ordinary 빈 밴드 대조군을 보존합니다. 기대 쪽수를 바꾸지 않았습니다. |

보정 후 dev 집중 nextest는 **36 PASS / 0 FAIL**, 53.050초입니다. 캡션·OLE·자동 번호,
#2097·#5714·#7203·#5585·#5846, #7114 저장/재열기 5건과 #2430·#3820 공백 대조군을
함께 실행했습니다. fmt와 diff-check도 통과했습니다. 최종 release 전쪽·전체 검증을 대신하지 않습니다.

같은 dev producer의 개인정보 전66쪽 Native는 최저 **92.53112% / 미달0쪽**이며,
PDF와66쪽이 일치합니다. 40쪽 review의 표 본체·다음 내용 원점은 직접 확인했고,
40쪽 자체 보조값은97.93097%입니다. 전쪽 fresh WASM·최종 release 증거는 아직 아닙니다.

## #4138의 쪽수 핀 자격 감사

두 split API의 stale 폭0·vpos 단조·inline control 줄 소속은 통과하지만 저장/재열기의
197쪽 assertion은 현재 196쪽으로 실패합니다. 이 숫자를 196으로 갱신하거나 잘못된
따옴표 금칙으로 되돌려 197쪽을 맞추지 않습니다.

고정 base에서 만든 분할 저장본은 양쪽 엔진 모두 197쪽이며 현재 브랜치의 분할 저장본은
양쪽 엔진 모두 196쪽입니다. 2774개 문단의 수평 캐시 중 `p178`만 base5줄 / 현재4줄입니다.
현재 입력의 한컴 2024 Print·재저장 HWPX는 현재4줄과 일치합니다. 따라서 이전5줄을
복원하는 가설은 기각했습니다. 각 입력의 독립 한컴 Print는 모두 197쪽입니다.

- base 분할 HWP SHA256: `5690b092e26f7473939047e5ab5a1587bf9e95441d23b8f68a729b36b75a56e1`
- 현재 분할 HWP SHA256: `4953689c6ac7f5f1819400abb87d2c6ffbd8bde0668c91464518991158879ffe`
- base Print PDF: `pdf/issue4138-fixed-base-split-20261014-2024.pdf`, 197쪽,
  SHA256 `85a24677a0d398e3f22e4c822ad0e88ac54ca9485779eafd96c98804c742276f`
- 현재 Print PDF: `pdf/issue4138-current-split-1x2-20261014-2024.pdf`, 197쪽,
  SHA256 `dd7a989867076e2aab0b1161fc1612f53245c5b858db722f7d48fc2fe09ea41a`
- job `a5b6ec90-8092-4035-b19a-1c688cc88fe7` / `f84c4b68-951a-4993-bc49-1933d014b53f`:
  Hancom13.0.0.3901, `frame_print_to_pdf_ex_one_up`, Print method0, preprocess none,
  session0 font scope verified(2 mapped/2 registered/0 failed).

base의 전197쪽 Native는 **최저13.60119% / 미달189쪽**입니다. 쪽수 일치는 정상
피델리티의 증거가 아닙니다. 같은 base 입력의 현재 엔진도 SVG 전197쪽의 내용·순서·세로
위치가 base와 같습니다. 수평 간격의 차이는 별도 비교 중이며 동일 입력 전쪽 TSV를
추가 산출합니다. base13쪽 review를 직접 열어 앞쪽 내용과 소제목의 잘못된 쪽/세로 배치를
확인했습니다. Native 결과이며 fresh WASM 자격 감사는 아직 실행하지 않았습니다.

#4138은 CLOSED이며 #7813은 원본 거대 셀과 네 cache/job 검사 범위를 기록합니다.
편집된 split의 쪽수 핀과 전쪽 미달을 자동으로 같은 등록 범위로 간주하지 않습니다.
검사 개선/이관은 같은 입력의 비교와 등록 범위를 확정한 뒤 결정합니다. 원본·PDF와 정상
parser/serializer·편집 계약은 보존합니다. GitHub 게시에는 새 사용자 승인이 필요합니다.

## 정본과 잔여

ignored 정본은 `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7882-20261014/`
아래 `final-v13/`, `v16-rest-focused-dev.log`, `v16-native-audit/`,
`giant4138-current-edited/`입니다. 초기 review helper 인자 오류 두 번은 최종 gate/PNG 생성
결과로 사용하지 않으며 이미 산출한 TSV/PNG를 재캡처 없이 확인한 retry를 기록합니다.

현재 전체 재검증 완료·PR 준비 완료를 주장하지 않습니다. 생물독소·개인정보·규제분석의
새 Native 전쪽 비교, fresh WASM/실제 소비·전체 nextest·필수 게이트를 계속합니다.
HYHWPEQ 공급 요청도 미응답이며 수식 PUA 내용 누락의 직접 판독은 보류입니다.
두 사용자 초안과 공유 target은 보존하고 커밋 대상에서 제외합니다.
