const MAX_USERS: usize = 100;
const MAX_NAME_LEN: usize = 50;
const MAX_EMAIL_LEN: usize = 50;
const MAX_PASSWORD_LENGTH: usize = 1000;
const INACTIVITY_THRESHOLD: i32 = 5;
const MAX_SESSION_TOKEN_LEN: usize = 32;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct UserStruct {
    pub password: [u8; MAX_PASSWORD_LENGTH],
    pub username: [u8; MAX_NAME_LEN],
    pub user_id: i32,
    pub email: [u8; MAX_EMAIL_LEN],
    pub inactivity_count: i32,
    pub is_active: i32,
    pub session_token: [u8; MAX_SESSION_TOKEN_LEN],
}

impl Default for UserStruct {
    fn default() -> Self {
        UserStruct {
            password: [0; MAX_PASSWORD_LENGTH],
            user_id: 0,
            email: [0; MAX_EMAIL_LEN],
            inactivity_count: 0,
            username: [0; MAX_NAME_LEN],
            session_token: [0; MAX_SESSION_TOKEN_LEN],
            is_active: 0,
        }
    }
}

#[derive(Debug)]
pub struct UserDatabase {
    // pub users: Vec<Option<Box<UserStruct>>>, 
    pub users: [Option<Box<UserStruct>>; MAX_USERS],
    pub count: i32,
    pub capacity: i32,
}

// Helper fnecs
// copy fns
fn copy_string(dest: &mut[u8], src: &str) {
    let src_bytes = src.as_bytes();
    // set a limit of the copy length, from the src, or capped at dest length -1 for null term
    let copy_length
     = std::cmp::min(src_bytes.len(), dest.len() - 1);
    dest.fill(0); // used to ensure always have proper null term even if smaller src
    dest[..copy_length
    ].copy_from_slice(&src_bytes[..copy_length
        ]);
    //copy 

}
// rust cant directly read byte array as a string
fn byte_to_string(bytes: &[u8]) -> String {
    let mut end = 0;
    while end < bytes.len() && bytes[end] != 0 {
        end += 1;
    }
    String::from_utf8_lossy(&bytes[..end]).to_string()
}


pub fn init_database() -> Box<UserDatabase> {
    let db = UserDatabase {
        users: std::array::from_fn(|_index| None),
        count: 0,
        capacity: MAX_USERS as i32,
    };
    // let db = UserDatabase {
    //     users: vec![None; MAX_USERS],  // Create Vec on heap
    //     count: 0,
    //     capacity: MAX_USERS as i32,
    // };
    println!("=== RUST DEBUG: UserDatabase created, boxing it ===");

    Box::new(db)
}

// NOTSURE: userstruct change to mut, not sure
pub fn add_user(db: &mut UserDatabase, mut user: Box<UserStruct>) {
    if (db.count as usize) >= MAX_USERS {
        return;
    }
    user.user_id = db.count + 1; // Start IDs from 1 to match expected output
    db.users[db.count as usize] = Some(user);
    db.count += 1;
}


pub fn create_user(username: &str, email: &str, user_id: i32, password: &str) -> Box<UserStruct> {
    let mut user = UserStruct {
        password: [0; MAX_PASSWORD_LENGTH],
        username: [0; MAX_NAME_LEN],
        user_id,
        email: [0; MAX_EMAIL_LEN],
        inactivity_count: 0,
        is_active: 1,
        session_token: [0; MAX_SESSION_TOKEN_LEN], //init cuz cant change userstruct
    };
    copy_string(&mut user.email, email);
    copy_string(&mut user.password, password);
    copy_string(&mut user.username, username);
    
    Box::new(user)
}


// <'a> is lifetime wildcard, ties the return value lifetime to parameters (references)
// fixes the need to return index thing
pub fn find_user_by_username<'a>(db: & 'a UserDatabase, username: &'a str) -> Option<&'a UserStruct> {
    for i in 0..(db.count as usize) {
        if let Some(ref user) = db.users[i] {
            let curr_username = byte_to_string(&user.username);
            if curr_username == username {
                return Some(user);
            }
        }
    }
    None
}
//same just add mut for ref
pub fn find_user_by_username_mut<'a>(db: &'a mut UserDatabase, username: & 'a str) -> Option<& 'a mut UserStruct> {
    // find 
    let mut found_index = None;
    for i in 0..(db.count as usize) {
        if let Some(ref user) = db.users[i] {
            let curr_username = byte_to_string(&user.username);
            if curr_username == username {
                found_index = Some(i);
                break;
            }
        }
    }
    
    // make it mut
    if let Some(index) = found_index {
        if let Some(ref mut user) = db.users[index] {
            return Some(user);
        }
    }
    
    None
}

pub fn print_database(db: &UserDatabase) {
    for i in 0..(db.count as usize) {
        if let Some(ref user) = db.users[i] {
            let curr_username = byte_to_string(&user.username);
            let curr_email = byte_to_string(&user.email);
            let curr_password = byte_to_string(&user.password);
            println!("User: {}, ID: {}, Email: {}, Inactivity: {}  Password = {}", 
                curr_username, user.user_id, curr_email, user.inactivity_count, curr_password);
        }
    }
}

pub fn update_database_daily(db: &mut UserDatabase) {
    // TODO: Implement this function from Part 1
    println!("=== RUST DEBUG: update_database_daily started, count = {} ===", db.count);

    for i in 0..(db.count as usize) {
        if let Some(ref mut user) = db.users[i] {
            if user.is_active == 0 && user.inactivity_count > INACTIVITY_THRESHOLD {
                // user.is_active = 0;
                db.users[i] = None;
            } else {
                user.inactivity_count += 1;
            }
        }
    }
}

pub fn user_login(db: &mut UserDatabase, username: &str) {
    if let Some(user) = find_user_by_username_mut(db, username) {
        user.inactivity_count = 0;
    }
}

fn main() {
    let mut db = init_database();
}