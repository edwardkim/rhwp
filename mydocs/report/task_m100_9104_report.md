# #9104 Native 대체 진단·말미 공백 보완 결과

## 기존 수용 상태와 이번 범위

#9105·#9127은 다른 collaborator가 #9143으로 수용해 CLOSED이며 #9103도 CLOSED다.
#9104는 실제 `nativeFontSupply` 대체 사실 기록이 남아 OPEN이다. 이미 수용된 가져오기·저장과
설치 CJK fallback을 재적용하지 않고 잔여 진단과 기존 로컬 Native 보완을 통합했다.

기준 devel: `4f44355f5ff234abd1289d340cb4a3fe13297982`.
제품 source: `1da7afbd64bbc59d9338a187ebcd21860ddc0842`.
최종 Rust 검사 head: `13bb8191967f7e2efc9c182deaf0e7e075576205`.

## 구현

실제 Native `for_text`의 선택 후보와 최종 Font에서 요청/해결 family·출처·요청 스타일을
관측한다. CLI `export-png --json`의 `nativeFontSupply.localFallbacks`에 상태·범위·records·
missingRequests·누락 수와 상한을 기록하며 실제 일반 대체의 요청 이름은 genericFallbacks에도
합친다. 준비 단계의 공급 계획과 실제 선택을 구분한다. 관측 때문에 공급 mappings나
FontEnvironment, 원본/명시 공급 우선순위를 변경하지 않는다.

관측은 최대 256개 항목·이름 합계 64 KiB이며 본문 문자는 보관하지 않는다.
metadata-only는 notRendered, 상한 초과 또는 페이지 실패는 partial이다. 명령에서 실제로 그린
쪽의 일반 텍스트 face 선택이 범위이며 이미지 내부 SVG/메타파일의 별도 선택까지 주장하지 않는다.
새 문자열도 기존 nativeFontSupply 전체의 불신 데이터 경계로 처리한다.

말미 공백은 공통 VisibleTextLineMeasure와 실제 TextRun 전진 배열로 처리한다.
가시 본문 원점과 작성된 공백·장식을 보존하면서 남은 줄 예산을 소비한다. 독립 스타일 입력란·
공백 전용 줄·음영/테두리·탭/개체·원고지 계약을 유지한다. 문서별 수치나 Paint trim을 추가하지 않았다.

## 실제 검증

- 최소 공개 글꼴 전체 회귀 **13,710 PASS**, 0 FAIL, 45 skip.
- Native 공식 3종 **4,073 PASS**, 실제 진단 집중 **12 PASS**.
- fmt·Native/WASM32/workspace Clippy·workspace build·suite/unit 정책 전부 PASS.
- Native/fresh WASM 각 **1,771쪽**, 보호 **3,542행 하락·누락 0**.
- 정확한 devel과의 전쪽 비교도 하락 0이며 **3,536쌍 PNG 동일**. 변경 6쌍은 중첩 표
  10쪽과 두 형식 편람 60쪽의 실제 출력이며 확대 대조했다. 중첩 표 10쪽은 양쪽 raw 100.00%다.
- 새 devel의 #9022·#9105·#9107·#9109·#9127·#9137·#9141은 66개 문서·74쪽에서
  양쪽 backend 각각 **전쪽 PNG 동일**이며 관련 동작 검사 38건도 최종 source에서 통과했다.
  [기능 보존 표](../working/assets/issue9104/final/incoming-feature-preservation.json)를 남긴다.
- 실제 편람 Native HWPX **383쪽 gate PASS**, 새 입력 보안 3종 clean.
- 오프라인 및 실제 다운로드 실패에서 **10문단 표시·8개 실제 대체 기록**. 일반/굵게/기울임
  요청과 진단 on/off PNG 동일, metadata-only 미출력·캐시 재생·상한·미지원도 확인했다.
- 기존 devel이 실패하던 본문 압축 관계 검사를 후보와 최종 전체 회귀에서 통과했다.
  검사 참조 57개를 점검해 기존 올바른 관계/완료 페이지 핀을 유지하고 새 관계 검사 4건을 추가했다.

[최종 증거 원장](../working/assets/issue9104/final/README.md),
[전체 실행](../working/assets/issue9104/final/final-gates.json),
[전쪽 시각 검증](../working/assets/issue9104/final/visual-validation.json),
[devel 대조](../working/assets/issue9104/final/devel-whole-comparison.json)를 연결한다.
기존 Canvas 회색 서식 영역의 색 차이는 devel과 같은 영역의 픽셀이 같음을 확인했고 직접 판독에
기록했다. 실루엣 100%를 전체 색상·픽셀 동일로 해석하지 않는다.

중간 기준 WASM이 공유 cache의 후보 산출물을 재사용한 것은 증거에서 제외했다.
실제 root 라이브러리를 재컴파일한 뒤 수정 전 본문 압축을 확인한 정확한 base로 전쪽을 다시
출력했다. 임시 글꼴 프로그램 11개도 원본에서 바이트 단위로 재구성되는 절차를 검증했다.
글꼴·SVG의 내장 글꼴·WASM 바이너리는 이 증거에 배포하지 않았다.

## 다음 단계

중첩 표 10쪽의 Native/WASM 두 행을 99.86%에서 100.00%로 상향했다. 다른 3,540행과
34개 데이터셋의 출처는 유지했다. [갱신 영수증](../working/assets/issue9104/final/floor-update.json),
[새 정본과 실제 전쪽 동일](../working/assets/issue9104/final/updated-baseline-matches-actual.json),
[consumer·실제 편람 gate](../working/assets/issue9104/final/final-baseline-consumer-validation.json),
[Python 계약 267건](../working/assets/issue9104/final/baseline-python-contracts.json)이 모두 PASS다.
consumer는 표 해시 연결만 바뀌며 제품·Rust 검사 source는 같다.

## 원격 코드 검증 완료

사용자의 원격 처리 승인 뒤 후보 `6f99912b5013b3a82da9869f536e1488859d4f83`를 push하고
devel 대상 [Open PR #9147](https://github.com/edwardkim/rhwp/pull/9147)을 생성했습니다.
동일 code head의 Full CI·CodeQL·Render Diff·Adapter·Proptest·최종 정책 감사가 모두 성공했습니다.
원격 A–E 실제 로그는 13,516 PASS·0 FAIL·45 skip이며 로컬 결과와 구분합니다.
[CI 영수증](../pr/assets/pr9147/code-ci.json)과 [self-review](../pr/archives/pr_9147_review.md)를 남깁니다.

이 문서 전용 기록 head의 검사·최신 병합 시뮬레이션을 확인한 뒤 승인된 merge·#9104 완료
댓글/close·devel 동기화·종료 정리를 진행합니다. 실행 전인 후속 조치는 완료로 기록하지 않습니다.
