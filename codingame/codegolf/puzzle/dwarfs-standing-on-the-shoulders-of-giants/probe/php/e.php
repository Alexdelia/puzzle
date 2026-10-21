<?$s=fread(STDIN,99);echo 2+preg_match_all('/^(\d+) (?=[^#]*#.* \1
)/ms',"$s#$s");