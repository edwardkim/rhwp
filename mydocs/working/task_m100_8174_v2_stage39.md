# #8174 v2-S39 — 글꼴 프로그램·레코드·CFB 공개 계약

Issue: #8174. 2026-10-26, 기준 `71a908e0f7`.
source unit 정책 보정의 다음 절편이며 생산 코드는 변경하지 않았다.

## 이관과 실행

`font_metrics_data`·`mini_cfb`·`record_writer`에 추가했던 source unit 4개를 제거하고
`tests/cases/issue_8174_font_program_and_wire.rs`의 공개 계약 4개를 실행해 통과했다.
원본 전체·SHA-256·source commit·Git blob·함수 목록은 ignored
`output/pr-review/issue8174/v2-s39-20261026/original/`과 `migration-inputs.json`에 보존했다.
세 파일의 `#[cfg(test)]` 이전 생산 부분은 바뀌지 않았으며 별도 JSON에 대조했다.

- 공개 Nimbus 별칭 세 개의 모든 Unicode cmap advance를 PDF에 실제 내장된 독립 CFF
  프로그램과 비교한다. 각 별칭에서 854개 이상 글리프와 em·공백/대시·결측 글리프를 확인했다.
  private provenance 포인터를 위한 새 지원 API를 만들지 않았다.
- 레코드 스트림은 독립 HWP bit 배치로 작은/확장 header와 payload 보존을 검사한다.
  public parser로 다시 읽어 tag·level·size·내용을 확인했다.
- CFB는 원래 검사와 같은 표준 stream 경로의 mini/일반 stream 내용과 root CLSID를 독립
  `cfb` reader로 확인한다. 내부 String uppercase 구현의 자기 비교를 공개 지원 계약으로
  올리지 않았다.
- 실제 public Session 첫 HWP 저장에서 최종 CFB backing을 예약할 수 없는 cap을 적용했다.
  거절한 요청량이 정상 산출물 길이와 같고 기존 live IR·실패 고정·새 요청 복구가 유지된다.
  private Storage 호출이나 가짜 1-byte 요청만으로 대신하지 않았다.

Native 집중 실행은 공유 target의 검증된 생산 API를 직접 연결했으며 위 source의 생산 부분은
이관 전후 동일하다. 최종 자동 suite와 새 전체 head의 실행은 T08에서 수행한다.
S38 대비 unit-tier 검사는 **4,136 tests / 304 modules**로 통과했다. cached devel 대비 전체
검사는 아직 실패한다. 기준선이나 기존 정상 테스트의 assertion을 완화하지 않았다.
최초 integration 소스의 format 인자 컴파일 오류는 바로잡고 실패 로그를 보존했다.

## 범위를 구분한 발견

탐색용으로 확대한 Unicode CFB 디렉터리는 reader의 name ordering 검사를 실패했다.
원래 cached devel writer를 별도 프로그램으로 실행해 **동일한 실패**를 확인했다.
이전 구현도 UTF-8 uppercase byte 길이로 정렬한다. 이관 대상의 원래 CFB 입력은 ASCII
stream 경로이며, 내부 uppercase String 동등 검사는 Unicode CFB 적합성 증거가 아니었다.
따라서 그 탐색 사례는 새로운 지원 계약으로 등록하지 않고 `unicode-directory-diagnostic.log`·
`baseline_probe.rs`·`baseline-probe.log`에 분리해 보존했다. 생산 코드의 Unicode 정렬을
수정했거나 임의 Unicode CFB를 지원 완료했다고 보고하지 않는다.

또한 cached devel 비교에서 증가로 표시되던 HWP3 sample16 검사는 이번 작업이 만든 검사가
아니다. `dcf64b4da05`에서 추가됐고, #8228 통합 `3e02fe2749`가 사용자 지정 제외에 따라
제거했으나 아직 이 branch에 rebase하지 않은 상태다. 해당 검사와 입력은 임의 이관·재검증
대상으로 삼지 않았다. T09의 base 동기화에서 그 제외와 새 devel 기능을 함께 보존한다.
원 branch base `9418ed0c7c` 대비 진단은 **4,136 > 4,101**, cached devel은 **4,136 > 4,100**이다.
이 차이를 감추려고 정책 검사의 실제 PR base를 바꾸지 않는다.

현재 source unit 잔여와 지원 항목, T04/T05의 남은 실행 경계 및 T06–T09는 계속 남는다.
