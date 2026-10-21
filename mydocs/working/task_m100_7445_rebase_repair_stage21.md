# #7445 리베이스 보정21 — 다단 fit과 후속 TAC 원본 흐름 분리

Issue: #7445, 관련 #702 / #2299.

브랜치 `fix/7445-completed-20261009`, parent `f437aa89fc783a96673afc90ec423c3b817854b6`,
base `2e3b4f880bd11d35398e6ea0100846611f13a72d`. 로컬 보정 체크포인트이며 PR 준비 완료가 아니다.
원격 push·PR·게시·merge는 수행하지 않았다.

## 실제 회귀와 원 계약

Stage20 전체 nextest780은11335실행11324PASS/11FAIL/45skip, 실행818.664초다.
로그의 실패 목록을 두 번 센22건을 실제 실패 수로 쓰지 않는다. 이 중 단축키 관련4검사는
Stage20의 다단 변경이 만든7→9쪽 회귀였다. 과학의 저장 줄 흐름을 보존하기 위해 모든
일반 다단 문단의 fit 전진까지 절대 저장 끝으로 바꾼 가정이 잘못됐다.

원 이슈702의 본문·논의·timeline과 원PR710의 본문·코멘트·diff를 다시 확인했다.
원 구현507b51361f3147bfa8d22a5a1aee59eaf94ccf31은 단정의 갱신과 짧은 배분 단의
vpos reset을 복구했고 당시10→8쪽, 잔여708/709를 구분했다. 현재7쪽/내용·단 소유 검사는
후속 로컬 보정d3bb24fb0ee2b374f5c7b4bd23d5fe4d42bce074의 계약이다.
원PR의8쪽 결과를 근거로 현재7쪽 검사를 완화하지 않았다. 현재 독립 기준은 기존
`pdf/basic/shortcut-2022.pdf` 전7쪽이다.

## 공통 소비 보정

`stored_multicolumn_tac_successor_flow`는 다음 원본 문단이 전체 단일 TAC 줄을 실제로
소유하고, 앞 문단의 저장 끝+마지막 줄간격+뒤 간격+다음 앞 간격이 그 원점에서 닫힐 때만
저장 후속 흐름을 전달한다. 편집·재조판·단나누기·다른 축은 기존 인증을 유지한다.
일반 다단 문단은 기존 `flow_advance_height`의 fit 전진을 유지한다. 문서명·쪽·내용별 분기는 없다.

생산은 content.rs의 원본 인접 TAC 질의이고, paragraph.rs의
`record_stored_paragraph_flow_end`/`record_paragraph_content_bottom`/`apply_full_paragraph_flow`가
같은 끝을 소비한다. 과학의p118→119와p120→121 원점 보존은 유지하고 단축키 일반 문단의
쪽말 fit을 되돌렸다. paint 단계에 별도 이동을 추가하지 않았다.

## 같은 입력 전쪽 결과와 판독

| 문서 | Native 쪽수/최저 | fresh WASM 쪽수/최저 |90% 미만|
|---|---:|---:|---:|
| shortcut |7/7 /91.78413%|7/7 /91.99857%|각0|
| exam_science |4/4 /96.14438%|4/4 /96.06833%|각0|

shortcut Native790/fresh WASM793, 과학Native789/fresh WASM792의 전체 TSV를 새로 산출했다.
canonical PDFium smooth_rgb_v1/raw2px90%, compat2022, 캡처8을 유지했다.
shortcut은192DPI/opensource+사용자 Fonts, 과학은 원PR8064와 같은96DPI/사용자 Fonts다.
단축키 부모713의 전쪽 최저91.78141%와 대조한 최대 하락은0.01663%p로0.1%p 미만이다.
Stage20의9/7쪽 실행786은 쪽수 gate에서 실패했으며 유효 전쪽 통과 증거로 쓰지 않는다.

과학8 PNG(두 backend×4쪽)는 Stage20의772/775와 각각 byte-identical이다.
입력·생산자·글꼴 조건과 PNG SHA를 연결해 이전3·4쪽 직접 판독을 재사용한다.
과학 인라인 본문의 미세 기준선 차이가 남는다는 기존 판정도 유지한다.
shortcut Native1·7쪽, WASM7쪽의 review/overlay를 직접 읽었다. 두 단 꼬리·단축키·구분 띠·
장식 번호7의 내용과 순서는 보존됐고 점선/잉크 가장자리 차이는 남는다. 완전 픽셀 일치 주장이 아니다.

[전쪽 TSV·입력/PDF/생산자·PNG 동일성](../pr/assets/issue7445/completed-20261009/stage21-20261021/verification.md)

[Native7쪽 review](../pr/assets/issue7445/completed-20261009/stage21-20261021/shortcut-native-p007-review.png) ·
[overlay](../pr/assets/issue7445/completed-20261009/stage21-20261021/shortcut-native-p007-overlay.png)

[fresh WASM7쪽 review](../pr/assets/issue7445/completed-20261009/stage21-20261021/shortcut-wasm-p007-review.png) ·
[overlay](../pr/assets/issue7445/completed-20261009/stage21-20261021/shortcut-wasm-p007-overlay.png)

## 정식 회귀

기존2299 편집/702 쪽 소유2검사와 overlap/oracle partition2검사가 수정 후 각각 통과했다
(788/795). 기존 과학4검사도 통과했다. 실행 범위가 다른 중복 검사를 전체 고유 건수로 합산하지 않는다.

새 `completed_shortcut_last_page_keeps_both_column_tails_and_print_baselines`는 전체7쪽에서
원문 항목이 마지막 쪽에 각각 한 번만 나오고 원본 문단·단축키·좌우 단 순서를 보존하는지 검사한다.
완료7쪽의 양쪽 raw99.65777/99.79309%와 직접 판독을 먼저 확보했다.
같은 동작 중 양쪽 최저가 높은7쪽을 선택했으며, 독립 PDF의 마지막 두 단 기준선을
CSS96 x121.28/589.92, y461.76의±2px로 보호한다. 후보 bbox에서 기대값을 복사하지 않았다.

최종 테스트 source 그대로 frozen771에서4PASS/1FAIL →787에서5PASS다(796 final 로그).
prepare 뒤 실제 자동 nextest797도5PASS/0FAIL/243미선택, 실행0.126초,
컴파일 포함123.512초/exit0이다. manifest1648sources/7197attrs/48targets 및 fmt check도 통과했다.

fresh WASM791은 Mac locked wrapper web/pkg/no-opt 실제 재컴파일198.010초/exit0이다.
Docker 최적화 결과가 아니다. root pkg/Studio public/frozen hash는 같고 실제 초기화 응답 확인은 남아 있다.

## 잔여

다단4건의 집중 복구를 새 전체 nextest 통과로 보고하지 않는다. 캡션3건·CSAP 간격·머리말 그림
줄 높이·PrEP 격자·공유 괘선 paint 검사의 나머지 실패를 각각 조사·보정한다.
과학 본문 기준선 잔여, 최신 전체 corpus/base 비교·전체 회귀·범위 게이트·actual36도 남았다.
모두 완료한 뒤 PR 진행 승인을 요청한다. 로그/진단은 ignored output의repair780,783–799에 보존한다.
