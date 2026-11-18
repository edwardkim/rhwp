# R41 리베이스 전 제품의 Native 전쪽 측정과 주장 페이지 판독

제품 source `09ada34bf1be3a3dcb60e36f2de016e8d35cfcd9`, 실제 검증 source
`7f88708f08d14d47537ef34578c969dc057ee6e8`의 완료 기록입니다. 문서 커밋
`651df3b1078b39f229df01eac0a97c8a5f6eb6e3`까지는 제품·테스트 source가 같았습니다.
이후 R41에서 새 devel을 반영했으므로 아래 결과를 새 후보의 실행으로 재표기하지 않습니다.
이 기록은 fresh WASM,
보호 문서 전쪽 대조, 최신 base 대조나 개별 PR 최종 판정의 완료 선언이 아닙니다.

## 완료한 검사와 실제 결과

| 검증 | 완료한 범위 | 실제 결과 |
| --- | --- | --- |
| 최소 글꼴 전체 Rust 회귀 | `nextest --tests --test-threads 12 --no-fail-fast` | 12,851건 모두 PASS, 기존 skip 45건, exit 0 |
| Native Skia lib | `cargo test --features native-skia --lib` | rhwp 3,853 PASS/기존 ignored 12, 나머지 workspace lib 182 PASS, exit 0 |
| 현재 PR Native 시각 검증 | 56개 입력·독립 Print 조합의 전체 662쪽 | 쪽수 일치, 모든 쪽 raw 90% 이상, 전체 최저 92.99% |
| canonical 편람 gate | HWPX Native 전체 383쪽 | 최저 99.96%, exit 0 |
| Studio·실제 브라우저 경로 | Node 2,753건의 동일 입력 증거 재사용 및 현재 WASM의 build/API/Canvas 경로 | 필수 11개 단계 exit 0 |
| 새 base 추가 기능 대조 | Native·브라우저 저장/반복 저장 등 31행 | 실제 입력 SHA를 고정한 비교 통과 |
| HWP3 제목 원본·HWP 저장·HWPX 저장 대조 | 각 16쪽, Native/fresh WASM 각각 48쪽 | Native 최저 99.58%, fresh WASM 최저 99.50%, 모두 독립 Print와 쪽수 일치 |

전체 Rust와 Native Skia lib는 공개 글꼴 5개만 노출한 최소 글꼴 환경에서 실행했습니다.
Cargo 빌드 잡은 8개, 테스트 스레드는 12개, 공용 산출물 경로는 `target/pr-review`입니다.
원시 실행 로그를 커밋하지 않고 실제 종료 결과를 기록합니다.

## 같은 문서·같은 페이지의 증적

- [Native 주장 70쪽의 실제 판독·원본/PDF SHA·PNG 지문 목록](native-claim-reading-r40.tsv)
- [Native 56개 입력의 전쪽 TSV·원본·Print 목록](native-full-results-r40.tsv)
  — `evidence` 열의 각 경로에 전체 TSV·실행 근거·대표 패널을 보존했습니다.
  같은 원본의 여러 주장 쪽이 있을 수 있습니다.
- 대표적인 실제 주장: [PR #8605](https://github.com/edwardkim/rhwp/pull/8605)의
  [육아휴직 권고안 1쪽](final-r38/primary/native/childcare/review-001.png)·
  [overlay](final-r38/primary/native/childcare/overlay-001.png),
  [18쪽](final-r38/primary/native/childcare/review-018.png)·
  [overlay](final-r38/primary/native/childcare/overlay-018.png).
  보호용 sample16 이미지를 원 PR의 실제 주장 문서 이미지로 바꾸어 쓰지 않습니다.
- [PR #8607](https://github.com/edwardkim/rhwp/pull/8607)의
  [위임사무표 1쪽](final-r38/primary/native/delegation-trimmed/review-001.png)·
  [overlay](final-r38/primary/native/delegation-trimmed/overlay-001.png).
- [PR #8681](https://github.com/edwardkim/rhwp/pull/8681)의
  [통합교부 품의 1쪽](final-r38/primary/native/stored-band/review-001.png)·
  [overlay](final-r38/primary/native/stored-band/overlay-001.png).

70쪽 중 58쪽은 현재 출력을 직접 판독했습니다. 나머지 12쪽은 이전에 직접 판독한
출력과 실제 RGBA 및 독립 PDF PNG SHA가 모두 같음을 확인하고 해당 판독 이력을
재사용했습니다. 단순 점수 일치나 파일명 일치로 판독을 재사용하지 않았습니다.
재사용한 판독의 관측 시각과 잔여 차이도 목록에 그대로 남겼습니다.

시각 검증에는 전체 시스템 글꼴과 기존 승인 대체 환경을 공급했습니다. 최소 공개
글꼴 회귀와 구분하며 OTF 등 비공개 글꼴 파일을 저장소에 추가하지 않았습니다.
기존 입력·기준 PDF는 같은 Git 경로와 SHA로 재사용하고 새 PDF를 만들지 않았습니다.

## 점수와 직접 판독의 구분

표 시작·종료·후속 본문, 중첩 셀의 내용·소속·순서, 각주와 캡션 표, 수식 첨자와 기호를
같은 쪽의 독립 Print와 대조했습니다. 요약문 2쪽과 결의서 `36474334`의 2쪽은
독립 PDF에도 있는 실제 빈 쪽임을 확인했습니다. 전체 쪽수에서 제외하지 않았습니다.

추적 플래그 0 대조군의 연구내용 일부 간격과 마지막 활용방안 자간은 차이가 남습니다.
97.75%를 전체 간격 해결이나 픽셀 동일로 해석하지 않습니다. 원 PR은 0x08 입력의
저장 줄을 보존하고 플래그 0 입력은 원본 프로그램으로 재조판하는 서로 다른 계약을
검사합니다. 해당 차이의 최신 고정 base 대조와 원 계약의 판정은 아직 남았습니다.

보고서 `628`의 기존 제목 들여쓰기, 일부 수식·영문 괄호 주변의 잉크/간격 차이와
대조군의 외곽선 차이도 판독 기록에 남겼습니다. 현재 각주·표 개선을 문서 전체의
모든 차이 해결로 확대하지 않습니다. 새로운 회귀인지 기존 차이인지는 같은 고정
base의 실제 출력과 대조하며, 점수 통과만으로 이번 변경의 결함을 면제하지 않습니다.

Native 공식 개별 Skia 검사·lint, fresh WASM 전체 측정·확대 판독, 보호 PR
[#8389](https://github.com/edwardkim/rhwp/pull/8389)·
[#8397](https://github.com/edwardkim/rhwp/pull/8397)의 표시값 무하락 및 최신 base
대비 0.1%p 조건이 모두 끝난 뒤 각 원 PR의 최종 리뷰를 갱신합니다.
