# 단계 395: 로드맵 경계 보정의 전체 시각 검증

## 대상과 기준

- 코드 후보 `b60dca726`: 첫 저장 표 프레임의 셀 경계를 통째 수용보다 먼저 보존했습니다.
- 원본과 독립 Print PDF는 단계 394와 같습니다. PDF는 302쪽입니다.
- 새 테스트나 기대값 변경 없이 기존 Native/HWPX 대조군을 실행합니다.

## 검증 계획

1. 공용 target에서 Native Skia release CLI 빌드 및 바이너리 SHA 고정.
2. Mac no-opt fresh WASM 빌드 및 public/pkg 정합 확인.
3. 기존 wrapper/native 쪽 소속·포함·겹침 대조군과 HWPX 프레임 대조군.
4. fmt 및 Native/WASM32 Clippy.
5. 같은 302쪽 원본의 Native/fresh WASM 전체 TSV와 43·44쪽 review PNG.

실행은 공유 Cargo target을 순차 사용합니다. 단계 394의 첫 두 검사 통과만으로
전체 문서가 90% 이상이거나 PR 준비가 완료됐다고 판단하지 않습니다.

실행 및 로그: `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/stage395-roadmap-after`.

## 캡처 동시 실행

사용자가 CPU 10개 환경에서 수정 후 캡처를 8개씩 실행하도록 지시했습니다.
저장소의 동일 웹폰트 캡처 명령을 8개씩 실행하며, 모든 PNG 완료 후 원래
`write_silhouette_tsv` 함수로 전체 302쪽을 비교합니다. 원본·Print PDF·글꼴·DPI와
비교 알고리즘, 90% gate는 변경하지 않습니다. 별도 target이나 회귀 검사는 만들지 않습니다.
기존 순차 출력과 병렬 대조 캡처 2쪽의 PNG SHA-256이 각각 일치했습니다.
검증 드라이버와 각 캡처 명령·종료 상태는 ignored output에 기록합니다.

## Native 전체 비교 결과

- 보정 전 302쪽 최저 92.38470%, 미달 0쪽. 보정 후도 최저 92.38470%, 미달 0쪽이다.
- 43쪽 96.09838% → 97.92471%, 44쪽 93.23054% → 98.13132%. ‘양질의 시간제’ 문단의 44쪽 소속을 실제 review PNG에서 확인했다.
- 기존 대조군 7 PASS, Native 빌드·Mac no-opt fresh WASM 빌드·fmt·Native/WASM32 Clippy가 통과했다. 전체 nextest 결과를 대신하지 않는다.
- 44쪽의 표 하단 빈 공간 차이가 남아 사용자가 추가 보정을 요청했다. 90% 통과만으로 이 차이를 해결했다고 보고하지 않는다. fresh WASM 전체 비교는 진행 중이며 결과를 별도로 추가한다.
- Native TSV·43/44쪽 PNG 및 해시 증적은 `mydocs/pr/assets/issue7445/completed-20261009/roadmap-source-frame-*`에 보존했다.
