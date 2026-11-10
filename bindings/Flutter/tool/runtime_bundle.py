#!/usr/bin/env python3
"""Build/verify a movable supplied Flutter runtime; never build or launch it."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
import struct
import sys
import tempfile

# Reuse the supplied-Native artifact reader's symlink/reparse and stable fd rules.
from native_artifact_provenance import _file_identity, _signature

SCHEMA = 'rhwp-flutter-desktop'
MAX_ENTRIES = 8192
MAX_BYTES = 2 * 1024**3
MAX_FILE_BYTES = 512 * 1024**2
MAX_MANIFEST_BYTES = 1024**2
NAMES = {
    'linux': ('librhwp_native_ffi.so', 'librhwp_flutter_owner.so'),
    'windows': ('rhwp_native_ffi.dll', 'rhwp_flutter_owner.dll'),
    'macos': ('librhwp_native_ffi.dylib', 'librhwp_flutter_owner.dylib'),
}
NOTICES = ('LICENSE', 'THIRD_PARTY.md', 'THIRD_PARTY_LICENSES.md')


def relative(value):
    if (not isinstance(value, str) or len(value.encode('utf-8')) > 4096 or
            not value or '\\' in value or ':' in value or '\x00' in value or
            value.startswith('/') or any(v in ('', '.', '..') for v in value.split('/'))):
        raise ValueError('invalid bundle-relative path')
    return value


def revision(value):
    if not isinstance(value, str) or not re.fullmatch('[0-9a-f]{40}', value):
        raise ValueError('source revision must be a full lowercase Git SHA')
    return value


def _read(path, *, maximum=MAX_FILE_BYTES, copy_to=None, prefix=4096):
    """Stream one stable regular input; do not follow links for executable input."""
    before = path.lstat()
    identity = _file_identity(before)
    if not stat.S_ISREG(before.st_mode) or before.st_nlink != 1 or before.st_size > maximum:
        raise ValueError('bundle input must be a bounded regular file without hardlinks')
    digest = hashlib.sha256()
    head = bytearray()
    flags = os.O_RDONLY | getattr(os, 'O_BINARY', 0) | getattr(os, 'O_NOFOLLOW', 0)
    with os.fdopen(os.open(path, flags), 'rb') as source:
        if _file_identity(os.fstat(source.fileno())) != identity:
            raise ValueError('bundle input changed before read')
        destination = None
        try:
            if copy_to is not None:
                copy_to.parent.mkdir(parents=True, exist_ok=True)
                destination = copy_to.open('xb')
            count = 0
            for chunk in iter(lambda: source.read(128 * 1024), b''):
                count += len(chunk)
                if count > maximum:
                    raise ValueError('bundle input grew beyond its bound')
                digest.update(chunk)
                if len(head) < prefix:
                    head.extend(chunk[:prefix-len(head)])
                if destination is not None:
                    destination.write(chunk)
            if count != before.st_size or _file_identity(os.fstat(source.fileno())) != identity:
                raise ValueError('bundle input changed during read')
            if destination is not None:
                destination.flush()
                os.fsync(destination.fileno())
        finally:
            if destination is not None:
                destination.close()
    if _file_identity(path.lstat()) != identity:
        raise ValueError('bundle input changed after read')
    if copy_to is not None:
        # Keep executable entry points usable on POSIX; do not copy setuid bits.
        copy_to.chmod(0o755 if before.st_mode & 0o111 else 0o644)
    return {'kind': 'file', 'bytes': before.st_size, 'sha256': digest.hexdigest(),
            'executable': bool(before.st_mode & 0o111)}, bytes(head)


def binary_identity(path, shared):
    """Read actual ELF/PE/Mach-O machine/type metadata, without loading code."""
    with path.open('rb') as source:
        length = os.fstat(source.fileno()).st_size
        def at(offset, size):
            if offset < 0 or size < 0 or offset + size > length:
                raise ValueError('truncated executable header')
            source.seek(offset)
            data = source.read(size)
            if len(data) != size:
                raise ValueError('truncated executable header')
            return data
        header = at(0, min(length, 64))
        if header.startswith(b'\x7fELF'):
            if len(header) < 64 or header[4:7] != b'\x02\x01\x01':
                raise ValueError('requires 64-bit little-endian ELF')
            kind, machine = struct.unpack_from('<HH', header, 16)
            if kind not in ((3,) if shared else (2, 3)):
                raise ValueError('unexpected ELF executable/library type')
            architecture = {62: 'x86_64', 183: 'arm64'}.get(machine)
            if architecture is None:
                raise ValueError('unsupported ELF machine')
            return 'linux', {architecture}
        if header.startswith(b'MZ'):
            if len(header) < 64:
                raise ValueError('truncated DOS header')
            offset = struct.unpack_from('<I', header, 60)[0]
            if offset > 1024**2:
                raise ValueError('PE header offset exceeds its bound')
            pe = at(offset, 26)
            if pe[:4] != b'PE\0\0' or struct.unpack_from('<H', pe, 24)[0] != 0x20b:
                raise ValueError('requires PE32+ executable')
            architecture = {0x8664: 'x86_64', 0xaa64: 'arm64'}.get(struct.unpack_from('<H', pe, 4)[0])
            if architecture is None or bool(struct.unpack_from('<H', pe, 22)[0] & 0x2000) != shared:
                raise ValueError('unexpected PE machine/library type')
            return 'windows', {architecture}
        def macho(offset, bound):
            head = at(offset, 32)
            if head[:4] != b'\xcf\xfa\xed\xfe' or bound < 32:
                raise ValueError('requires 64-bit little-endian Mach-O')
            cpu, kind = struct.unpack_from('<I', head, 4)[0], struct.unpack_from('<I', head, 12)[0]
            architecture = {0x1000007: 'x86_64', 0x100000c: 'arm64'}.get(cpu)
            if architecture is None or kind not in ((6, 8) if shared else (2,)):
                raise ValueError('unexpected Mach-O machine/library type')
            return architecture
        if header[:4] == b'\xcf\xfa\xed\xfe':
            return 'macos', {macho(0, length)}
        if header[:4] in (b'\xca\xfe\xba\xbe', b'\xca\xfe\xba\xbf'):
            count = struct.unpack('>I', at(4, 4))[0]
            width = 32 if header[:4] == b'\xca\xfe\xba\xbf' else 20
            if not 1 <= count <= 16:
                raise ValueError('invalid fat Mach-O slice count')
            architectures = set()
            spans = []
            for i in range(count):
                entry = at(8 + i * width, width)
                cpu = struct.unpack_from('>I', entry)[0]
                offset, size = struct.unpack_from('>QQ' if width == 32 else '>II', entry, 8)
                if offset < 8 + count * width or size < 32 or offset + size > length:
                    raise ValueError('invalid fat Mach-O slice range')
                if any(offset < end and start < offset + size for start, end in spans):
                    raise ValueError('overlapping fat Mach-O slices')
                architecture = macho(offset, size)
                if cpu != {'x86_64': 0x1000007, 'arm64': 0x100000c}[architecture]:
                    raise ValueError('fat Mach-O machine differs from slice')
                spans.append((offset, offset + size))
                architectures.add(architecture)
            return 'macos', architectures
        raise ValueError('unknown executable format')


def _scan(root):
    """Retain internal relative framework links, never traverse them."""
    _signature(root.lstat())
    root = root.resolve(strict=True)
    if not root.is_dir():
        raise ValueError('runtime root must be a directory')
    entries = {}
    folded = set()
    pending = [(root, 0)]
    total = 0
    while pending:
        directory, depth = pending.pop()
        if depth > 32:
            raise ValueError('runtime directory depth exceeds its bound')
        for path in sorted(directory.iterdir()):
            name = relative(path.relative_to(root).as_posix())
            if name.casefold() in folded:
                raise ValueError('case-colliding runtime entries')
            folded.add(name.casefold())
            if len(folded) > MAX_ENTRIES:
                raise ValueError('runtime entry count exceeds its bound')
            info = path.lstat()
            if stat.S_ISLNK(info.st_mode):
                target = relative(os.readlink(path))
                resolved = path.resolve(strict=True)
                resolved.relative_to(root)
                entries[name] = {'kind': 'link', 'target': target}
            else:
                _signature(info)
                if stat.S_ISDIR(info.st_mode):
                    entries[name] = {'kind': 'directory', 'signature': _signature(info)}
                    pending.append((path, depth + 1))
                elif stat.S_ISREG(info.st_mode) and info.st_nlink == 1:
                    if info.st_size > MAX_FILE_BYTES:
                        raise ValueError('runtime file exceeds its bound')
                    total += info.st_size
                    if total > MAX_BYTES:
                        raise ValueError('runtime bytes exceed their bound')
                    entries[name] = {'kind': 'file', 'signature': _signature(info)}
                else:
                    raise ValueError('runtime entries require regular files or internal relative links')
    return root, entries


def _copy_tree(root, destination):
    root, entries = _scan(root)
    result = {}
    for name, item in sorted(entries.items()):
        src, dst = root / name, destination / name
        if item['kind'] == 'directory':
            dst.mkdir(parents=True, exist_ok=True)
            result[name] = {'kind': 'directory'}
        elif item['kind'] == 'link':
            dst.parent.mkdir(parents=True, exist_ok=True)
            os.symlink(item['target'], dst, target_is_directory=src.is_dir())
            result[name] = item
        else:
            if _signature(src.lstat()) != item['signature']:
                raise ValueError('runtime input changed before copy')
            result[name], _ = _read(src, copy_to=dst)
    if _scan(root)[1] != entries:
        raise ValueError('runtime tree changed during copy')
    return result


def checked(root, name):
    name = relative(name)
    path = root / name
    # Every component is below root after resolution; manifest links are
    # validated as their own entries. Native/dispatcher/entrypoint are regular.
    path.resolve(strict=True).relative_to(root)
    return path


def verify(directory, *, expected_platform=None, expected_architecture=None):
    root, scanned = _scan(Path(directory).absolute())
    path = root / 'rhwp-runtime.json'
    raw, encoded = _read(path, maximum=MAX_MANIFEST_BYTES, prefix=MAX_MANIFEST_BYTES)
    if raw['bytes'] == 0:
        raise ValueError('empty runtime manifest')
    # Preserve duplicate-key rejection from supplied-artifact provenance.
    from native_artifact_provenance import _object
    manifest = json.loads(encoded, object_pairs_hook=_object, parse_constant=lambda _: (_ for _ in ()).throw(ValueError('nonfinite JSON')))
    if not isinstance(manifest, dict) or type(manifest.get('schemaVersion')) is not int or manifest.get('schemaVersion') != 1 or manifest.get('kind') != SCHEMA:
        raise ValueError('unsupported runtime manifest')
    platform, architecture = manifest.get('platform'), manifest.get('architecture')
    if platform not in NAMES or architecture not in ('x86_64', 'arm64'):
        raise ValueError('unsupported runtime target')
    if expected_platform is not None and platform != expected_platform:
        raise ValueError('runtime platform mismatch')
    if expected_architecture is not None and architecture != expected_architecture:
        raise ValueError('runtime architecture mismatch')
    sources = manifest.get('sources')
    if not isinstance(sources, dict) or set(sources) != {'runtime', 'native', 'dispatcher'}:
        raise ValueError('missing declared source identities')
    for value in sources.values():
        revision(value)
    files = manifest.get('files')
    if (not isinstance(files, dict) or not 1 <= len(files) <= MAX_ENTRIES or
            set(scanned) != set(files) | {'rhwp-runtime.json'}):
        raise ValueError('incomplete runtime file manifest')
    total = 0
    for name, expected in files.items():
        file = checked(root, name)
        if not isinstance(expected, dict):
            raise ValueError('invalid runtime file description')
        actual = scanned[name]
        if expected.get('kind') == 'directory':
            if actual.get('kind') != 'directory' or expected != {'kind': 'directory'}:
                raise ValueError('runtime directory mismatch')
        elif expected.get('kind') == 'link':
            if actual != expected:
                raise ValueError('runtime link mismatch')
        elif expected.get('kind') == 'file':
            if (set(expected) != {'kind', 'bytes', 'sha256', 'executable'} or
                    type(expected.get('bytes')) is not int or type(expected.get('executable')) is not bool or
                    not isinstance(expected.get('sha256'), str) or
                    not re.fullmatch('[0-9a-f]{64}', expected['sha256'])):
                raise ValueError('invalid runtime file metadata')
            current, _ = _read(file)
            if current != expected:
                raise ValueError('runtime digest, size, or mode mismatch: ' + name)
            total += current['bytes']
        else:
            raise ValueError('unknown runtime file kind')
    if total > MAX_BYTES:
        raise ValueError('runtime bytes exceed their bound')
    roles = {'native': 'native/' + NAMES[platform][0], 'dispatcher': 'native/' + NAMES[platform][1]}
    runtime = manifest.get('runtime')
    if not isinstance(runtime, str) or not runtime.startswith('app/'):
        raise ValueError('invalid runtime executable')
    roles['runtime'] = runtime
    for role, name in roles.items():
        if name not in files or files[name].get('kind') != 'file':
            raise ValueError('missing regular executable role')
        actual_platform, architectures = binary_identity(checked(root, name), shared=role != 'runtime')
        if platform != actual_platform or architecture not in architectures:
            raise ValueError('runtime executable platform/architecture mismatch')
    if manifest.get('native') != roles['native'] or manifest.get('dispatcher') != roles['dispatcher']:
        raise ValueError('noncanonical native executable roles')
    abi = manifest.get('abi')
    if (not isinstance(abi, dict) or any(type(v) is not int for v in abi.values()) or
            abi != {'native': 1, 'dispatcher': 2}):
        raise ValueError('unsupported declared native/dispatcher ABI')
    if any(files.get('notices/' + name, {}).get('kind') != 'file' for name in NOTICES):
        raise ValueError('missing runtime license notices')
    if _scan(root)[1] != scanned:
        raise ValueError('runtime changed during verification')
    return manifest


def package(runtime, executable, native, dispatcher, notices, output, platform, architecture, sources):
    """Copy explicit trusted inputs; their Git source association is caller-declared."""
    if platform not in NAMES or architecture not in ('x86_64', 'arm64'):
        raise ValueError('unsupported runtime target')
    for value in sources.values():
        revision(value)
    executable = relative(executable)
    output = Path(output).absolute()
    if os.path.lexists(output):
        raise FileExistsError('runtime bundle output already exists')
    if not output.parent.is_dir():
        raise ValueError('runtime output parent must exist')
    # Read-only packaging; never dlopen or run the supplied code here.
    runtime = Path(runtime).absolute()
    native, dispatcher = Path(native).absolute(), Path(dispatcher).absolute()
    with tempfile.TemporaryDirectory(prefix='.rhwp-runtime-', dir=output.parent) as temp:
        staging = Path(temp)
        files = {'app/' + name: item for name, item in _copy_tree(runtime, staging / 'app').items()}
        for role, source, name in [('native', native, NAMES[platform][0]),
                                   ('dispatcher', dispatcher, NAMES[platform][1])]:
            destination = 'native/' + name
            files[destination], _ = _read(source, copy_to=staging / destination)
        for name in NOTICES:
            destination = 'notices/' + name
            files[destination], _ = _read(Path(notices) / name, copy_to=staging / destination)
        # Copy the project's existing redistribution notices as supplied; the
        # Flutter build's own license assets remain in app/ unchanged.
        licenses = Path(notices) / 'licenses'
        if licenses.exists():
            files.update({'notices/licenses/' + name: item for name, item in _copy_tree(licenses, staging / 'notices/licenses').items()})
        manifest = {'kind': SCHEMA, 'schemaVersion': 1, 'platform': platform, 'architecture': architecture,
                    'sources': sources, 'abi': {'native': 1, 'dispatcher': 2},
                    'runtime': 'app/' + executable, 'native': 'native/' + NAMES[platform][0],
                    'dispatcher': 'native/' + NAMES[platform][1], 'files': dict(sorted(files.items()))}
        for name, item in _scan(staging)[1].items():
            if item['kind'] == 'directory':
                manifest['files'][name] = {'kind': 'directory'}
        manifest['files'] = dict(sorted(manifest['files'].items()))
        encoded = json.dumps(manifest, indent=2, sort_keys=True, allow_nan=False) + '\n'
        if len(encoded.encode()) > MAX_MANIFEST_BYTES:
            raise ValueError('runtime manifest exceeds its bound')
        (staging / 'rhwp-runtime.json').write_text(encoded, encoding='utf-8')
        verify(staging)
        # mkdir is exclusive on all supported OSes; never overwrite a prior
        # bundle or remove a caller-owned directory when a race wins creation.
        output.mkdir()
        try:
            for child in staging.iterdir():
                shutil.move(str(child), str(output / child.name))
            verify(output)
        except BaseException:
            shutil.rmtree(output)
            raise
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    commands = parser.add_subparsers(dest='command', required=True)
    create = commands.add_parser('create', allow_abbrev=False)
    for key in ('runtime', 'native', 'dispatcher', 'notices', 'output'):
        create.add_argument('--' + key, type=Path, required=True)
    create.add_argument('--executable', required=True)
    create.add_argument('--platform', choices=tuple(NAMES), required=True)
    create.add_argument('--architecture', choices=('x86_64', 'arm64'), required=True)
    for role in ('runtime', 'native', 'dispatcher'):
        create.add_argument('--' + role + '-source', required=True)
    check = commands.add_parser('verify', allow_abbrev=False)
    check.add_argument('--bundle', type=Path, required=True)
    check.add_argument('--platform', choices=tuple(NAMES))
    check.add_argument('--architecture', choices=('x86_64', 'arm64'))
    args = parser.parse_args()
    try:
        if args.command == 'create':
            result = package(args.runtime, args.executable, args.native, args.dispatcher, args.notices,
                             args.output, args.platform, args.architecture,
                             {name: getattr(args, name + '_source') for name in ('runtime', 'native', 'dispatcher')})
        else:
            result = verify(args.bundle, expected_platform=args.platform, expected_architecture=args.architecture)
        print(json.dumps({'ok': True, 'kind': result['kind'], 'platform': result['platform'],
                          'architecture': result['architecture'], 'files': len(result['files']),
                          'nativeLoaded': False, 'runtimeLaunched': False}))
        return 0
    except (OSError, ValueError, KeyError, TypeError) as error:
        print('rhwp runtime bundle: ' + str(error), file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
