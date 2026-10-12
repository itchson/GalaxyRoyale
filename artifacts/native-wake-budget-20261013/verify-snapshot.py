import pathlib, zipfile, subprocess, hashlib, json
own = pathlib.Path(__file__).resolve().parent
root = own.parent.parent
ref = '60e16acaba7ceda12afd02bf497f6aadd34da885'
lines = subprocess.check_output(['git', 'ls-tree', '-r', ref], cwd=root).decode().splitlines()
git_blobs = {line.split('\t', 1)[1]: line.split('\t', 1)[0].split()[2] for line in lines}
sha = lambda b: hashlib.sha256(b).hexdigest()
compiled = {}
probe = (own / 'probe.rs').read_bytes()
for archive in ['frozen-native-source.zip', 'frozen-test-fixtures.zip']:
    with zipfile.ZipFile(own / archive) as z:
        for member in z.infolist():
            if member.is_dir(): continue
            source = z.read(member)
            canonical = source
            blob = hashlib.sha1(b'blob ' + str(len(canonical)).encode() + b'\0' + canonical).hexdigest()
            if git_blobs[member.filename] != blob:
                canonical = source.replace(b'\r\n', b'\n')
                blob = hashlib.sha1(b'blob ' + str(len(canonical)).encode() + b'\0' + canonical).hexdigest()
            assert git_blobs[member.filename] == blob, member.filename
            actual = (own / 'snapshot' / member.filename).read_bytes()
            is_probe_module = member.filename == 'crates/game_core/src/co_op_wake_tests.rs'
            assert actual == (source + b'\n' + probe if is_probe_module else source), member.filename
            compiled[member.filename] = {'canonicalGitBlob': blob, 'canonicalSha256': sha(canonical), 'archiveExportSha256': sha(source), 'archiveExportBytes': len(source), 'archiveCRLFConversion': source != canonical, 'compiledSha256': sha(actual), 'compiledBytes': len(actual), 'workOnlyProbeAppended': is_probe_module}
result = {'ref': ref, 'actualFilesVerified': len(compiled), 'allCompiledArchiveFilesVerified': True, 'sourceChangeOnlyWorkProbeAppend': True, 'files': compiled}
(own / 'compiled-inputs.json').write_text(json.dumps(result, indent=2) + '\n', encoding='utf8')
print(json.dumps({'ref': ref, 'verifiedFiles': len(compiled), 'compiledInputsSha256': sha((own / 'compiled-inputs.json').read_bytes())}))
