{ fetchNixpkgs, system, ... }:
let
  nixpkgs1909 = fetchNixpkgs {
    rev = "19.09";
    sha256 = "0mhqhq21y5vrr1f30qd2bvydv4bbbslvyzclhw0kdxmkgg3z4c92";
  };
  pkgs = import nixpkgs1909.path {
    inherit system;
    config.php = {
      systemd = false;
      imap = false;
      ldap = false;
      mysqlnd = false;
      mysqli = false;
      pdo_mysql = false;
      apxs2 = false;
      curl = false;
      gettext = false;
      pcntl = false;
      pdo_odbc = false;
      postgresql = false;
      pdo_pgsql = false;
      readline = false;
      zlib = false;
      openssl = false;
      gd = false;
      exif = false;
      fpm = false;
      gmp = false;
      sodium = false;
      phpdbg = false;
      cgi = false;
      valgrind = false;
    };
  };
  version = "7.3.9";
  php = pkgs.php73.overrideAttrs (old: {
    name = "php-${version}";
    inherit version;
    src = pkgs.fetchurl {
      url = "https://www.php.net/distributions/php-${version}.tar.bz2";
      sha256 = "a39c9709a8c9eb7ea8ac4933ef7a78b92f7e5735a405c8b8e42ee39541d963c4";
    };
    buildInputs = old.buildInputs ++ [ pkgs.zlib ];
    postPatch = ''
      substituteInPlace ext/zip/config.m4 --replace "for i in /usr/local /usr" "for i in ${pkgs.zlib.dev}"
    '';
    configureFlags = old.configureFlags ++ [
      "--disable-session"
      "--enable-shmop"
      "--enable-sysvmsg"
      "--enable-sysvsem"
      "--enable-sysvshm"
      "--enable-wddx"
    ];
  });
in
pkgs.symlinkJoin {
  name = "php-${version}";
  paths = [ php ];
}
