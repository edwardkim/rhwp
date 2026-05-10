# pdf-large/ — 대용량 PDF (Git LFS 추적)

50 MB 초과 PDF 영역 영역 의 LFS 격리 폴더.

## 본질

GitHub 권장 50 MB 초과 PDF 영역 영역 일반 git history 영역 영역 누적 부담을 차단하기 위해 Git LFS 영역 영역 격리 추적한다.

## 사용 규칙

- **본 폴더**: 50 MB 초과 PDF 만 배치 (한컴 편집기 1-up 인쇄 영역 영역 대용량 PDF 등)
- 다른 폴더 (`pdf/`, `pdf-2020/`, `pdf-2010/`) 영역 영역 일반 git 영역 영역 보존 (50 MB 미만)

## 명명 규약

상위 `pdf/` README 와 동일:

| 한컴 버전 | 명명 패턴 |
|----------|----------|
| 한글 2022 | `{원본 stem}-2022.pdf` |
| 한글 2020 | `{원본 stem}-2020.pdf` |
| 한글 2010 | `{원본 stem}-2010.pdf` |

원본 파일이 하위 폴더 (`samples/basic/` / `samples/hwp3/`) 에 있는 경우 PDF 도 동일 하위 폴더 구조 유지.

## Git LFS 처리

`.gitattributes` 영역 영역 `pdf-large/**/*.pdf filter=lfs diff=lfs merge=lfs -text` 패턴 등록.

신규 PDF 추가 시 자동 LFS pointer 변환:
```bash
cp /path/to/big.pdf pdf-large/big-2022.pdf
git add pdf-large/big-2022.pdf
git commit -m "..."
git push  # LFS 자동 업로드
```

## Clone / Fork 시

LFS 미설치 환경 영역 영역 `pdf-large/` 의 PDF 가 LFS pointer (135 byte) 만 보임. 실제 PDF 영역 영역:

```bash
git lfs install
git lfs pull
```

또는 사용자 PATH 영역 영역 git-lfs binary 직접 다운로드:
- https://github.com/git-lfs/git-lfs/releases
