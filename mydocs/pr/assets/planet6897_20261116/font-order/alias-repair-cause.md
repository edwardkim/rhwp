# 한양중고딕의 TTF 설치 이름 누락

## 보정 전 실제 출력

제품 source `10d0d6ae43273430f74f7055c39b3d74948c1d7f`에서 검증 JSON을 지정하지 않고
중첩 표 원본 17쪽을 실제 Visual Sweep으로 출력했다. Native 최저 97.54%, fresh WASM
최저 97.75%이며 5쪽은 각각 99.01%, 99.06%다.
[전쪽 TSV와 실제 PNG](pr9096-visual/validation.json)는 명시적인 TTF 공급 조건과 구분한다.
이 결과는 새 코드의 결과가 아니다.

## 원인과 공통 수정

한양중고딕의 TTF 설치 파일 H2GTRM.TTF는 실제 name table에서
`HY중고딕`·`HYGothic-Medium`으로 등록된다. 원본 이름 `한양중고딕`을 조회하는
`installed_font_program_aliases`에는 이 대응이 빠져 있었다. 형식 정렬 함수 자체는
TTF를 우선해도 조회 후보에 그 설치 이름이 없으므로 기존 CFF 프로그램만 선택했다.

원본 글꼴의 설치 이름을 같은 공통 후보 목록에 연결한다. 설치 여부나 파일명만으로
바이트 공급을 인정하지 않으며, Native/Studio/WASM이 실제 로드한 프로그램의 기존 공통
TTF → TTC face → OpenType 선택을 소비하게 한다. 원본 TTF/TTC가 없으면 CFF를 유지한다.
다른 family의 임의 TTF를 먼저 선택하거나 글리프·원본·PDF·점수 문턱을 바꾸지 않는다.
문서별 ID·쪽수·좌표 분기를 추가하지 않는다.

`tests/cases/installed_font_program_order.rs`에 사설 글꼴 파일 없이 공급 snapshot으로
TTF 존재·TTC만 존재·CFF만 존재하는 세 관계를 검사한다. 실제 시각 검증의 보정 전 결과는
위 원본 전쪽 측정이며, 새 관계형 검사의 보정 전 실행은 stale 생성 harness 때문에 중단했다.
그 실행을 FAIL/PASS 근거로 쓰지 않는다. 보정 후 검사와 JSON 없는 Native/fresh WASM 재출력,
#9096 원 사례·정본 보호 검증을 새 제품 source에서 완료한 뒤 수용을 판정한다.

## 지침 반영

사용자 지시에 따라 보정 후 실제 Visual Sweep을 먼저 마친 뒤
`mydocs/manual/verification/visual_sweep_guide.md`의 글꼴 공급·고정 JSON 재사용 절차를 수정한다.
현재 단계에서 매뉴얼은 수정하지 않았다.
