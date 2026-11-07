---
kind: report
status: active
canonical: mydocs/pr/assets/issue7375/selected-face-identity-20261107/README.md
last_verified: 2026-11-07
---

# 실제 face 선택과 메트릭 범위 보존

코드 `55cfac286a`는 호스트가 선택한 face 조회 이름을 출력 체인의 첫 요청으로 보존한다.
논리 family는 메트릭·선택 인증에 계속 사용한다. 같은 family의 Bold·Light를 다른 기본 굵기로
합치지 않으며 특정 문서·쪽 번호에 따른 분기는 추가하지 않는다.

`tests/cases/issue_1994_document_font_program_scope.rs`의 공개 API 관계 검사 4개는
사용자 선택 범위·다른 원본의 paint 스타일·전진·저장 바이트 보존, 잘못된 선택 거부,
전역/언어 슬롯의 실제 프로그램 조회 이름을 확인한다. 새 테스트를 `src/`에 추가하지 않는다.
두 face 조회 검사는 수정 전 실제 assertion 2 FAIL, 수정 후 PASS이며 집중 회귀 전체는 17 PASS다.
초기 검사 작성 중의 컴파일/입력 오류는 결함 재현으로 세지 않았다.

독립 PDF는 기존 `pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2024.pdf`를 재사용했다.
최소 글꼴 격리·공개 웹폰트·동일 PDF·정본 pdftoppm/2px 비교로 전383쪽을 다시 산출했다.
9·27·183·251·277쪽은 각각 100%로 복구됐고 전쪽 최저는 99.96%, 기준 하락은 0쪽이다.
이는 판정값이며 바이트 단위의 PNG 동일성이나 모든 픽셀의 완전 동일성을 뜻하지 않는다.

| 자료 | 수정 전 | 수정 후 |
| --- | --- | --- |
| 전383쪽 TSV | [Native](before-native-handbook.tsv) | [Native](after-native-handbook.tsv) |
| 277쪽 review | [수정 전](before-native-p277-review.png) | [수정 후](after-native-p277-review.png) |
| 277쪽 overlay | [수정 전](before-native-p277-overlay.png) | [수정 후](after-native-p277-overlay.png) |

새 WASM을 실제 Studio7700에서 초기화한 응답 해시도 확인했다. 새 Chrome의 일반 문서
로드·초기화·재로드에서 옥수수 두 실제 프로그램이 준비됐고, 초기화 버튼 자체는 목록만
지우고 감지 질문을 띄우지 않았다. 기존에 장시간 열린 사용자 Chrome의 글꼴 목록이
자동 갱신됐다고 주장하지 않는다. 설치 글꼴 바이너리는 제출하지 않는다.

소스·생성기·검사 결과 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7375/selected-face-identity-20261107/validation.json`)에 실제 확인한 범위를 기록했다.
후보 전체 Native/fresh WASM·전체 회귀·실제 소비 경로 결과는 최종 검증에서 별도로 판정한다.

## 기존 반환값 검사 검토

#8191의 `6097253468`에서 추가한 기존 두 검사는 설치 원본이 준비되면 환경이 null이라는
표현을 사용했다. 이후 `855e2a1185`는 원본의 실제 프로그램 신원을 환경으로 인증하도록
바꿨고 공개 선택 프로토콜도 이 계약을 사용한다. 반환 여부를 원본 우선순위로 오인하지 않도록
두 기존 검사에서 원본 programId·조회 이름·대체 경계·사용자 선택 보존을 확인한다.
실물 제목의 좌표 검사는 유지했다. 보정 전 확장 실행은 104 PASS/2 FAIL,
보정 후 관련 집중 실행은 19 PASS이며 전체 회귀는 별도 최종 검증 대상이다.
