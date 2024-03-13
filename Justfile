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