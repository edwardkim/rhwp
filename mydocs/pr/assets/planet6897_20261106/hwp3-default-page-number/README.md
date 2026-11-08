# HWP3 자동 쪽번호의 기본 글꼴 소유 복원

HWP3 사양의 10.14절 쪽번호 제어는 위치·형식만 저장하고 글꼴을 저장하지 않는다.
명시한 `쪽 번호`/`Page Number` 스타일이 없을 때 모호한 `바탕` 이름을 쓰면 설치된 Windows
Batang TTC를 선택해 독립 한컴2020 Print의 Haansoft Batang과 줄표 폭·숫자 시작점이 달라졌다.
Native HWP3의 암시적 기본값만 명확하게 하고 측정·paint가 같은 글꼴 소유자를 소비하도록 했다.
명시 스타일·HWP5/HWPX 기본값·입력·Print·DPI·판정 기준은 바꾸지 않았다.

동일 PDF·96 DPI의 기존 Linux PDFium PNG를 SHA 검증 후 재사용하고 compat2022·canonical 2px raw로 sample10 Native/fresh WASM 각각763쪽을
끝까지 산출했다. Native 최저99.41%, WASM99.38%; 90% 미달·쪽 누락·중복은0이다.
629쪽은 두 경로 모두99.96→100.00%다. review와 overlay에서 번호의 시작점·줄표 폭을
직접 확인했고 미세한 기준선·안티앨리어싱 차이를 남아 있는 차이로 구분했다.
Native 보호 TSV 하락0/상승11쪽, WASM 하락8/상승5쪽이다. **전체 보호값 복원 완료가 아니다.**

WASM 하락8쪽을 수정 전 frozen 패키지로 다시 측정했다. 7쪽의 표시값은 같고658쪽은
99.93→99.94%다. 629쪽을 포함한9쪽의 전후 PNG 차이는 모두 자동 번호 영역에만 있다.
이 진단의 영역 분리는 원인 확인용이며 시각 gate에서 영역을 제외하거나 점수를 다시 계산하지 않았다.
나머지 보호값 차이는 계속 보정해야 한다.

명시 스타일 HWPX 대조군2쪽은 두 경로 모두100.00%이고 기존 Native PNG와 바이트가 같다.
전체93조건의89개 원본 형식·스타일 감사에서 Native HWP3는 sample10 및 명시 쪽번호 스타일을
가진2개뿐이었다. 다른 문서의 검증을 대신하는 전쪽 통과 주장은 하지 않는다.

새3개 검사는 수정 전1 PASS/2 FAIL, 수정 후3 PASS이고 Mac 최소 글꼴 sandbox에서도3 PASS다.
설치 글꼴을 읽는 검사가 아니라 생성 번호의 소유·기본값·독립 Print 시작점/폭/기준선 ±2 CSSpx를
검사한다. 잘못된 진단 precondition으로 중단된 초기 초안2개는 이 실패→통과 근거에 포함하지 않았다.
정식 자동 수집 경로도 최소 글꼴 환경에서3 PASS/0 FAIL(312건은 선택 밖)다.
불필요한 반복문 정리 후 suite 갱신을 누락해0건 수집한 재실행1회는 무효로 보존하고,
`--prepare` 재생성 뒤 위3건을 실제 재검증했다. 정식 자동 수집 경로의 결과는 [validation.json](validation.json)에 기록한다.

[Native review](after-p629-review.png) · [overlay](after-p629-overlay.png) ·
[fresh WASM review](after-wasm-p629-review.png) · [overlay](after-wasm-p629-overlay.png).

Mac WASM은 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.
Docker 최적화 통과가 아니다. 루트pkg·Studio public·frozen의 JS/WASM hash 일치는 receipt에 있다.
기존 전체 nextest13072 PASS는 수정 전 code head의 결과이며 최종 후보 전체 검증으로 인용하지 않는다.

Native Skia lib·해당 suite Clippy(`-D warnings`), fmt, 현재 base의 manifest48/48, 문서 링크 검사도 통과했다.
최종 전체 회귀·나머지 보호 문서·실제 소비 경로 검증은 이 집중 검증으로 대신하지 않는다.
