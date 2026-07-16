# PR #2272 검토 — PDF 폰트 채번 비결정 원인 규명 + 정규화 비교 도구 (planet6897, #2269)

- 검토일: 2026-07-15 / base: devel / 2파일 +192 (도구 + tech 문서, 소스 무변경) / CLEAN, CI 11 green
- 배경: 어제(#2266 검토) 등록한 파생 이슈 #2269 를 하루 만에 처리.

## 검증

### 원인 규명 — 크레이트 소스로 실증

`svg2pdf-0.13.0/src/util/context.rs` — `fonts: HashMap<ID, Option<Font>>`
(:23) 를 `for font in self.fonts.values_mut()` (:80-81) 로 반복하며 객체
ref 할당. **주장한 위치·기전 그대로 확인.** 우리 측 renumber 는 결정적이나
입력 순서가 비결정이라는 분석도 정합.

### 도구 실증 (로컬 재현)

| 케이스 | 결과 |
|--------|------|
| 양성: 같은 문서(treatise) 2회 export | raw diff 466,050 바이트 → **정규형 동일, exit 0** |
| 정규형 파일 대조 (`--emit`) | 두 실행의 정규형 바이트 동일 |
| 음성: 스트림 내용 1바이트 변조 | **exit 1 + diff 위치·문맥 출력** (진짜 회귀 검출) |
| `py_compile` | 통과 |

### 발견 — 경미한 견고성 결함 (비차단 개선 요청)

입력 파일 부재 시 **traceback + exit 1** — docstring 계약("2=인자 오류")과
불일치하고, 게이트 소비자가 경로 오타를 "진짜 회귀"로 오독할 수 있다.
FileNotFoundError → 메시지 + exit 2 처리 권장 (2줄 수정). 게이트 도구의
exit code 계약이므로 후속 반영 요청.

> 검증 중 #2268(export-pdf 간헐 행)이 2회 재발(sungeo, timeout 180/240s)
> — 본 PR 무관, #2268 기록 보강 대상.

## 판단

**approve → merge 수용 권고** (CLEAN — 일반 merge 가능). 견고성 결함은
비차단 코멘트로 반영 요청.

**#2269 처리 옵션** (merge 시 자동 close 아님 — closes 키워드 없음):
- A안: 완료 조건("정규화 비교 도구 제공") 충족으로 **close** — 업스트림
  근본 정정(svg2pdf 기여/벤더 패치)은 별도 이슈로 분리.
- B안: 근본 정정까지 #2269 를 open 유지.
