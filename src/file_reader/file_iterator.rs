use std::io::prelude::*;
use std::iter::Iterator;

use std::collections::HashMap;
use std::{fs::File, io::BufReader};

use crate::alglobo::transaction::Transaction;

pub struct FileIterator {
    reader: BufReader<File>,
    ended: bool,
}

impl FileIterator {
    /// Funcion destinada a crear una instancia de `FileIterator`
    /// # Errors:
    /// Arroja error si el archivo no existe
    /// PRE: la variable path hace referencia a un archivo. La funcion devolvera Err si no se
    /// encuentra al archivo
    pub fn new(path: &str) -> Result<FileIterator, String> {
        if let Ok(file) = File::open(path) {
            return Ok(FileIterator {
                reader: BufReader::new(file),
                ended: false,
            });
        }
        Err("[Sistema Error] Archivo de reservas no encontrado".to_string())
    }

    /// La funcion devuelve el atributo ended
    #[must_use]
    pub fn ended(&self) -> bool {
        self.ended
    }
}

impl Iterator for FileIterator {
    type Item = Transaction;

    /// Implementacion del metodo next de la interfaz Iterador, para la clase
    /// `FileIterator`
    /// #Errors
    /// Arroja error si el archivo termino
    fn next(&mut self) -> Option<Transaction> {
        let mut line = String::new();
        let len = self
            .reader
            .read_line(&mut line)
            .expect("Read line should not fail");
        if len == 0 {
            self.ended = true;
            return None;
        }
        line = line.replace("\n", "");
        let params: Vec<&str> = line.split(',').collect();
        if params.len() < 4 {
            return None;
        }
        let mut services_info: HashMap<String, f64> = HashMap::new();
        services_info.insert("Airline".to_string(), params[1].parse::<f64>().unwrap());
        services_info.insert("Bank".to_string(), params[2].parse::<f64>().unwrap());
        services_info.insert("Hotel".to_string(), params[3].parse::<f64>().unwrap());

        Some(Transaction::new(
            params[0].parse::<u64>().unwrap(),
            services_info,
        ))
    }
}
