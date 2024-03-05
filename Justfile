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

bundle-debug:
    rm -rf dotnetrawfilereader-sys/lib/*
    cd librawfilereader && dotnet publish -c Debug -o ../dotnetrawfilereader-sys/lib/

check:
    cargo r -- tests/data/small.RAW 1