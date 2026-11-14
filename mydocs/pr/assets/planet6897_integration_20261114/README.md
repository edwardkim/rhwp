# planet6897 PR 5개 통합 검증

최종 구현 검증 source는 `836615963eedfac9170d45c15cbba32d23e1b4d0`, base는 `a210861488468868b5992231c55b78798e622e98`다. 이후 증적·리뷰 문서만 바뀐 head의 runtime 동일성은 코드 diff와 producer 해시로 구분한다.

원 PR #8976·#9064·#9065·#9067·#9068의 원15개 커밋과 필요한 공통 메트릭 보정3개를 포함한다. 원 PR의 10k 코퍼스 기록은 작성자의 선행 기록이며 이번 통합에서 새로 수행한 전수 검사로 표시하지 않는다.

## 최종 전쪽 출력

| PR | 문서 | 쪽수/백엔드 | Native 최저 raw | fresh WASM 최저 raw |
| --- | --- | ---: | ---: | ---: |
| #8976 | owned-tac | 11 | 95.63889% | 95.64295% |
| #9064 | ws-host | 2 | 100.00000% | 100.00000% |
| #9065 | page-band | 4 | 95.97412% | 96.01502% |
| #9067 | title-mark | 10 | 94.38083% | 94.48671% |
| #9068 | repeated | 5 | 99.79582% | 99.79577% |
| #9068 | orphan | 8 | 95.42298% | 95.42139% |

전체6문서·40쪽/백엔드이며 canonical96DPI·compat2022·2px raw 기준이다. 원본/PDF·문턱을 바꾸지 않았다. 정상 설치 글꼴 환경에서 Visual Sweep을 실행했고, 회귀만 최소 글꼴 sandbox에서 실행했다. Mac fresh WASM은 locked wrapper `--target web --out-dir pkg --no-opt`의 로컬 대체 빌드이며 Docker 최적화 결과가 아니다.

## 정본 증적

- [입력·PDF 해시, 전쪽 정밀값, TSV 바이트·해시](claim-validation.json), [고정 producer·Native/JS/WASM 해시](final-claim-producer.json).
- [Native/fresh WASM 보호 문서](protected-validation.json): 각각18종·1,771쪽, 총3,542쪽, 공식 두 자리 표시값 하락0쪽. [페이지별 비교](protected/comparison.tsv). 편람 HWPX Native383쪽 `--handbook-gate`도 통과했다.
- [전체 회귀](full-nextest-result.json): 13,533통과·0실패·45skip. [최소 글꼴 감사](minimum-font/inventory-audit.json).
- [원 PR/적용 SHA와 보정 출처](integration-provenance.json), [최신 devel·원 PR 간·누적21건 병합 시뮬레이션](merge-simulation.json).
- [현재 base 비교에서 과거 고정 base 증거를 재사용한 경로 감사](rebase-base-read-path-audit.json), [페이지별 base/후보 비교](base-candidate-comparison.json). 실제 쪽수가 다른 base의 공통 쪽 번호 비교는 진단값이고, 후보 전체 내용 소유·쪽수 검사와 구분한다.
- [직접 판독 및 동일 raw 재사용](direct-reading-final.json). 대표 review와 standalone overlay는 항상 같은 입력·페이지다.
- [#8973 중첩 표 줄 보정](row8973/cause.md), [표 뒤 앞 간격 보정](closing-line/cause.md), [1쪽 종료 글자 높이 보정](leading-line/cause.md).
- [원문 #9062 마지막 두 쪽 소유·쪽수](original9062-ownership.json): 원문 전체 시각 수용과 구분한다.

## 실제 기준 PDF

owned-tac·ws-host·page-band는 데스크톱 한컴2020 Print, title-mark·repeated·orphan는 실제 PDF Creator 기준 한컴2022 `12.0.0.4547` Print다. 파일명의 연도나 서비스 engine 표기와 실제 제품을 혼동하지 않는다. 웹한글기안기/Cairo PDF로 판정하지 않았다. 유효한 기존 독립 Print를 재사용했다.

## 실행 중인 추가 검증

[현재 Rust 게이트](rust-validation.json)는 fmt·manifest·집중 회귀·전체 회귀·Skia·Native/WASM/all-target Clippy·workspace build 29단계 모두 통과했다. 리베이스 반영 기능·실제 Studio 소비 검증은 완료 후 별도 기록한다. svg-consumers/·actual-studio-before-pr9068/·title-repeated-save.json은 원래 source에 대한 이력이며 최신 실행 완료로 읽지 않는다. 최종 PR 제출 전 이 절을 실제 결과로 교체한다.
