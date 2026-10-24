#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/lib.sh"

usage="usage: sandbox.sh <command> [args...]; runs it in bwrap: private home, no runtime dir (session bus, keyring, systemd), repo writable except agent/tool, git config and hooks, and the files the host runs"
(($#)) || { echo "$usage" >&2 && exit 2; }
repo=$(git -C "$root" rev-parse --show-toplevel)
runtime=${XDG_RUNTIME_DIR:-/run/user/$UID}
mkdir -p "$sandbox_home" "$report_socket_dir"

mounts=(
	--ro-bind / /
	--dev /dev
	--proc /proc
	--tmpfs /tmp
	--tmpfs /var/tmp
	--tmpfs /run/user
	--dir "$runtime"
	--ro-bind "$report_socket_dir" "$report_socket_dir"
	--tmpfs /home
	--bind "$sandbox_home" "$HOME"
	--ro-bind "$repo" "$repo"
	--bind "$root" "$root"
	--bind "$repo/.git" "$repo/.git"
)
bind_existing() {
	local mode=$1 path
	shift
	for path in "$@"; do
		[[ -e $path ]] && mounts+=("$mode" "$path" "$path")
	done
	return 0
}
bind_existing --ro-bind "$repo"/.git/{config,hooks,info,modules} "$root"/agent/tool "$root"/{data,.doc,.claude,.direnv,.envrc} \
	"$root"/{flake.nix,flake.lock,opencode.json,start.sh,stop.sh,status.sh,metrics.sh,submit.sh} \
	"$HOME"/.config/{codingame,opencode}
bind_existing --bind "$HOME"/.local/share/opencode "$HOME"/.local/state/opencode "$HOME"/.cache/opencode
if [[ ${1##*/} == claude ]]; then
	bind_existing --bind "$HOME"/.claude "$HOME"/.claude.json
	mounts+=(--tmpfs "$HOME/.claude/projects")
	bind_existing --ro-bind "$HOME/.claude/projects/${repo//\//-}/memory"
	for private in paste-cache file-history shell-snapshots session-env backups debug; do
		[[ -d $HOME/.claude/$private ]] && mounts+=(--tmpfs "$HOME/.claude/$private")
	done
	[[ -e $HOME/.claude/history.jsonl ]] && mounts+=(--ro-bind /dev/null "$HOME/.claude/history.jsonl")
fi

hidden_env=()
for name in DBUS_SESSION_BUS_ADDRESS SSH_AUTH_SOCK GNOME_KEYRING_CONTROL GPG_AGENT_INFO WAYLAND_DISPLAY DISPLAY XAUTHORITY SWAYSOCK I3SOCK \
	$(compgen -e | grep '^CLAUDE_CODE_MESSAGING_' || true); do
	hidden_env+=(--unsetenv "$name")
done
exec "${CG_BWRAP:+$CG_BWRAP/bin/}bwrap" --die-with-parent --new-session --unshare-pid --unshare-ipc \
	"${mounts[@]}" "${hidden_env[@]}" --chdir "$PWD" -- "$@"
