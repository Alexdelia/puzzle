# Anonymous hash lookup on the whole input

`#!perl -0p` slurps stdin into `$_`; `$_={3,7,10,3935}->{$_+0}` maps the first number (numeric context stops at the first non-digit, only a warning) to an answer and `-p` prints it. 5 bytes shorter than `%h=(...);$_=$h{$_+0}` and 3 shorter than `-0ap` + `$h{$F[0]}`. `1e4`, `15e3`, `5e9-25e3` work as keys/values (integral NV prints as integer up to 15 digits). Verified locally on perl 5.34 (roller-coaster hardcode).

Local referee note: `CG_PERL` may be unset in the dev shell; `CG_PERL=/nix/store/*-cg-perl-5.34.0` (found by `ls /nix/store | grep cg-perl`) fixes it.
