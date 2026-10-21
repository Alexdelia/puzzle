# PHP: perl one-liner via system()

`<?system('perl -Minteger -ape\'...\'');` costs 15 bytes over the bash perl core (only `'` escaped as `\'`, `$` untouched in single-quoted PHP string). Beats backticks (`$x` interpolates, each needs `\$`) and nowdoc (+22). Verified local 21/21, 208 bytes on mayan-calculation.
