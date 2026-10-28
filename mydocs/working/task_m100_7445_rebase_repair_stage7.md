# Issue #7445: 최신 devel 리베이스와 WMF 마스크 색 보존

## 리베이스와 보존

upstream/devel9415b920c8dec299e8ada69b01dbd1c3c044f1f1(#7964)을 fetch했다.
직전 보정은d9cc469f50으로 로컬 커밋한 뒤250개 후보를 다시 적용했다.
리베이스 HEAD는ac5d062c8d7e0194c3254cf58ee925bd055b94a7이다.
image_resolver의 upstream 색상 모듈과 이 브랜치 crop 원본 좌표 helper를 함께 보존했다.
문단 원점은 upstream의 source-line 들여쓰기 공통 결과를 유지하고 과거 clamp로 되돌리지 않았다.
이미 upstream에 있는 raw90/raw98 Visual Sweep 패치는 중복 적용하지 않고 확장된7751 보고서를 보존했다.
초안/PDF 등42개 파일 SHA를 모두 복원했다. backup/7445-before-9415b920-20261017과
stash f4eea7a90560c339cdfa4d36c9b63d2523e64941도 보존했다.
기존 stage6 보고서는2026-10-14 이력이며 덮어쓰지 않는다.

## WMF 원인과 후보

사용자 수동 Print64쪽과 원본 BIN0003.WMF, Windows GDI+ Metafile.DrawImage의
분홍색 패널을 대조했다. 원본은 표준 WMF이며 GDI+ 변환 후 header의 IsWmf=false를
원본 EMF의 증거로 쓰지 않았다. 두 XOR 바깥 사각형 안에 완전히 포함된 DPA 흑백
마스크는 checker 무늬/8단위 크기에 의존하지 않고 원본 브러시색을 선택한다.
불완전 연산·변경된 최종 사각형·clip·비흑백 팔레트와 밖으로 튀어나온 비checker는
이 정확한 포함 규칙의 대상이 아니다. 기존 별도 halftone 경계 근사는 확대하지 않았다.

Native127.449초/fresh WASM235.296초의 후보 actual PNG 전64쪽은 각각
최저93.07399/95.87967%,쪽수64/64·미달0이다. Mac web/no-opt 로컬 대체 빌드이며
Root pkg/Studio public/동결 pkg JS·WASM hash를 대조했다. Docker 최적화 통과가 아니다.
18쪽 전쪽 raw98.01466/97.96086%,중요19쪽99.43242/99.61477%와 양쪽 review/overlay를
직접 판독했다. 18쪽은 관계형 마스크 검사를 추가하며 양쪽98% 미달이므로 px로 고정하지 않는다.
19쪽 기존 제목 px 검사의98% 선행 조건은 유지한다. 새 그림의 PNG hash가 이전과 달라
이전 직접 판독만 재사용하지 않았다. 이전 후보 전64쪽 대비0.01%p 하락은0쪽이며
이를 새 upstream 비교 완료로 기록하지 않았다.

[전쪽 TSV·독립 마스크 근거·전후 review/overlay](../pr/assets/issue7445/completed-20261009/regression-policy-20261015/wmf-contained-mask-20261017/README.md)를 보존했다.
정식 회귀2건은 새 전쪽 선행 검증 후 추가했다. 동일 ac5d062c의 WMF 포함 규칙만 되돌려
실제17건15PASS/2FAIL(0.137초), 복원 후17PASS(0.097초)를 확인했다.
컴파일 포함248.681/158.995초이며 새 두 검사만 의도한 mask 소유 assertion에서 실패했다.
리베이스된 글꼴/탭16개도16PASS(컴파일 포함8.508초)다. 동결 PNG producer의 모든
production source SHA와 복원된 작업장이 같음을 확인했다. 전체회귀 완료가 아니다.

## 잔여와 공유 빌드

새9415b920 Native의 거대 셀80쪽은94.40109%,후보96.82269%다.
같은 새 기준의 sample16은65쪽이므로 독립64쪽과 whole TSV를 만들 수 없다.
마지막65쪽에도 합의각서 본문624자가 있어 빈 쪽으로 제거하거나64쪽으로 자르지 않았다.
원문/PDF·65개 render tree와 canonical 불일치 오류를 보존하고 현재 후보64쪽 수용과 구분한다.
기준 WASM과 전체48조건 비교·최신 전체회귀/gates·실제68소비는 아직 미완료다.

공유 target 대기 검사에서 다른 main 작업장의 wasm-pack을 발견해 첫 baseline 시도는
Cargo 시작 전에 중단했다. 해당 프로세스/산출물을 변경하지 않고 종료까지 기다린 뒤
새 baseline 빌드를 시작했다. 첫 시도를 컴파일 또는 테스트 실패로 합산하지 않는다.

거대 셀80쪽의 실제 Canvas 선폭0.5 대 독립 PDF0.480000019CSSpx도 별도로 진단했다.
무변경 spy는 canonical PNG와 byte-identical이다. paint 폭만0.48로 바꾼 진단은
raw97.50476%로 같고, 원점도 과거 snap으로 옮기면96.81942%로 낮아졌다.
이 counterfactual은 생산 코드·통과 증거로 채택하지 않았다. 최소 선폭·허용치·mask를
수치 하락을 숨기도록 바꾸지 않고 실제 원점·본문 원인과 새 기준 비교를 계속 조사한다.

새 baseline도 Native191.126초/fresh WASM299.082초로 완료했다. JS null 환경 선택이
46개 원본에서 변경false·쪽수/첫쪽 layout hash 유지임을 실제 새 package에서 다시 확인했다.
Native와 WASM 모두 sample16의65vs64 불일치로 canonical 전체 비교가 종료되어 잘라 만든 TSV가 없다.
WASM giant80은 새 기준97.54273→후보97.50476%,0.03797%p 하락이 남는다.
지표가 같은 paint-width-only 진단이나 수치가 낮아진 과거 snap 진단을 생산 코드로 채택하지 않았다.


## C87 후보 — 같은 런의 쪽번호도 동일한 오른쪽 탭 끝점 사용

HEAD acd4c6e316의 전체 nextest는10664건 중10662PASS/2FAIL/45skip,
실행466.433초(컴파일 포함765.119초)로 끝났다. 실패는 기존 issue1116의
HWP3/HWP5 목차 오른쪽 끝 정렬 두 검사다. 기준을 바꾸거나 검사를 제거하지 않았다.
사용자 수동 Print p2의26개 숫자 블록 오른쪽 끝은 모두641.405396CSSpx다.
현재 출력은641.4933과637.159967의 두 그룹이며 차이는17.3333px 글자 크기의
0.25em이다. 숫자15/16/20 등은 탭과 같은 런에 있어 C85의 교차 런 보정 밖에 있었다.

`right_leader_tab_target_rel → estimate_text_width_exact → 문단 폭`과
`right_leader_tab_target_rel → compute_char_positions → TextRun 문자 위치`를
대조했다. 공통 helper가 숫자 끝점에서 리더 뒤 여백0.25em을 빼고 있었다.
숫자 블록은 탭 정지에 정렬하고 리더 여백은 선의 끝에만 적용하도록 그 감산을 제거했다.
명시적 LEFT/DECIMAL·비숫자·일반 탭 경로와 리더 앞뒤 여백 규칙은 바꾸지 않았다.
문서명·쪽번호·목표 좌표에 의한 구현 분기는 없다.

최종 후보 source의 기존 issue1116은13PASS(0.364초), 글꼴/탭 소유 대조군16PASS
(0.176초), WMF 포함 마스크17PASS(0.109초)다. 13건의 최초 후보 실행도13PASS(0.291초)였고,
그 뒤 불필요해진 지역 변수만 제거한 최종 source로 다시 실행했다.
세부 시간/종료 코드는 ignored `toc-anchor-repair/`의 로그·pipeline-commands.json을
정본으로 한다. 이 집중 검사 통과를 새 전체 회귀 통과로 보고하지 않는다.
새 Native/fresh WASM·전64쪽·실제 Studio 소비와 최종 전체 gate는 후속 검증 중이다.

기존 acd4 실제 Studio96DPI p2/18/41 review/overlay도 직접 열었다. p2의 일부 숫자
오정렬은 raw99.23573%이어도 결함으로 유지했고, p18 분홍 패널 복원과 세부 WMF
글자/선 차이를 구분했다. p41 HCI 기호와 내용 순서는 보존하지만 glyph raster 차이는 남는다.
이 범위는 portable compat2024 전64쪽과 별개인 실제 제품 default2022 경로다.

전64쪽 최저인63쪽도 양쪽 review/overlay로 따로 대조한다. 원문 한양신명조 기호 슬롯의
○는 metric trace에서 heuristicFullwidth이며, 기준보다 큰 원 모양이 보인다.
이를 글자 유무·폰트 설치만으로 해결됐다고 하거나 새 정상 golden으로 고정하지 않는다.
Native93.07399/WASM95.87967%는 이전 acd4 출력의 수치이며 새 후보 통과 수치가 아니다.


### C87 전쪽·실제 소비와 기존 리더 핀 보완

production code4541014e3e와 같은 source의 새 Native148.492초/fresh WASM196.611초를
사용했다. Mac locked wrapper web/no-opt 대체 빌드이며 Docker 최적화 통과가 아니다.
수동 Print64쪽 전체는 Native93.07399/WASM95.87967%·미달0·쪽수64/64다.
2쪽은97.57913→97.97410%,98.33185→98.71502%로 각각 개선됐고, 이전 후보 전64쪽
대비0.01%p 이상 하락은 양쪽0쪽이다. 이는 이전 후보 비교이며 upstream 비교가 아니다.
같은 source의19쪽 raw99.43242/99.61477%와 PNG byte identity도 확인했다.

Native/fresh WASM2쪽 review/overlay를 직접 열어 숫자 끝점 정렬과 한 탭의 단일 리더를
확인했다. 실제 Studio default2022 Canvas2D Print96DPI는99.33039%이며 초기화 응답
JS/WASM2개의 hash가 새 동결 package와 일치한다. 그 review/overlay도 직접 열었다.
실제96DPI 결과를 portable192DPI 쪽의98% 자격으로 바꿔 쓰지 않았다.

전쪽 검증 뒤 기존 `sample16_hwp5_toc_leaders_stop_before_page_numbers`의x650 핀을
같은 글줄의 실제 숫자 glyph를 리더가 침범하지 않는 관계로 바꿨다. 숫자는 리더 시작과
font-size에서 얻은 같은 글줄 관계로 찾고 고정된 용지 영역으로 선택하지 않는다.
같은 issue1116의13개 검사는 모두 유지하여13PASS(0.354초)를 확인했다.
처음 body 수정으로 suite weight가 달라진 뒤 derive만 보고 실행한 시도는0건/exit4였다.
이 실행은 무효로 보존하고 --prepare로 실제15번 suite를 재생성한 뒤13건을 실행했다.
0건을 통과 증거로 합산하지 않는다. source·원본·독립 기대값·허용치는 변경하지 않았다.

[전쪽 TSV·전후 review/overlay·실제 Studio 소비](../pr/assets/issue7445/completed-20261009/regression-policy-20261015/toc-anchor-20261017/README.md)를 연결했다.
전체 nextest·필수 gate·전체48조건의 새 devel 비교·실제68경로는 별도로 남아 있다.

### 추가 원본 HWP3의 기준 확인 — 완료 범위와 구분

issue1116의 원본 HWP3도 별도로 확인했다. 기존 Cairo PDF를 독립 한컴 출력으로
승격하지 않고 원본SHA559fd94860cf836d5800436d56055c23a73e59eedef3da273a5ba365e6ed9e16에서
engine2020의 실제 한컴11.0.0.9136 Print(method0,one-up)을 새로 확보했다.
PDF SHA8f0bfd27ae567481e244f39de6f7b596b3afa8b227dce895d4ebf9cebc416bd0,
64쪽/2150230바이트이며 기존 PDF·원본은 보존했다. 출력 job은
`e5e48314-3620-40cb-a9a8-b90ce3e31348`이다.
현재64쪽의 최저는Native13.84189/WASM14.60170%,각44쪽 미달이다.
3쪽 review/overlay를 직접 대조해 본문 세로 배치·도형·글자 차이를 확인했다.
변환된 HWP5 수동 Print64쪽의 완료 결과와 원본 HWP3 결과를 섞지 않는다.
원본 HWP3는 기존48조건 밖의 추가 자격 조사이며, 현재 전쪽 개선 완료를 주장하지 않는다.
같은 새 PDF/글꼴/compat2022의 고정 devel 전쪽 비교는 진행 중이다.
이 조사에서 기존 원본 HWP3 검사·기대값을 삭제하거나 낮추지 않았다.
