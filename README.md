# Kernel avec système de fichier en FAT32

## Objectif

Ce kernel permet de charger un système de fichier en FAT32 et d'effectuer des opérations basiques dessus.

Comme:
- Lister les fichiers
- Lire un fichier
- Écrire un fichier
- Créer un fichier
- Supprimer un fichier
- Renommer un fichier 
- Déplacer un fichier 
- Créer un dossier
- Navigation dans les dossiers
- Affichage du chemin actuel


## Compilation

Pour compiler le kernel, il suffit de lancer la commande `cargo bootimage`.

## Lancement

Pour lancer le kernel, il faut avoir qemu installé et lancer la commande `qemu-img create -f raw disk.img 100M` pour créer un disque virtuel de 100M. 

Ensuite, il faut effectuer la commande `qemu-system-x86_64 -drive format=raw,file=<chemin> -drive format=raw,file=disk.img,if=ide` en remplaçant le chemin du fichier par le chemin du fichier généré par la compilation suite au lancement de la commande `cargo bootimage`. (car il vous donnera le chemin du fichier généré .bin)

## Fonctionnalités

- [x] Lister les fichiers avec la commande `ls`
- [x] Lire un fichier avec la commande `cat` 
- [x] Écrire un fichier avec la commande `echo`
- [x] Créer un fichier avec la commande `touch`
- [x] Supprimer un fichier avec la commande `rm`
- [x] Renommer un fichier avec la commande `mv`
- [x] Déplacer un fichier avec la commande `mv`
- [x] Créer un dossier avec la commande `mkdir`
- [x] Navigation dans les dossiers avec la commande `cd`
- [x] Affichage du chemin actuel avec la commande `pwd`
- [x] Support des fichiers de plus de 1 cluster

## Fonctionnalités à venir

- Fini (pour l'instant)

## Auteurs

- Léo Haidar
- Luc Martin

## License

Ce projet est sous license MIT