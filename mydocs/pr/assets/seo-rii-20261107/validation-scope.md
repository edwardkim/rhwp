# 검증 source와 결과 재사용 범위

## 고정 source

| 구분 | SHA | 의미 |
| --- | --- | --- |
| 최신 devel | `b8dd67c5001dfb0410e9e192528676396cdd6a4e` | 비교 기준, 2026-11-07 원격 재조회에서도 같음 |
| 원 기여 누적 tip | `629f204f82e95c4f2f1a12d318698d1a01fe1cf7` | Ready 29개 head의 원 history를 포함 |
| SDK·검사 보정 | `e976fd93938781fc668da23cf4d1db35d52362fb` | Windows 공개 registrar, 변수 가림, 공개 fixture 글꼴 공급, Flutter 자동 migration |
| Metal 결정 | `c2648ffdfe983a00d9edc4150f4f5bfc8f59fb39` | default-disabled 실험 기반에 한정한 작업지시자 예외 |
| 최종 제품 보정 | `cda7c8cb33bb7ded970d033ed1543aabb49c63a2` | 원 수식 face 부재 시 공통 표시 코드의 의미 문자 복원 |
| 최종 회귀 source | `263b1e753c006d1631d25794588b6638fa1172ae` | 실제 producer의 4·3 회귀와 측정 증거; 제품 코드는 바로 위와 같음 |

마지막 단계에서 제품·검사 source를 다시 바꾸면 영향 범위의 검증을 갱신한다. 문서·TSV·PNG만 추가한 commit은 위 실행 source와 구분해 기록하며 새로운 실행인 것처럼 표시하지 않는다.

## 실제 실행과 재사용

| 검증 | 실제 source | 판정 범위 |
| --- | --- | --- |
| Linux Native·Qt GL/CPU, snapshot·lease·worker·접근성·저장·IME | 원 stack 및 SDK 보정 이후 source, 상세 실행 원장 참조 | 최종 숫자 보정은 `hancom_glyphs.rs`와 Native equation painter 두 파일뿐이다. host/FFI/Flutter protocol·quota·수명 코드는 같으므로 이 계약 결과를 재사용한다. |
| Flutter SDK analyze·70개 test, Linux C++ 13 cases/226 checks | SDK 보정 이후 source | 마지막 Native 내부 숫자 보정은 Dart/C++ source·공개 ABI를 바꾸지 않는다. 실제 producer 의미 경계는 별도 최종 Native 집중 회귀로 확인한다. |
| Windows SDK C++ compile/link·ABI lifetime | `e976fd9393` 이후 | protected registrar 접근과 `/W4 /WX` 변수 가림을 보정한 실제 Windows SDK 결과다. Linux stub 성공을 Windows 증거로 쓰지 않는다. |
| Windows Native D3D compile/link·4종 ABI·2종 하드웨어 D3D | `263b1e753c` | 최종 제품 보정이 포함된 DLL을 실제 MSVC로 다시 빌드해 실행했다. [최종 원장](platform-execution/windows-final-correction.json)을 따른다. |
| Native 전 24쪽·fresh WASM Canvas 전 24쪽 | `cda7c8cb33` | 실제 최적화 산출물과 독립 Print를 사용했다. 최종 회귀 commit은 제품 코드를 바꾸지 않는다. |
| devel/후보 PNG 대조 | `b8dd67c500` / `cda7c8cb33` | 같은 입력·글꼴·96 DPI·Print/compat 2022의 48쌍이 바이트 단위로 같다. [전쪽 비교](visual/base-candidate-comparison.json)와 양쪽 TSV를 보존한다. |
| Native 편람 383쪽 | `cda7c8cb33` | 최저 raw 99.96%, 기준 대비 회귀 0쪽. [최종 gate](platform-execution/handbook-native-final-correction.json) |
| 실제 Direct PDF radial 6쪽 | `cda7c8cb33` | base의 non-linear gradient 거부와 후보의 vector shading 출력 차이를 검증했다. 전쪽 raw 100%, shading 6개·image XObject 0개. [실행 근거](direct-pdf/README.md) |
| 실제 producer 회귀의 음성 대조 | 검사 `263b1e753c`, 제품 두 파일만 `c2648ffdfe` | 기존 5개 검사는 통과하고 신규 실제 숫자 검사가 missing-ink assertion에서 실패했다. 두 제품 파일은 최종 보정으로 복구했다. |

## 실행 수집기의 오류와 실제 결과 구분

- 테스트 source의 크기가 바뀌면서 현재 suite 배정이 달라졌다. 오래된 ignored harness로 선택한 첫 실행은 검사 0개여서 nextest가 거부했다. 현재 manifest로 harness를 준비하고 48/48 수집 경로를 확인한 뒤 실제 음성 대조를 실행했다. 0개 실행을 실패 재현이나 성공으로 세지 않는다.
- 음성 대조 수집기는 처음에 의미 불일치 assertion만 예상했지만, 실제 새 검사는 그보다 앞선 missing-ink assertion에서 실패했다. 실행 로그와 5 pass/1 fail을 확인해 수집 기록을 정정했다. 검사 assertion·기대값·제품 코드를 통과시키려고 변경하지 않았다.
- Windows GPU driver는 JSON 앞에 `GPU_PROBE_RESPONSE` 표식을 출력한다. 전체 stdout을 JSON으로 읽은 첫 수집 오류는 실제 compile/runtime 실패가 아니다. 이미 빌드한 DLL을 유지하고 표식 뒤 응답을 읽어 4종 ABI·2종 하드웨어 검사의 최종 결과를 기록했다.

## 지원 완료로 확대하지 않는 경계

- Linux GL/Vulkan의 소프트웨어 장치는 물리 GPU 성능·전력·VRAM 검증이 아니다.
- Windows D3D12 producer의 opaque/alpha/stale 실행 성공은 최종 Flutter D3D11 consumer sampling·지속 표시 qualification을 뜻하지 않는다. 기존 bounded immutable/quarantine 예산과 `continuousPresentationSupported:false`를 유지한다.
- Metal은 작업지시자가 승인한 default-disabled 실험 기반이다. Apple SDK compile/link·실물 Metal은 미검증이며 이를 지원 완료로 기록하지 않는다.
- P83은 명시한 bounded HWP5 upright 경로다. HWPX·mixed·sideways·일반 bidi 지원으로 확장해 설명하지 않는다.
- 정적 Direct PDF preflight는 공개 Rust API의 bounded 정적 진단이다. 동적 실행·모든 문서의 무손실 편집을 보장하지 않는다.

## 직접 판독에서 남은 기존 대조군 차이

`eq-01.hwp`의 본문 굵기·수식 간격/분수 모양과 `pic-crop-01.hwp`의 가는 테두리는 독립 Print와 차이가 보인다. 이 입력들은 원 숫자 보정의 새 결함이 아니며 같은 base/후보 PNG가 모두 같다. Native/fresh WASM 전쪽이 90% 이상이라는 사실로 차이를 면제하거나 해결 완료로 기록하지 않는다. 원격 수용 단계에서 이 기존 대조군 차이에 한정한 예외 여부를 별도로 명시한다. 현재는 그 예외 승인을 받지 않았다.

## 원격 처리 전 필수 결정

원 기여 tip의 history에는 Draft #8646의 P73 두 commit도 포함돼 있다. Draft를 별도 작업 대상으로 삼거나 상태를 수정하지 않았지만, 같은 history를 merge하면 GitHub가 그 PR을 간접 MERGED로 표시할 수 있다. 이 효과를 포함한 원격 범위 승인을 받은 뒤에만 통합 branch push·Open PR·CI·merge·후속 처리를 한다. #536·#8701 전체 로드맵은 OPEN을 유지한다.

## 최종 완료와 주석 보완의 동등성

검증 source `263b1e753c`에서 [17개 필수 묶음](local-gates.json)이 모두 통과했다. 전체 nextest 12,825 pass / 45 skip, Native 집중 194 pass, Native 공식 3종 4,077 pass(lib 4,040 + placeholder 2 + Direct PDF 35, lib ignore 12), doctest 8 pass / 3 ignore, Studio 2,745 pass / 2 skip이다. 각 명령의 범위가 겹치므로 고유 검사 수로 합산하지 않는다.

검증 종료 후 11개 API·수명·합성 래스터 테스트 파일의 첫머리 module documentation에 검사 유형·정확한 갱신 시각·직전 파일 commit·비해당 이유·정식 근거를 보완했다. [파일별 body hash](test-header-metadata-receipt.json)를 실제 Git source와 다시 대조해 첫머리를 제외한 파일 본문이 바이트 단위로 같고 제품·assertion·fixture 변경이 없음을 확인했다. 기존 실행 source를 새로 실행한 것처럼 표시하지 않는다. 보완 후 fmt·diff·현재 자동 수집 48/48과 변경 문서 검사를 통과했다.

## 원격 승인·실행 확정

[통합 PR #8739](https://github.com/edwardkim/rhwp/pull/8739)의 head `0d2548fd88ea76b10e53eec841376b4bcbecbc4a`에서 필수 CI·self-review를 완료했고, 원 기여 이력을 보존하는 merge `f5f0fbee95b5fb0e32e99f8f639d228ae55389dd`로 수용했다. 작업지시자는 기존 eq-01 본문 굵기·수식 간격/분수 모양 및 picture-crop 가는 테두리 차이에 한정한 예외와 Draft #8646의 간접 MERGED 효과를 승인했다. 새 숫자 오류·회귀·필수 검사 실패는 예외에 포함하지 않았다. #536·#8701은 OPEN을 유지하며 #8337 완료·원 PR별 후속 댓글·정리는 정본 순서에 따라 이어 간다.

위의 승인 대기·미수행 표현은 당시 로컬 단계의 기록이며 현재 상태는 [최종 운영 기록](remote/completion.md)을 따른다.
