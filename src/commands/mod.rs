// src/command/mod.rs

// fichier pour les commandes

use alloc::string::String;
use alloc::vec::Vec;
use crate::print;
use crate::println;

const MAX_CMD_LENGTH: usize = 100;

pub struct CommandBuffer { // structure pour stocker la commande
    buffer: String,
}

impl CommandBuffer {
    pub fn new() -> Self {
        CommandBuffer {
            buffer: String::with_capacity(MAX_CMD_LENGTH),
        }
    }

    pub fn add_char(&mut self, c: char) { // juste pour ajouter un caractère à la commande
        if self.buffer.len() < MAX_CMD_LENGTH {
            self.buffer.push(c);
            print!("{}", c);
        }
    }

    pub fn backspace(&mut self) {
        if !self.buffer.is_empty() {
            self.buffer.pop();
            print!("\x08 \x08"); // backspace pour effacer le caractère et le remplacer par un espace
        }
    }

    pub fn execute(&mut self) {
        println!(); // New line after command
        let cmd = self.buffer.trim();
        if !cmd.is_empty() {
            match cmd {
                "help" => print_help(),
                _ => println!("Commande non reconnue: '{}'. Mettez 'help' afin d'afficher la lsite des commandes.", cmd),
            }
        }
        self.buffer.clear();
        print!("> "); // nouveau prompt
    }
}

fn print_help() {
    println!("Commandes valables:");
    println!("  help    - Aide sur les commandes disponibles");
    println!("  TODO: Plein de commandes arrivent bientôt !");
}