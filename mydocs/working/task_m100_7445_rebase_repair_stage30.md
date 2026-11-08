# #7445 보정30 — 저장 후행 간격과 실제 출력 축의 공통 소유

Issue: #7445, 관련 #7009/#7751/#7390.

저장 plain/empty 문단의 닫힌 간격을 현재 커서에서 소비하되, 절대 source 축이 없는 로컬 전진을 paint 끝점으로 발행하지 않도록 공통 결과를 분리했다. dirty 사다리도 실제 축으로 재사용하지 않는다. 편람242쪽의9.973333CSSpx 앞당김을 원문748HU 누락으로 추적해 보정했다.

[원인·원 해결 계보·소비 경로·전후 검사·전체 TSV·review/overlay](../pr/assets/issue7445/completed-20261009/stage30-20261021/README.md).

Native/fresh 각각 primary637쪽은 전부90% 이상, 별도 기존 미달 수도시설3쪽은 구분했다. 같은 기준 PNG로1280쌍을 감사해0.1%p 이상 후퇴0이었다. 중간 후보에서 실제 전쪽 검증이 발견한 PrEP70·RowBreak HWP14 후퇴2개도 구현 복구하고 같은 Native PNG로 회복을 확인했다. 해당 두 반례와 편람 원문/독립 Print를 정식4검사로 보호한다. 기존 #7751의 crop 소비 뒤 잘못된 그림 선택만 소유 ID로 교정하고 원래 종이·본문·머리말 검사는 유지한다.

정식 자동 suite의 새4검사와 기존#7751의2검사 모두PASS, Native/WASM/전체target strict Clippy·workspace build·manifest/unit-tier·fmt/diff도PASS다. Native Skia3종도 root3859PASS/12ignored+다른libs182PASS, PNG placeholder2PASS, 직접PDF export14PASS로 완료했다. 전체 integration nextest 통과로 쓰지 않는다. 이번 단계는 전체 브랜치 제출 완료가 아니며 법령 #8064 Native 하한·과학 기준선·전체 scope 회귀/게이트와 실제 소비36건 등을 계속한다. PR은 사용자 새 승인 전 생성하지 않는다.
