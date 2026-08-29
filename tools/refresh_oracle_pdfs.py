#!/usr/bin/env python3
"""원본 저장 제품에 맞는 한컴 PDF 정답지를 endpoint별 단일 큐로 갱신한다.

각 원본에 대해 `rhwp info --json`을 먼저 읽어 한컴 저장 제품을 확인한다. 2024 저장본은
MCP engine 2024로, 그 외와 메타데이터 없는 저장본은 문서화된 2020 호환 engine으로 변환한다.
각 MCP endpoint에는 변환 작업을 하나만 보낸다. 여러 endpoint의 완전한 환경 파일을 지정하면
endpoint별 한 작업씩만 병렬로 실행한다. 성공한 결과만 `pdf/`의 canonical 이름으로 원자 교체하므로
실패한 항목은 기존 PDF와 기준 원장을 훼손하지 않는다.

기본 실행은 canonical PDF가 없는 원장 행만 보충한다. 기존 기준 PDF도 다시 뽑으려면
`--refresh-existing`을 준다. 대량 갱신은 재개 가능하도록 `--limit` 또는 `--source`로 나눠 실행한다.
경로 식별자 도입 전의 PDF는 `--migrate-success-log`로, 성공 로그에 남은 원본 대응이 확인되는
경우에만 재변환 없이 새 이름으로 이관한다.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import threading
import time
import re
from zoneinfo import ZoneInfo

TOOLS_DIR = pathlib.Path(__file__).resolve().parent
if str(TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(TOOLS_DIR))

from oracle_pdf_selection import (
    canonical_filename,
    canonical_pdf_path,
    engine_for_product,
    path_token_canonical_filename,
)


FIXTURE = pathlib.Path('tests/fixtures/oracle_page_count_baseline.tsv')
KST = ZoneInfo('Asia/Seoul')


class DocumentPreflightRejected(RuntimeError):
    """MCP가 문서 자체의 unattended 변환을 허용하지 않은 경우다."""


class RunLogger:
    """시각이 붙은 재개 가능 변환 로그. 비공개 env 내용이나 명령행은 기록하지 않는다."""

    def __init__(self, path):
        self.path = pathlib.Path(path) if path else None
        self.file = None
        self.lock = threading.Lock()
        if self.path:
            self.path.parent.mkdir(parents=True, exist_ok=True)
            self.file = self.path.open('a', encoding='utf-8')

    def emit(self, level, message):
        timestamp = datetime.now(KST).strftime('%y-%m-%d %H:%M:%S KST')
        line = '%s [%s] %s' % (timestamp, level, message)
        with self.lock:
            print(line, flush=True)
            if self.file:
                print(line, file=self.file, flush=True)

    def detail(self, label, content):
        if not content.strip():
            return
        for line in content.rstrip().splitlines():
            self.emit('DETAIL', '%s: %s' % (label, line))

    def close(self):
        if self.file:
            self.file.close()


def default_env_file():
    configured = os.environ.get('HWP2024_MCP_ENV_FILE')
    candidates = [
        configured,
        os.path.expanduser('~/hwp-convert-2024/.env.local'),
        os.path.expanduser('~/Cloud/Devel/hwp-convert-2024/.env.local'),
    ]
    for candidate in candidates:
        if candidate and pathlib.Path(candidate).is_file():
            return candidate
    return candidates[0] if candidates[0] else candidates[1]


def fixture_sources():
    sources = []
    with FIXTURE.open(encoding='utf-8') as fh:
        for raw in fh:
            if raw.startswith('#') or not raw.strip():
                continue
            source = raw.split('\t', 1)[0]
            if source.lower().endswith(('.hwp', '.hwpx')):
                sources.append(source)
    return sources


def source_metadata(rhwp, source):
    result = subprocess.run(
        [rhwp, 'info', source, '--json'],
        capture_output=True,
        text=True,
        encoding='utf-8',
        errors='replace',
    )
    if result.returncode:
        raise RuntimeError('%s: rhwp info 실패 (%s)' % (source, result.stderr.strip()))
    try:
        data = json.loads(result.stdout)
    except json.JSONDecodeError as exc:
        raise RuntimeError('%s: rhwp info JSON 해석 실패: %s' % (source, exc)) from exc
    product = (data.get('lastSavedWith') or {}).get('product')
    return product, engine_for_product(product)


def existing_canonical_pdfs(source):
    """메타데이터 조회 없이 확인할 수 있는 기존 canonical PDF 후보를 돌려준다."""
    return [
        (engine, pathlib.Path(canonical_pdf_path(source, engine)))
        for engine in ('2020', '2024')
        if pathlib.Path(canonical_pdf_path(source, engine)).is_file()
    ]


def mcp_command(args, env_file, action, extra):
    return [
        args.npx,
        '-y',
        '--package=file:%s' % args.package,
        '--',
        'hwp2024-mcp-convert',
        action,
        '--env-file', env_file,
        *extra,
    ]


def configured_server_urls(env_file):
    """공통 인증을 쓰는 endpoint URL 목록을 비공개 env에서 읽는다."""
    key = 'HWP2024_MCP_SERVER_URLS='
    for raw in pathlib.Path(env_file).read_text(encoding='utf-8').splitlines():
        if raw.startswith(key):
            return [url.strip() for url in raw[len(key):].split(',') if url.strip()]
    return []


def run_mcp(command, label, logger):
    result = subprocess.run(
        command,
        capture_output=True,
        text=True,
        encoding='utf-8',
        errors='replace',
    )
    logger.detail('%s stdout' % label, result.stdout)
    logger.detail('%s stderr' % label, result.stderr)
    if result.returncode:
        raise RuntimeError('%s 실패 (exit %d)' % (label, result.returncode))
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as exc:
        raise RuntimeError('%s JSON 응답 해석 실패: %s' % (label, exc)) from exc


def nested_value(data, key):
    if isinstance(data, dict):
        if key in data:
            return data[key]
        for value in data.values():
            found = nested_value(value, key)
            if found is not None:
                return found
    if isinstance(data, list):
        for value in data:
            found = nested_value(value, key)
            if found is not None:
                return found
    return None


def convert_one(args, source, engine, destination, logger, env_file, lane):
    output_name = destination.name
    with tempfile.TemporaryDirectory(prefix='rhwp-oracle-pdf-') as temp_dir:
        logger.emit('INFO', 'MCP start: lane=%s source=%s output=%s engine=%s' %
                    (lane, source, destination, engine))
        started = time.monotonic()
        start_response = run_mcp(
            mcp_command(args, env_file, 'start', [
                '--input', source,
                '--target', 'pdf',
                '--engine', engine,
                '--output-filename', output_name,
                '--timeout-seconds', str(args.timeout_seconds),
            ]),
            'MCP start lane=%s' % lane,
            logger,
        )
        job_id = nested_value(start_response, 'job_id')
        if not isinstance(job_id, str) or not job_id:
            raise RuntimeError('%s: MCP start 응답에 job_id가 없다' % source)
        logger.emit('INFO', 'MCP job 생성: job_id=%s' % job_id)

        deadline = started + args.timeout_seconds + args.status_interval_seconds
        terminal_success = {'success', 'succeeded'}
        terminal_failure = {'failed', 'failure', 'cancelled', 'canceled', 'terminated'}
        while True:
            for attempt in range(1, args.status_retry_count + 1):
                try:
                    status_response = run_mcp(
                        mcp_command(args, env_file, 'status', ['--job-id', job_id]),
                        'MCP status lane=%s job_id=%s' % (lane, job_id),
                        logger,
                    )
                    break
                except RuntimeError as exc:
                    if attempt == args.status_retry_count:
                        raise
                    delay = args.status_retry_delay_seconds * attempt
                    logger.emit('WARN', 'MCP status 재시도: lane=%s job_id=%s attempt=%d/%d delay=%ss reason=%s' %
                                (lane, job_id, attempt, args.status_retry_count, delay, exc))
                    time.sleep(delay)
            status = nested_value(status_response, 'status')
            terminal = nested_value(status_response, 'terminal')
            elapsed = time.monotonic() - started
            logger.emit('INFO', 'MCP 상태: job_id=%s status=%s terminal=%s elapsed=%.1fs' %
                        (job_id, status or 'unknown', terminal, elapsed))
            if status in terminal_success:
                break
            remote_error = nested_value(status_response, 'error')
            if (isinstance(remote_error, str)
                    and remote_error.startswith('input preflight rejected the document:')):
                raise DocumentPreflightRejected('%s: %s' % (source, remote_error))
            if status in terminal_failure or terminal is True:
                raise RuntimeError('%s: MCP job %s 비성공 종료 (status=%s, terminal=%s, %.1fs)' %
                                   (source, job_id, status, terminal, elapsed))
            if time.monotonic() >= deadline:
                raise RuntimeError('%s: MCP job %s 상태 대기 시간 초과 (%.1fs)' %
                                   (source, job_id, elapsed))
            time.sleep(args.status_interval_seconds)

        download_response = run_mcp(
            mcp_command(args, env_file, 'download', [
                '--job-id', job_id,
                '--output-dir', temp_dir,
            ]),
            'MCP download lane=%s job_id=%s' % (lane, job_id),
            logger,
        )
        del download_response
        produced = pathlib.Path(temp_dir) / output_name
        if not produced.is_file() or produced.stat().st_size == 0:
            raise RuntimeError('%s: MCP가 유효한 PDF를 저장하지 않았다' % source)
        destination.parent.mkdir(parents=True, exist_ok=True)
        temporary = destination.with_suffix(destination.suffix + '.tmp')
        shutil.copy2(produced, temporary)
        os.replace(temporary, destination)
        logger.emit('INFO', 'MCP 성공: source=%s output=%s bytes=%d elapsed=%.1fs job_id=%s' %
                    (source, destination, destination.stat().st_size,
                     time.monotonic() - started, job_id))


SUCCESS_RECORD = re.compile(r'MCP 성공: source=(.+?) output=(pdf/.+?\.pdf) bytes=')


def endpoint_env_files(args, configured, directory):
    """lane별 env를 만든다.

    `--endpoint-env-file`은 endpoint와 token이 모두 다른 서버용이다. URL만 다른 서버들은
    공통 인증 env를 복제해 URL만 바꾼 임시 파일을 사용한다.
    """
    if args.endpoint_env_file:
        seen = set()
        lanes = []
        for index, raw_path in enumerate(configured, 1):
            path = pathlib.Path(raw_path).expanduser().resolve()
            if not path.is_file():
                raise ValueError('endpoint 환경 파일이 없다: %s' % path)
            if path in seen:
                raise ValueError('같은 endpoint 환경 파일을 중복 지정할 수 없다')
            seen.add(path)
            lanes.append(('endpoint-%d' % index, str(path)))
        return lanes

    if configured == [None]:
        return [('endpoint-1', args.env_file)]
    if len(configured) != len(set(configured)):
        raise ValueError('같은 --server-url을 중복 지정할 수 없다')
    base = pathlib.Path(args.env_file).read_text(encoding='utf-8')
    base = re.sub(r'^HWP2024_MCP_SERVER_URLS=.*(?:\n|$)', '', base, flags=re.MULTILINE)
    lanes = []
    for index, endpoint in enumerate(configured, 1):
        replacement = 'HWP2024_MCP_SERVER_URL=%s' % endpoint
        content, changed = re.subn(
            r'^HWP2024_MCP_SERVER_URL=.*$', replacement, base, flags=re.MULTILINE)
        if not changed:
            content = base.rstrip() + '\n' + replacement + '\n'
        path = directory / ('endpoint-%d.env' % index)
        path.write_text(content, encoding='utf-8')
        lanes.append(('endpoint-%d' % index, str(path)))
    return lanes


def migrate_success_log(args, log_path, logger):
    """기존 성공 로그가 증명하는 PDF만 경로 식별자 이름으로 이관한다."""
    records = []
    with pathlib.Path(log_path).open(encoding='utf-8') as fh:
        for raw in fh:
            match = SUCCESS_RECORD.search(raw)
            if match:
                records.append((match.group(1), match.group(2)))
    logger.emit('INFO', '기존 성공 로그 이관 계획: 기록=%d' % len(records))
    migrated = skipped = failures = 0
    for source, old_output in records:
        source_path = pathlib.Path(source)
        old_path = pathlib.Path(old_output)
        if not source_path.is_file():
            failures += 1
            logger.emit('ERROR', '이관 제외(원본 없음): %s' % source)
            continue
        try:
            product, engine = source_metadata(args.rhwp, source)
        except RuntimeError as exc:
            failures += 1
            logger.emit('ERROR', '이관 제외(metadata 실패): %s' % exc)
            continue
        legacy_path = pathlib.Path('pdf') / canonical_filename(source, engine)
        if old_path != legacy_path:
            failures += 1
            logger.emit('ERROR', '이관 제외(기존 이름 불일치): source=%s output=%s engine=%s' %
                        (source, old_output, engine))
            continue
        token_path = pathlib.Path('pdf') / path_token_canonical_filename(source, engine)
        available = [path for path in (legacy_path, token_path) if path.is_file()]
        if not available:
            skipped += 1
            logger.emit('INFO', '이관 제외(기존 PDF 없음): source=%s output=%s' % (source, old_output))
            continue
        if len(available) != 1:
            failures += 1
            logger.emit('ERROR', '이관 제외(둘 이상의 이전 PDF): source=%s outputs=%s' %
                        (source, ','.join(str(path) for path in available)))
            continue
        source_pdf = available[0]
        destination = pathlib.Path(canonical_pdf_path(source, engine))
        if source_pdf == destination:
            skipped += 1
            logger.emit('INFO', '이관 제외(이미 canonical): source=%s output=%s' % (source, destination))
            continue
        if destination.exists():
            skipped += 1
            logger.emit('WARN', '이관 제외(대상 PDF 존재): source=%s output=%s' % (source, destination))
            continue
        logger.emit('INFO', '이관: source=%s output=%s product=%s engine=%s' %
                    (source, destination, product or 'null', engine))
        if not args.dry_run:
            destination.parent.mkdir(parents=True, exist_ok=True)
            os.replace(source_pdf, destination)
        migrated += 1
    logger.emit('INFO', '기존 성공 로그 이관 완료: 이관=%d 제외=%d 오류=%d' %
                (migrated, skipped, failures))
    return 1 if failures else 0


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--rhwp', default='target/release/rhwp')
    parser.add_argument('--npx', default=shutil.which('npx') or 'npx')
    parser.add_argument(
        '--package',
        default=str(TOOLS_DIR.parent / 'tools/hwp-convert-mcp-2024-client-20260824-011002.tar.gz'),
    )
    parser.add_argument('--env-file', default=default_env_file())
    parser.add_argument('--timeout-seconds', type=int, default=1800)
    parser.add_argument('--status-interval-seconds', type=int, default=15)
    parser.add_argument('--status-retry-count', type=int, default=4)
    parser.add_argument('--status-retry-delay-seconds', type=int, default=5)
    parser.add_argument('--endpoint-env-file', action='append', default=[],
                        help='endpoint와 token이 함께 든 lane별 env 파일 (반복 지정 시 파일당 단일 worker)')
    parser.add_argument('--server-url', action='append', default=[],
                        help='공통 인증 env를 사용할 MCP URL (반복 지정 시 URL당 단일 worker)')
    parser.add_argument('--source', action='append', default=[], help='특정 상대 원본 경로(반복 가능)')
    parser.add_argument('--limit', type=int, default=None, help='이번 실행에서 변환할 최대 문서 수')
    parser.add_argument('--refresh-existing', action='store_true')
    parser.add_argument('--migrate-success-log',
                        help='기존 MCP 성공 로그의 검증된 source/output 기록만 새 canonical 이름으로 이관')
    parser.add_argument('--dry-run', action='store_true')
    parser.add_argument('--log-file', help='시각·상세 MCP 로그 파일(환경 파일의 내용은 기록하지 않음)')
    args = parser.parse_args()

    logger = RunLogger(args.log_file)
    try:
        if args.migrate_success_log:
            return migrate_success_log(args, args.migrate_success_log, logger)
        configured_lanes = (
            args.endpoint_env_file
            or args.server_url
            or configured_server_urls(args.env_file)
            or [None]
        )
        logger.emit('INFO', '배치 시작: 기존 canonical PDF 제외=%s, dry_run=%s, endpoint=%d, endpoint별 단일 큐' %
                    (not args.refresh_existing, args.dry_run, len(configured_lanes)))
        selected = args.source or fixture_sources()
        pending = []
        inspection_failures = []
        skipped_existing = 0
        for source in selected:
            if not pathlib.Path(source).is_file():
                logger.emit('WARN', '제외(원본 없음): %s' % source)
                continue
            if not args.refresh_existing:
                existing = existing_canonical_pdfs(source)
                if len(existing) == 1:
                    engine, destination = existing[0]
                    skipped_existing += 1
                    logger.emit('INFO', '유지(기존 canonical PDF, metadata 생략): source=%s output=%s engine=%s' %
                                (source, destination, engine))
                    continue
                if len(existing) > 1:
                    logger.emit('WARN', '기존 canonical PDF가 둘 이상이므로 metadata 확인: source=%s outputs=%s' %
                                (source, ','.join(str(path) for _, path in existing)))
            try:
                product, engine = source_metadata(args.rhwp, source)
            except RuntimeError as exc:
                inspection_failures.append(str(exc))
                logger.emit('ERROR', '제외(rhwp info 실패): %s' % exc)
                continue
            destination = pathlib.Path(canonical_pdf_path(source, engine))
            if destination.exists() and not args.refresh_existing:
                skipped_existing += 1
                logger.emit('INFO', '유지(기존 PDF): source=%s output=%s product=%s engine=%s' %
                            (source, destination, product or 'null', engine))
                continue
            pending.append((source, product, engine, destination))

        if args.limit is not None:
            pending = pending[:args.limit]
        failures = list(inspection_failures)
        logger.emit('INFO', '변환 계획: 대상=%d 기존 PDF 제외=%d metadata 실패=%d' %
                    (len(pending), skipped_existing, len(inspection_failures)))
        if not args.dry_run:
            with tempfile.TemporaryDirectory(prefix='rhwp-mcp-endpoints-') as endpoint_dir:
                lanes = endpoint_env_files(args, configured_lanes, pathlib.Path(endpoint_dir))
                lane_lock = threading.Lock()
                stop_event = threading.Event()
                next_position = 0

                def convert_next(lane, env_file):
                    nonlocal next_position
                    lane_failures = []
                    while not stop_event.is_set():
                        with lane_lock:
                            if stop_event.is_set() or next_position >= len(pending):
                                return lane_failures
                            position = next_position + 1
                            source, product, engine, destination = pending[next_position]
                            next_position += 1
                        logger.emit('INFO', '[%d/%d] 변환: lane=%s source=%s output=%s product=%s engine=%s' %
                                    (position, len(pending), lane, source, destination,
                                     product or 'null', engine))
                        try:
                            convert_one(args, source, engine, destination, logger, env_file, lane)
                        except DocumentPreflightRejected as exc:
                            logger.emit('WARN', '제외(MCP input preflight): lane=%s %s' % (lane, exc))
                            continue
                        except Exception as exc:
                            message = '%s: %s' % (source, exc)
                            lane_failures.append(message)
                            logger.emit('ERROR', 'lane=%s %s' % (lane, message))
                            stop_event.set()
                    return lane_failures

                with ThreadPoolExecutor(max_workers=len(lanes), thread_name_prefix='oracle-pdf') as executor:
                    futures = [executor.submit(convert_next, lane, env_file) for lane, env_file in lanes]
                    for future in futures:
                        failures.extend(future.result())
        logger.emit('INFO', '배치 완료: 변환 대상=%d 기존 PDF 제외=%d 실패=%d' %
                    (len(pending), skipped_existing, len(failures)))
        return 1 if failures else 0
    finally:
        logger.close()


if __name__ == '__main__':
    sys.exit(main())
