---
kind: evidence
status: active
canonical: mydocs/working/task_m100_7445_rebase_repair_stage5.md
last_verified: 2026-10-16
---

# 분배 정렬의 마지막 글리프 점유 폭 보정

Issue: #7445. 이 기록은 마지막 자간 보정과 아래 두 문서의 검증 범위다.
브랜치 전체의 PR 준비 완료나 전체 회귀 통과를 뜻하지 않는다.

## 원인과 소비 경로

마지막 글자 뒤 자간은 다음 글자가 없는 가시 상자의 끝에 포함하지 않는다.
기존 나눔 정렬은 음수 자간만 정수 반올림한 폭으로 되돌렸고, 양쪽 정렬은 마지막
전진폭을 그대로 분배했다. PII 38쪽의 마지막 `를`은 자연 폭20CSSpx를 유지하면서
원점이699.72여서 저장 줄 끝을 약1.6px 넘었다. 독립 Print의 원점은697.920003이다.

`paragraph_layout.rs`의 공통 마지막 글리프 메트릭 질의가 signed 소수 자간을 계산한다.
`compute_line_extra_spacing`은 말미 구분 공백을 제외한 가시 폭에서 이를 제외해
양쪽·나눔·배분 정렬의 여분을 확정한다. 해당 결과가 같은 글줄의 `extra_word_spacing` /
`extra_char_spacing`으로 전달되고 실제 문자 위치 계산과 emitted run이 소비한다.
모델 문자열·줄 소속·논리 전진은 보존한다. 말미 개체 앞 전진과 필드·겹침·주석·탄력
leader의 별도 점유를 일반 마지막 글자 자간으로 해석하지 않는다. 문서 ID·고정 위치
분기와 좌표 clamp는 추가하지 않았다.

## 입력과 독립 Print

| 문서 | 입력 SHA-256 | PDF SHA-256 |
| --- | --- | --- |
| `samples/task2097/75544_pii_bunseok.hwpx` | `f429768ec02e5a849d08c06678cbee029fdc8618ca9e2bdcdbba485087fe71f1` | `0fef38f44b3825f1c97869c2720112536504eaf30cec2a2d2af6d4399ffe8770` |
| `samples/issue7445/80250_regulatory_analysis.hwp` | `e68d903c20e89ed1325b3a5633c3c0606d21b042345b76a1785cad1e1eca8589` | `65bbfb5a1ae7c4a71774cda4c91156990edd8a41177b6509109ee08ec286f286` |

PDF는 각각 `pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf`,
`pdf/80250_regulatory_analysis-hwp-2024.pdf`다. compat는2022/2024, Print192DPI,
정본 Visual Sweep의2px 이웃 관용 실루엣을 사용했다. 원본·PDF·글꼴·허용치를 변경하지 않았다.
PII는 기존 유효 기본 공급 조건대로 요청하지 않은 HFT 프로그램을 공급하지 않았으며,
설치 글꼴과 기존 `font-supply`를 사용했다. 규제분석은 기존 공통 글꼴 공급 조건을 유지했다.

## 현재 후보의 전쪽 결과

생산 소스는 부모 `372db04051807530cd7e335eb4229b637e1c00f6`와 본 커밋의 Rust 변경이다.
Native SHA-256은 `410a20dcf8a646943832958d86fbd95819e6017e18db77c07ecdfb568d2ec11b`,
fresh WASM은 `81dd807b61bc2e943b765de1449e42c822ad2420bfa8f4166d107470a959ffc0`다.
Mac locked wrapper의 `--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며
Docker 최적화 결과가 아니다. root pkg·Studio public·보존 pkg의 JS/WASM hash는 같다.
새 후보의 Studio 실제 소비36건은 아직 수행하지 않았다.

| 문서 | 쪽수 Native/WASM/PDF | Native 최저 | fresh WASM 최저 | 90% 미만 |
| --- | --- | --- | --- | --- |
| 개인정보 분석 | 66/66/66 | 93.77937% | 93.77937% | 양쪽0쪽 |
| 규제분석 | 17/17/17 | 94.12693% | 94.07717% | 양쪽0쪽 |

| 주요 쪽 | Native 보정 전 → 후 | fresh WASM 후 | 직접 확인 |
| --- | --- | --- | --- |
| PII11 | 99.90464 →100.00000% | 100.00000% | 표 셀의 마지막 글자·기준선·내용·괘선 |
| PII38 | 98.40210 →99.96598% | 99.96599% | 본문 마지막 글자·소제목·쪽번호·줄 소속 |
| 규제분석16 | 99.36660 →99.90503% | 99.90354% | 본문 간격·끝점·표·후속 내용 |

두 문서 전83쪽씩 총166쪽을 새 후보에서 산출했다. 같은 끝점 동작의 px 검사는
양쪽100%인 PII11을99%대의38쪽보다 우선 선정했다. 원본 표110/개체0/셀(row1,col2)의
원본 문단0과 내용으로 글줄을 선택한다. 독립 PDF의 마지막 `개` 원점·baseline을72→96CSS로
환산한 x685.7596028645833 / y268.79998779296875에 ±2CSSpx를 적용한다.
셀 문단의 내용·순서·줄 프레임 포함 관계도 유지한다. 원래 결함인38쪽의 문단363/줄1은
절대 좌표 없이 원본 프레임 안의 마지막 글리프와 문단 전체 보존을 검사한다.

## 검사와 남은 검증

기존 정렬 단위 검사에 음수·양수·0·소수 자간, 양쪽/나눔/배분 정렬, 말미 구분 공백,
뒤 인라인 개체를 포함했다. 올바른 원점+자연 상자 검사로 수정 전1FAIL → 관련4PASS다.
처음 선언 자간 px를 그대로 뺀 진단에는 글꼴별 전진 환산 오차0.0267px가 있어 기록을
보존하고 교정했으며 그 진단을 최종 FAIL→PASS 근거로 사용하지 않는다.

정식 `issue_7445_negative_indent_ownership::justified_terminal_glyph_preserves_the_source_frame_and_content`
검사를 이전 생산 소스로 실제 실행해1FAIL(end703.8/frame703.2666667)을 확인했다.
후보의 정식 파일 전체4건은4PASS다. 이전 frozen rlib의 의존 crate hash가 공용 캐시에 없어
실행하지 못한 compile은 실패 기록이며, 같은 이전 생산 소스를 다시 빌드한 실제 실패로 대체했다.
후보 파일을 정확히 복원한 SHA도 ignored 증적에 남겼다. 기존 검사·기대값은 제거하지 않았다.

정본 nextest에서도 같은 파일4PASS/0FAIL(0.171초,228skip)을 확인했다. 최초0건 선택은
생성 묶음과 현재 배정의 불일치여서 실패로 보존했고, 최종 `--prepare` 후 derive/generated
일치를 확인하고 실제4건을 실행했다. fmt와 Native/WASM Clippy도 통과했다.

전체 새 head nextest·Skia·전체48조건의 전쪽 비교·actual36은 제출 전 잔여다.
중간 C67 대비 Native PII12/26와 규제분석2의 작은 감소도 전후 review/overlay로 보존했다.
이전 devel 비교를 이번 후보 전체 통과로 재인용하지 않는다. 참고 비교에는 PII20 Native의
0.03042%p 감소가 남아 있고, PII의 공통 명시 환경과 실제 기본 출력의 새 후보 동일성까지
확인한 최종 devel 대조가 필요하다. 점수 상승으로 이 보류를 상쇄하지 않는다.
거대 표 분할본7~10쪽과 기존 제목 굵기·미세 간격 잔여도 계속 보정한다.

## 대표 이미지

- PII38 보정 전: [review](pii38-before-native-review.png) / [overlay](pii38-before-native-overlay.png)
- PII38 Native: [review](pii38-native-review.png) / [overlay](pii38-native-overlay.png)
- PII38 fresh WASM: [review](pii38-wasm-review.png) / [overlay](pii38-wasm-overlay.png)
- PII11 Native: [review](pii11-native-review.png) / [overlay](pii11-native-overlay.png)
- PII11 fresh WASM: [review](pii11-wasm-review.png) / [overlay](pii11-wasm-overlay.png)
- 규제분석16 Native: [review](regulatory16-native-review.png) / [overlay](regulatory16-native-overlay.png)
- 규제분석16 fresh WASM: [review](regulatory16-wasm-review.png) / [overlay](regulatory16-wasm-overlay.png)

로그·전쪽 TSV·producer·전후 비교·직접 판독 기록은 ignored
`/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/regression-policy-audit-20261015/rebase-4662c069-20261016/`
아래 `candidate-c70`, `c70-whole-native`, `c70-whole-wasm`, `c70-canonical-reviews`,
`c70-fixture-before-rebuild` 및 `c70-*` 로그에 보존한다.
