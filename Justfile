schema:
    flatc --gen-all --rust -o thermorawfilereader/src/gen/ schema/schema.fbs
    flatc --gen-all --csharp -o librawfilereader/gen/ schema/schema.fbs

clean-dotnet:
    cd librawfilereader && dotnet clean

dotnet:
    cd librawfilereader && dotnet build -c Debug

bundle:
    rm -rf dotnetrawfilereader-sys/lib/*
    cd librawfilereader && dotnet publish -c Release -o ../dotnetrawfilereader-sys/lib/
    md5sum dotnetrawfilereader-sys/lib/* | sort | tee dotnetrawfilereader-sys/lib/checksum

bundle-debug:
    rm -rf dotnetrawfilereader-sys/lib/*
    cd librawfilereader && dotnet publish -c Debug -o ../dotnetrawfilereader-sys/lib/
    md5sum dotnetrawfilereader-sys/lib/* | sort | tee dotnetrawfilereader-sys/lib/checksum

index := "1"

check i=index:
    cargo r -- tests/data/small.RAW {{i}}

test:
    cargo t

alias t := test

changelog:
    #!/usr/bin/env python

    import subprocess
    import re

    new_content = subprocess.check_output(['git', 'cliff', '-s', 'all', '-u', '--bump'], stderr=subprocess.DEVNULL).decode()

    new_version = subprocess.check_output(
        ["git", "cliff", "--bumped-version"], stderr=subprocess.DEVNULL
    ).decode().strip()

    buffer = open('CHANGELOG.md').read()

    buffer = buffer.replace("## ", f"{new_content}## ", 1).splitlines()

    offset = buffer.index("<!-- Versions -->") + 1
    line_to_patch = buffer[offset]
    buffer[offset] = re.sub(r"v\d+\.\d+\.\d+[^\.]*", new_version, line_to_patch)
    version_link_template = buffer[offset + 1]
    version_link_template = re.sub(
        r"\d+\.\d+\.\d+[^\.]*(?=\])", new_version[1:], version_link_template
    )
    version_link_template = re.sub(
        r"v\d+\.\d+\.\d+[^\.]*", new_version, version_link_template
    )

    buffer.insert(offset + 1, version_link_template)

    buffer = '\n'.join(buffer)
    open('CHANGELOG.md', 'wt').write(buffer)

