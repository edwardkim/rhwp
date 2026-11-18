# 기존 hft2ttf 환경에서 필요한 수정만 선별

2026-11-17의 정리 기준은 upstream/devel 1e2488ebd432deb7c326be532f8a91945f260231이다.
기존 전체 head e858e569144537b38e9581e8a9cf6329064fefba와 준비 head
d09ba70c83e9cee61fb2d08f1b1fd36b08990109는 별도 backup 브랜치와 Git bundle에 보존했다.
이 경로는 기존 측정 증거를 유지하는 위치이며 닫힌 #8371을 새 작업 이슈로 간주하지 않는다.

## 남긴 그림 수정

독립 한컴 2020 Print의 NASM 로고는 26쪽에서 설명과 다음 본문보다 위에 있다.
devel은 원본 HWP3의 이전 쪽 호스트에 그림을 남겨 25쪽 제목에 겹쳤다.
이 문제는 hft2ttf 공급 조건에서도 재현되므로 폰트 이름 복원으로 해결되지 않는다.
저장 프레임의 넘침·명시적 쪽 경계·다음 쪽의 실제 예약 밴드로 그림 소속을 결정한다.
그림이 예약한 공간을 다음 본문에서 다시 계상하지 않으며 그림→설명→본문 순서를 보호한다.

원 수정 35d00ad613을 재적용했다. 기존 그림 ID·저장 바이트·21쪽 그림 검사와
sample4_nasm_picture_and_following_text_keep_their_source_page 관계 검사를 유지했다.

## 제외한 수정

- 622c8feb81, 52479c0c54: 유형 미상 HFT 신원의 확장과 그 경로의 저장 왕복 보정.
- a1d4446fd2, 8cf730c6ae, e2e8353541, 062e7a4db5: 문단 배경·빈 줄 밴드 확장 및 관련 검사.
  전역 overflow 진단 변화가 따르는 별도 범위이므로 이번 필수 수정 묶음에서 제외한다.
- HFT 신원·제목 폭의 새 검사, hft2otf 조건의 TSV, 중복 이미지와 과거 raw 로그는 제외한다.
  원 코드·검사·증거는 백업에 남아 있다. 기존 upstream 검사와 baseline은 변경하지 않는다.

## 보존한 시각 증거와 현재 검증의 구분

native-hft2ttf.tsv, wasm-hft2ttf.tsv와 final-p021/025/026-*.png는
과거 a27afaf 실행의 독립 Print 대조 자료다. 현재 정리 head의 측정으로 재명명하지 않는다.
현재 head의 집중 회귀와 선택 순서·기호 판정 검사 결과는 후속 실행 기록에 덧붙인다.
새 PR의 전체 검증이나 제출 완료를 주장하는 기록은 아니다.

## Native 빈 기호 판정

원 수정 146a6e5d9a는 Windows DWrite face가 직접 메모리 데이터를 주지 않아도
기존 공통 SFNT 복구 경로에서 실제 빈 기호를 확인하게 한다. 이름별 대체 규칙은 추가하지 않는다.
독립 Print의 HCR 글리프·원래 한컴바탕 우선 공급 순서를 대조한 기존 진단과 확대 자료를 보존했다.
handbook-ordered-probe.tsv 및 handbook-ordered-symbol-preservation-v6.json은 과거 실행 자료이며,
200쪽 개선과 다른 9쪽의 출력 보존 근거다. typeface-data-probe-summary.json에 직접 API 관측을 요약한다.
이 증거도 현재 정리 head의 새로운 전383쪽 gate 통과로 표시하지 않는다.
