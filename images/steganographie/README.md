Exemples de lignes de commandes :

./target/release/steg -w -l -i ./tests/celeste-3.png ./tests/results/celeste-3-0.5-sha2-p-c.png -t long.txt -g sha256 -c aes_key -p 0.50
./target/release/steg -w -l -i ./tests/celeste-3.png ./tests/results/celeste-3-1.0-sha2-p-c.png -t long.txt -g sha256 -c aes_key -p 0.999
./target/release/steg -w -l -i ./tests/earth.png ./tests/results/earth-1.0-sha2-p-c.png -t verylong.txt -g sha256 -c aes_key -p 0.999

steganographie/steg -w -l -i christmas.jpg results/christmas-sha256.jpg -t steganographie/message.txt -g sha256 -c aes_key -p 0.7

Informations sur les arguments

./target/release/steg => l'emplacement de l'executable
./tests/celeste-3.png ./tests/results/celeste-3-0.5-sha2-p-c.png => image 1 => image de départ, image 2 => image de sortie avec message caché
-t long.txt => ton fichier txt
-c aes_key => la clé aes (que je t'ai donné)
-p 0.50 => le pourcentage max de pixels modifiés (attention pour -p 1.00 ça prend beaucoup de calcul)