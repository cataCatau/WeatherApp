use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Favorites {
    pub locations: Vec<String>,
}

impl Favorites {
    pub fn load() -> Self {
        let path = Path::new("favorites.json");
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(content) => match serde_json::from_str::<Favorites>(&content) {
                    Ok(favs) => return favs,
                    Err(e) => eprintln!("Corrupted JSON in favorites.json: {}", e),
                },
                Err(e) => eprintln!("Couldn't read from favorites.json: {}", e),
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(e) = fs::write("favorites.json", json) {
                    eprintln!("Critical error writing to favorites.json: {}", e);
                }
            }
            Err(e) => eprintln!("Error during serialization: {}", e),
        }
    }

    pub fn add(&mut self, city: String) {
        if !self.locations.contains(&city) {
            self.locations.push(city);
            self.save();
        }
    }

    pub fn remove(&mut self, city: &String) {
        self.locations.retain(|x| x != city);
        self.save();
    }
}
