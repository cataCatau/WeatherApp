use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Favorites {
    pub locations: Vec<String>,
}

impl Favorites {
    pub fn load() -> Self {
        if Path::new("favorites.json").exists()
            && let Ok(content) = fs::read_to_string("favorites.json")
            && let Ok(favs) = serde_json::from_str(&content)
        {
            return favs;
        }

        Self::default()
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self)
            && let Err(e) = fs::write("favorites.json", json)
        {
            eprintln!("Eroare la salvarea favoritelor: {}", e);
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
