while :;do
readarray -n 8 a 
printf "%d\n" ${a[@]}|awk 'NR==1{m=$1;mi=0}{if($1>m){m=$1;mi=NR-1}}END{print mi}'
done
