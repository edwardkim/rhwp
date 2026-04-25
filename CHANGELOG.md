# Changelog

이 프로젝트의 주요 변경 사항을 기록합니다.

## [Unreleased]

### 수정
- **목차 리더 도트 + 페이지번호 우측 탭 정렬** (#279)
  — 분석·구현 by [@seanshin](https://github.com/seanshin)
  - `fill_type=3` 점선 리더를 round cap 원형 점으로 표현 (한컴 동등)
  - `find_next_tab_stop` RIGHT 탭 클램핑 제외 — 들여쓰기 문단의 페이지번호 정렬 보정
  - 메인테이너 추가 보강: 셀 padding 인지 leader 시멘틱, 페이지번호 폭별 leader 길이 차등화, 공백 only run carry-over

## [0.6.0] — 2026-04-04

> 조판 품질 개선 + 비기능성 기반 구축 — "알을 깨고 세상으로"

### 추가
- **GitHub Actions CI**: 빌드 + 테스트 + Clippy 엄격 모드 (#46, #47)
- **GitHub Pages 데모**: https://edwardkim.github.io/rhwp/ (#48)
- **GitHub Sponsors**: 후원 버튼 활성화
- **그림 자르기(crop)**: SVG viewBox / Canvas drawImage로 이미지 crop 렌더링 (#43)
- **이미지 테두리선**: Picture border_attr 파싱 + 외곽선 렌더링 (#43)
- **머리말/꼬리말 Picture**: non-TAC 그림 절대 위치 배치, TAC 그림 인라인 배치 (#42)
- **로고 에셋 관리**: assets/logo/ 기준 원본 관리, favicon 생성
- **비기능성 작업 계획서**: 6개 영역 13개 항목 3단계 마일스톤 (#45)

### 수정
- **같은 문단 TAC+블록 표**: 중간 TAC vpos gap 음수 역행 방지 (#41)
- **분할 표 셀 세로 정렬**: 분할 행에서 Top 강제, 중첩 표 높이 반영 (#44)
- **TAC 표 trailing ls**: 경계 조건 순환 오류 해결 (#40)
- **통화 기호 렌더링**: ₩€£¥ Canvas 맑은고딕 폴백, SVG 폰트 체인 (#39)
- **반각/전각 폭 정밀화**: Bold 폴백 보정 제거, 스마트 따옴표/가운뎃점 반각 (#38)
- **폰트 이름 JSON 이스케이프**: 백슬래시 포함 폰트명 로드 실패 수정 (#37)
- **머리말 표 셀 이미지**: bin_data_content 전달 경로 수정 (#36)
- **Clippy 경고 제거**: unnecessary_unwrap, identity_op 등 6건 수정 (#47)

## [0.5.0] — 2026-03-29

> 뼈대 완성 — 역공학 기반 HWP 파서/렌더러

### 핵심 기능
- **HWP 5.0 / HWPX 파서**: OLE2 바이너리 + Open XML 포맷 지원
- **렌더링 엔진**: 문단, 표, 수식, 이미지, 차트, 머리말/꼬리말/바탕쪽/각주
- **페이지네이션**: 다단 분할, 표 행 단위 분할, shape_reserved 처리
- **SVG 내보내기**: CLI (`rhwp export-svg`)
- **Canvas 렌더링**: WASM/Web 기반
- **웹 에디터**: rhwp-studio (텍스트 편집, 서식, 표 생성)
- **hwpctl 호환 API**: 30 Actions, Field API (한컴 웹기안기 호환)
- **VS Code 확장**: HWP/HWPX 뷰어 (v0.5.0~v0.5.4)
- **755+ 테스트**

### 조판 엔진
- 줄간격 (고정값/비율/글자에따라), 문단 여백, 탭 정지
- 표 셀 병합, 테두리 스타일, 셀 수식 계산
- 다단 레이아웃, 문단 번호/글머리표
- 세로쓰기, 개체 배치 (자리차지/글자처럼/글앞/글뒤)
- 인라인 TAC 표/그림/수식 렌더링

### 수식 엔진
- 분수(OVER), 제곱근(SQRT/ROOT), 첨자
- 행렬: MATRIX, PMATRIX, BMATRIX, DMATRIX
- 경우(CASES), 정렬(EQALIGN), 적분/합/곱 연산자
- 15종 텍스트 장식, 그리스 문자, 100+ 수학 기호
