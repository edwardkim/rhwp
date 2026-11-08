# 저장 호스트의 원자 행 프레임·문단 범위 보존

## 원인과 공통 보정

#7728의 전18쪽98% 보정 뒤 전체 nextest에서 CBTA의 본문 넘침·후속 줄 위치와 KPS의
저장 rowspan 문단 범위·첫 프레임, 총4건이 실패했다. 기준 원장·기대값·공차를 바꾸지 않았다.
같은 정식 검사를 고정0896/429 및 단계별 frozen rlib에 실제 링크한 결과 phase9까지3PASS,
원자 행을 재귀 컷으로 넓힌 단계부터3FAIL이었다. 별도 첫 driver의 의존 검색 경로 오류는
컴파일 실패이며 이 비교의 실제 FAIL로 세지 않았다.

70833의9×4·13×4 자식 호스트는 저장 글줄이 없다. CBTA·KPS의 해당 호스트에는 저장 글줄과
원본 첫 조각 프레임이 있다. 새 재귀 경로가 후자의 원자 행까지 다시 해석하며 기존 문단 slice와
후속 줄 예약을 덮었다. 실제 내부 글줄 컷 또는 저장 글줄이 없는 재조판 호스트만 같은 재귀 원장을
사용하도록 공통 판별을 공유했다. 저장 호스트는 기존 행 조각 경로의 첫 프레임·문단 범위를 유지한다.
문서명·쪽 번호·좌표 상수 예외와 검사를 지우는 우회는 없다.

## 실제 결과

- 기존4FAIL 전부 집중 재검증 PASS. 관련 Native 실제17PASS(7728의12건, 그림 글머리표1건,
  KPS2건, CBTA 후속 줄1건, 본문 넘침 partition1건)다. 본문 partition은89입력을 실제 검사했다.
- CBTA Native59쪽 raw34.70778%→98.77747%,60쪽35.55472%→98.90589%.
  fresh WASM58/59/60쪽은98.13700/98.76943/98.91913%다.
- KPS Native65/66쪽은99.33994/100%,fresh WASM99.32808/100%다. 같은 고정0896 Native의
  65·66쪽과 현재 PNG가 각각 hash 동일하다. 잘못된 중간 결과의65쪽99.73849%를 정상 계약으로
  고정하지 않는다. raw가 높아도 model 문단 범위가 사라진 것과 선언 프레임 훼손은 실제 결함이다.
- 원 PR #8187의 실제 Chrome API36개와 레이어 `modelParaRange`의[0,3]/[3,5] 관계2개,
  총38개가 통과했다. HWP/HWPX 저장·재열기의 쪽수와 라벨 소유도 포함한다. 원본 검사 assertion은
  유지하고 고유 output 경로·추가 관계 관측만 적용했다. 이는 실제 Studio 화면 검증의 대용이 아니다.
- #7728 Native/fresh WASM 전18쪽을 새로 출력했다. 각각 최저98.83905%/98.83565%,
  미달·누락0이며 각18개의 PNG는 phase10과 hash 동일하다. 전쪽 TSV도 각각 동일하다.
  [전18쪽 TSV·대표 그림](../phase10-all-pages-98-20261106/README.md)을 같은 입력·새 producer의
  hash 대응으로 재사용한다. 이전 실행을 새 source에서 캡처했다고 바꾸어 쓰지 않는다.
- CBTA59쪽 before/after Native review와 after WASM overlay, KPS65쪽 after Native review와
  after WASM overlay를 직접 판독했다. 표 아래 후속 문단·제목과 라벨/이어받기 소속을 확인했다.
  가는 선·글리프의 미세 차이는 남고100% RGB 동일 출력으로 과장하지 않는다.

## 출처와 미완료

`verification.json`에 parent+정확한 runtime diff, Native/WASM/JS·전18쪽 PNG/TSV 대응을 고정했다.
원본·독립 Print PDF는 기존 경로와 hash를 유지한다. CBTA는 현재 정본의2020 Print이고,
KPS는 `rhwp-studio/public/samples/kps-ai.hwp`와 원 PR의
`mydocs/pr/assets/platy-native-editing/inputs/kps-ai.hwp.hancom.pdf`다.

Mac locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며 Docker 최적화가 아니다.
Root pkg·Studio public·frozen package의JS/WASM hash를 확인했다. 초기 API driver의 추가 응답
감사는 Chrome inspector cache 오류였고 API 실패로 세지 않았다. 해당 관측을 제거하고 원 PR의
검사+추가 관계로 재실행했다. 실제 초기화 응답 hash는 다음 정식 Studio/E2E 단계에서 확인한다.

전체 nextest·Skia·lint/build·전쪽 정상 대조군과 실제 Studio 및 편람 gate는 새 source로 다시
진행한다. 이 단계의 집중·선택 쪽 결과를 CBTA204쪽/KPS77쪽 전체 수용이나 PR 준비 완료로 쓰지 않는다.
로그·진단 driver·중간 JSON·응답 감사 실패와 기존 전체 실패는 ignored output에 보존한다.
