---
kind: working
status: active
canonical: mydocs/working/task_m100_7727_stage1.md
last_verified: 2026-11-14
---

# #7727·#7729·#7730·#7731·#7732 원본 피델리티 분석과 보정

갱신: 2026-11-14T21:39:40+09:00.

## 목표와 현재 판정

다섯 이슈의 실제 원본을 같은 독립 한컴 Print PDF와 대조하여 **모든 쪽 98% 이상**으로
복구하고, 가능한 차이는 100%까지 개선한다. 98% 이상으로 검증된 범위에는 독립 근거에
맞는 회귀 검사를 추가한다. 미달 쪽을 검사에서 빼거나 기준 PDF·글꼴·일치율 판정을 바꾸지 않는다.

브랜치 `task/7727-related-fidelity`, 고정 base/parent는
`53da4d2da28ae0cacf8feac3a989cc6a69fb2622`다. 현재 구현·검증이 진행 중이며 PR 제출 준비가
완료되지 않았다. 아래 수치는 해당 생산자에 한정한다. 뒤의 변경으로 앞선 전쪽 결과를
새 후보의 결과로 바꾸어 부르지 않는다.

## 원본과 독립 기준

| 이슈 | 원본 | 독립 PDF | 쪽수·실제 출력 제품 |
| --- | --- | --- | --- |
| #7727 | `samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp` | `pdf/issue7727/cbta-original-hancom2020-print-20261106.pdf` | 204쪽·한컴 2020 |
| #7729 | `samples/issue6844/30269-anticorruption-recommendation-toc.hwp` | `pdf/30269-anticorruption-recommendation-toc-2020.pdf` | 22쪽·한컴 2022 |
| #7730 | `samples/exam_eng.hwp` | `pdf/exam_eng-hwp-2020.pdf` | 8쪽·한컴 2022 |
| #7731 | `mydocs/pr/assets/issue7445/hwpctl_API_v2.4.hwp` | `mydocs/pr/assets/issue7731/api-reference/hwpctl_API_v2.4-hwp-2020.pdf` | 105쪽·한컴 2022 |
| #7732 | `mydocs/pr/assets/issue7445/issue1937_rowbreak_footnote_overpagination.hwp` | `pdf/issue7382-regression-review/issue1937_rowbreak_footnote_overpagination-2020.pdf` | 50쪽·한컴 2020 |

파일명의 `2020`과 실제 제품 버전을 구분했다. 웹한글기안기 출력물을 기준으로 쓰지 않았다.
입력/PDF SHA-256·쪽수·Creator·Producer는 작업 증적의 `cases.json`에 고정했다.
시각 검증 조건은 정상 설치 글꼴, Native PNG/fresh WASM, 96 DPI, compat2022,
공통 Visual Sweep raw 2px, 픽셀 차이 문턱 32, 페이지 작업자 총 8개다.
현재 정본 도구의 PDFium 변환을 사용하며 기준 PDF 바이트는 수정하지 않았다.
회귀 검사 실행의 최소 글꼴 조건과 실물 시각 검증의 원본 글꼴 조건은 구분한다.

## 동일 생산자의 전쪽 결과

`closing-span-clean-native` 및 `closing-span-clean-pkg`는 API 종료 프레임·CBTA 종료 병합 칸
보정까지 포함한다. 이후의 캡션 테두리·NBSP 압축·밑줄 변경은 포함하지 않는다.

| 문서 | Native 전체 최저: base → 보정 | fresh WASM 전체 최저 | 보정 후 98% 미만 |
| --- | --- | --- | --- |
| CBTA 204쪽 | 96.23% → 96.35% | 96.35% | 96·118·162·166쪽 |
| 권고안 22쪽 | 87.21% → 97.44% | 97.44% | 4쪽 |
| 영어 시험지 8쪽 | 97.12% → 97.12% | 97.11% | 2쪽 |
| API 105쪽 | 24.28% → 99.55% | 99.55% | 없음 |
| 각주 표 50쪽 | 98.58% → 98.58% | 98.58% | 없음 |

두 경로 모두 389쪽의 쪽수가 일치했다. Native의 383쪽이 98% 이상이며 6쪽이 남았다.
API는 Native 77쪽이 공식 두 자리 표시값 100.00%다. 이 결과는 전체 nextest·보호 문서·
최종 실제 소비 경로의 통과를 뜻하지 않는다.

## 원인별 보정 규칙

### API 12→13쪽: 표 앞 바깥여백 중복 소비

`fragment/budget.rs`에서 저장 host 종료 뒤 재동기화한 현재 커서를 opening 프레임의
원점으로 비교했다. 이미 소비한 바깥 위 여백 3.773px를 다시 차감해 행 소유가 달라졌다.
불변 `anchor_y`와 host 앞 점유로 실제 opening 프레임을 판별한다.
기존 #8148 계열의 저장 프레임 계약을 추적했으며 명시적 다음 프레임과 페이지 컷은 유지한다.

### 권고안 6·22쪽: 단 시작 전 닫힌 인라인 개체 줄

`stored_column_spacing.rs`가 한 줄에 여러 도형이 있거나 앞 문단이 중립 컨트롤만 가지면
닫힌 줄을 인정하지 못했다. 원본 문단 99의 두 도형은 3370/3338HU이며 줄 높이는 3370HU다.
같은 줄의 개체 높이는 합이 아닌 최댓값으로 닫고 바깥여백을 포함한다.
저장된 앞 500HU 간격을 누락하던 경로를 복구했다. 기존 #7642·#8917 구현을 대조했다.
6쪽 87.21%→99.95%, 22쪽 91.73%→99.60%다.

### API 35→36쪽: 빈 첫 줄과 동일 원점 재시작

첫 빈 문단과 다음 코드 문단이 모두 vpos=0이어도 첫 빈 문단의 실제 줄 높이가 소유된다.
선언 프레임 닫힘으로 검증한 경우 빈 문단의 종료 전진을 보존하고,
`stored_empty_successor_geometry`가 그 전진을 다시 지우지 않도록 했다.
초기 후보는 36쪽을 악화시켜 폐기했으며, 정정한 후보의 35·36쪽은 모두 100.00%다.

### 권고안 4쪽: 저장 오른쪽 탭의 선언 프레임

문단의 명시적 RIGHT 탭이 현재 프레임 바깥에 있고 실제 저장 확장 정보가 있을 때
그 저장 목표를 소비한다. 프레임 안쪽 탭·자동 오른쪽 탭의 계약은 유지한다.
`inline_right_tab.rs`에서 처리했으며 97.34%→97.44%다. 잔여 차이가 있어 완료가 아니다.

### API 65쪽: 폭 0 어울림 표 host의 앞·뒤 점유

독립 글줄이 아닌 빈 host에서 문단 앞 간격과 표 바깥여백을 중복 소비했다.
공통 `stored_empty_square_host_uses_paragraph_origin`으로 측정과 paint가 같은 원점을 쓴다.
뒤 간격도 원본 식 `34574−500+141+22784+141=57140HU`로 실제 종료를 인증한 경우에만
바깥 아래 여백으로 닫는다. 일반 host의 뒤 간격을 일괄 제거하지 않는다.
65쪽은 72.99%→94.53%(앞만 보정)→100.00%(끝 프레임 함께 보정)다.

### API 27·53·56·67쪽: 첫 재시작과 마지막 종료의 이중 인증

`native_text_frame_terminal_successor_gap`은 셀 placeholder 크기만으로 선언 opening을
제외하고, 실제 종료 줄에 뒤 간격을 더했다. 첫 저장 reset의 줄+안여백이 선언 높이를
닫는지 확인하고 마지막 줄+여백이 다음 문단 원점에 정확히 닿는지도 확인한다.
원본의 두 독립 경계를 만족할 때만 다음 프레임의 간격을 중복 소비하지 않는다.
네 쪽 모두 100.00%다. 문서명·쪽 번호 예외를 넣지 않는다.

### API 69쪽: TAC 표 뒤 독립 빈 줄과 fit 예산

표를 소유한 저장 줄 앞의 실제 prefix와 뒤의 독립 빈 줄을 구분하지 않아,
표 fit 계산에 이미 소비한 줄이 다시 포함되고 paint는 공백 host의 뒤 줄을 생략했다.
표 소유 줄의 실제 인덱스로 뒤 시작을 정하고, source prefix가 실제 전진과 일치할 때
소유 표 줄의 높이·간격만 표 예산으로 쓴다. 별도 빈 줄은 자기 위치를 가진다.
초기 시각 통과 뒤에도 fit의 4px 누락을 확인해 추가 보정했다. 최종 69쪽은 100.00%다.

### CBTA 176·182쪽: 이번 행에서 닫히는 병합 칸의 몫 누락

선행 실제 수정 `0024df763a`의 내용·선언·병합 칸 몫의 최댓값 계약을 추적했다.
`crossing_span_page_end_band`의 엄격 부등식이 이번 행에서 끝나는 병합 칸을 제외해
근거 없는 남은 쪽 공간 전체를 끝 행에 배분했다. 182쪽 행222의 17.0933px가 32.44px로
늘어났지만 독립 Print는 약21.43px다.
종료 병합 칸의 실제 내용과 선언 높이 중 큰 점유에서 이미 수용한 앞 행 상자만 뺀다.
줄·안여백의 전체 최소는 기존 공개 측정 API로 구한다. 뒤로 이어지는 칸의 계약은 유지한다.
176쪽 97.45%→99.94%, 182쪽 96.23%→99.94%이며 전204쪽으로 확인했다.

### CBTA 118쪽: 위 캡션 이동에서 소유 테두리 누락

`layout.rs::offset_inline_image_y`가 그림만 옮긴 뒤 반환해, 형제 테두리를 캡션 위에 남겼다.
원본 image y=212.238px, border y=184.918px로 27.32px 분리됐다.
이미지 ID와 `RectangleNode.image_frame_owner`로 같은 소유자의 테두리를 찾아 함께 이동한다.
좌표·색·문서명으로 테두리를 추정하지 않는다.
별도 Native/fresh WASM 집중 결과는 모두 96.85%→98.44%다. review·overlay를 함께 직접 확인했다.
얇은 테두리의 출력 농도 차이는 남아 있어 100% 해결로 쓰지 않는다.
새 관계형 검사 `issue_7727_top_caption_picture_frame`은 실제 보정 전 1 FAIL→보정 후 1 PASS다.
문서 전체가 아직 98% 미만이므로 절대 위치를 고정하는 검사로 등록하지 않았다.

## 영어 시험지의 추가 분석

NBSP의 실제 tracking을 소비하면서 압축 슬롯 수에서는 제외한 공통 분모 불일치를 확인했다.
압축 슬롯에 NBSP를 포함한 후보는 Native 전8쪽에서 최저 97.14%다. 아직 목표 미달이다.
앞서 언어 경계를 강제로 이어받긴 별도 후보는 97.12%→96.85%로 악화시켜 되돌렸다.
그 후보의 결과를 최종 증거로 사용하지 않는다.

밑줄 차이는 별도 한컴 2020 인쇄 대조군으로 분리했다. 동일한 12pt 글자·밑줄에
글자 위치 -12/0/+12를 적용했다. MCP job `9e395d33-57c9-4fb5-932e-eea17e70b484`,
실제 버전 11.0.0.9136, Print method 0, 1쪽, 출력 13초다.

| 글자 위치 | 글자 기준선(pt) | 밑줄 y(pt) | 밑줄−글자 기준선(pt) |
| --- | ---: | ---: | ---: |
| -12 | 79.4731 | 82.9490 | 3.4759 |
| 0 | 100.0906 | 102.1280 | 2.0374 |
| +12 | 120.7081 | 121.3070 | 0.5989 |

밑줄은 약19.179pt인 줄 간격만 따라가고 글자 위치 변화는 따라가지 않는다.
아래 밑줄의 공통 replay에서만 글자 위치를 제외하고, 텍스트·장식의 원본 run은 동일하게
보존한다. 최초 후보가 원본 장식 run을 바꾸어 canonical 소유 검증에 거절된 것을 실제 확인했고,
검증을 완화하지 않고 replay 단계로 보정을 옮겼다. 종이 프레임의 밑줄 원점도 같은 줄 기준선이다.
Native 전8쪽 최저97.79%, fresh WASM 전8쪽 최저97.78%다. 개선됐지만 목표 미달이다.

새 밑줄 관계형 검사는 이전 코드에서 1 PASS / 1 FAIL, 보정 후 2 PASS다.
NBSP 끝 전진 관계를 추가한 3개 검사는 보정 후 모두 PASS다. NBSP의 이전 라이브러리 검사는
폐기한 언어 슬롯 후보를 포함한 진단 생산자이므로 base 자체의 결과로 부르지 않는다.
위 밑줄·취소선과 원본 글자 위치 보존을 대조군으로 함께 확인했다.
API의 독립 PDF 표 경계 13·65·69쪽 고정형 검사와 69쪽 독립 빈 줄 소유 관계를 추가했다.
정확한 base Native에서 3 FAIL, 보정 후보에서 3 PASS다. 임시 독립 rustc 실행이며
정식 파생 suite의 최소 글꼴 Native Skia nextest도 실행해 새3개 파일의 **7 PASS / 0 FAIL**을
확인했다(3.353초, 선택 밖1024건). 첫 실행의 임시 폴더 helper 직접 의존성 누락은 표준
라이브러리의 검사 소유 폴더와 Drop 정리로 수정했다. 전체 회귀는 아직 실행하지 않았다.

Mac 격리에서 CoreText1875→528개, system_profiler의 글꼴 파일235개와 CoreText의
모든 실제 경로가 OS 영역에 있음을 확인했다. 기본 한글 프로그램 파일·표시 PNG의 SHA는
일반 환경과 같고, 일반 프로세스의 사용자 글꼴 등록 목록도 전후 동일했다.
[최소 글꼴 집중 검증 기록](../pr/assets/issues7727_7732_20261114/focused-minimal-font-validation.json)을 따른다.

독립 대조군은 [원본 HWPX](../../tests/fixtures/issue7730/underline-position-probe.hwpx),
[한컴2020 Print PDF](../../pdf/issue7730/underline-position-probe-2020.pdf),
[원인·전후 측정·MCP 영수증](../pr/assets/issues7727_7732_20261114/underline-baseline/cause.json)에
같은 SHA로 보존했다. 글꼴 바이너리는 포함하지 않는다.

## 남은 분석과 완료 조건

- CBTA 96쪽: 저장 줄은 `교육훈`/`련 단계 간 비교`로 나뉘지만 Print는 `교육훈련`/`단계 간 비교`다.
  자동 번호 뒤의 실제 가용 폭·언어별 전진·저장 줄의 소유를 원본에서 확인한다.
- CBTA 162·166쪽: 종료 행 상자가 약6px 더 크다. 162쪽은 문단5의 표 조각173..189,
  scan 단계 끝 행 override 45.2933px다. 176·182쪽과 같은 원인이라고 단정하지 않는다.
- 권고안 4쪽과 영어 시험지 2쪽은 남은 실제 차이를 해결한다.
- 최종 source의 Native/fresh WASM 전389쪽을 같은 조건으로 재검증한다.
- 회귀를 최소 글꼴 환경에서 실행하고, 독립 근거로 자격을 얻은 쪽에는 현행 지침에 따른
  정확한 고정형 또는 관계형 검사를 추가한다. 검사의 상단 주석에 종류·측정 근거를 명시한다.
- 전체 nextest·범위별 lint/build·보호 TSV·필수 편람·실제 소비 경로를 최종 head로 확인한다.
  미실행 항목을 이전 head의 통과나 PR CI로 대신하지 않는다.

## 증적 위치

현재 작업 증적은 저장소 기준 `output/pr-review/issues-7727-7732-20261114/`다.
임시 driver·실행 파일·글꼴/SVG 중간 자료는 이 경로에만 두며 Git에 추가하지 않는다.

- `cases.json`: 입력과 기준 PDF의 SHA·쪽수·출처.
- `closing-span-clean-native-state.json`, `closing-span-clean-wasm-state.json`: 동일 source 전쪽 결과.
- `closing-span-clean-native-producer.json`, `closing-span-clean-wasm-producer.json`: 생산자 SHA.
- `logs/closing-span-clean-native-driver.log`, `logs/closing-span-clean-wasm-driver.log`: 전쪽 실행 로그.
- `caption-frame-native-producer.json`, `logs/caption-frame-cbta.log`: 캡션 집중 후보와 측정.
- `logs/caption-frame-before-test.log`, `logs/caption-frame-after-test.log`: 새 검사 전후 실패/통과.
- `caption-frame-direct-review/review/review_118.png`와 `overlay/overlay_118.png`: 같은 쪽 직접 판독 쌍.
- `underline-position-cause.json`, `underline-position-print-{start,status,download}.json`: 독립 밑줄 대조군 영수증.

위 경로는 진행 중인 로컬 증적이다. 최종 제출 때에는 필요한 PNG·TSV·영수증과 독립 표본을
정식 assets/PDF 경로로 선별해 함께 커밋하고 이 문서의 링크와 판정을 갱신한다.
