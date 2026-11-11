# #8946 외부 그림을 포함한 독립 Print 확보와 표지 검증

2026-11-12 KST. 제품 코드는 `eec02db951`, 직전 검증 기록은 `b7b3442635`다.
이번 단계는 남은 표지 검증 자료를 확보하는 작업이며 호스트 글꼴 옵션·수명 처리라는
#8946의 범위나 수용 기준을 변경하지 않는다.

## 경로 조사와 선택

- 현재 배포된 한컴 변환 MCP는 단일 HWP/HWPX blob을 받는다. 조사한 tool schema의
  `options`에는 timeout 및 별도의 HWP5 정규화 옵션만 있고 외부 그림 첨부·임의 파일 쓰기
  인터페이스는 없다. 알려지지 않은 endpoint나 서버 경로를 추측해 호출하지 않았다.
- 원본을 그대로 열려면 Windows의 `D:\Work\Gwjang\temp\images`에 같은 그림을 공급해야
  한다. [한컴 공식 FAQ](https://www.hancom.co.kr/support/faqCenter/faq/detail/2925)는 연결
  그림의 원래 위치 복구와 문서 내부에 포함하는 방식을 안내한다. 현재 변환 인터페이스로는
  전자의 서버 파일 배치를 실행할 수 없다.
- [한컴의 HWPX 구조 설명](https://tech.hancom.com/hwpxformat/)은 `BinData`가 그림 등
  바이너리 자원이고 `Contents/content.hpf`가 패키지 파일 목록임을 설명한다. 이를 근거로
  같은 그림 바이트를 내부 자원으로 전달하는 **전송용 사본**을 별도로 만들고 검증한다.
  이 설명만으로 두 방식의 출력 일치를 가정하지 않고 아래의 실제 동등성 대조를 수행한다.

## 전송용 사본의 변경 경계

원본 SHA-256은 `3395e19bebea8b6689f383df1f4ea1ddb253dee91c4320392cc40e90e2e4f191`,
전송용 사본 SHA-256은 `77f6a3f38cf79b681cf61483b08ed3bb2ea7310331676c548a1af9aefcc58158`다.
원본을 덮어쓰지 않았다. 원본 ZIP의 11개 기존 entry 중 `Contents/content.hpf` 하나만
달라졌고 나머지 10개는 바이트와 SHA가 동일하다. 본문 `section0.xml`, 글꼴·문단 설정
`header.xml`, 저장 제품·호환성·미리보기 정보도 그대로다.

변경은 `image1`~`image3`의 `href`를 내부 `BinData/<원본 basename>`으로 지정하고
`isEmbeded="0"`을 `"1"`로 바꾸는 것뿐이다. 그림 ID·컨트롤·크기·좌표·본문·저장 LineSeg·
서식은 바꾸지 않았다. `samples/oracle.gif`, `samples/rdb02.gif`, `samples/s1.jpg`의
실제 바이트를 추가했다. 그림 변환·재인코딩·색 변환은 하지 않았다.

전송용 사본은 원본과 SHA가 다른 입력이다. 결과 PDF의 입력을 원본 파일 자체로 표기하지
않으며, 자원 전달 방식 변경의 동등성 증거를 통해 원본에 같은 자원을 공급한 출력을
대조한다. 원본 Windows 경로에 대한 한컴의 외부 파일 탐색을 시험했다는 주장도 하지 않는다.

## 사본의 실제 출력 동등성

최적화 Native와 fresh WASM을 각각 실행했다. 원본에 같은 그림을 외부 공급한 직전 출력과
전송용 사본에서 내부 그림을 읽은 출력은 두 경로 모두 763쪽 전체에서 RGB 픽셀과 렌더
트리가 동일하다. Native PNG의 바이트 차이는 이번 직접 export에 DPI metadata를 지정하지
않았기 때문이며 실제 크기와 RGB는 전쪽 동일하다. WASM은 PNG 바이트까지 모두 같다.
따라서 새 한컴 PDF와의 수치 비교에는 직전 원본 출력의 PNG를 그대로 사용한다.

실제 Canvas2D 호스트도 전송용 사본으로 763쪽을 열었다. 복원한 같은 프로그램 목록
521개에서 67개 적재·pending 0·failed 0이며, 본문 2~763쪽의 트리·PNG는 원본 실행과
동일하다. 표지에는 이번에 그림이 정상 공급되어 차이가 생겼다. 글꼴 정책이나 제품
렌더링 코드는 변경하지 않았다.

한컴 2024에 `start → status → download`의 정식 Print 변환을 요청했다.
job은 `2f6bae57-2c8f-4dba-835e-4e57766a9f23`이다. 약 25분 22초 뒤 성공했고, 전체 PDF를 내려받아 아래 검증을 완료했다.
준비했던 1쪽 보조 입력은 서버에 보내지 않았고 수용 근거에 사용하지 않았다.


## 독립 Print 실행과 최종 결과

실제 한컴 `13.0.0.3901`, engine `2024`, `pdf_print_method: 0`,
`pdf_output_mode: frame_print_to_pdf_ex_one_up`, `input_preprocess: none`을 확인했다.
실제 출력은 763쪽이며 PDF 크기는 90,804,612바이트,
SHA-256은 `45bb4d29b0afea67fb4818a9b605a84201cbdf2998b6c31310d36fb85294cad8`다.

전체 파일은 `pdf/issue8946/hwp3-sample10-hwpx-images-embedded-2024.pdf`에 로컬 보관한다.
50 MiB 이상 PDF의 정본 보존 규칙에 따라 이 파일과 직전 재인쇄 PDF의 정확한 경로를
`.git/info/exclude`에 등록했다. PDF를 삭제하거나 분할해 커밋하지 않았다.
전송 입력·Print 실행 정보·해시·측정 범위는
[검증 메타데이터](../pr/assets/issue8946/stage4-20261112/verification.json)에,
전송용 입력은 [HWPX 사본](../pr/assets/issue8946/stage4-20261112/hwp3-sample10-hwpx-images-embedded.hwpx)에 보존했다.

새 독립 PDF와 직전 독립 PDF를 같은 canonical PDFium 96dpi로 전쪽 렌더했다.
**달라진 쪽은 표지뿐이며 본문 2~763쪽은 RGB 픽셀이 모두 동일하다.**
[전쪽 독립 Print 대조](../pr/assets/issue8946/stage4-20261112/independent-print-parity.json)와
[원본/전송 사본 출력 동등성](../pr/assets/issue8946/stage4-20261112/source-transport-output-parity.json)을
함께 확인한다. 원본과 사본의 파일 동일성, 실제 출력 동등성, 독립 Print의 본문 보존을
서로 다른 근거로 기록했다.

| 무마스크 비교 | 표지 raw | 전쪽 최저 raw | 90% 미만 쪽 |
|---|---:|---:|---:|
| 원본 + 같은 외부 그림, Native | 100.00000% | 96.34886% | 0 / 763 |
| 원본 + 같은 외부 그림, fresh WASM | 100.00000% | 99.56218% | 0 / 763 |
| 전송 사본 + 실제 Canvas2D 호스트 공급 | 99.94899% | 99.35730% | 0 / 763 |

Native/fresh WASM 표지의 미대응 픽셀은 각각 0이다. 색상 경계 보정 전 2px 실루엣 raw를
사용했으며 그림 제외 영역·임의 crop·resize를 비교에 적용하지 않았다. `100%`는 이
raw 지표의 값이며 PDF와 PNG의 모든 색상 픽셀이 동일하다는 뜻이 아니다.
Host Screen의 원래 794×1123 크기도 유지했다.

[Native 전쪽 TSV](../pr/assets/issue8946/stage4-20261112/native-full.tsv),
[fresh WASM 전쪽 TSV](../pr/assets/issue8946/stage4-20261112/wasm-full.tsv),
[Host Screen 전쪽 TSV](../pr/assets/issue8946/stage4-20261112/host-screen-full.tsv),
[정밀 수치·PNG 해시](../pr/assets/issue8946/stage4-20261112/exact-page-metrics.tsv)를 보존했다.
원본 Native/WASM PNG는 단계 3에서 생성한 동일 바이트를 재사용했고,
사본 전쪽 실행의 실제 픽셀·트리 동등성을 먼저 검증했다. 제품 코드·WASM은 그 뒤 바뀌지 않았다.

### 표지 직접 확대 판독

[독립 Print 표지](../pr/assets/issue8946/stage4-20261112/independent-print-cover-96.png)와
[Native](../pr/assets/issue8946/stage4-20261112/native-cover-96.png),
[WASM](../pr/assets/issue8946/stage4-20261112/wasm-cover-96.png),
[호스트 화면](../pr/assets/issue8946/stage4-20261112/host-screen-cover-96.png)을 확인했다.
추가 300dpi 삼자 대조에서
[로고와 작은 글자](../pr/assets/issue8946/stage4-20261112/cover-logo-300.png),
[영문 제목](../pr/assets/issue8946/stage4-20261112/cover-title-300.png),
[중앙 그림](../pr/assets/issue8946/stage4-20261112/cover-illustration-300.png),
[하단 그림·연구실 이름](../pr/assets/issue8946/stage4-20261112/cover-footer-300.png)을
직접 판독했다. 그림 안의 글자·선·색·크기·배치와 제목 및 한글의 글리프·장평·기준선·
띄어쓰기·앞뒤 간격을 확인했다. 그림/글자의 누락·겹침·잘못된 글리프·배치 후퇴는
관찰하지 않았다. 원래 bitmap의 저해상도와 backend별 보간·안티앨리어싱 차이는 남는다.
표지에는 수식·첨자·장식 개체가 없으며 이전에 보정한 본문 제목 23개 줄의 확대 근거는
본문의 독립 PDF 및 Native/WASM 출력이 그대로이므로 단계 3의 판독을 연결한다.

### 기존 검사 확인과 판정

이번에 보정한 HWPX 경로와 이전 `samples/hwp3-sample10-hwpx.hwpx` 참조를 따라
`tests/cases/issue_7810_hwp3_face_names_follow_hangul_converter.rs`, `tests/issue_1142.rs`,
`tests/issue_1143.rs`, 변환 corpus 및 deferred manifest를 확인했다. 활성 검사는 글꼴 이름,
외부 그림 ID·경로·적재 상태, 그림 주입 후 캐시/내용 포함 관계를 보호하며 이 HWPX의 절대
px·고정 영역·과거 쪽수 핀을 새로 교체할 대상은 없었다. 원본 자원 개수·ID 등의 정확한
저장 계약은 유지했다. 더 넓은 #7774의 deferred 범위나 다른 원본 검사를 임의로 복원·
제거하지 않았다. 새 회귀 테스트·golden·기대값 변경은 없다.

**같은 그림을 공급한 조건의 남은 표지 시각 미달은 해소됐고, 이 문서의 전쪽 시각 판정은
통과다.** 외부 그림의 실제 전달 경계 차이가 원인이었으며 제품 좌표·글꼴 정책·수용
기준을 바꾸어 수치를 맞추지 않았다. 원본 Windows 외부 경로를 한컴 서버에서 탐색하는
동작까지 검증한 것은 아니다. 동일 자원의 내부 전달과 원본 외부 공급 출력의 동등성을
근거로 독립 Print에 대조했다.

이는 #8946 전체 완료나 PR 제출 준비 완료를 뜻하지 않는다. 후속 제출 시에는 해당
최종 head의 나머지 필수 로컬 게이트·회귀 보호와 이슈 수용 항목을 별도로 대조한다.
이번에는 원격 push·이슈 댓글·PR 생성 없이 조사와 표지 재검증 결과를 로컬에 보존했다.
