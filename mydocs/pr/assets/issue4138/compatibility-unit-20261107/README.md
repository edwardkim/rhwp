# #4138 CJK 호환 단위 뒤의 공통 줄 경계

2026-11-07, 부모 `fc038e4d54b9edad6e09c0c92935440af01525da`에 줄 끝 후보 보정만 적용했다.
validation.json (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue4138/compatibility-unit-20261107/validation.json`)에 제품 파일·입력·독립 PDF·실행 파일·WASM·글꼴 공급·검사
출처를 고정했다. Mac의 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.
Docker 최적화 빌드로 보고하지 않는다. 로그·임시 driver·글꼴 프로그램은 ignored output에만 둔다.

## 위반 규칙과 독립 근거

실제 분할 저장본
`mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/issue4138-current-production/saved-split-1x2.hwp`
(SHA-256 `4a96f1fc838eb7239d8c85b7698b9e04f83fc6b2f271c948ccae038a1c6c89b9`)과 기존
`pdf/issue4138-ascii-split-20261016-2024.pdf` 전197쪽을 그대로 사용한다. 새 입력이나 PDF로 교체하지 않았다.

독립 Print120쪽의 문단1575는 앞줄이 `밀도0.5㎥`로 끝나고 다음 줄이 `/kg로 계산한다.`로
시작한다. 보정 전에는 단위 뒤 줄 끝 후보를 잃어 `0.5㎥/kg로 계산한다.` 전체를 다음 줄로
되감았다. `㎥`는 개별 기호 토큰이지만 공통 줄 끝 후보 판별이 한글·숫자·한자·일부 리더만
허용했다. 토큰의 실제 폭이 들어가도 그 지점을 줄 끝으로 선택할 수 없었다.

[Unicode17 LineBreak 데이터](https://www.unicode.org/Public/17.0.0/ucd/LineBreak.txt)는
U+3300..U+33FF CJK 호환 기호 전체를 ID로 분류한다. 이 범위가 한자처럼 줄 끝 후보를
소유하도록 공통 판별자에 반영했다. `/kg` 시작 경계는 실제 한컴 Print로 확인한 계약이며,
Unicode 기본 줄 나눔의 모든 규칙을 그대로 구현했다고 주장하지 않는다.
글꼴 슬롯·전진폭·열 폭·fit 공차·원문은 바꾸지 않았다. 측정 경로와 실제 줄 채움 경로가
같은 판별자를 쓰며 문서명·쪽 번호·너비 예외는 없다.

선행 원인은 #4138의 분할 셀 재조판과 이전 두 단계의 숫자·금칙 줄 소유까지 추적했다.
앞 단계는 `140℃`, 한점 리더와 한자 뒤 괄호를 복구했지만 CJK 호환 단위 뒤 후보는 빠져 있었다.
그 검사는 그대로 유지하고 이번 누락을 공통 계층에서 보완했다.

## 실제 출력과 회귀

현행 Visual Sweep, PDFium·192DPI·compat2024·색상 경계 보정 전2px raw다.

| 경로 | 보정 전120쪽 | 보정 후120쪽 | 보정 후 전197쪽 최저 | 98% 미달 |
| --- | ---: | ---: | ---: | --- |
| Native | 95.84016% | 100.00000% | 94.83034% | 21·44·129·130쪽 |
| fresh WASM | 95.90529% | 100.00000% | 94.98570% | 21·44·129·130쪽 |

전쪽 TSV 원문은 [Native](native-silhouette.tsv)·[WASM](wasm-silhouette.tsv)이다.
각197쪽과 독립 PDF 쪽수가 일치하며 누락·90% 미만은 없다. 앞 단계의 동일 입력·PDF·글꼴
전197쪽과 대조하면 두 경로 모두120쪽만 개선되고 나머지196쪽 PNG는 바이트가 같다.
앞 단계는 리베이스 전의 역사적 실행이며 현재 부모fc038의 전쪽 baseline으로 재명명하지 않았다.
현재 부모fc038의 보정 전120쪽은 두 경로에서 별도로 새로 출력했다.

[Native before review](before-native-review.png)·[overlay](before-native-overlay.png),
[after review](after-native-review.png)·[overlay](after-native-overlay.png),
[WASM before review](before-wasm-review.png)·[overlay](before-wasm-overlay.png),
[after review](after-wasm-review.png)·[overlay](after-wasm-overlay.png)를 직접 대조했다.
본문 확대 [Native](native-detail.png)·[WASM](wasm-detail.png)·[PDF](pdf-detail.png)는 같은 원본
PNG의 같은 영역을 재표본화 없이 자른 것이다. 단위·슬래시·앞뒤 문장·들여쓰기·기준선을 확인했다.
집중120쪽과 전체 실행의120쪽 PNG 해시도 같다. raw100%는2px 관용값이며 RGB 완전 동일은 아니다.

새 관계형1건은 앞줄 종료·다음 줄 시작과 원문 문단의 누락·중복 없는 보존을 검사한다.
새 고정형1건은 같은 페이지의 두 경로100%와 직접 판독 후 독립 PDF의 단위·다음 줄 원점과
기준선을192DPI ±2px로 보호한다. 기대값은 구현이 아니라 PDF의 문자 위치에서 얻었다.
두 검사 모두 실제 수정 전2 FAIL→수정 후2 PASS였다. 기존 저장 프레임·금칙·기호 슬롯·
영문 낱말·앞 단계 글상자 검사를 포함한 Mac 최소 글꼴 집중 묶음은75 PASS다.

앞 단계의 원본3종과 글상자 반례5종도 최종 source에서 두 경로 전16건을 다시 출력했다.
모두100%이며 이전 커밋의 원본·PDF PNG와 해시가 같다. 루트 pkg·Studio public·frozen package의
JS/WASM 파일 해시도 일치한다. 브라우저 초기화 응답 검증을 완료한 것으로 바꾸어 쓰지 않는다.
fmt·workspace all-target Clippy를 통과했다. 전체 nextest·나머지 최종 게이트는 아직 완료가 아니다.

전쪽98% 미달은5쪽에서4쪽으로 줄었으나 사용자 목표는 아직 충족하지 못했다.
남은 수식·간격과 다른 기존 문서를 계속 보정하며 PR 제출은 보류한다.
