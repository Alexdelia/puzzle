<?=2+preg_match_all('/(\d+) (?=[^#]*#.* \1
)/s',($s=fread(STDIN,99))."#$s");