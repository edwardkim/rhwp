# Stage137: PR #8191의 3536 리베이스 검증과 후속 범위

Issue: #7445. 검증 code head는 `8ba0980feb5995dffbfdb84601e53085b49f810d`,
고정 base는 `3536e72819e165dbe959ee42cfd4e4cf8db33ce7`이다. 이전
[Stage127](task_m100_7445_rebase_repair_stage127.md)의 source77/e7 결과와 구분한다.
기존 401개 패치는 range-diff에서 모두 보존됐고 자동 병합 tree와 리베이스 tree가 같다.
같은 PR 원격 브랜치에 exact lease로 반영하여 GitHub의 충돌 해소를 확인했다.

## 범위와 전쪽 비교의 생산 출처

새 HWP 보정 대상을 추가하지 않았다. 별도 PR의 sample16 HWP/HWPX 및 바이트가 같은
2010 별칭은 제외하고, 서로 다른 2022 저장본과 HWP3 원본은 유지했다.
제출 범위는 backend당 **49조건·1,772쪽**이다. 조건 수를 해결한 이슈 수로 세지 않는다.

| 경로 | 최저 raw | 90% 미달 | 후보 쪽수 불일치 | 고정 devel 대비 2%p 이상 하락 |
| --- | ---: | ---: | ---: | ---: |
| Native Skia | 93.06876% | 0 | 0 | 0 |
| fresh WASM Canvas | 92.99291% | 0 | 0 | 0 |

전쪽 raster의 실제 producer는 **source77/base e7**이다. 이번에 같은 입력/PDF·글꼴·정본
도구를 감사하고 **196 TSV를 전쪽 다시 산출**했다. 새 code head의 전쪽 재캡처라고 쓰지 않는다.
API 변경은 읽기 전용 JSON getter와 그 설명, Studio 변경은 셀 명령/선택 경로다. JSON getter
밖의 쿼리 코드와 렌더링·파서·글꼴·캡처 입력의 동일성, 196조건의 원래 actual/reference
**7,078쌍** 바이트를 확인했다. 원래 producer/build/font hash를 유지했고 TSV의 쪽 순서·값도 같다.
실제 새 head의 72관측이 이전에 판독한 대응 PNG 쌍과 모두 같은 점도 별도로 확인했다.

base의 5조건×2경로는 독립 PDF와 쪽수가 다르다. 미대응 쪽을 숨기지 않았고 수치상 같은 쪽을
같은 내용이라고 간주하지 않았다. 후보의 전체 쪽수·내용 소속·순서·누락/중복 검사와 기존 대표
38쌍 판독의 출처를 유지했다. canonical은 PDFium152.0.7947.0/pypdfium2 5.12.1,
smooth RGB v1·96DPI·raw 2px/90%·devel 하락 2%p이다. 예외·마스크·문턱 완화는 없다.

## 현재 head의 실제 소비와 전체 회귀

| 실제 소비 | 관측 | 최저 raw | 90% 미달 |
| --- | ---: | ---: | ---: |
| Native CLI PNG | 30 | 93.13188% | 0 |
| Studio Canvas2D | 30 | 93.13770% | 0 |
| 지원 Studio CanvasKit | 12 | 99.87307% | 0 |

72관측의 actual/reference 쌍은 새 head에서 생산하고 원래 쌍과 각각 SHA를 대조했다. 동일
바이트를 확인한 뒤 기존 판독을 재사용했으며 새로 전부 열었다고 하지 않는다. 실제 browser
초기화 **52관측**의 JS/WASM 응답을 확인했다. Vite JS는 허용된 URL 변환과 원본 source-map,
WASM은 원시 hash를 대조했다. 자원/글꼴 준비 미달로 Canvas2D에 fallback한 10관측은
CanvasKit 통과가 아니다. backend·software surface·readiness·그림 준비 guard를 유지했다.

- release-test/native-skia 전체 nextest: **11,766 PASS / 0 FAIL / 45 skip**, 실제 검사
  1,035.984초. 명령 전체 1,705.27초에는 컴파일도 포함된다.
- prepare·manifest·unit tiers·fmt·Native/WASM/workspace/Skia all-target Clippy,
  workspace/release build·Skia lib·placeholder·직접 PDF·release lib·doc·Studio unit/TS 등
  **22단계 exit0**.
- Skia root lib 3,859 PASS/12ignore와 다른 libs182 PASS, placeholder2·직접 PDF14 PASS.
  release root lib3,802 PASS/12ignore와 다른 libs182 PASS, doc8 PASS/3ignore.
- Studio/editor/shared PDF **2,421 PASS / 0 FAIL / 2skip**. 현재 head의 Studio·Chrome·Firefox
  build와 VSCode compile, 글꼴/바인딩11·실제 browser 글꼴10 계약도 모두 PASS이다.
- 각 묶음에는 중복 검사가 있으므로 고유 검사 수로 합산하지 않는다. 선택 POC 등 과거 미실행을
  새 실행으로 바꾸어 쓰지 않는다. source preflight의 source_check PASS와 publish=false인
  내부 패키지의 배포 준비도 구분한다.

Mac WASM은 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.
Docker 최적화 통과가 아니다. 검증 중 root/pkg·Studio/public·frozen artifact의 hash가 같았다.
후보 JS `f90e8e9655657ea0f23b467ccb89b37b6af24f8f1b387dab9055dae747aea5b9`,
WASM `69aced2fc42726a35d62182ef7931cf4254bfcd8a388f72f3496eb1d5653ddc9`를 사용했다.
검증 종료 뒤 자동 생성된 public JS의 JSDoc 4줄만 작업 tree에서 복원했다. 주석을 제외한
바이트 동일성과 frozen 원본을 보존했으며, 정리 뒤 root/public JS hash까지 같다고 하지 않는다.

## 반영한 #8180·#8182의 실제 기능 보존

정확한 base archive SHA는
`b7c9aa76aa0cb93a680ec2414535c6d6849d547be2c2ae5a98ce038003f6f950`이다.
base Native/fresh WASM은 동일 공유 target에서 순차 빌드하고 후보 pkg/public을 바꾸지 않았다.
base Studio도 그 archive의 코드와 pkg를 사용하고 동일 locked 의존성만 연결했다.

- #8180: **base/head 각각 셀 편집 6진입 경로**, 중첩/텍스트상자 및 이웃 소속 보존,
  undo/redo와 잘못된 대상9건의 snapshot/mutation 전 거부를 실제 Studio에서 확인했다.
- #8182: 현재 전체 nextest에 포함된 Native7건이 통과했다. base/head 각각 직접 WASM
  **5계약 묶음**과 실제 Studio dev getter4입력을 확인했다. 모델 text/charX와 표시
  displayText/displayCharX를 구분하고 모델 오프셋·선택/삽입·SVG가 유지됨을 대조했다.
- 두 source의 실제 초기화 응답 hash도 각각 검증했다. API 계약 검사를 문서의 전쪽 Canvas
  시각 검증으로 승격하지 않았다. 같은 입력의 표시/모델 계약은 exact base와 같다.

앞선 #8190 코덱의 단독/동시 구성·114 Native관측·68 시각 출력은 Stage127의 source77 생산
근거다. 이번에는 그 code 경로가 바뀌지 않았음을 확인하고 보존했다. 모든 코덱 matrix를8ba에서
다시 실행했다고 하지 않는다. 현재 기본 WASM/Worker 계약은 별도로 PASS이다.

## 추가 직접 판독과 이슈 종료 범위

후속 이슈 판단을 위해 기존 producer77 PNG의 아래 Native/WASM **10쌍**에서 현재 정본
도구로 review·standalone overlay를 만들고 모두 다시 열었다. 새 raster 캡처는 아니다.

| 대상·쪽 | Native review/overlay | WASM review/overlay |
| --- | --- | --- |
| regulatory17 13쪽 | [review](../pr/assets/issue7445/completed-20261009/rebase-3536e72819-20261024/followup/native-regulatory17-p013-review.png) · [overlay](../pr/assets/issue7445/completed-20261009/rebase-3536e72819-20261024/followup/native-regulatory17-p013-overlay.png) | [review](../pr/assets/issue7445/completed-20261009/rebase-3536e72819-20261024/followup/wasm-regulatory17-p013-review.png) · [overlay](../pr/assets/issue7445/completed-20261009/rebase-3536e72819-20261024/followup/wasm-regulatory17-p013-overlay.png) |
| asset60 41쪽 | [review](../pr/assets/issue7445/completed-20261009/rebase-3536e72819-20261024/followup/native-asset60-p041-review.png) · [overlay](../pr/assets/issue7445/completed-20261009/rebase-3536e72819-20261024/followup/native-asset60-p041-overlay.png) | [review](../pr/assets/issue7445/completed-20261009/rebase-3536e72819-20261024/followup/wasm-asset60-p041-review.png) · [overlay](../pr/assets/issue7445/completed-20261009/rebase-3536e72819-20261024/followup/wasm-asset60-p041-overlay.png) |
| sample16-2022 24쪽 | [review](../pr/assets/issue7445/completed-20261009/rebase-3536e72819-20261024/followup/native-sample16-2022-p024-review.png) · [overlay](../pr/assets/issue7445/completed-20261009/rebase-3536e72819-20261024/followup/native-sample16-2022-p024-overlay.png) | [review](../pr/assets/issue7445/completed-20261009/rebase-3536e72819-20261024/followup/wasm-sample16-2022-p024-review.png) · [overlay](../pr/assets/issue7445/completed-20261009/rebase-3536e72819-20261024/followup/wasm-sample16-2022-p024-overlay.png) |
| press12 12쪽 | [review](../pr/assets/issue7445/completed-20261009/rebase-3536e72819-20261024/followup/native-press12-p012-review.png) · [overlay](../pr/assets/issue7445/completed-20261009/rebase-3536e72819-20261024/followup/native-press12-p012-overlay.png) | [review](../pr/assets/issue7445/completed-20261009/rebase-3536e72819-20261024/followup/wasm-press12-p012-review.png) · [overlay](../pr/assets/issue7445/completed-20261009/rebase-3536e72819-20261024/followup/wasm-press12-p012-overlay.png) |
| ktx003 1쪽 | [review](../pr/assets/issue7445/completed-20261009/rebase-3536e72819-20261024/followup/native-ktx003-p001-review.png) · [overlay](../pr/assets/issue7445/completed-20261009/rebase-3536e72819-20261024/followup/native-ktx003-p001-overlay.png) | [review](../pr/assets/issue7445/completed-20261009/rebase-3536e72819-20261024/followup/wasm-ktx003-p001-review.png) · [overlay](../pr/assets/issue7445/completed-20261009/rebase-3536e72819-20261024/followup/wasm-ktx003-p001-overlay.png) |

규제분석의 13쪽 표/비표시 안내문, 자산관리의 붙임4·직인·연락처, 2022 저장본의 구성은
시작 문장·복구시간 표, 보도자료의 1~7행·마지막 별첨/출처 소속을 대조했다. 이 범위의 내용
누락·소속 차이를 보지 못했다. 표 선 농도·글리프 획·미세 자간/기준선 차이는 남아 있으며
높은 raw를 RGB 완전 동일이나 원 글꼴의 동일성으로 해석하지 않는다.

**KTX-003의 노선도/소요시간/운임안내 제목 획·그림자 차이는 남아 있다.** raw99%대만으로
#7780을 완전 해결로 닫지 않는다. 새 원인을 추정해 보정 범위를 넓히지 않고 기존 이슈에서
부분 개선으로 추적한다. #7813도 원본 HWP/HWPX115쪽과 저장본197쪽의 시각 개선과,
보존된 scoped-cache/resumable API 회귀4건의 복구를 구분해 열린 상태로 남긴다.

현재 전쪽 결과와 실제 정식 회귀 통과를 대조한 종료 후보는 **#7716·#7717·#7718·#7719·#7720·
#7724·#7725·#7726·#7758·#7809**다. 기존 미달 이관과 현재 개선을 구분하며 병합 뒤 source/
merge SHA·전쪽 수치·회귀·시각/CI 링크를 각 이슈에 남긴 뒤 종료하고 #7445 해당 행을 취소선
처리한다. 대표 이슈와 원기록/연결은 보존한다. 이미 닫힌 #8060은 조각 높이 규칙의 해결이며
71352 전쪽 잔여는 #8100, 11쪽 넘침은 #8095에서 계속 추적한다.

## 원격 완료와 증적 보존

현재 exact code head의 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/40064292787),
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/40064292763),
[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/40064292747), Adapter·Proptest·Gym 계약의
완료·성공을 확인했다. Full Gym benchmark는 정책상 skip이며 전체 성능 통과로 세지 않는다.
이 결과 뒤 정식 self-review·오늘할일을 같은 PR의 문서 trailing으로 반영한다.
trailing head 검사·merge·이슈 close·cleanup은 이 기록 시점의 후속 조건이다.

현재 base 대비 새 Rust 회귀 함수는 **228개(integration224·unit4)**이며 현재 전체 nextest에서
모두 실제 PASS를 확인했다. 기존 검사 교체/이름 변경12개와 파일 이동3개는 새 검사 수에서
제외했다. 소스 `#[test]` 정의의 순증가218개와 새 검사 수를 구분하며 Python/JS 계약을 더하지 않는다.

원 로그·TSV·JSON·임시 실행기는 ignored
`output/pr-review/7445-completed-20261009/rebase-3536e72819-20261024/`에 있다.
원장은 stage128~136과 issue7445-followup이며 이번 보존은 필요한 Markdown/대표 PNG뿐이다.
사용자 초안·공유 target·원본·PDF·설치 글꼴·backup/stash는 보존했다.
