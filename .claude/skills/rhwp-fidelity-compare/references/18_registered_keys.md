# 18 — 등록 키 REG

하네스 `REG` 는 ASCII 글롭으로 원본과 기준 PDF 를 고른다. 한글 argv /
NFC·NFD 함정을 피하기 위해서다.

| 키 | 원본 글롭 | PDF 글롭 | 등급 | 특성 |
| --- | --- | --- | --- | --- |
| plan | `samples/2022* *.hwp` | `pdf/2022* *-2022.pdf` | 한컴 2022 기준 | 보고서 35쪽 |
| manual | `samples/2025 *.hwpx` | `pdf/2025 *-2024.pdf` | 한컴 2024 기준 | 장문 편람 |
| bunjang | `samples/21868765*.hwp` | `samples/21868765*.pdf` | 참고 PDF | 표. 승격 금지 |
| korexam | `samples/21_*.hwp` | `pdf/21_*-2022.pdf` | 한컴 2022 기준 | A3 2단 15쪽 |
| math | `samples/exam_math.hwp` | `pdf/exam_math-2022.pdf` | 한컴 2022 기준 | 수식 20쪽 |
| eng | `samples/exam_eng.hwp` | `pdf/exam_eng-2022.pdf` | 한컴 2022 기준 | 영어 8쪽 |

글롭이 0건이면 `글롭 미해결`. 여러 건이면 짧은 경로를 고른다.
새 키를 이 스킬이 `REG` 에 추가하지 않는다. 없으면 19장 direct pair.

## 호출

```bash
venv/bin/python tools/fidelity_compare/fidelity_compare.py <키> <시작> <끝>
# 시작·끝은 0-based, 끝 포함
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 34
venv/bin/python tools/fidelity_compare/fidelity_compare.py math 0 19 --text-only
venv/bin/python tools/fidelity_compare/fidelity_compare.py korexam 0 2
```

키 오타:

```
등록되지 않은 문서 키: pala (선택: bunjang, eng, korexam, manual, math, plan)
```

positional 이 세 개가 아니면 사용법 오류. direct pair 와 섞지 말 것.

## bunjang 주의

`samples/` 동반 PDF 다. 등급이 참고 다. 사용자가 "공식이야" 라고
증명하기 전에 F17. 표 중심이라 텍스트 후보보다 시트/레이아웃 원장이
더 쓸모 있을 수 있다. 그래도 등급은 참고로 남긴다.

## 실측 앵커를 키에 묶기

- plan 35쪽 전수: #3385
- math: diff 6~11%, 수식 강함
- korexam: 자간 미세, `RHWP_FONT_PATH_DIR` 후속

이 앵커는 "항상 같은 숫자" 가 아니다. 바이너리와 글꼴이 바뀌면
순위가 바뀐다. 키를 고르는 힌트일 뿐이다.

## 샘플이 없는 머신

sparse checkout 이나 슬림 클론은 `samples/` · `pdf/` 가 비어 있을 수
있다. 글롭 실패면 그 키를 건너뛰고, 계약 시험은 파일 존재에 의존하지
않는다 (`samples.json` 의 `liveCompareNotRequiredForContract`).

## 에이전트 금지

- 일곱 번째 키를 README 표에만 추가
- 한글 별칭 키 (`업무계획`) 를 argv 로 발명
- bunjang 을 한컴 2022 기준으로 승격
- 글롭이 고른 파일이 의도한 파일이 아닌데 확인 없이 전수
