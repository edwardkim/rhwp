# Stage127: e7a0dec55c 리베이스 검증과 제출 범위 정리

Issue: #7445. 실행 source는 `77f2c50e222c0f5530f8162b7664aee257e4187e`,
고정 base는 `e7a0dec55c6c6c6365e4ff50babe7583e1cf1c70`이다. #7445 전체 해결을
주장하지 않으며 새 HWP 보정 대상을 추가하지 않았다. PR #8191의 이전 원격 후보
`9712311c`와 이번 source의 검증을 구분한다.

## 제출 범위와 전쪽 결과

사용자 지시에 따라 `samples/hwp3-sample16-hwp5.hwpx`도 별도 PR 범위로 제외했다.
이미 제외한 같은 이름의 HWP 및 바이트가 같은 별칭과 함께 이번 제출 성과에 포함하지 않는다.
원본·PDF·일반 정상 검사·기존 생산 기록은 보존했다. 이는 미달 예외가 아니며 이번 후보의
해당 HWPX 64쪽도 이미 통과한 상태였다.

| 경로 | 제출 조건 | 독립 쪽수 | 최저 raw | 90% 미달 | 후보 쪽수 불일치 | devel 대비 2%p 이상 하락 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Native Skia | 49 | 1,772 | 93.06876% | 0 | 0 | 0 |
| fresh WASM Canvas | 49 | 1,772 | 92.99291% | 0 | 0 | 0 |

불변 50조건 생산은 base/head 각 backend에서 모두 완료했다. 그 전체 200 TSV를 감사한 뒤
별도 PR HWPX의 4 TSV를 제외한 **196 TSV·98대조 조건**으로 제출 집계를 분리했다.
실제 입력/PDF hash, 전쪽 순서와 PNG 쪽 집합의 누락·중복을 확인했다. 새 후보의 제출 범위
3,544 PNG는 모두 새 캡처이며 이전 후보 `1f54c3df`의 대응 PNG와 바이트가 같았다.
이를 이전 실행을 새 실행으로 바꿔 적는 근거로 사용하지 않았다.

base의 로드맵·low-slack·보도자료·court·tab-period 분할 표는 5조건×2 backend의 쪽수가
독립 PDF와 다르다. 미대응 쪽을 명시했고 같은 수치 쪽의 비교만으로 의미상 대응을 주장하지
않았다. 후보의 독립 쪽수·내용 소속/순서/보존 회귀와 대표 38쌍 판독 근거를 함께 확인했다.
이번 source의 7쌍을 다시 열었고 31쌍은 새 actual/reference PNG 각각의 정확한 바이트 동일성을
확인한 뒤 기존 판독을 재사용했다. TSV는 전쪽 새로 산출했다.

실제 제품 진입점은 Native 30·Canvas2D 30·지원 CanvasKit 12의 **72관측**이며 모두 raw 90%
이상이다. 실제 초기화 응답 JS/WASM, software surface·readiness·그림 준비와 소유를 확인했다.
글꼴/자원 미준비로 Canvas2D에 fallback한 10관측은 CanvasKit 통과에 포함하지 않았다.
제외 전 74관측 이력과 이번 72관측 집계를 구분했다.

canonical 기준은 PDFium 152.0.7947.0 / pypdfium2 5.12.1 smooth RGB v1, 96DPI,
raw 2px·90%와 devel 대비 2%p 경계다. Mac WASM은 locked wrapper의
`--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며 Docker 최적화 통과가 아니다.

## 현재 source의 전체 회귀와 필수 게이트

- 전체 release-test/native-skia nextest: **11,759 PASS / 0 FAIL / 45 skip**, 1,048.744초.
- prepare·manifest·unit tiers·fmt·Native/WASM/workspace/Skia all-target Clippy,
  workspace/release build·Skia lib·placeholder2·직접 PDF14·release lib·doc·Studio unit/TS 등
  **22단계 exit0**. source preflight는 PASS이며 publish=false인 내부 패키지를 게시 준비로
  해석하지 않았다.
- Studio 2,409 PASS·2skip, Studio/Chrome/Firefox build·VSCode compile,
  binding/font 11·실제 browser 10 계약 PASS. 패키징한 JS/WASM도 frozen 산출물과 대조했다.
- 변경한 trace 25 PASS·선택 비공개 POC1skip, workflow wiring 5 PASS, routing 95 PASS,
  actionlint exit0. 서로 중복되는 묶음을 고유 테스트 수로 합산하지 않았다.

## 새 base #8190의 기능 보존

코덱·자원 처리·CLI/WASM 진입점 503파일의 Git blob 목록이 base/head에서 같음을 확인했지만,
정적 동일성으로 실행 검증을 대신하지 않았다.

- Native JBIG2 단독 5/5·JPEG2000 단독 7/7·동시 사용 7/7을 실제 수집·실행했다.
- 선택 구성의 Native/WASM Clippy, 정확한 archive의 base Native 및 base/head 동시 코덱
  WASM 생산을 포함한 11단계가 순차 실행으로 모두 통과했다.
- 실제 WASM/Worker 4구성에서 각각 19입력을 확인했다. Native 6구성×19입력은 **114관측**으로,
  정상 56개는 정확한 base의 디코딩 RGBA와 같고 비지원/손상 58개는 crash가 아닌 깨끗한
  exit1·출력 미생성으로 확인했다. 자원 한도 초과 3건은 기존 출력 바이트를 보존하고
  `PDF_RESOURCE_LIMIT`를 반환했다.
- 기존 PDF 17개의 base/head·Native/WASM **68출력**은 모두 독립 1쪽과 일치하고 raw 90%
  이상이다. base Native 최저 100%, WASM 98.33874%, 후보 두 경로는 모두100%이며
  devel 대비 2%p 이상 하락은 없다. 기존 세 Ghostscript 독립 기준과 원본 PDF의 출처를 유지했다.
  새 HWP 보정 대상을 추가하거나 비교 문턱·예외를 바꾸지 않았다.
- 실행 스크립트가 Native에 JSON null을 FontEnvironment 구조체로 전달한 최초 34실패는
  원 로그에 보존했다. 인자를 바로잡아 실패한 Native 34건만 재실행했고 유효한 WASM 34건은
  실제 producer/hash를 확인해 유지했다. 해당 17개 가져오기 결과에는 비어 있지 않은 문자 run이
  없음을 확인했다. 제품 source나 검사 문턱을 바꿔 이 오류를 숨기지 않았다.
- 전체 17입력을 대표하는 28개의 서로 다른 actual/reference 쌍에서 review와 overlay를
  판독했다. 모든 38개 서로 다른 픽셀 쌍을 개별 판독했다고 주장하지 않는다. 알파·색상·마스크,
  반복 무늬 및 스캔 표의 내용·여백·경계를 확인했다. 얇은 선의 미세 위상·안티앨리어싱·엄격 RGB
  차이는 잔여이며 raw 100%와 구분한다.

대표 스캔 1쪽:

- Native [review](../pr/assets/issue7445/completed-20261009/rebase-e7a0dec55c-20261024/incoming8190/native-scan-jbig2-200-p001-review.png) · [overlay](../pr/assets/issue7445/completed-20261009/rebase-e7a0dec55c-20261024/incoming8190/native-scan-jbig2-200-p001-overlay.png)
- fresh WASM [review](../pr/assets/issue7445/completed-20261009/rebase-e7a0dec55c-20261024/incoming8190/wasm-scan-jbig2-200-p001-review.png) · [overlay](../pr/assets/issue7445/completed-20261009/rebase-e7a0dec55c-20261024/incoming8190/wasm-scan-jbig2-200-p001-overlay.png)

## 잔여와 다음 기준

sample11의 사용자 UI Print에도 존재하는 회색/갈색 면, 원본 sample16의 수식/dot 경계,
각 문서의 얇은 괘선·획·미세 간격은 기존 잔여다. 점수 통과를 RGB·원 글꼴·절대 좌표의
완전 동일성이나 모든 CanvasKit 입력 지원으로 확대하지 않는다.

제출 직전 upstream이 `3536e72819e165dbe959ee42cfd4e4cf8db33ce7`로 전진한 것을 확인했다.
추가된 #8180의 셀 진입/삭제 대상 및 #8182의 표시 문자열/모델 오프셋 분리를 이후 리베이스에서
보존·검증해야 한다. **이 문서는 e7 base/source 77의 완료 증거이며 그 새 HEAD의 통과나
GitHub CI·merge·후속 처리 완료를 주장하지 않는다.**

원 로그·전체 TSV/PNG·JSON·임시 실행기는 ignored
`output/pr-review/7445-completed-20261009/rebase-e7a0dec55c-20261024/`에 보존한다.
핵심 원장은 stage111,114,120,122,123,124,126이다. 이번 커밋에는 필요한 Markdown과
대표 PNG만 추가하며 사용자 초안·공유 target·원본·기준 PDF·설치 글꼴을 보존한다.
