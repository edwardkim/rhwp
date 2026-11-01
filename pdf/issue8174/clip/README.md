# PDF 연속 MoveTo와 선택 쪽수 대조군

이 PDF는 RHWP 렌더러를 사용하지 않고 직접 PDF syntax 또는 PyMuPDF로 생성했다.
가져오기 전의 각 원본 PDF 자체가 독립 표시 기준이다. 비공개 사용자 원본의 내용은 포함하지 않는다.

| 파일 | 쪽 | 독립 계약 |
| --- | --- | --- |
| `consecutive-moves.pdf` | 5 | 사각형, 빈 연속 이동·q/Q 뒤 사각 polygon, 빈 이동 뒤 `re`·evenodd, 볼록 삼각형, affine Form |
| `text-consecutive-moves.pdf` | 3 | 같은 텍스트·사각 clip을 보통 `re`, 빈 이동·q/Q 뒤 polygon, 빈 이동 뒤 `re`로 표현 |
| `batch-operator-control.pdf` | 2 | 쪽마다 250,000회 미만인 동일 이미지와 160,000여 연산을 batch로 가져와도 유지 |
| `compound-rejected.pdf` | 1 | 실제 선분이 있는 두 subpath의 이미지 clip; #8413에서 alpha 합성 지원 |
| `curve-rejected.pdf` | 1 | 실제 곡선 이미지 clip; #8413에서 Bézier alpha 합성 지원 |

생성기와 실제 source·제품 hash·전쪽 Native/fresh WASM TSV·확대 판독 자료는
[S08 증거](../../../mydocs/pr/assets/issue8174/v2-s08/README.md)에 있다.
이미지의 alpha·crop·소유·저장 왕복은 `issue_8174_pdf_consecutive_moves` 관계 검사로 확인한다.
파일명의 `rejected`는 S08 당시의 미지원 판정을 추적하기 위해 유지한다. #8413에서는
독립 원본의 Native/fresh WASM 전쪽 raw 100%를 확인한 뒤 거부 기대를 alpha·저장 보존
검사로 교체했다. [후속 구현 기록](../../../mydocs/working/task_m100_8413_implementation.md).
source clip의 정규화를 위해 sidecar 상한을 해제하지 않으며 곡선의 실제 내부와 외부를 검사한다.

텍스트 PDF는 DejaVu Sans의 subset 프로그램을 포함한다. [DejaVu 고지](LICENSE-DejaVu)를 보존한다.
PyMuPDF의 BaseFont 선언은 `DejaVu Sans Book`이며 실제 프로그램 family는 `DejaVu Sans`다.
검증은 동일한 실제 프로그램을 공급하고 [명시적 글꼴 환경](../../../mydocs/pr/assets/issue8174/v2-s08/dejavu-family-environment.json)을
양쪽 runtime에 적용한다. 임의의 대체 프로그램이나 글꼴 예외가 아니다. 이름 차이를 방치한 실패
캡처와 올바른 공급 뒤의 결과는 구분해 S08 보고에 기록했다.
