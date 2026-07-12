# PR #1959 처리 계획

## 적용 커밋

- `93d9ff491c06d937f6b931f7c96369a1593542c1` Task #1956: WrapAroundPara 쪽나누기 무시 + 전체 폭 밴드 오매칭 수정
- `f7dfbce0f4eb28d7bdf319488f9fccd3d343d04f` tools: verify_pi_page_vs_hangul --list 옵션 + BOM 내성
- `3b0fc347166e775dfa046eb51b12074f3382b9b3` Task #1955: 글뒤로/글앞으로 다중 쪽 표 후행 빈 문단 pi 귀속 수정
- `ca3574c1adc3edaf434598a3f22757d2e2964eff` Task #1921: 네이티브 HWP5/HWPX 비영 near-top vpos reset 트리거 추가
- `43f40f47cc06f9fde2df2c23c4e769d0a756c150` debug: RHWP_TABLE_DRIFT 출력에 available 구성요소 추가

## 처리 기록

- 누적 검토 브랜치에서 #1958 이후 적용.
- 충돌 없음.
- `debug:` 커밋은 환경변수 기반 진단 출력 확장임을 별도 확인.

## 후속 절차

- merge 전 최신 GitHub Actions와 Render Diff 성공 상태 재확인.
- 관련 이슈 #1955, #1956 및 #1921 후속 축 상태를 확인한다.
