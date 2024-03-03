schema:
    flatc --gen-all --rust -o thermorawfilereader/src/gen/ schema/schema.fbs
    flatc --gen-all --csharp -o librawfilereader/gen/ schema/schema.fbs

clean-dotnet:
    cd librawfilereader && dotnet clean

dotnet:
    cd librawfilereader && dotnet build -c Release

bundle:
    cd librawfilereader && dotnet publish -c Release -o ../lib