# Task 392: GitHub 업스트림 공개 준비 계획 수립

## 목표

rhwp 프로젝트를 MIT 라이선스 오픈소스로 GitHub에 공개한다.
**기존 GitLab 리포는 수정하지 않고**, `/home/edward/mygithub/rhwp/`에 **신규 리포를 구성**한다.

## 핵심 원칙

```
기존 리포 (GitLab)                     신규 리포 (GitHub)
/home/edward/vsworks/rhwp/            /home/edward/mygithub/rhwp/
─────────────────────────             ──────────────────────────
CLAUDE.md (민감정보 포함)              → CLAUDE.md (민감정보 제거된 공개용)
.env.docker (로컬 전용)               → .env.docker.example (템플릿)
samples/ (민감 HWP 포함)              → samples/ (자체 생성 공개 샘플만)
hwp_webctl/ (공공기관 코드)           → 복사 안 함
mydocs/manual/hwp/Help/ (한컴 CHM)    → 복사 안 함
mydocs/manual/hwpctl/*.hwp (한컴 문서) → 복사 안 함
mydocs/convers/ (대화기록)            → 복사 안 함
rhwp-studio/dist/fonts/ (상용 폰트)   → 오픈소스 폰트만
mydocs/tech/hwp_spec_5.0.md (한컴 스펙) → 복사 안 함 (재배포 조건 불명)
────────────────────────────────────────────────────────────
src/ (전체)                           → 전체 복사
mydocs/orders/ (일일 기록)            → 복사 (교육 자료)
mydocs/plans/ (계획서)                → 복사 (교육 자료)
mydocs/feedback/ (피드백)             → 복사 (교육 자료, 민감 제외)
mydocs/report/ (보고서)               → 복사 (교육 자료)
mydocs/tech/ (기술 문서)              → 선별 복사 (자체 작성분만)
mydocs/manual/onboarding_guide.md     → 복사
mydocs/troubleshootings/              → 복사
rhwp-studio/src/ (전체)               → 전체 복사
rhwp-studio/e2e/ (E2E 테스트)         → 복사
scripts/ (대시보드)                   → 복사
```

## 구현 방식: 복사 스크립트

`scripts/prepare-github.sh`를 작성하여 선별 복사를 자동화한다.

```bash
#!/bin/bash
# 사용법: ./scripts/prepare-github.sh
# 결과: /home/edward/mygithub/rhwp/ 에 GitHub용 리포 생성

SRC=/home/edward/vsworks/rhwp
DST=/home/edward/mygithub/rhwp

# 1. 소스 코드 복사
rsync -av --exclude-from=github-exclude.txt $SRC/ $DST/

# 2. CLAUDE.md 공개용으로 교체
cp $DST/CLAUDE.public.md $DST/CLAUDE.md

# 3. .env.docker.example 복사
cp $SRC/.env.docker.example $DST/.env.docker.example

# 4. 공개 샘플만 복사
cp $SRC/samples/public/* $DST/samples/

# 5. LICENSE 생성
# 6. git init + 커밋
```

## 빌드 영향 없음 확인

| 명령 | GitHub 리포에서 실행 가능 | 비고 |
|------|------------------------|------|
| `cargo build` | O | 소스 코드 완전 |
| `cargo test` | O | 민감 샘플 참조 테스트는 skip 처리 |
| WASM 빌드 | O | `.env.docker.example` → `.env.docker`로 복사 |
| `rhwp export-svg` | O | 공개 샘플 사용 |
| E2E 테스트 | O | 공개 샘플 사용 |

## 제외 대상 상세

### 민감 정보
- CLAUDE.md의 GitLab IP/계정/비밀번호/SSH 키 경로
- .env.docker의 환경변수 값
- hwp_webctl/ (실제 공공기관 예산요구서 코드+데이터)
- mydocs/convers/ (작업 대화 기록)

### 한컴/상용 라이선스
- `mydocs/manual/hwp/Help/` — 한컴 hwpkor.chm 추출본 (953개 HTML)
- `mydocs/manual/hwpctl/*.hwp` — 한컴 hwpctl 공식 문서 원본
- `mydocs/manual/hwpctl/*.md` — 한컴 문서 변환본
- `mydocs/tech/hwp_spec_5.0.md` — 한컴 공개 스펙 (재배포 조건 불명)
- `mydocs/tech/hwp_spec_chart.md` — 한컴 차트 스펙
- `mydocs/tech/hwp_spec_equation.md` — 한컴 수식 스펙
- `mydocs/tech/hwp_spec_3.0_hwpml.md` — 한컴 HWPML 스펙
- `rhwp-studio/dist/fonts/h2hdrm*` — 함초롬돋움 (한컴 폰트)
- `rhwp-studio/dist/fonts/Times*` — MS 상용 폰트
- `rhwp-studio/dist/fonts/Tahoma*` — MS 상용 폰트
- `rhwp-studio/dist/fonts/Verdana*` — MS 상용 폰트
- `rhwp-studio/dist/fonts/Malgun*` — MS 상용 폰트

### 민감 샘플 파일
- `samples/bodo-01.hwp, bodo-02.hwp` — 실제 보도자료
- `samples/kps-ai.hwp` — 입찰 제안서
- `samples/gonggo-01.hwp` — 공고문
- `samples/synam-001.hwp` — 실제 문서
- `samples/exam_*.hwp` — 수능 시험지 (저작권)
- `samples/bsbc01_10_000.hwp, data.json` — 예산양식
- `rhwp-studio/public/samples/kps-ai.hwp` 등 동일

### 자체 작성 (포함 가능)
- `mydocs/tech/hwp_spec_errata.md` — 스펙 오류 정리 (자체 작성)
- `mydocs/tech/hwp_table_rendering.md` — 표 렌더링 가이드 (자체 작성)
- `mydocs/tech/equation_support_status.md` — 수식 지원 현황 (자체 작성)
- `mydocs/tech/dev_roadmap.md` — 개발 로드맵 (자체 작성)
- `mydocs/tech/rendering_engine_design.md` — 렌더링 엔진 설계 (자체 작성)

## 단계별 타스크 분할

| 타스크 | 내용 | 우선순위 | 비고 |
|--------|------|---------|------|
| 393 | prepare-github.sh 스크립트 + 제외 목록 작성 | P0 | 선별 복사 자동화 |
| 394 | CLAUDE.public.md + .env.docker.example 작성 | P0 | 민감 정보 제거된 공개 버전 |
| 395 | samples/ 공개 샘플 자체 생성 | P0 | gen-table 등으로 테스트용 HWP 생성 |
| 396 | LICENSE (MIT) + Cargo.toml/package.json 업데이트 | P0 | |
| 397 | README.md 공개용 작성 | P0 | 영문 또는 한영 병기 |
| 398 | CONTRIBUTING.md 작성 | P1 | |
| 399 | GitHub Actions CI 설정 | P1 | |
| 400 | GitHub 리포지토리 생성 + 최초 push | P0 | /home/edward/mygithub/rhwp/ |
