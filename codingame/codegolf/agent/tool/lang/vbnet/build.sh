#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
dir=$(cache_dir vbnet "$src")
if [[ ! -x $dir/run ]]; then
	mkdir -p "$dir/src"
	cp "$src" "$dir/src/main.vb"
	cat >"$dir/src/main.vbproj" <<-'EOF'
		<Project Sdk="Microsoft.NET.Sdk">
		  <PropertyGroup>
		    <OutputType>Exe</OutputType>
		    <TargetFramework>net8.0</TargetFramework>
		    <LangVersion>16.9</LangVersion>
		    <UseAppHost>false</UseAppHost>
		    <AssemblyName>Answer</AssemblyName>
		    <RootNamespace>Answer</RootNamespace>
		    <Optimize>false</Optimize>
		    <DebugType>portable</DebugType>
		    <DefineConstants></DefineConstants>
		    <GenerateAssemblyInfo>false</GenerateAssemblyInfo>
		  </PropertyGroup>
		</Project>
	EOF
	cat >"$dir/src/nuget.config" <<-'EOF'
		<configuration>
		  <packageSources>
		    <clear />
		  </packageSources>
		</configuration>
	EOF
	DOTNET_CLI_HOME=$dir/home DOTNET_NOLOGO=1 DOTNET_CLI_TELEMETRY_OPTOUT=1 DOTNET_SKIP_FIRST_TIME_EXPERIENCE=1 \
		NUGET_PACKAGES=$dir/nuget MSBUILDDISABLENODEREUSE=1 \
		"$CG_VBNET/bin/dotnet" build "$dir/src/main.vbproj" -c CodinGame -o "$dir/out" -nologo -v q -clp:ErrorsOnly -nodeReuse:false >&2
	write_launcher "$dir" env LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8 DOTNET_CLI_TELEMETRY_OPTOUT=1 DOTNET_NOLOGO=1 "$CG_VBNET/bin/dotnet" "$dir/out/Answer.dll"
fi
echo "$dir/run"
