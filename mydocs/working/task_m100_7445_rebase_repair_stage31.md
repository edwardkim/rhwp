# #7445 보정31 — 리베이스 ClipPath 소비와 공급 번호의 원본 메트릭

Issue: #7445. #8120을 포함한80968497 base로 리베이스한 뒤 새 ClipPath 자식 순회·Skia RenderProfile 인자와 TAC26 자동번호의 원본 metric face 선택을 복구했다.

[소비 경로·최신 빌드 해시·전쪽 TSV·review/overlay·회귀 및 잔여](../pr/assets/issue7445/completed-20261009/stage31-20261021/README.md).

Native/fresh WASM 각각 주요8문서769쪽은 전부90% 이상이고 TAC26쪽은 양쪽raw100%·직접판독을 확인했다. 최종 정식 새 case source의 frozen 보정 전1FAIL/후1PASS 및 suite011 수집을 확인했으며 자동suite/전체회귀 완료라고 쓰지 않는다. 리베이스 전후 공통640쪽/backend에서0.1%p 이상 하락0이지만, 별도#8064전179조건 검증이 언어14쪽 등 실제회귀를 검출했다. 원본 저장 줄·문단 여백 소유의 공통 보정을 다음 단계에서 계속한다. 정확한809base 전쪽 대조·필수 게이트/Actual36와 알려진 법령·과학 차이가 남아 PR 준비 완료가 아니다.
