use alloc::string::String;
use crate::{print, println};
use crate::filesystem::FS;

const MAX_CMD_LENGTH: usize = 100;

pub struct CommandBuffer {
    buffer: String,
}

impl CommandBuffer {
    pub fn new() -> Self {
        CommandBuffer {
            buffer: String::with_capacity(MAX_CMD_LENGTH),
        }
    }

    pub fn add_char(&mut self, c: char) {
        if self.buffer.len() < MAX_CMD_LENGTH {
            self.buffer.push(c);
            print!("{}", c);
        }
    }

    pub fn backspace(&mut self) {
        if !self.buffer.is_empty() {
            self.buffer.pop();
            print!("\x08 \x08"); // backspace pour effacer le caractère
        }
    }

    pub fn execute(&mut self) {
        println!();
        let input = self.buffer.trim();
        
        if !input.is_empty() {
            let parts: alloc::vec::Vec<&str> = input.split_whitespace().collect();
            let command = parts[0];
            let args = &parts[1..];

            match command {
                "help" => print_help(),
                "touch" => {
                    if args.len() != 1 {
                        println!("Usage: touch <filename>");
                    } else {
                        match FS.lock().create_file(args[0]) {
                            Ok(_) => println!("Fichier créé: {}", args[0]),
                            Err(e) => println!("Erreur de création du fichier: {:?}", e),
                        }
                    }
                },
                "cat" => {
                    if args.len() != 1 {
                        println!("Usage: cat <filename>");
                    } else {
                        match FS.lock().read_file(args[0]) {
                            Ok(content) => println!("{}", content),
                            Err(e) => println!("Erreur de lecture du fichier: {:?}", e),
                        }
                    }
                },
                "write" => {
                    if args.len() < 2 {
                        println!("Usage: write <filename> <content>");
                    } else {
                        let content = args[1..].join(" ");
                        match FS.lock().write_file(args[0], &content) {
                            Ok(_) => println!("Contenu écrit dans le fichier: {}", args[0]),
                            Err(e) => println!("Erreur d'écriture du fichier: {:?}", e),
                        }
                    }
                },
                "ls" => {
                    match FS.lock().list_files() {
                        Ok(files) => {
                            if files.is_empty() {
                                println!("Aucun fichier dans le répertoire");
                            } else {
                                for file in files {
                                    println!("{}", file);
                                }
                            }
                        },
                        Err(e) => println!("Erreur de lecture du répertoire: {:?}", e),
                    }
                },
                _ => println!("Commande inconnue: {}, taper 'help' pour afficher l'aide", command),
            }
        }
        self.buffer.clear();
        print!("> ");
    }
}

fn print_help() {
    println!("Commandes disponibles:");
    println!("  help             - Affichage de l'aide");
    println!("  touch <filename> - Création d'un fichier");
    println!("  cat <filename>   - Affichage du contenu d'un fichier");
    println!("  write <filename> <content> - Ecrire dans un fichier");
    println!("  ls              - Liste des fichiers");
}