# Native 글꼴 보고의 출처 선언 보정

## 기존 계약과 원인

- #3787 S1(원 구현 `0d63fb935b`)의 출처 표지 계약은 JSON 봉투의 문서 파생 문자열을 단일 출처 지도에서 선언하도록 정한다. 실제 본문 토큰으로 모든 JSON 명령을 실행하는 `tests/provenance_contract.rs`는 선언 누락을 검출하며 완화하지 않는다. 원래 `src/provenance.rs`는 현재 `crates/rhwp-contracts/src/provenance.rs`로 이동했다.
- #7903(통합 `116b2b6ae0`)의 PNG 글꼴 관측과 #8565(통합 `5551091085418c17d8321c0fee2626365911481b`)의 Native 웹폰트 공급은 실제 JSON에 글꼴명·대체 매핑을 싣는다. SVG·PDF·PNG의 MAP 항목은 문서 문자열이 없다는 이전 선언을 유지했다.
- 새 base `1924d371d6` 위 후보 `ef6d302937`의 최소 글꼴 stock 전체 nextest는 12,475건 중 12,474 PASS / 1 FAIL / 45 skip였다. 실패는 SVG·PDF의 `nativeFontSupply.unavailable[]`에 문서 문자열이 실리지만 untrustedContent=false, 경로 선언 누락인 네 진단이다. Native feature에서만 실행되는 PNG도 같은 공급 결과와 글꼴 환경/대체 관측을 낸다.

## 보정 범위

공통 출처 지도에 SVG·PDF의 nativeFontSupply/fontEnvironment, PNG의 같은 필드와 observedFontFallbacks/declaredFontEnvironment를 선언한다. 객체 키에도 문서 글꼴명이 실리므로 해당 보고·환경 객체 전체를 문서 파생 데이터로 취급한다. 기존 marked의 실제 필드 존재 확인은 유지한다. 존재하지 않거나 null/빈 값인 필드는 표지에 추가하지 않는다. 검사·허용치·schemaVersion·출력 내용·폰트 선택·레이아웃·paint·저장 알고리즘은 바꾸지 않는다.

기존 출처 계약 11건의 기본/Native 실행, Rust lint·workspace·stock 전체 회귀를 새 코드에서 확인하며 아직 보정 후 완료 판정은 하지 않았다. Native 32입력/편람383 출력의 producer는 ef6이며 이 메타데이터 선언 보정의 재실행 결과로 바꾸어 적지 않는다. 실제 렌더링 source 동일성을 확인해 재사용 범위를 명시한다. 최적화 fresh WASM과 실제 API·저장 검증은 새 보정 head에서 이어 실행한다.

공통 contracts 라이브러리 기존 단위 검사 15건과 fmt check는 보정 후 통과했다. 전체 명령 봉투의 실제 출처 계약과 Native PNG 경로는 새 code head에서 이어 검증한다.

## 소비 계약의 동기화

#5295의 원 구현 `ee5bb1c0af74b0ec6314d276910793d44e50e87f`은 오프라인 소비 fixture와 실제 출처 지도가 같은 경로를 선언하는지 검사한다. `63532b1ee5`에서 실제 JSON 계약 11건과 Rust lint·workspace build는 통과했지만, stock 전체 회귀는 12,475건 중 12,473 PASS / 2 FAIL / 45 skip였다. 두 실패는 소비용 fixture의 이전 빈 선언과 새 지도 사이의 불일치다.

`.claude/skills/rhwp-provenance/fixtures/command-untrusted-fields.json`의 SVG·PDF 2경로/PNG 4경로와 해당 소비 문서·공통 기술 문서를 실제 지도에 동기화한다. 원 검사 assertion은 유지한다. 제품 Rust 코드는 `63532b1ee5`와 같고, 소비 fixture와 문서만 바뀐다. 집중 소비 계약과 stock 전체 회귀를 다시 실행하고, Native·최적화 WASM의 남은 필수 검증을 이어 진행한다.
