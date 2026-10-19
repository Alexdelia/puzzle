#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
dir=$(cache_dir csharp "$src")
if [[ ! -x $dir/run ]]; then
	mkdir -p "$dir/src"
	cp "$src" "$dir/src/main.cs"
	cat >"$dir/src/main.csproj" <<-'EOF'
		<Project Sdk="Microsoft.NET.Sdk">
		  <PropertyGroup>
		    <OutputType>Exe</OutputType>
		    <TargetFramework>net8.0</TargetFramework>
		    <LangVersion>12.0</LangVersion>
		    <ImplicitUsings>disable</ImplicitUsings>
		    <Nullable>disable</Nullable>
		    <AllowUnsafeBlocks>true</AllowUnsafeBlocks>
		    <UseAppHost>false</UseAppHost>
		    <AssemblyName>Answer</AssemblyName>
		    <Optimize>false</Optimize>
		    <DebugType>portable</DebugType>
		    <DefineConstants></DefineConstants>
		    <DisableImplicitFrameworkDefines>true</DisableImplicitFrameworkDefines>
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
		"$CG_CSHARP/bin/dotnet" build "$dir/src/main.csproj" -c CodinGame -o "$dir/out" -nologo -v q -clp:ErrorsOnly -nodeReuse:false >&2
	write_launcher "$dir" env LC_ALL=en_US.UTF-8 DOTNET_CLI_TELEMETRY_OPTOUT=1 DOTNET_NOLOGO=1 "$CG_CSHARP/bin/dotnet" "$dir/out/Answer.dll"
fi
echo "$dir/run"
