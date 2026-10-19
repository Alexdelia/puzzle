#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
dir=$(cache_dir fsharp "$src")
if [[ ! -x $dir/run ]]; then
	sdk=$(echo "$CG_FSHARP"/sdk/8.*)
	refs=$(echo "$CG_FSHARP"/packs/Microsoft.NETCore.App.Ref/8.*/ref/net8.0)
	runtime_version=$(basename "$(echo "$CG_FSHARP"/shared/Microsoft.NETCore.App/8.*)")
	cp "$src" "$dir/main.fs"
	cp "$sdk/FSharp/FSharp.Core.dll" "$dir/"
	export DOTNET_CLI_TELEMETRY_OPTOUT=1 DOTNET_NOLOGO=1 DOTNET_CLI_HOME=$dir
	"$CG_FSHARP/bin/dotnet" "$sdk/FSharp/fsc.dll" --nologo --target:exe --debug:portable --optimize- --tailcalls- --nowarn:FS0020,FS0025,FS0064,FS3370 \
		--targetprofile:netcore --noframework --simpleresolution \
		-r:"$dir/FSharp.Core.dll" $(printf -- '-r:%s ' "$refs"/*.dll) \
		-o:"$dir/Answer.dll" "$dir/main.fs" >&2
	jq -n --arg version "$runtime_version" \
		'{runtimeOptions: {tfm: "net8.0", framework: {name: "Microsoft.NETCore.App", version: $version}}}' \
		>"$dir/Answer.runtimeconfig.json"
	write_launcher "$dir" env LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8 DOTNET_CLI_TELEMETRY_OPTOUT=1 DOTNET_NOLOGO=1 \
		"$CG_FSHARP/bin/dotnet" "$dir/Answer.dll"
fi
echo "$dir/run"
