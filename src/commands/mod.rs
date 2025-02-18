use alloc::string::String;
use crate::{print, println};
use crate::filesystem::FS;

const MAX_CMD_LENGTH: usize = 16; // commandes tres courtes

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
            print!("\x08 \x08");
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
                        println!("usage: touch <filename>");
                    } else {
                        match FS.lock().create_file(args[0]) {
                            Ok(_) => println!("fichier cree: {}", args[0]),
                            Err(e) => println!("erreur: {:?}", e),
                        }
                    }
                },
                "cat" => {
                    if args.len() != 1 {
                        println!("usage: cat <filename>");
                    } else {
                        match FS.lock().read_file(args[0]) {
                            Ok(content) => println!("{}", content),
                            Err(e) => println!("erreur: {:?}", e),
                        }
                    }
                },
                "write" => {
                    if args.len() < 2 {
                        println!("usage: write <filename> <contenu>");
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
                        Ok(files) => {
                            if files.is_empty() {
                                println!("pas de fichiers");
                            } else {
                                for file in files {
                                    println!("{}", file);
                                }
                            }
                        },
                        Err(e) => println!("erreur: {:?}", e),
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
}