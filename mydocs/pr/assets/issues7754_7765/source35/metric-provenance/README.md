# 표시용 원 glyph와 저장 HFT 전진폭 계약

## 독립 입력과 실제 반례

`tests/fixtures/fonts/RHWPHftOutline.otf`는 저장소의 MIT 독창 사각형이다.
독립 한컴 Print나 실제 배치 정답지가 아닌 font-only 계약 입력이다.
원 cmap의 U+AC00을 메모리에서 U+F53A로 옮기고 hmtx 900/1000 UPM을 보존했다.
20px 원 HFT 한양신명조의 전진은 20px이며, 표시 프로그램의 자연 폭은 디바이스
양자화 후 18.026666666666667px다. 두 값은 각기 다른 계약이다.

검증된 동결 라이브러리로 직접 실행한 `source33-result.json`과
`source35-result.json`은 표시용 CFF의 공급만으로 소스 35에서 20px가 18.0267px로
바뀌는 오류를 입증한다. 같은 코드의 진단에서 소스 33은 표시 폭을 유지하지만
명시적 자연 메트릭을 아직 소비하지 못한다. 제품 소스 36은 원 HFT·자연 메트릭
요청 여부로 측정 분기만 제한한다. glyph paint의 원 자형 공급은 유지한다.
이 문서를 작성하는 시점의 소스 36은 아직 빌드·재측정 전이다.

원인 연결은 `NativePuaFonts::register_otf_for_source`의 원 face 가시 glyph/hmtx
수집 → `NativeOtfMetrics::compatibility_cluster_advance`의 공급 순서 판정 →
`pua_glyph::source_program_glyph` → `EmbeddedTextMeasurer`의 old glyph 폭 소비다.
Native와 Canvas paint도 같은 선택을 사용한다. 측정에서만 원 HFT 폭의 소유를
보존해야 한다. 프로그램 신원이 선택됐다는 사실만으로 표시 대체를 원 폭으로
승격하지 않는다. TTF → 일치 TTC → OTF의 원래 공급 순서는 바꾸지 않는다.

새 `issue_7754_split_hft_program_contract.rs`는 원 HFT 폭 보존·명시적 자연 메트릭,
앞 프로그램의 원 자모 소유 유지, 뒷 분할 CFF 원 glyph의 실제 Native 래스터를
검사한다. 래스터는 같은 사각형을 U+AC00로 공급한 대조군과 비교하며 실제 문서의
절대 좌표나 해시를 고정하지 않는다. 시스템 또는 비공개 글꼴에 의존하지 않는다.
기존 검사 탐색은 `pua_oldhangul`, composer의 원 자모 보존 및 HFT outline 계약을
확인했다. 원문 주소 보존 검사는 유지하고 같은 검사를 중복 추가하지 않는다.

`before-contract-tests.json`의 실제 실행에서 소스 33은 1 PASS/2 FAIL이고,
소스 35는 2 PASS/1 FAIL이다. 소스 33의 Native paint 실패는 소스 35에서 통과하고,
소스 35의 HFT 표시 폭 실패는 소스 36에서 보정할 대상이다. 소스 36의 통과로
미리 기록하지 않는다. 이 실행은 고정 rlib를 직접 링크한 반례 재현이며 최종
정식 suite·전체 Rust 검증을 대체하지 않는다.

## 실제 문서의 별도 시각 증거

소스 35 `3045dc9768ffe0be3f1329ab55d33bc5b48527d2`에서 원본
`samples/HWP5-nopassword-123456.hwpx`와 독립 Print
`pdf/HWP5-nopassword-123456-hwpx-2024.pdf`를 2026-11-22에 96dpi,
색상 경계 보정 전 2px 내용 실루엣으로 전 24쪽 비교했다.
Native 최저 99.50%, fresh WASM 최저 99.48%이며 누락이 없다.
1쪽 99.81%/99.72%, 같은 옛한글 동작의 우선 후보인 2쪽 99.99%/99.99%,
5쪽 99.98%/99.98%다. 전쪽 TSV·입력/PDF 해시·생산물 신원·명령은
`../native-focused/`, `../wasm-focused/`, `../gates/`에 보존했다.
직접 확대 판독은 `../direct-review/`의 네 패널과 JSON에 기록했다.
#8399 A·D·E HWP/HWPX 6개 어울림 대조군은 양쪽 모두 100.00%다.
이는 위 합성 계약의 시각 측정값이 아니며, 보정 후 코드의 재측정을 면제하지 않는다.

초기 보조 라이브러리 복사에서 오래된 최상위 rlib를 선택한 오류는 실행 실패로
확인했고, 실제 사용한 `release-test/deps/librhwp.rlib`로 고쳐 재실행했다.
원 Native CLI와 fresh WASM 생산물·실제 문서 수치는 잘못된 보조 복사본을 사용하지 않았다.
검증에 성공한 라이브러리의 해시는 `builds-corrected-libraries.json`에서 추적한다.

## 소스 36의 보정 후 실행과 정식 Native 연결

소스 36 최적화 Native 라이브러리에서 위 세 합성 계약은 3 PASS/0 FAIL이다.
정식 근거는
`mydocs/pr/assets/issues7754_7765/source36/gates/split-hft-contract-after.json`에 있다.
기본 suite에서 Native feature가 꺼진 채 수집되는 것만으로 paint 검사를 실행한 것으로
보지 않았다. 세 검사 전체를 기존 Native CI 호출 대상인
`tests/cases/issue_8153_print_glyph_geometry.rs::split_hft_original_programs`로 이관했다.
assertion 본문은 동일하며, 기존 함수·기대값은 유지했다. 별도 workflow 호출은 추가하지 않았다.
이관 경로와 동등성 및 최종 정식 실행의 대기 상태는 `source36/gates/regression-native-routing.json`에 있다.

추가 실물 검사는 같은 원본을 실제 CLI로 출력하고 1쪽의 원 문단 여백·내용 순서와
2쪽의 옛한글·후속 글자 원점을 검사한다. 빈 명시 환경으로 원 HFT 메트릭을 유지하므로
설치 글꼴이나 네트워크에 의존하지 않는다. 소스 33에서는 여백 검사가 206.7px 대
160.1px로 실패했고, 소스 36에서는 두 검사 모두 통과했다. 독립 좌표·전쪽 수치·판독
근거는 `mydocs/pr/assets/issues7754_7765/source36/`에 있다. DocumentCore를 추가로 직접
링크하려던 진단의 E0463은 현재 공유 dependency와 동결 rlib의 링크 준비 실패로
기록하며 테스트의 실패나 통과로 계산하지 않았다. 정식 Cargo 검증은 별도로 수행한다.
