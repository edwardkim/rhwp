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
