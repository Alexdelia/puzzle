# PHP env on CG

Verified 2026-10-19 with test games on chuck-norris:

- `PHP_VERSION` = `7.3.9`, CLI, NTS, 64-bit ints (`PHP_INT_SIZE` 8): no arrow functions (`fn`, 7.4), no `??=` (7.4), no spread in arrays with string keys, `match`/named args/nullsafe (8.x) absent
- no php.ini (`php_ini_loaded_file()` false); ini: `short_open_tag=1` (`<?` opener works), `error_reporting=32767` (E_ALL), `display_errors=stderr` (notices/warnings go to stderr, not judged; so bare-word undefined constants used as strings, a 7.3 warning, should cost nothing: unverified on CG), `memory_limit=512M`, `disable_functions=ini_set`, `precision=14`, `serialize_precision=-1`, `register_argc_argv=1`, `date.timezone` unset (UTC)
- extensions: `Core date libxml pcre sqlite3 bcmath calendar ctype dom fileinfo filter ftp hash iconv intl json mbstring SPL PDO standard posix Reflection shmop SimpleXML soap sockets pdo_sqlite Phar sysvmsg sysvsem sysvshm tokenizer wddx xml xmlreader xmlwriter zip eio`
- absent on CG: zlib (no `gzinflate`/`gzcompress` packing), gmp, openssl, session, gd, sodium, exif, readline, pcntl
- locale `LC_CTYPE=en_US.UTF-8`, rest `C`
- echo of floats: `0.1+0.2` prints `0.3`, `1e20` prints `1.0E+20`, `7/3` prints `2.3333333333333`
- local launcher (`agent/tool/lang/php/build.sh`) mirrors ini and extension set except `eio` (PECL) and locale; startup ~10 ms
