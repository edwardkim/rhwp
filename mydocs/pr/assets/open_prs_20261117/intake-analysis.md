# 열린 PR 3건 통합 검토 — 선행 분석

검토 브랜치는 `review/integration-20261117-open-prs`, 고정 base는
`1e2488ebd432deb7c326be532f8a91945f260231`이다. 원 PR별 리뷰어 `jangster77` 지정과
API 확인 후 #9144 → #9149 → #9152 순서의 11개 기능·테스트·증적 커밋을 체리픽했다.
devel별 3건·PR 쌍별 3건·누적 3건의 머지 시뮬레이션은 모두 종료 코드 0이었다.
이 사실은 기능·시각 검증 통과를 뜻하지 않는다.

| 원 PR | 원 head | 검토할 주장 |
| --- | --- | --- |
| [#9144](https://github.com/edwardkim/rhwp/pull/9144) | `5cffd27772756f21796569060de9bc7132a9a1e1` | dump-pages가 rhwp 합성 줄을 한컴 저장 높이 근거로 사용하지 않음 |
| [#9149](https://github.com/edwardkim/rhwp/pull/9149) | `6e9f7fc98f78e8c6a180fb2e46c2cf57cc0a68da` | 저장 줄 없는 한/글 빈 문단의 문단 기준 위아래 자리차지 직선이 앵커 줄을 소유함 |
| [#9152](https://github.com/edwardkim/rhwp/pull/9152) | `acdb778b35dd9efe30ad63a8ba9d3f1ef6b88e4b` | 줄 밖 명시적 탭 정지 앞뒤 글의 줄 소속 분리, 워드 호환 대조 보존 |

## 공식 PDF와 원 PR 참고 PDF의 구분

원 PR #9149·#9152가 제시한 웹한글기안기 PDF는 공식 수용 점수·회귀 기대값·보호 하한의
근거로 사용하지 않는다. 같은 입력으로 Windows 데스크톱 한컴의 Hancom PDF Print를
MCP에서 새로 출력했다. Microsoft Print to PDF나 PDF 저장/내보내기 출력이 아니다.
입력 저장 메타데이터는 두 파일 모두 `hancom-office-2024`, `13.0.0.3622`이며 실제 출력
제품은 한컴 2024 `13.0.0.3901`이다. MCP engine은 `2024`, Print method는 `0`, 출력 모드는
`frame_print_to_pdf_ex_one_up`, 입력 전처리는 `none`이다. 두 PDF 모두 1쪽이며
Creator/Producer는 `Hancom PDF 1.3.0.550`이다.

- [입력·PDF 해시와 MCP 실행 영수증](reference-pdfs.json)
- [직선 앵커 공식 PDF](../../../../pdf/pr-review-open-prs-20261117/hwp_band-2024.pdf)
- [줄 밖 탭 공식 PDF](../../../../pdf/pr-review-open-prs-20261117/tab_stop_beyond_line-2024.pdf)

데스크톱 Print의 텍스트 위치에서 BAND→AFTER의 간격은 세로 오프셋 2374/600/0에 대해
각각 약 5789.67/5490.00/5490.00 HU다. 탭 재현물의 `가`→`X`는 약 29.248px(96dpi)이며
두 글자의 시작 x는 같다. 이는 PDF에서 추출한 독립 측정값이고 제품 출력의 통과 판정이 아니다.

## 현재 검증 상태와 다음 판정

고정 base의 Native 전쪽 비교는 직선 앵커 36.37336%, 줄 밖 탭 55.75355%다.
수용 조건은 후보의 같은 입력·공식 PDF 전쪽 Native/fresh WASM 90% 이상,
확대 판독·원 PR 주장 경계·최신 정본 보호 TSV 하락 없음과 필수 회귀·lint·build 통과다.
원 PR의 HWP5 변환 경로 검사와 원본 HWPX 경로를 구분하며, 한 경로의 통과로 다른 경로를
완료 처리하지 않는다. #9144는 실제 진단 호출처와 저장·다시 열기 결과를 확인한다.

새 base의 #9147은 Native 실제 대체 글꼴 관측과 표 뒤 종단 장식·줄 관계를 보완했다.
이 기능 및 상향된 보호 하한을 함께 보존해야 한다. 이전 #9143의 결과를 현재 base의 결과로
재사용하지 않는다. 검증용 글꼴은 기존 설치와 명시한 원본 공급 조건을 유지하며,
사용자가 비교를 요청한 신규 HFT ZIP은 설치하거나 보호 검증에 혼합하지 않는다.
