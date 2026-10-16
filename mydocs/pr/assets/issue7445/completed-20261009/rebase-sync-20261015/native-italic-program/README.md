# Native의 실제 face와 합성 기울임 소유

## 결함과 경로

`TextStyle.italic → SkiaTextReplay → 실제 선택 Typeface → finish_font → draw_str`에서 기울임 face를 요청했더라도 단일 직립 face가 반환되면 출력 Font에 기울임이 적용되지 않았다. 공통 조판·글자 원점·advance는 그대로 두고 실제 face가 Upright일 때만 요청 기울임을 합성한다. 이미 Italic/Oblique인 프로그램에는 추가하지 않는다. 그림자·외곽선도 같은 Font를 소비한다. 폼 라벨·수식은 별도 경로이며 이번 변경 대상이 아니다.

프로젝트 문서의 독립 Print는 Batang·휴먼명조 직립 프로그램을 `83 0 27.639 -83` 텍스트 행렬로 출력했다. 대조한 x/y shear는0.333이다. 문서명·문자열 분기가 아니라 요청 스타일과 실제 face의 기울임 속성으로 적용한다.

## 검증

- 공개 Native PNG 검사2건: 동일한 직립 face·원점·advance에서 기울임은 위쪽 잉크를 오른쪽으로 옮겨야 한다. 실제 Italic/Oblique 프로그램은 요청 비트로 이중 기울어지지 않아야 한다. 수정 전1FAIL/1PASS,수정 후 정식 nextest2PASS(0.109초). 원본 fixture 프로그램은 기존 공개 테스트용 글꼴이며 새 글꼴 바이너리를 추가하지 않았다.
- Native 빌드·manifest·fmt/check 통과. Mac fresh WASM web/no-opt 로컬 대체 빌드 후 루트pkg/Studio public hash 일치. 이 변경 모듈은 Native 전용이다.
- Native/fresh WASM 각각6문서42쪽 전쪽90% 이상·독립 쪽수 일치: 프로젝트2,변환본16,법원2,위첨자3,그림 채움14,근린시설 표5. 각 TSV와 수치는 evidence.json（로컬 중간 자료 `evidence.json`）에 있다. 문서별 원래96/192DPI 조건을 유지했다.
- 프로젝트2쪽 Native 최저93.81061%,fresh WASM93.79837%. 같은 명시 글꼴 환경의 고정 devel463c48cc2 대비 양쪽 전2쪽0.01%p 이상 하락0쪽. Native 첫 쪽은 직전 후보93.93027%보다 낮지만 고정 devel92.88471%보다 높다. 이 차이를 숨기지 않는다.
- `review.png`를 직접 열어 직립으로 남던 글자가 기울어진 것을 확인했다. 글자 굵기·일부 간격·밑줄의 잔여 차이는 남아 있으며 완전 일치를 주장하지 않는다.

위첨자3쪽과 그림 채움2쪽의 작은 devel 하락은 별도 보정 대상이다. 이42쪽 확인을 원43조건·전체 회귀·actual36 완료로 인용하지 않는다. 실행 로그와 전체 생산 소스/hash는 ignored output `rebase-sync-20261015/native-italic-program-candidate-v29/`에 보존했다.
