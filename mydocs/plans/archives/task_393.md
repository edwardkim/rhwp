# Task 393: prepare-github.sh 스크립트 + 제외 목록 작성

## 목표

`/home/edward/vsworks/rhwp/` → `/home/edward/mygithub/rhwp/`로 선별 복사하는 스크립트를 작성한다.

## 구현 계획 (3단계)

### 1단계: github-exclude.txt 제외 목록 작성
- 민감 정보 파일
- 한컴/상용 라이선스 리소스
- 민감 샘플 파일
- 빌드 아티팩트 (target/, pkg/, output/, dist/)
- 내부 전용 파일 (hwp_webctl/, mydocs/convers/)

### 2단계: scripts/prepare-github.sh 스크립트 작성
- rsync로 제외 목록 기반 복사
- CLAUDE.md 공개용으로 교체 (Task 394에서 작성 예정, 여기서는 placeholder)
- .env.docker.example 복사
- 실행 후 결과 트리 출력

### 3단계: 검증
- 스크립트 실행 → /home/edward/mygithub/rhwp/ 생성 확인
- 민감 파일 미포함 확인
- cargo build 가능 여부 확인
