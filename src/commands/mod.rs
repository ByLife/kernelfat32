use alloc::string::String;
use crate::{print, println};
use crate::filesystem::FS;

const MAX_CMD_LENGTH: usize = 64; // augmenté à 64 caractères

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

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn backspace(&mut self) {
        if !self.buffer.is_empty() {
            self.buffer.pop();
            // déplace le curseur en arrière, efface le caractère, et redéplace le curseur
            print!("\u{8} \u{8}");
        }
    }

    pub fn execute(&mut self) { // execute la commande
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
                        println!("usage: touch <fichier>");
                    } else {
                        match FS.lock().create_file(args[0]) {
                            Ok(_) => println!("fichier cree: {}", args[0]),
                            Err(e) => println!("erreur: {:?}", e),
                        }
                    }
                },
                "cat" => {
                    if args.len() != 1 {
                        println!("usage: cat <fichier>");
                    } else {
                        match FS.lock().read_file(args[0]) {
                            Ok(content) => println!("{}", content),
                            Err(e) => println!("erreur: {:?}", e),
                        }
                    }
                },
                "write" => {
                    if args.len() < 2 {
                        println!("usage: write <fichier> <contenu>");
                    } else {
                        let content = args[1..].join(" ");
                        match FS.lock().write_file(args[0], &content) {
                            Ok(_) => println!("ecrit dans: {}", args[0]),
                            Err(e) => println!("erreur: {:?}", e),
                        }
                    }
                },
                "ls" => {
                    match FS.lock().list_files() {
                        Ok(entries) => {
                            if entries.is_empty() {
                                println!("pas de fichiers");
                            } else {
                                for (name, is_dir) in entries {
                                    if is_dir {
                                        print!("[DIR] ");
                                    }
                                    println!("{}", name);
                                }
                            }
                        },
                        Err(e) => println!("erreur: {:?}", e),
                    }
                },
                "rm" => {
                    if args.len() != 1 {
                        println!("usage: rm <fichier>");
                    } else {
                        match FS.lock().remove(args[0]) {
                            Ok(_) => println!("supprime: {}", args[0]),
                            Err(e) => println!("erreur: {:?}", e),
                        }
                    }
                },
                "mkdir" => {
                    if args.len() != 1 {
                        println!("usage: mkdir <repertoire>");
                    } else {
                        match FS.lock().create_directory(args[0]) {
                            Ok(_) => println!("repertoire cree: {}", args[0]),
                            Err(e) => println!("erreur: {:?}", e),
                        }
                    }
                },
                "cd" => {
                    if args.len() != 1 {
                        println!("usage: cd <repertoire>");
                    } else {
                        match FS.lock().change_directory(args[0]) {
                            Ok(_) => (),
                            Err(e) => println!("erreur: {:?}", e),
                        }
                    }
                },
                "pwd" => {
                    match FS.lock().print_working_directory() {
                        Ok(path) => println!("{}", path),
                        Err(e) => println!("erreur: {:?}", e),
                    }
                },

                "mv" => {
                if args.len() != 2 {
                    println!("usage: mv <source> <destination>");
                } else {
                    match FS.lock().move_node(args[0], args[1]) {
                        Ok(_) => println!("deplace: {} -> {}", args[0], args[1]),
                        Err(e) => println!("erreur: {:?}", e),
                    }
                }
            },

                _ => println!("commande inconnue: '{}'. tape 'help'", command),
            }
        }
        self.buffer.clear();
        print!("> ");
    }
}

fn print_help() {
    println!("commandes dispo:");
    println!("  help             - montre l'aide");
    println!("  touch <fichier>  - cree un fichier");
    println!("  cat <fichier>    - affiche un fichier");
    println!("  write <fichier> <texte> - ecrit dans un fichier");
    println!("  ls              - liste les fichiers");
    println!("  rm <fichier>     - supprime un fichier");
    println!("  mkdir <repertoire> - cree un repertoire");
    println!("  cd <repertoire>  - change de repertoire");
    println!("  pwd             - affiche le repertoire courant");
    println!("  mv <src> <dst>  - deplace un fichier ou repertoire");
}