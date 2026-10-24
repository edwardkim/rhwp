# Stage79: Print 벡터 쪽의 화면 합성과 그림 표본 보존

Issue: #7445. 부모 HEAD는 `f1f52d57bdf5cb42b4cea2a2515a9985e2dc67b7`, base는
`d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`이다. 별도 PR 대상으로 제외한 sample16 HWP5와
동일 별칭은 다시 보정하지 않았다. 기존 50조건의 범위를 유지한다.

## 원인과 공통 소비 경로

Stage78은 실제 Print 종이의 장치 표본을 식별했지만, 그림이 없는 쪽에도 `crisp-edges`를 적용했다.
소수 CSS 크기로 합성할 때 최근접 표본 복제가 가는 괘선의 coverage를 바꿨다. 페이지 크기나
조판을 바꾸지 않고, 기존 PageRenderer의 레이어 요약에서 실제 그림 소유를 확인하도록 보정했다.

`PageRenderer의 그림·raw SVG·복합 쪽 배경 → hasRasterContent → CanvasView의 Print 필터`
경로를 연결했다. 벡터만 있는 Print 쪽은 일반 보간을 사용한다. 그림·raw SVG·복합 배경 또는
메타데이터를 확인할 수 없는 쪽은 기존 표본 소유 판정을 유지한다. 확인 불가를 그림 없음으로
간주하지 않는다. Screen, 원본 PageInfo, CSS·스크롤 크기와 낮은 해상도 예산은 유지했다.
문서명·쪽 번호·일치율 분기는 없다. Rust·글꼴·원본·PDF·시각 문턱은 변경하지 않았다.

## 회귀와 실제 검증

- 기존 refreshPages 관계 검사에 실제 PageRenderer 요약 파서를 연결했다. 벡터 Print의 실제
  1 FAIL을 복구했고, 복합 배경의 잘못된 그림 없음 판정도 실제 1 FAIL 후 복구했다.
  관련 cache·zoom·scroll 검사는 최종 71 PASS다. 기존 검사 안에서 조건을 확장했다.
- 최종 3개 source hash가 일치하는 상태에서 TypeScript exit 0, Studio 2,411건 중
  2,409 PASS / 0 FAIL / 2 skip, production build exit 0을 확인했다.
- 같은 source의 실제 독립 Print 31건은 Canvas2D 27건·지원 CanvasKit 4건이며,
  최저 93.15185%, 90% 미만 0건이다. backend와 초기화 JS/WASM 응답 hash도 확인했다.
- 확대·DPR 전환과 복원 20건의 최저는 90.00026%, 90% 미만 0건이다. 같은 source로 이미
  성공한 3개 입력의 15건은 producer·PNG·Canvas 감사 hash를 검증해 재사용했고, 관측기가
  실패했던 로드맵 1개 입력의 5건은 다시 캡처했다. 20건 전부를 새 캡처라고 주장하지 않는다.
- 저장 줄 결측 대조군의 zoom 1.25 / DPR 1.5는 이전 정책 86.78369%에서 현재 정책
  90.00026%로 개선됐다. Git 부모의 실제 메서드를 같은 완성 Canvas에 적용한 정책 대조이며,
  전체 base 앱을 별도로 빌드한 전후 비교는 아니다. 정책 전후 Canvas 바이트 불변을 감사했다.
- 편집용 caret은 실제 `CaretRenderer.hide()`로 숨기고 전후 모든 Canvas PNG hash의 일치를
  확인했다. 캡처 PNG를 편집하거나 비교 영역을 가리지 않았다. 이전 92픽셀 차이는 문서 내용이
  아닌 caret이었고, 관측 조건을 정정했다.

저장 줄 결측 대조군의 review/overlay에서 표·별표·본문 소속 보존을 직접 확인했다. 내부 가로선과
글자의 미세 래스터 차이는 남아 있다. sample11 원본 50쪽의 문단·상하선·쪽 번호와 도형 소속,
로드맵 153쪽의 산점·축·출처·쪽 번호도 대조했다. sample11 기준 Print에 있는 PC·라우터 아래
면과 그림 색상 차이를 이번 CSS 보정이 해결했다고 주장하지 않는다. 대표 그림과 수치의 범위를
전체 50조건의 전쪽 완료로 확대하지 않는다.

## 자체 오류와 정정

첫 관측기가 Canvas supplemental metrics가 활성화된 상태에서 portable 전용 overlay export를
직접 호출해 로드맵 캡처 1건이 실패했다. 제품의 정상 메타데이터/트리 fallback 경로를 관측하도록
고쳤으며 guard를 우회하거나 metrics 모드를 강제로 바꾸지 않았다. 정정 후 Print 31건과 로드맵
확대 5건을 완료했다. 실패 실행은 output에 보존하고 수용 증거에서 제외했다.

이전 정책의 TypeScript 메서드를 추출하는 첫 진단은 설치된 TypeScript 7의 native port를
기존 JS API로 취급해 실패했다. Node의 내장 타입 제거로 정확한 Git 메서드 본문을 추출하고
producer hash를 남긴 정정 실행만 사용했다. 최종 증적 수집 시 중복된 정상 JS 응답 2회를
단일 응답으로 잘못 가정한 감사도 경로별 응답 hash 검증으로 정정했다. 제품 source 변화는 없다.

## 전체 게이트와 잔여

부모 f1f52d57의 Rust source는 이 단계에서 그대로다. 해당 source의 전체 nextest는
11,752 PASS / 0 FAIL / 45 skip, 실행 918.516초이며 필수 게이트 18개 모두 exit 0이다.
Native Skia lib는 3,859 PASS / 12 ignore와 다른 lib 182 PASS, placeholder 2 PASS,
PDF 14 PASS다. release lib는 3,802 PASS / 12 ignore와 다른 lib 182 PASS,
doc test는 8 PASS / 3 ignore다. Native/WASM/workspace/Skia lint, build, manifest,
unit tier도 완료했다. fresh WASM은 locked wrapper의 `--target web --out-dir pkg --no-opt`
로컬 대체 빌드이며 Docker 최적화 통과가 아니다.

새 base의 Native와 fresh WASM 빌드도 각각 exit 0으로 완료했다. 구체적인 새 커밋에 연결한
50조건·backend당 기준 1,836쪽의 base/head Native/fresh WASM TSV와 내용 소속·누락·중복·
쪽수 및 대표 PNG 판독은 계속 필요하다. 기존 bitmap 대조군의 JPEG/장치 표본 차이 진단은
제품 보정 통과로 간주하지 않는다. 이 단계는 PR 제출 준비 완료가 아니다.

[최종 source·소비 검증, TSV와 대표 review/overlay](../pr/assets/issue7445/completed-20261009/rebase-d9fd85ed-20261023/vector-print-compositor/README.md).
실행 로그·임시 관측기·기각 후보는 ignored output에만 보존했다. 지침 파일은 변경하지 않았다.
