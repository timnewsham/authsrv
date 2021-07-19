
// Mock DB for initial testing.
// todo: replace with real database

use std::collections::HashMap;
use std::collections::HashSet;
//use std::time::SystemTime as Time;

pub struct User {
    pub name: String,
    pub hash: String, // XXX
    pub scopes: HashSet<String>,
    //pub enabled: bool,
    //pub retire: Time,
}

pub struct DB {
    users: HashMap<String, User>,
    scopes: HashSet<String>,
}

// XXX for testing. "adminadmin"
const HASH: &str = "$argon2i$v=19$m=4096,t=3,p=1$cmFuZG9tc2FsdA$HXrbCSqkWTwH9W4z4JTyyJuuhEX/DNDs5tgTDfo+dHI";

#[allow(dead_code)]
impl DB {
    pub fn new() -> Self {
        let mut db = DB { 
            users: HashMap::new(),
            scopes: HashSet::new(),
        };
        db.add_scope("authadmin");
        db.add_user("admin", HASH, &vec!["authadmin"]);
        db
    }

    pub fn get_scopes(&self) -> HashSet<String> {
        self.scopes.clone()
    }
    pub fn add_scope(&mut self, scope: &str) -> Option<()> {
        if self.scopes.contains(scope) {
            None
        } else {
            self.scopes.insert(scope.to_owned());
            Some(())
        }
    }
    pub fn delete_scope(&mut self, scope: &str) -> Option<()> {
        if self.scopes.remove(scope) {
            // XXX remove scope from all users?
            Some(())
        } else {
            None
        }
    }

    pub fn get_user(&self, user: &str) -> Option<&User> {
        // XXX prune any scopes that no longer exist?
        self.users.get(user)
    }
    pub fn add_user(&mut self, name: &str, hash: &str, scopes: &Vec<&str>) -> Option<()> {
        if self.users.contains_key(name) {
            return None;
        } 

        let u = User{ 
            name: name.to_owned(),
            hash: hash.to_owned(),
            scopes: scopes.iter().copied().map(str::to_owned).collect(),
        };
        self.users.insert(name.to_owned(), u);
        Some(())
    }
}

