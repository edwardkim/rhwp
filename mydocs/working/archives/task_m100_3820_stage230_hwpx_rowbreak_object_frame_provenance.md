# Stage 230: HWPX RowBreak object frame provenance 제한

## 목적

Issue #3820의 HWPX RowBreak object-frame 보정을 유지하면서,
`59043_regulatory_analysis.hwp`가 한글 2022 기준 37쪽에서 35쪽으로 줄어든 회귀를
source provenance로 분리한다.

## 원인 확인

- 최신 `upstream/devel`에서 `issue_1921_59043_pagination_pin`은 5건 모두 통과한다.
- good=`upstream/devel`, bad=현재 누적 변경으로 실행 가능한 revision만 대상으로 이분
  탐색했다. 컴파일 불가 중간 revision은 `bisect skip`으로 제외했다.
- 최초 실행 가능 회귀는 `8a757a03b` (`fix: 저장 RowBreak object의 쪽 소유권을 보존한다`)다.
- Stage 223의 `saved_rowbreak_object_frame`은 HWPX `issue2006`의 단일 host LineSeg와
  declared object frame을 연결해 같은 physical page에 소유시키는 규칙이다.
- 그러나 조건에 실제 HWPX 컨테이너 provenance가 없어 원본 HWP인 59043도 같은 모양의
  RowBreak 표를 object frame으로 처리했다. HWP5 table host LineSeg는 HWPX object-frame
  좌표 계약이 아니므로 fragment를 앞쪽에 과도하게 배치했고, p11/p12 및 문서 말미가
  앞당겨져 35쪽이 됐다.

## 구현

- `saved_rowbreak_object_frame`의 첫 조건을 `st.profile.hwpx_container()`로 제한했다.
- `hwpx_stored_layout()`은 rhwp HWPX→HWP 변환 계보도 포함하므로 사용하지 않는다.
  container 전용 physical-frame 보정에는 실제 OWPML HWPX만 참인 `hwpx_container()`이
  정확한 계약이다.
- native HWP는 기존 RowBreak row scanner와 measured fragment geometry를 유지한다.
- 고정 px allowance, 문서명·페이지·행 번호 기반 예외는 추가하지 않았다.

## 검증 범위

- `issue_1921_59043_pagination_pin`: 한글 2022 37쪽 및 p8/p11/p12/p35-p36 소유권.
- `issue_2006_1790387_prep_pagination_pin`: HWP 2020 MCP 140쪽 object-frame 정합.
- #3820 집중 게이트: `issue_3820_rowbreak_rowspan_band`, `issue_3930_hwpx_hwp_save_layout`,
  `issue_1733`.
- 집중 게이트 뒤 전체 `--lib` 및 `--tests` 회귀를 별도로 수행한다.
